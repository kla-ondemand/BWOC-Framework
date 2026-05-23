//! Policy / permission system and safety guardrails.
//!
//! Three-layer safety pipeline.  Every tool call passes through these layers
//! **before** sandbox execution, in the order below:
//!
//! ```text
//! GUARDRAILS → PERMISSION → SANDBOX → execute
//! ```
//!
//! ## Layer 1 — Guardrails (`guardrails`)
//!
//! Hard policy engine grounded in Sīla 5 + Taṇhā 3.  Runs first, always.
//! Cannot be overridden by permission config or any operator action.
//! Blocks: `rm -rf` of repo/worktree root; secret writes; identity spoof;
//! gate-bypass flags (`--no-verify`, `--force`/`-f` on push); privilege
//! escalation (`sudo`, `su`, `doas`).
//!
//! Returns [`guardrails::GuardrailViolation`] on a hit.  The violation is
//! fed back to the model as the tool result.
//!
//! ## Layer 2 — Permission (`permission`)
//!
//! Per-tool / per-pattern `allow | ask | deny` modes loaded from
//! `config.manifest.json` and `.bwoc/harness-policy.toml`.  `ask` prompts
//! the operator on TTY; in non-TTY / autonomous mode it falls back to the
//! policy default (deny).  Denials are fed back to the model as tool results.
//!
//! ## Layer 3 — Sandbox (`crate::sandbox`)
//!
//! Confines all tool effects to the agent's worktree: filesystem write
//! allowlist; `run_command` with env scrub and arg-level scan; OS-level
//! sandbox stub (macOS / Linux pluggable trait, v1 is worktree+allowlist).

pub mod guardrails;
pub mod permission;

pub use guardrails::{GuardrailViolation, check as guardrail_check};
pub use permission::{
    HarnessPolicy, Mode, PermissionDecision, Policy, evaluate as permission_evaluate,
};

/// The outcome of the full policy pipeline (guardrails + permission).
///
/// Used by `agent_loop` to decide whether to proceed to the sandbox and
/// execute the tool, or to return a denial message to the model.
#[derive(Debug, Clone)]
pub enum PolicyOutcome {
    /// All policy layers approved; proceed to sandbox then execute.
    Proceed,
    /// A guardrail rule fired.  The model receives this as the tool result.
    GuardrailBlocked(GuardrailViolation),
    /// The permission layer denied the call.  The model receives this as the
    /// tool result so it can adapt (e.g., try a different approach).
    PermissionDenied(String),
}

impl PolicyOutcome {
    /// Convert to the string that will be fed back to the model as the
    /// tool result when the call is blocked.
    pub fn into_tool_result(self) -> Option<String> {
        match self {
            PolicyOutcome::Proceed => None,
            PolicyOutcome::GuardrailBlocked(v) => Some(format!(
                "BLOCKED by safety guardrail [{rule}]: {reason}",
                rule = v.rule,
                reason = v.reason,
            )),
            PolicyOutcome::PermissionDenied(reason) => {
                Some(format!("DENIED by permission policy: {reason}"))
            }
        }
    }
}

