//! Daemon-side Kalyāṇamitta-7 refusal logic. Spec:
//! `modules/agent-template/interconnect/trust.md` §"Refusal Semantics".
//!
//! Behind the `BWOC_TRUST_GATING=1` env opt-in (v1 safety). When enabled
//! AND the recipient's manifest declares a non-empty `requiredTrust`,
//! the daemon resolves each new inbox envelope's sender, reads the
//! sender's `trust.declared`, and writes a refusal record to
//! `<agent>/.bwoc/inbox.refusals.jsonl` if any required quality is
//! missing. The original envelope in `inbox.jsonl` is NEVER deleted —
//! auditability matters. `bwoc inbox` joins the two files at read time
//! so `select(.refused)` works against the resulting JSON.

use std::path::{Path, PathBuf};

use bwoc_core::manifest::Manifest;
use bwoc_core::workspace::AgentsRegistry;

/// Daemon trust posture, built once at `--serve` startup.
pub struct TrustContext {
    /// Recipient's `requiredTrust` list (own manifest). Empty ≡ no gating
    /// regardless of env opt-in.
    pub required: Vec<String>,
    /// Walked-up workspace root holding `.bwoc/agents.toml`. `None` ≡
    /// daemon is running outside a workspace; sender lookup is impossible
    /// so gating refuses every non-`user` envelope when on.
    pub workspace_root: Option<PathBuf>,
    /// Reflects `BWOC_TRUST_GATING=1`. When false, `evaluate` always
    /// returns `None` (permissive).
    pub gating_enabled: bool,
}

impl TrustContext {
    /// Build from the recipient's own manifest + cwd. Reads env at call
    /// time so daemon can be relaunched with new env without code change.
    pub fn build(own: &Manifest, cwd: &Path) -> Self {
        let required: Vec<String> = own
            .trust
            .as_ref()
            .map(|t| t.required_trust.clone())
            .unwrap_or_default();
        let gating_enabled = std::env::var("BWOC_TRUST_GATING").ok().as_deref() == Some("1");
        let workspace_root = find_workspace_root(cwd);
        Self {
            required,
            workspace_root,
            gating_enabled,
        }
    }

    /// Returns `true` if this context will never refuse anything (gating
    /// off or no requirements declared). Lets the daemon skip per-envelope
    /// JSON parsing when there's nothing to check.
    pub fn is_inert(&self) -> bool {
        !self.gating_enabled || self.required.is_empty()
    }
}

/// A refusal verdict ready to be serialized into
/// `.bwoc/inbox.refusals.jsonl`. Kept generic over time-source so the
/// caller stamps `ts` from a single helper (matches the JSONL envelope
/// timestamps that `bwoc send` writes).
pub struct Refusal {
    pub envelope_offset: u64,
    pub envelope_ts: String,
    pub envelope_from: String,
    pub reason: &'static str,
    pub missing: Vec<String>,
}

