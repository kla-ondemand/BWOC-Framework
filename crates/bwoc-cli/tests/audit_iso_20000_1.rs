//! End-to-end smoke for the `audit-iso-20000-1` runtime (BWOC-33).
//!
//! Verifies the round-trip `bwoc audit run --plugin audit-iso-20000-1 --json`
//! against a tempdir workspace that declares both an attestation block and a
//! sample block. 20000-1 is the first runtime that mixes two evidence kinds,
//! so this confirms the dispatcher accepts the BWOC-27 `attestation` AND
//! `sample` kinds end-to-end: the plugin routes each criterion by its
//! `expected_evidence_kind`, emits the matching evidence shape, the dispatcher
//! validates + re-emits it in the canonical `--json` envelope, and the exit
//! code reflects the fail count.
//!
//! Skipped on Windows for the same reason as `smoke.rs` — the embedded
//! template's `*.md` symlinks don't round-trip on the Windows filesystem
//! without admin perms. The Windows audit path is exercised by inline unit
//! tests in `src/audit.rs`.

#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_bwoc"))
}

/// Copy the plugin dir verbatim from the worktree into the tempdir. Preserves
/// `audit.sh` executable bit (the dispatcher only runs entries with `+x`).
fn install_plugin(workspace: &Path) {
    let src = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../modules/plugins/audit-iso-20000-1")
        .canonicalize()
        .expect("canonicalize audit-iso-20000-1 plugin source");
    let dst = workspace.join("modules/plugins/audit-iso-20000-1");
    std::fs::create_dir_all(&dst).expect("mkdir plugin dst");
    for entry in std::fs::read_dir(&src).expect("read plugin src dir") {
        let entry = entry.expect("read dir entry");
        let from = entry.path();
        let to = dst.join(entry.file_name());
        std::fs::copy(&from, &to).expect("copy plugin file");
        // Preserve mode (audit.sh needs +x).
        let perms = std::fs::metadata(&from).expect("stat src").permissions();
        std::fs::set_permissions(&to, perms).expect("chmod dst");
    }
}

fn run_audit(ws: &Path) -> std::process::Output {
    Command::new(bin())
        .args(["audit", "run", "--plugin", "audit-iso-20000-1", "--json"])
        .args(["--workspace"])
        .arg(ws)
        .output()
        .expect("spawn bwoc audit run")
}

/// Workspace.toml for the mixed-evidence smoke: two attestations (one with
/// `valid_through`, one without) and two samples (one with `window`, one
/// without). The remaining four criteria — `service-catalogue` (attestation),
/// `service-level-management` / `problem-management` / `continual-improvement`
/// (sample) — have no evidence and must fail.
const WORKSPACE_TOML: &str = r#"[workspace]
name = "bwoc-33-smoke"
version = "0.1.0"

[plugins.audit-iso-20000-1]
enabled = true

[[plugins.audit-iso-20000-1.attestations]]
criterion_id  = "20000-1-service-policy-and-objectives"
statement     = "Service management policy v2.1 ratified 2026-01-15; objectives reviewed quarterly."
signer        = "Service Owner: Tonkla K."
signed_at     = "2026-01-15"
valid_through = "2027-01-15"

[[plugins.audit-iso-20000-1.attestations]]
criterion_id = "20000-1-service-management-system-scope"
statement    = "SMS scope covers the hosted API platform and its support services."
signer       = "Service Owner: Tonkla K."
signed_at    = "2026-01-15"

[[plugins.audit-iso-20000-1.samples]]
criterion_id  = "20000-1-incident-management"
summary       = "49 of 50 incidents resolved within SLA"
sampled_count = 49
sampled_of    = 50
window        = "2026-Q1"

[[plugins.audit-iso-20000-1.samples]]
criterion_id  = "20000-1-change-management"
summary       = "30 of 30 changes followed the approval workflow"
sampled_count = 30
sampled_of    = 30
"#;