/// Run the full policy pipeline for one tool call.
///
/// # Arguments
/// - `tool_name`      — the tool being called
/// - `arguments_json` — the raw JSON argument string from the model
/// - `worktree_root`  — absolute path of the agent's worktree root
/// - `policy`         — the loaded permission policy
/// - `is_tty`         — whether a controlling TTY is available for `ask` prompts
///
/// # Returns
/// [`PolicyOutcome::Proceed`] if all layers approve, or a blocking variant
/// that the caller should surface as the tool result.
pub fn run_pipeline(
    tool_name: &str,
    arguments_json: &str,
    worktree_root: &std::path::Path,
    policy: &Policy,
    is_tty: bool,
) -> PolicyOutcome {
    // Layer 1: Guardrails (non-overridable, always runs first).
    if let Err(violation) = guardrail_check(tool_name, arguments_json, worktree_root) {
        return PolicyOutcome::GuardrailBlocked(violation);
    }

    // Layer 2: Permission.
    match permission_evaluate(policy, tool_name, arguments_json, is_tty) {
        PermissionDecision::Allow => PolicyOutcome::Proceed,
        PermissionDecision::Deny { reason } => PolicyOutcome::PermissionDenied(reason),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn allow_policy() -> Policy {
        Policy {
            default_mode: Mode::Allow,
            tools: std::collections::HashMap::new(),
            patterns: Vec::new(),
        }
    }

    fn deny_policy() -> Policy {
        Policy {
            default_mode: Mode::Deny,
            tools: std::collections::HashMap::new(),
            patterns: Vec::new(),
        }
    }

    fn wt() -> &'static Path {
        Path::new("/tmp/agent-oracle/test-worktree")
    }

    // ── Guardrails fire before permission ────────────────────────────────────

    #[test]
    fn guardrail_blocks_before_permission_allow() {
        // Even with allow_policy, a guardrail violation must block.
        let policy = allow_policy();
        let outcome = run_pipeline(
            "run_command",
            r#"{"command": "rm -rf /"}"#,
            wt(),
            &policy,
            false,
        );
        assert!(matches!(outcome, PolicyOutcome::GuardrailBlocked(_)));
    }

    #[test]
    fn guardrail_blocks_no_verify_before_permission() {
        let policy = allow_policy();
        let outcome = run_pipeline(
            "run_command",
            r#"{"command": "git commit --no-verify -m 'skip'"}"#,
            wt(),
            &policy,
            false,
        );
        assert!(matches!(outcome, PolicyOutcome::GuardrailBlocked(_)));
    }

    // ── Permission deny after guardrails pass ────────────────────────────────

    #[test]
    fn permission_deny_blocks_safe_command() {
        let policy = deny_policy();
        let outcome = run_pipeline(
            "run_command",
            r#"{"command": "echo hello"}"#,
            wt(),
            &policy,
            false,
        );
        assert!(matches!(outcome, PolicyOutcome::PermissionDenied(_)));
    }

    // ── Proceed when both layers pass ────────────────────────────────────────

    #[test]
    fn proceed_when_all_layers_pass() {
        let policy = allow_policy();
        let outcome = run_pipeline(
            "read_file",
            r#"{"path": "README.md"}"#,
            wt(),
            &policy,
            false,
        );
        assert!(matches!(outcome, PolicyOutcome::Proceed));
    }

    // ── into_tool_result ─────────────────────────────────────────────────────

    #[test]
    fn into_tool_result_proceed_is_none() {
        assert!(PolicyOutcome::Proceed.into_tool_result().is_none());
    }

    #[test]
    fn into_tool_result_guardrail_blocked_contains_rule() {
        let v = GuardrailViolation {
            rule: "sila_panatatipata",
            reason: "test".to_string(),
        };
        let msg = PolicyOutcome::GuardrailBlocked(v)
            .into_tool_result()
            .unwrap();
        assert!(msg.contains("sila_panatatipata"));
        assert!(msg.contains("BLOCKED by safety guardrail"));
    }

    #[test]
    fn into_tool_result_permission_denied_contains_reason() {
        let msg = PolicyOutcome::PermissionDenied("operator said no".to_string())
            .into_tool_result()
            .unwrap();
        assert!(msg.contains("DENIED by permission policy"));
        assert!(msg.contains("operator said no"));
    }

    // ── Denial is NOT a hard error — it is a tool result ────────────────────
    // (This is a documentation-as-test: the outcome never panics)

    #[test]
    fn denial_does_not_panic() {
        let policy = deny_policy();
        // This must return a PolicyOutcome, not panic or return Err.
        let outcome = run_pipeline(
            "run_command",
            r#"{"command": "sudo rm -rf /"}"#,
            wt(),
            &policy,
            false,
        );
        // Guardrail fires first (sudo → bhava_tanha_escalation).
        assert!(matches!(outcome, PolicyOutcome::GuardrailBlocked(_)));
    }
}