impl Refusal {
    /// Render as a single JSONL line including the supplied refusal timestamp.
    pub fn to_jsonl(&self, ts: &str) -> String {
        let value = serde_json::json!({
            "ts": ts,
            "envelopeOffset": self.envelope_offset,
            "envelopeTs": self.envelope_ts,
            "envelopeFrom": self.envelope_from,
            "reason": self.reason,
            "missing": self.missing,
        });
        serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Evaluate a single inbox envelope against the daemon's trust posture.
/// Returns `Some(Refusal)` when gating is on AND the sender fails at
/// least one required quality. Returns `None` for the permissive paths:
/// gating off, no requirements, `from=user`, or sender satisfies all.
///
/// `envelope_offset` is the byte offset of the envelope's line within
/// `inbox.jsonl` — it's the join key `bwoc inbox` uses when overlaying
/// refusals onto the envelope view.
pub fn evaluate(ctx: &TrustContext, envelope_line: &str, envelope_offset: u64) -> Option<Refusal> {
    if ctx.is_inert() {
        return None;
    }
    let env: serde_json::Value = serde_json::from_str(envelope_line).ok()?;
    let from = env
        .get("from")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if from == "user" {
        // Spec: user-originated messages always pass.
        return None;
    }
    let envelope_ts = env
        .get("ts")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Without a workspace, we can't look up the sender's manifest.
    // Gating is on AND we can't verify → refuse, naming the reason.
    let Some(ws) = ctx.workspace_root.as_ref() else {
        return Some(Refusal {
            envelope_offset,
            envelope_ts,
            envelope_from: from,
            reason: "no_workspace",
            missing: ctx.required.clone(),
        });
    };

    let registry = match AgentsRegistry::load(ws) {
        Ok(r) => r,
        Err(_) => {
            return Some(Refusal {
                envelope_offset,
                envelope_ts,
                envelope_from: from,
                reason: "registry_unreadable",
                missing: ctx.required.clone(),
            });
        }
    };

    let Some(entry) = registry.agents.iter().find(|a| a.id == from) else {
        return Some(Refusal {
            envelope_offset,
            envelope_ts,
            envelope_from: from,
            reason: "unknown_sender",
            missing: ctx.required.clone(),
        });
    };

    let manifest_path = ws.join(&entry.path).join("config.manifest.json");
    let declared = match Manifest::load_from_path(&manifest_path) {
        Ok(m) => m.trust.map(|t| t.declared).unwrap_or_default(),
        Err(_) => {
            return Some(Refusal {
                envelope_offset,
                envelope_ts,
                envelope_from: from,
                reason: "sender_manifest_unreadable",
                missing: ctx.required.clone(),
            });
        }
    };

    let missing: Vec<String> = ctx
        .required
        .iter()
        .filter(|q| !declared.has(q))
        .cloned()
        .collect();
    if missing.is_empty() {
        return None;
    }
    Some(Refusal {
        envelope_offset,
        envelope_ts,
        envelope_from: from,
        reason: "missing_trust",
        missing,
    })
}

/// Walk up from `start` looking for `.bwoc/workspace.toml`. Same chain as
/// the CLI's `resolve_workspace` minus the explicit-path / env arms.
fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    let mut cur = start.to_path_buf();
    loop {
        if cur.join(".bwoc/workspace.toml").is_file() {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bwoc_core::manifest::{TrustBlock, TrustDeclared};
    use std::sync::Mutex;

    /// Serializes env-mutating tests. Rust 2024 marks `set_var/remove_var`
    /// as `unsafe` because env is process-wide — two parallel tests on the
    /// same var race. The first run after promoting this module hit that
    /// (1/15 flake on `build_reads_required_from_manifest`). The mutex
    /// pins env-touching tests to one-at-a-time without forcing
    /// `--test-threads=1` globally.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn ctx_with(required: Vec<String>, gating: bool, ws: Option<PathBuf>) -> TrustContext {
        TrustContext {
            required,
            workspace_root: ws,
            gating_enabled: gating,
        }
    }

    #[test]
    fn evaluate_passes_when_gating_off() {
        let ctx = ctx_with(vec!["vatta".into()], false, None);
        let line = r#"{"ts":"t","from":"agent-x","to":"agent-me","message":"hi"}"#;
        assert!(evaluate(&ctx, line, 0).is_none());
    }

    #[test]
    fn evaluate_passes_when_required_empty() {
        let ctx = ctx_with(vec![], true, None);
        let line = r#"{"ts":"t","from":"agent-x","to":"agent-me","message":"hi"}"#;
        assert!(evaluate(&ctx, line, 0).is_none());
    }

    #[test]
    fn evaluate_passes_for_user_origin() {
        let ctx = ctx_with(vec!["vatta".into()], true, None);
        let line = r#"{"ts":"t","from":"user","to":"agent-me","message":"hi"}"#;
        assert!(evaluate(&ctx, line, 0).is_none());
    }

    #[test]
    fn evaluate_refuses_no_workspace_for_agent_sender() {
        let ctx = ctx_with(vec!["vatta".into()], true, None);
        let line = r#"{"ts":"t","from":"agent-x","to":"agent-me","message":"hi"}"#;
        let r = evaluate(&ctx, line, 42).expect("should refuse");
        assert_eq!(r.reason, "no_workspace");
        assert_eq!(r.envelope_offset, 42);
        assert_eq!(r.envelope_from, "agent-x");
        assert_eq!(r.missing, vec!["vatta"]);
    }

    #[test]
    fn evaluate_silently_skips_malformed_envelope() {
        let ctx = ctx_with(vec!["vatta".into()], true, None);
        assert!(evaluate(&ctx, "{not json}", 0).is_none());
    }

    #[test]
    fn refusal_jsonl_shape_is_camel_case() {
        let r = Refusal {
            envelope_offset: 7,
            envelope_ts: "2026-05-23T00:00:00Z".into(),
            envelope_from: "agent-x".into(),
            reason: "missing_trust",
            missing: vec!["vatta".into()],
        };
        let line = r.to_jsonl("2026-05-23T00:00:01Z");
        assert!(line.contains("\"envelopeOffset\":7"));
        assert!(line.contains("\"envelopeTs\":\"2026-05-23T00:00:00Z\""));
        assert!(line.contains("\"envelopeFrom\":\"agent-x\""));
        assert!(line.contains("\"reason\":\"missing_trust\""));
        assert!(line.contains("\"missing\":[\"vatta\"]"));
    }

    #[test]
    fn build_reads_required_from_manifest() {
        let _guard = ENV_LOCK.lock().unwrap();
        let mut m = sample_manifest();
        m.trust = Some(TrustBlock {
            schema_version: 1,
            declared: TrustDeclared::default(),
            required_trust: vec!["vatta".into(), "noCatthana".into()],
        });
        unsafe {
            std::env::remove_var("BWOC_TRUST_GATING");
        }
        let ctx = TrustContext::build(&m, Path::new("/nonexistent-anywhere"));
        assert_eq!(ctx.required, vec!["vatta", "noCatthana"]);
        assert!(!ctx.gating_enabled);
        assert!(ctx.is_inert());
    }

    #[test]
    fn build_empty_required_when_no_trust_block() {
        let _guard = ENV_LOCK.lock().unwrap();
        let m = sample_manifest();
        unsafe {
            std::env::set_var("BWOC_TRUST_GATING", "1");
        }
        let ctx = TrustContext::build(&m, Path::new("/nonexistent-anywhere"));
        assert!(ctx.required.is_empty());
        // Even with gating env on, empty required ≡ inert.
        assert!(ctx.is_inert());
        unsafe {
            std::env::remove_var("BWOC_TRUST_GATING");
        }
    }

    fn sample_manifest() -> Manifest {
        Manifest {
            name: "me".into(),
            agent_id: "agent-me".into(),
            agent_role: "demo".into(),
            primary_model: "m".into(),
            fallback_model: None,
            memory_path: "memories/".into(),
            sessions_path: None,
            deep_memory_cmd: None,
            lint_cmd: "true".into(),
            format_cmd: "true".into(),
            test_cmd: "true".into(),
            build_cmd: "true".into(),
            worktree_base: None,
            scope_description: None,
            out_of_scope: None,
            trust: None,
            version: "2.0".into(),
        }
    }
}