#[test]
fn audit_iso_20000_1_emits_attestation_sample_and_fail_findings() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let ws = tmp.path();

    std::fs::create_dir_all(ws.join(".bwoc")).expect("mkdir .bwoc");
    std::fs::write(ws.join(".bwoc/workspace.toml"), WORKSPACE_TOML).expect("write workspace.toml");
    install_plugin(ws);

    let output = run_audit(ws);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // 8 criteria; 4 have evidence (pass), 4 do not (fail). Exit = fail count = 4.
    assert!(
        !stderr.contains("framework error") && !stderr.contains("not in closed enum"),
        "dispatcher rejected a finding shape:\n{stderr}\n\nstdout:\n{stdout}"
    );
    assert_eq!(
        output.status.code(),
        Some(4),
        "expected exit code 4 (one per missing-evidence fail); stdout=\n{stdout}\nstderr=\n{stderr}"
    );

    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("parse --json envelope");

    let summary = &envelope["summary"];
    assert_eq!(summary["pass_count"], 4);
    assert_eq!(summary["fail_count"], 4);
    assert_eq!(summary["framework_error"], false);

    let runs = envelope["runs"].as_array().expect("runs is array");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0]["plugin"], "audit-iso-20000-1");
    assert_eq!(runs[0]["version"], "0.2.0");

    let findings = runs[0]["findings"].as_array().expect("findings is array");
    assert_eq!(findings.len(), 8, "expected 8 findings (one per criterion)");

    let find = |id: &str| -> serde_json::Value {
        findings
            .iter()
            .find(|f| f["criterion_id"] == id)
            .unwrap_or_else(|| panic!("{id} finding present"))
            .clone()
    };

    // Sample (with window) — incident management.
    let incident = find("20000-1-incident-management");
    assert_eq!(incident["status"], "pass");
    assert_eq!(incident["evidence"]["kind"], "sample");
    assert_eq!(
        incident["evidence"]["value"],
        "49 of 50 incidents resolved within SLA"
    );
    assert_eq!(incident["evidence"]["sampled_count"], 49);
    assert_eq!(incident["evidence"]["sampled_of"], 50);
    assert_eq!(incident["evidence"]["window"], "2026-Q1");
    assert!(
        incident.get("remedy").is_none(),
        "pass must not carry remedy"
    );

    // Sample (no window) — change management. window key must be dropped.
    let change = find("20000-1-change-management");
    assert_eq!(change["status"], "pass");
    assert_eq!(change["evidence"]["kind"], "sample");
    assert_eq!(change["evidence"]["sampled_count"], 30);
    assert_eq!(change["evidence"]["sampled_of"], 30);
    assert!(
        change["evidence"]
            .as_object()
            .unwrap()
            .get("window")
            .is_none(),
        "window omitted in workspace.toml — must not surface in envelope"
    );

    // Attestation (with valid_through) — service policy.
    let policy = find("20000-1-service-policy-and-objectives");
    assert_eq!(policy["status"], "pass");
    assert_eq!(policy["evidence"]["kind"], "attestation");
    assert_eq!(policy["evidence"]["signer"], "Service Owner: Tonkla K.");
    assert_eq!(policy["evidence"]["signed_at"], "2026-01-15");
    assert_eq!(policy["evidence"]["valid_through"], "2027-01-15");

    // Attestation (no valid_through) — SMS scope. valid_through key dropped.
    let scope = find("20000-1-service-management-system-scope");
    assert_eq!(scope["status"], "pass");
    assert_eq!(scope["evidence"]["kind"], "attestation");
    assert!(
        scope["evidence"]
            .as_object()
            .unwrap()
            .get("valid_through")
            .is_none(),
        "valid_through omitted in workspace.toml — must not surface in envelope"
    );

    // Missing attestation — service catalogue. Fail pointing at workspace.toml.
    let catalogue = find("20000-1-service-catalogue");
    assert_eq!(catalogue["status"], "fail");
    assert_eq!(catalogue["evidence"]["kind"], "file");
    assert_eq!(catalogue["evidence"]["value"], ".bwoc/workspace.toml");
    let cat_remedy = catalogue["remedy"].as_str().expect("fail carries remedy");
    assert!(
        cat_remedy.contains("[[plugins.audit-iso-20000-1.attestations]]")
            && cat_remedy.contains("20000-1-service-catalogue"),
        "attestation remedy must name the block and criterion: {cat_remedy}"
    );

    // Missing sample — service level management. Fail pointing at workspace.toml.
    let slm = find("20000-1-service-level-management");
    assert_eq!(slm["status"], "fail");
    assert_eq!(slm["evidence"]["kind"], "file");
    assert_eq!(slm["evidence"]["value"], ".bwoc/workspace.toml");
    let slm_remedy = slm["remedy"].as_str().expect("fail carries remedy");
    assert!(
        slm_remedy.contains("[[plugins.audit-iso-20000-1.samples]]")
            && slm_remedy.contains("20000-1-service-level-management"),
        "sample remedy must name the block and criterion: {slm_remedy}"
    );
}

