//! End-to-end smoke for the `audit-iso-27001` runtime (BWOC-34) — the last
//! EPIC-3 ISMS runtime.
//!
//! Verifies the round-trip `bwoc audit run --plugin audit-iso-27001 --json`
//! against a tempdir workspace that declares attestations, a Statement of
//! Applicability (`[[…soa]]`), and Annex A samples. 27001 is the only runtime
//! whose sampling population is operator-declared: the SoA's in-scope set
//! (`applicable = true`) is the population `M`, and the count of this plugin's
//! Annex A controls in scope is `K` (`sampled_count`). This test confirms the
//! dispatcher accepts the BWOC-27 `attestation` + `sample` kinds AND the
//! `not_applicable` status end-to-end, that `K` is computed by SCOPE (an
//! in-scope control missing its sample fails without deflating a sibling's
//! `K`), and that `not_applicable` does not count toward the fail exit code.
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
        .join("../../modules/plugins/audit-iso-27001")
        .canonicalize()
        .expect("canonicalize audit-iso-27001 plugin source");
    let dst = workspace.join("modules/plugins/audit-iso-27001");
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
        .args(["audit", "run", "--plugin", "audit-iso-27001", "--json"])
        .args(["--workspace"])
        .arg(ws)
        .output()
        .expect("spawn bwoc audit run")
}

fn write_workspace(ws: &Path, toml: &str) {
    std::fs::create_dir_all(ws.join(".bwoc")).expect("mkdir .bwoc");
    std::fs::write(ws.join(".bwoc/workspace.toml"), toml).expect("write workspace.toml");
    install_plugin(ws);
}

/// Mixed-evidence workspace: two attestations (one with `valid_through`, one
/// without); a Statement of Applicability with two in-scope plugin controls
/// (A.5.15, A.5.24), one justifiably-excluded plugin control (A.5.29), and one
/// in-scope NON-plugin control (A.8.5 — counts toward `M` but is never sampled
/// by this plugin); and one Annex A sample (for A.5.15 only).
///
/// Derived population: M = 3 (A.5.15, A.5.24, A.8.5 are applicable=true);
/// K = 2 (A.5.15, A.5.24 are this plugin's in-scope Annex A controls — A.5.24
/// counts toward K *by scope* even though it has no sample and will fail).
const WORKSPACE_TOML: &str = r#"[workspace]
name = "bwoc-34-smoke"
version = "0.1.0"

[plugins.audit-iso-27001]
enabled = true

[[plugins.audit-iso-27001.attestations]]
criterion_id  = "27001-isms-scope"
statement     = "ISMS scope covers the hosted API platform and its support services."
signer        = "CISO: Tonkla K."
signed_at     = "2026-01-15"
valid_through = "2027-01-15"

[[plugins.audit-iso-27001.attestations]]
criterion_id = "27001-information-security-policy"
statement    = "Information security policy v1 ratified 2026-01-15."
signer       = "CISO: Tonkla K."
signed_at    = "2026-01-15"

[[plugins.audit-iso-27001.soa]]
control       = "A.5.15"
applicable    = true
justification = "Access control is central to protecting source, credentials, and customer data."

[[plugins.audit-iso-27001.soa]]
control       = "A.5.24"
applicable    = true
justification = "Incident management readiness is required for the hosted platform."

[[plugins.audit-iso-27001.soa]]
control       = "A.5.29"
applicable    = false
justification = "No formal continuity programme; risk accepted by management for a solo workspace."

[[plugins.audit-iso-27001.soa]]
control       = "A.8.5"
applicable    = true
justification = "Secure authentication is enforced on all in-scope systems."

[[plugins.audit-iso-27001.samples]]
criterion_id = "27001-access-control"
summary      = "Access reviews completed across in-scope systems; 0 orphaned accounts found."
window       = "2026-Q1"
"#;

