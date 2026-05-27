//! End-to-end smoke for the `audit-iso-9001` runtime (BWOC-28).
//!
//! Verifies the round-trip `bwoc audit run --plugin audit-iso-9001 --json`
//! against a tempdir workspace that declares an attestation block. Confirms
//! the dispatcher accepts the BWOC-27 `attestation` evidence kind end-to-end:
//! the plugin emits attestation findings, the dispatcher validates + re-emits
//! them in the canonical `--json` envelope, exit code reflects fail count.
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
        .join("../../modules/plugins/audit-iso-9001")
        .canonicalize()
        .expect("canonicalize audit-iso-9001 plugin source");
    let dst = workspace.join("modules/plugins/audit-iso-9001");
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

/// Authoritative workspace.toml for the smoke run: enables the plugin and
/// declares two attestations (one with `valid_through`, one without). The
/// remaining six criteria have no attestation — they should emit `status=fail`
/// with the workspace.toml-pointing remedy.
const WORKSPACE_TOML: &str = r#"[workspace]
name = "bwoc-28-smoke"
version = "0.1.0"

[plugins.audit-iso-9001]
enabled = true

[[plugins.audit-iso-9001.attestations]]
criterion_id  = "9001-management-review"
statement     = "Management review held 2026-04-15 covering Q1 QMS performance, customer feedback, internal audit results, improvement opportunities."
signer        = "Quality Manager: Tonkla K."
signed_at     = "2026-04-15"
valid_through = "2027-04-15"

[[plugins.audit-iso-9001.attestations]]
criterion_id = "9001-leadership-and-policy"
statement    = "Quality policy v1.2 ratified 2026-01-10 — aligned with strategic direction (FY2026)."
signer       = "Top Management: Tonkla K."
signed_at    = "2026-01-10"
"#;

#[test]
fn audit_iso_9001_emits_attestation_and_fail_findings() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let ws = tmp.path();

    std::fs::create_dir_all(ws.join(".bwoc")).expect("mkdir .bwoc");
    std::fs::write(ws.join(".bwoc/workspace.toml"), WORKSPACE_TOML).expect("write workspace.toml");
    install_plugin(ws);

    let output = Command::new(bin())
        .args(["audit", "run", "--plugin", "audit-iso-9001", "--json"])
        .args(["--workspace"])
        .arg(ws)
        .output()
        .expect("spawn bwoc audit run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // criteria.toml declares 8 criteria; 2 have attestations (pass) and 6 do
    // not (fail). Exit code = fail count = 6.
    assert!(
        !stderr.contains("framework error"),
        "framework error in stderr — dispatcher rejected attestation shape:\n{stderr}\n\nstdout:\n{stdout}"
    );
    assert_eq!(
        output.status.code(),
        Some(6),
        "expected exit code 6 (one per missing-attestation fail); stdout=\n{stdout}\nstderr=\n{stderr}"
    );

    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("parse --json envelope");

    // Summary block: 2 pass (attestation), 6 fail (missing), framework_error=false.
    let summary = &envelope["summary"];
    assert_eq!(summary["pass_count"], 2);
    assert_eq!(summary["fail_count"], 6);
    assert_eq!(summary["framework_error"], false);

    let runs = envelope["runs"].as_array().expect("runs is array");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0]["plugin"], "audit-iso-9001");

    let findings = runs[0]["findings"].as_array().expect("findings is array");
    assert_eq!(findings.len(), 8, "expected 8 findings (one per criterion)");

    // Find the management-review finding — the most-decorated attestation.
    let mr = findings
        .iter()
        .find(|f| f["criterion_id"] == "9001-management-review")
        .expect("management-review finding present");
    assert_eq!(mr["status"], "pass");
    assert_eq!(mr["evidence"]["kind"], "attestation");
    assert_eq!(mr["evidence"]["signer"], "Quality Manager: Tonkla K.");
    assert_eq!(mr["evidence"]["signed_at"], "2026-04-15");
    assert_eq!(mr["evidence"]["valid_through"], "2027-04-15");
    assert!(mr.get("remedy").is_none(), "pass must not carry remedy");

    // The leadership attestation has no valid_through — confirm the key
    // is dropped from the canonical envelope.
    let lp = findings
        .iter()
        .find(|f| f["criterion_id"] == "9001-leadership-and-policy")
        .expect("leadership-and-policy finding present");
    assert_eq!(lp["status"], "pass");
    assert_eq!(lp["evidence"]["kind"], "attestation");
    assert!(
        lp["evidence"]
            .as_object()
            .unwrap()
            .get("valid_through")
            .is_none(),
        "valid_through was omitted in workspace.toml — must not surface in envelope"
    );

    // A criterion without an attestation block must fail and point at workspace.toml.
    let irc = findings
        .iter()
        .find(|f| f["criterion_id"] == "9001-internal-audit")
        .expect("internal-audit finding present");
    assert_eq!(irc["status"], "fail");
    assert_eq!(irc["evidence"]["kind"], "file");
    assert_eq!(irc["evidence"]["value"], ".bwoc/workspace.toml");
    let remedy = irc["remedy"].as_str().expect("fail must carry remedy");
    assert!(
        remedy.contains("[[plugins.audit-iso-9001.attestations]]"),
        "remedy does not name the workspace.toml block: {remedy}"
    );
    assert!(
        remedy.contains("9001-internal-audit"),
        "remedy does not name the criterion: {remedy}"
    );
}

#[test]
fn audit_iso_9001_fails_all_when_no_attestations_present() {
    // Workspace with the plugin enabled but no attestations declared — every
    // criterion fails with the missing-attestation remedy. Confirms the
    // first-run "empty workspace.toml" state is honest about absent evidence
    // rather than silently passing or erroring out.
    let tmp = tempfile::tempdir().expect("create tempdir");
    let ws = tmp.path();

    std::fs::create_dir_all(ws.join(".bwoc")).expect("mkdir .bwoc");
    std::fs::write(
        ws.join(".bwoc/workspace.toml"),
        "[workspace]\nname = \"empty\"\nversion = \"0.1.0\"\n\n\
         [plugins.audit-iso-9001]\nenabled = true\n",
    )
    .expect("write workspace.toml");
    install_plugin(ws);

    let output = Command::new(bin())
        .args(["audit", "run", "--plugin", "audit-iso-9001", "--json"])
        .args(["--workspace"])
        .arg(ws)
        .output()
        .expect("spawn bwoc audit run");

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
}