#[test]
fn audit_iso_20000_1_fails_all_when_no_evidence_present() {
    // Plugin enabled but no attestations and no samples — every criterion
    // fails with its missing-evidence remedy. Confirms the first-run state is
    // honest about absent evidence rather than silently passing or erroring.
    let tmp = tempfile::tempdir().expect("create tempdir");
    let ws = tmp.path();

    std::fs::create_dir_all(ws.join(".bwoc")).expect("mkdir .bwoc");
    std::fs::write(
        ws.join(".bwoc/workspace.toml"),
        "[workspace]\nname = \"empty\"\nversion = \"0.1.0\"\n\n\
         [plugins.audit-iso-20000-1]\nenabled = true\n",
    )
    .expect("write workspace.toml");
    install_plugin(ws);

    let output = run_audit(ws);
    assert_eq!(
        output.status.code(),
        Some(8),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse --json envelope");
    assert_eq!(envelope["summary"]["fail_count"], 8);
    assert_eq!(envelope["summary"]["pass_count"], 0);
    assert_eq!(envelope["summary"]["framework_error"], false);

    // Every fail finding points at workspace.toml (kind=file), never "none".
    let findings = envelope["runs"][0]["findings"]
        .as_array()
        .expect("findings array");
    for f in findings {
        assert_eq!(f["status"], "fail");
        assert_eq!(f["evidence"]["kind"], "file");
    }
}

#[test]
fn audit_iso_20000_1_degrades_invalid_sample_to_fail_not_framework_error() {
    // A sample with sampled_of < sampled_count is operator data that's wrong,
    // not a plugin bug. The runtime must catch it and emit a `fail` finding
    // (kind=file) rather than emit invalid sample evidence that the dispatcher
    // would reject as a framework error (exit 255).
    let tmp = tempfile::tempdir().expect("create tempdir");
    let ws = tmp.path();

    std::fs::create_dir_all(ws.join(".bwoc")).expect("mkdir .bwoc");
    std::fs::write(
        ws.join(".bwoc/workspace.toml"),
        "[workspace]\nname = \"bad-sample\"\nversion = \"0.1.0\"\n\n\
         [plugins.audit-iso-20000-1]\nenabled = true\n\n\
         [[plugins.audit-iso-20000-1.samples]]\n\
         criterion_id  = \"20000-1-incident-management\"\n\
         summary       = \"impossible rate\"\n\
         sampled_count = 60\n\
         sampled_of    = 50\n",
    )
    .expect("write workspace.toml");
    install_plugin(ws);

    let output = run_audit(ws);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_ne!(
        output.status.code(),
        Some(255),
        "runtime emitted invalid evidence instead of degrading to fail; stderr:\n{stderr}"
    );
    let envelope: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse --json envelope");
    assert_eq!(envelope["summary"]["framework_error"], false);

    let findings = envelope["runs"][0]["findings"]
        .as_array()
        .expect("findings array");
    let incident = findings
        .iter()
        .find(|f| f["criterion_id"] == "20000-1-incident-management")
        .expect("incident finding present");
    assert_eq!(incident["status"], "fail");
    assert_eq!(incident["evidence"]["kind"], "file");
    let remedy = incident["remedy"].as_str().expect("fail carries remedy");
    assert!(
        remedy.contains("sampled_of") && remedy.contains("sampled_count"),
        "remedy should explain the count/of inversion: {remedy}"
    );
}