#[test]
fn audit_iso_27001_emits_attestation_sample_not_applicable_and_fail_findings() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let ws = tmp.path();
    write_workspace(ws, WORKSPACE_TOML);

    let output = run_audit(ws);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // 8 criteria. pass: isms-scope, info-sec-policy, access-control = 3.
    // fail: risk-assessment, statement-of-applicability, incident-management
    // (in scope but no sample), internal-audit = 4. not_applicable:
    // business-continuity (justified exclusion) = 1. Exit = fail count = 4
    // (not_applicable does NOT count as a fail).
    assert!(
        !stderr.contains("framework error") && !stderr.contains("not in closed enum"),
        "dispatcher rejected a finding shape:\n{stderr}\n\nstdout:\n{stdout}"
    );
    assert_eq!(
        output.status.code(),
        Some(4),
        "expected exit code 4 (fails only; not_applicable excluded); stdout=\n{stdout}\nstderr=\n{stderr}"
    );

    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("parse --json envelope");

    let summary = &envelope["summary"];
    assert_eq!(summary["pass_count"], 3);
    assert_eq!(summary["fail_count"], 4);
    assert_eq!(summary["not_applicable_count"], 1);
    assert_eq!(summary["framework_error"], false);

    let runs = envelope["runs"].as_array().expect("runs is array");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0]["plugin"], "audit-iso-27001");
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

    // Attestation (with valid_through) — ISMS scope.
    let scope = find("27001-isms-scope");
    assert_eq!(scope["status"], "pass");
    assert_eq!(scope["evidence"]["kind"], "attestation");
    assert_eq!(scope["evidence"]["signer"], "CISO: Tonkla K.");
    assert_eq!(scope["evidence"]["signed_at"], "2026-01-15");
    assert_eq!(scope["evidence"]["valid_through"], "2027-01-15");

    // Attestation (no valid_through) — InfoSec policy. valid_through dropped.
    let policy = find("27001-information-security-policy");
    assert_eq!(policy["status"], "pass");
    assert_eq!(policy["evidence"]["kind"], "attestation");
    assert!(
        policy["evidence"]
            .as_object()
            .unwrap()
            .get("valid_through")
            .is_none(),
        "valid_through omitted in workspace.toml — must not surface in envelope"
    );

    // SoA-driven sample — access control (A.5.15). sampled_count = K = 2
    // (A.5.15 + A.5.24 in scope), sampled_of = M = 3 (… + A.8.5). The operator
    // never typed these numbers; they are SoA-derived. A.5.24 contributes to K
    // by SCOPE even though it has no sample and fails below — findings stay
    // independent.
    let access = find("27001-access-control");
    assert_eq!(access["status"], "pass");
    assert_eq!(access["evidence"]["kind"], "sample");
    assert_eq!(
        access["evidence"]["value"],
        "Access reviews completed across in-scope systems; 0 orphaned accounts found."
    );
    assert_eq!(
        access["evidence"]["sampled_count"], 2,
        "K = this plugin's in-scope Annex A controls (A.5.15 + A.5.24), by scope"
    );
    assert_eq!(
        access["evidence"]["sampled_of"], 3,
        "M = all SoA controls applicable=true (A.5.15 + A.5.24 + A.8.5)"
    );
    assert_eq!(access["evidence"]["window"], "2026-Q1");
    assert!(access.get("remedy").is_none(), "pass must not carry remedy");

    // not_applicable — business continuity (A.5.29 excluded + justified).
    // evidence.kind = none, value empty, remedy required and echoes the
    // justification.
    let continuity = find("27001-business-continuity");
    assert_eq!(continuity["status"], "not_applicable");
    assert_eq!(continuity["evidence"]["kind"], "none");
    assert_eq!(continuity["evidence"]["value"], "");
    let cont_remedy = continuity["remedy"]
        .as_str()
        .expect("not_applicable carries remedy");
    assert!(
        cont_remedy.contains("A.5.29") && cont_remedy.contains("solo workspace"),
        "not_applicable remedy must echo the SoA justification: {cont_remedy}"
    );

    // In-scope Annex A control with NO recorded sample — incident management
    // (A.5.24). Fails pointing at workspace.toml; remedy names the samples block.
    let incident = find("27001-incident-management");
    assert_eq!(incident["status"], "fail");
    assert_eq!(incident["evidence"]["kind"], "file");
    assert_eq!(incident["evidence"]["value"], ".bwoc/workspace.toml");
    let inc_remedy = incident["remedy"].as_str().expect("fail carries remedy");
    assert!(
        inc_remedy.contains("[[plugins.audit-iso-27001.samples]]")
            && inc_remedy.contains("27001-incident-management"),
        "in-scope-without-sample remedy must name the samples block and criterion: {inc_remedy}"
    );

    // Missing attestation — risk assessment. Fail pointing at workspace.toml.
    let risk = find("27001-risk-assessment");
    assert_eq!(risk["status"], "fail");
    assert_eq!(risk["evidence"]["kind"], "file");
    let risk_remedy = risk["remedy"].as_str().expect("fail carries remedy");
    assert!(
        risk_remedy.contains("[[plugins.audit-iso-27001.attestations]]")
            && risk_remedy.contains("27001-risk-assessment"),
        "attestation remedy must name the block and criterion: {risk_remedy}"
    );
}

#[test]
fn audit_iso_27001_fails_all_when_no_evidence_present() {
    // Plugin enabled but no attestations, no SoA, no samples — every criterion
    // fails with its missing-evidence remedy (attestations absent; Annex A
    // controls absent from the SoA). Confirms the first-run state is honest
    // about absent evidence rather than silently passing or erroring.
    let tmp = tempfile::tempdir().expect("create tempdir");
    let ws = tmp.path();
    write_workspace(
        ws,
        "[workspace]\nname = \"empty\"\nversion = \"0.1.0\"\n\n\
         [plugins.audit-iso-27001]\nenabled = true\n",
    );

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
    assert_eq!(envelope["summary"]["not_applicable_count"], 0);
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
fn audit_iso_27001_unjustified_soa_exclusion_fails_not_not_applicable() {
    // An Annex A control marked applicable=false WITHOUT a justification is an
    // incomplete SoA entry, not a valid exclusion: 6.1.3 requires a
    // justification for exclusions too. It must `fail` (pointing at
    // workspace.toml), NOT silently emit `not_applicable`. Guards the Musāvāda
    // line — an unexplained exclusion is not a defensible scoping decision.
    let tmp = tempfile::tempdir().expect("create tempdir");
    let ws = tmp.path();
    write_workspace(
        ws,
        "[workspace]\nname = \"bad-soa\"\nversion = \"0.1.0\"\n\n\
         [plugins.audit-iso-27001]\nenabled = true\n\n\
         [[plugins.audit-iso-27001.soa]]\n\
         control    = \"A.5.29\"\n\
         applicable = false\n",
    );

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
    let continuity = findings
        .iter()
        .find(|f| f["criterion_id"] == "27001-business-continuity")
        .expect("business-continuity finding present");
    assert_eq!(
        continuity["status"], "fail",
        "unjustified exclusion must fail, not be not_applicable"
    );
    assert_eq!(continuity["evidence"]["kind"], "file");
    let remedy = continuity["remedy"].as_str().expect("fail carries remedy");
    assert!(
        remedy.contains("justification") && remedy.contains("A.5.29"),
        "remedy should require a justification for the exclusion: {remedy}"
    );
}
