//! Tier 2 "deep memory" backend — pluggable seam.
//!
//! Tier 2 is entirely optional. When `deepMemoryCmd` is absent from the
//! manifest (or is the unconfigured placeholder `# (Tier 2 not configured)`),
//! the framework uses [`DisabledDeepMemory`], which is a harmless no-op that
//! prints a clear status message. The agent continues to work normally.
//!
//! ## The interface
//!
//! Any external tool that speaks the three sub-commands below can be
//! plugged in by setting `deepMemoryCmd` in `config.manifest.json`:
//!
//! ```text
//! <cmd> wake-up                  — session start: emit prior context to stdout
//! <cmd> search "<query>"        — find relevant past decisions/notes
//! <cmd> mine <path> --mode <m>  — persist session learnings at session end
//! ```
//!
//! ## Testability seam
//!
//! [`ShellDeepMemory`] accepts an injectable `runner` — a `fn(&str, &[&str])
//! -> Result<String, String>`. Tests pass a closure that records calls and
//! returns canned output **without ever spawning a real process**.

use std::path::Path;

/// Tier 2 deep-memory operations. Every implementation must be non-fatal
/// when Tier 2 is unavailable — callers check [`DeepMemoryStatus`] and
/// surface a user-visible note, but never hard-fail.
pub trait DeepMemory {
    /// Emit prior context at session start (`wake-up` sub-command).
    /// Returns the tool's stdout as a `String`.
    fn wake_up(&self) -> Result<String, DeepMemoryError>;

    /// Search past decisions/notes for `query` (`search "<query>"` sub-command).
    /// Returns the tool's stdout as a `String`.
    fn search(&self, query: &str) -> Result<String, DeepMemoryError>;

    /// Persist session learnings at session end (`mine <path> --mode <mode>`
    /// sub-command). `path` is the agent's sessions directory; `mode` is a
    /// tool-defined string (e.g. `"convos"`). Returns `Ok(())` on success.
    fn mine(&self, path: &Path, mode: &str) -> Result<(), DeepMemoryError>;

    /// Whether this backend is actually configured (i.e. not the disabled
    /// no-op). Callers use this to decide whether to surface "Tier 2 not
    /// configured" messages.
    fn status(&self) -> DeepMemoryStatus;
}

/// Outcome of [`DeepMemory::status`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeepMemoryStatus {
    /// A real command is configured and will be invoked.
    Configured { cmd: String },
    /// No `deepMemoryCmd` set (or is the placeholder). All operations are no-ops.
    Disabled,
}

/// Errors from Tier 2 operations.
#[derive(Debug, thiserror::Error)]
pub enum DeepMemoryError {
    /// The external tool exited non-zero.
    #[error("deep-memory command failed (exit {exit_code}): {stderr}")]
    ExitError { exit_code: i32, stderr: String },
    /// The command runner itself failed (e.g. binary not found).
    #[error("failed to invoke deep-memory command: {0}")]
    InvokeError(String),
    /// Tier 2 is disabled — this error is only surfaced if a caller
    /// explicitly calls an operation on the disabled backend directly.
    /// Normal usage should check `status()` first.
    #[error("Tier 2 deep memory is not configured (deepMemoryCmd is unset or placeholder)")]
    Disabled,
}

// ---------------------------------------------------------------------------
// Runner seam
// ---------------------------------------------------------------------------

/// Command runner type alias. Receives `(program, args)` and returns
/// `Ok(stdout)` or `Err(message)`. The default impl shells out via
/// `std::process::Command`; tests inject a closure instead.
pub type RunnerFn = fn(&str, &[&str]) -> Result<String, String>;

/// Default production runner — shells out via `std::process::Command`.
/// Captures stdout on success; maps non-zero exit and I/O errors to `Err`.
pub fn default_runner(program: &str, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("failed to spawn '{program}': {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let code = output.status.code().unwrap_or(-1);
        Err(format!("exit {code}: {stderr}"))
    }
}

// ---------------------------------------------------------------------------
// ShellDeepMemory — real shell-out implementation
// ---------------------------------------------------------------------------

/// Invokes `<cmd> wake-up`, `<cmd> search "<q>"`, and
/// `<cmd> mine <path> --mode <mode>` via the injectable `runner`.
///
/// Split on whitespace so `deepMemoryCmd` can be `"my-tool --flag"` as well
/// as a bare binary name. The first token becomes the program; the rest
/// become leading arguments prepended before the sub-command args.
pub struct ShellDeepMemory {
    /// Full command as configured in the manifest (e.g. `"mem-tool"` or
    /// `"npx mem-cli --store /data"`). Split on first whitespace run.
    pub cmd: String,
    /// Injectable runner. Use [`default_runner`] for production;
    /// pass a test closure to verify dispatch without spawning.
    pub runner: RunnerFn,
}

impl ShellDeepMemory {
    /// Construct with the production runner.
    pub fn new(cmd: impl Into<String>) -> Self {
        Self {
            cmd: cmd.into(),
            runner: default_runner,
        }
    }

    /// Construct with a custom runner (for tests).
    pub fn with_runner(cmd: impl Into<String>, runner: RunnerFn) -> Self {
        Self {
            cmd: cmd.into(),
            runner,
        }
    }

    /// Split `self.cmd` into `(program, prefix_args)`.
    fn split_cmd(&self) -> (String, Vec<String>) {
        let mut parts = self.cmd.split_whitespace();
        let program = parts.next().unwrap_or("").to_string();
        let prefix: Vec<String> = parts.map(str::to_string).collect();
        (program, prefix)
    }

    /// Build the full argument list and call the runner.
    fn invoke(&self, sub_args: &[&str]) -> Result<String, DeepMemoryError> {
        let (program, prefix) = self.split_cmd();
        if program.is_empty() {
            return Err(DeepMemoryError::InvokeError(
                "deepMemoryCmd is empty".to_string(),
            ));
        }
        // Build [prefix..., sub_args...]
        let prefix_refs: Vec<&str> = prefix.iter().map(String::as_str).collect();
        let mut all: Vec<&str> = Vec::with_capacity(prefix_refs.len() + sub_args.len());
        all.extend_from_slice(&prefix_refs);
        all.extend_from_slice(sub_args);

        (self.runner)(&program, &all).map_err(|e| {
            // Parse "exit N: <stderr>" from default_runner; otherwise treat
            // as an invoke error.
            if let Some(rest) = e.strip_prefix("exit ") {
                let mut iter = rest.splitn(2, ": ");
                let code = iter
                    .next()
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(-1);
                let stderr = iter.next().unwrap_or("").to_string();
                DeepMemoryError::ExitError {
                    exit_code: code,
                    stderr,
                }
            } else {
                DeepMemoryError::InvokeError(e)
            }
        })
    }
}

impl DeepMemory for ShellDeepMemory {
    fn wake_up(&self) -> Result<String, DeepMemoryError> {
        self.invoke(&["wake-up"])
    }

    fn search(&self, query: &str) -> Result<String, DeepMemoryError> {
        self.invoke(&["search", query])
    }

    fn mine(&self, path: &Path, mode: &str) -> Result<(), DeepMemoryError> {
        let path_str = path.to_string_lossy();
        self.invoke(&["mine", &path_str, "--mode", mode])?;
        Ok(())
    }

    fn status(&self) -> DeepMemoryStatus {
        DeepMemoryStatus::Configured {
            cmd: self.cmd.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// DisabledDeepMemory — no-op used when Tier 2 is not configured
// ---------------------------------------------------------------------------

/// No-op backend used when `deepMemoryCmd` is absent or the placeholder
/// `# (Tier 2 not configured)`. All methods return `Err(Disabled)` so
/// callers that check `status()` first never see this error in normal use.
pub struct DisabledDeepMemory;

impl DeepMemory for DisabledDeepMemory {
    fn wake_up(&self) -> Result<String, DeepMemoryError> {
        Err(DeepMemoryError::Disabled)
    }

    fn search(&self, _query: &str) -> Result<String, DeepMemoryError> {
        Err(DeepMemoryError::Disabled)
    }

    fn mine(&self, _path: &Path, _mode: &str) -> Result<(), DeepMemoryError> {
        Err(DeepMemoryError::Disabled)
    }

    fn status(&self) -> DeepMemoryStatus {
        DeepMemoryStatus::Disabled
    }
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

/// The placeholder value written by `bwoc new` when Tier 2 is not set.
pub const UNCONFIGURED_PLACEHOLDER: &str = "# (Tier 2 not configured)";

/// Build a boxed [`DeepMemory`] from the manifest's `deep_memory_cmd` field.
///
/// Returns [`DisabledDeepMemory`] when:
/// - `cmd` is `None`
/// - `cmd` is `Some("")` (empty string)
/// - `cmd` is the placeholder `# (Tier 2 not configured)`
///
/// Returns [`ShellDeepMemory`] (with the production runner) otherwise.
pub fn from_manifest_cmd(cmd: Option<&str>) -> Box<dyn DeepMemory> {
    match cmd {
        None | Some("") => Box::new(DisabledDeepMemory),
        Some(s) if s.trim() == UNCONFIGURED_PLACEHOLDER => Box::new(DisabledDeepMemory),
        Some(s) => Box::new(ShellDeepMemory::new(s)),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::path::PathBuf;
    use std::rc::Rc;

    // --- factory tests -------------------------------------------------------

    #[test]
    fn factory_none_returns_disabled() {
        let b = from_manifest_cmd(None);
        assert_eq!(b.status(), DeepMemoryStatus::Disabled);
    }

    #[test]
    fn factory_empty_returns_disabled() {
        let b = from_manifest_cmd(Some(""));
        assert_eq!(b.status(), DeepMemoryStatus::Disabled);
    }

    #[test]
    fn factory_placeholder_returns_disabled() {
        let b = from_manifest_cmd(Some(UNCONFIGURED_PLACEHOLDER));
        assert_eq!(b.status(), DeepMemoryStatus::Disabled);
    }

    #[test]
    fn factory_real_cmd_returns_configured() {
        let b = from_manifest_cmd(Some("my-mem-tool"));
        assert_eq!(
            b.status(),
            DeepMemoryStatus::Configured {
                cmd: "my-mem-tool".into()
            }
        );
    }

    // --- DisabledDeepMemory --------------------------------------------------

    #[test]
    fn disabled_wake_up_returns_disabled_error() {
        let b = DisabledDeepMemory;
        assert!(matches!(b.wake_up(), Err(DeepMemoryError::Disabled)));
    }

    #[test]
    fn disabled_search_returns_disabled_error() {
        let b = DisabledDeepMemory;
        assert!(matches!(
            b.search("anything"),
            Err(DeepMemoryError::Disabled)
        ));
    }

    #[test]
    fn disabled_mine_returns_disabled_error() {
        let b = DisabledDeepMemory;
        assert!(matches!(
            b.mine(&PathBuf::from("/tmp"), "convos"),
            Err(DeepMemoryError::Disabled)
        ));
    }

    // --- ShellDeepMemory dispatch --------------------------------------------

    // Shared call-log for injection. Using a static via thread_local so the
    // fn-pointer constraint is satisfied without capturing.
    thread_local! {
        static CALL_LOG: RefCell<Vec<(String, Vec<String>)>> = const { RefCell::new(Vec::new()) };
    }

    fn recording_runner(program: &str, args: &[&str]) -> Result<String, String> {
        CALL_LOG.with(|log| {
            log.borrow_mut().push((
                program.to_string(),
                args.iter().map(|s| s.to_string()).collect(),
            ));
        });
        Ok("stub output".into())
    }

    fn drain_log() -> Vec<(String, Vec<String>)> {
        CALL_LOG.with(|log| std::mem::take(&mut *log.borrow_mut()))
    }

    #[test]
    fn shell_wake_up_dispatches_correct_args() {
        let b = ShellDeepMemory::with_runner("mem-tool", recording_runner);
        let out = b.wake_up().unwrap();
        assert_eq!(out, "stub output");
        let calls = drain_log();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "mem-tool");
        assert_eq!(calls[0].1, vec!["wake-up"]);
    }

    #[test]
    fn shell_search_dispatches_correct_args() {
        let b = ShellDeepMemory::with_runner("mem-tool", recording_runner);
        b.search("prior art for caching").unwrap();
        let calls = drain_log();
        assert_eq!(calls[0].1, vec!["search", "prior art for caching"]);
    }

    #[test]
    fn shell_mine_dispatches_correct_args() {
        let b = ShellDeepMemory::with_runner("mem-tool", recording_runner);
        b.mine(&PathBuf::from("/tmp/sessions"), "convos").unwrap();
        let calls = drain_log();
        // Expected: ["mine", "/tmp/sessions", "--mode", "convos"]
        assert_eq!(calls[0].1[0], "mine");
        assert_eq!(calls[0].1[2], "--mode");
        assert_eq!(calls[0].1[3], "convos");
    }

    #[test]
    fn shell_cmd_with_prefix_args_splits_correctly() {
        // deepMemoryCmd = "npx mem-cli --store /data"
        let b = ShellDeepMemory::with_runner("npx mem-cli --store /data", recording_runner);
        b.wake_up().unwrap();
        let calls = drain_log();
        assert_eq!(calls[0].0, "npx");
        assert_eq!(calls[0].1, vec!["mem-cli", "--store", "/data", "wake-up"]);
    }

    #[test]
    fn shell_status_returns_configured_with_cmd() {
        let b = ShellDeepMemory::new("my-tool");
        assert_eq!(
            b.status(),
            DeepMemoryStatus::Configured {
                cmd: "my-tool".into()
            }
        );
    }

    #[test]
    fn shell_runner_exit_error_maps_to_exit_error_variant() {
        fn failing_runner(_program: &str, _args: &[&str]) -> Result<String, String> {
            Err("exit 1: something went wrong".into())
        }
        let b = ShellDeepMemory::with_runner("bad-tool", failing_runner);
        match b.wake_up() {
            Err(DeepMemoryError::ExitError { exit_code, stderr }) => {
                assert_eq!(exit_code, 1);
                assert_eq!(stderr, "something went wrong");
            }
            other => panic!("expected ExitError, got {other:?}"),
        }
    }

    #[test]
    fn shell_runner_invoke_error_maps_to_invoke_error_variant() {
        fn no_binary_runner(_program: &str, _args: &[&str]) -> Result<String, String> {
            Err("failed to spawn 'ghost-tool': No such file".into())
        }
        let b = ShellDeepMemory::with_runner("ghost-tool", no_binary_runner);
        assert!(matches!(b.wake_up(), Err(DeepMemoryError::InvokeError(_))));
    }

    // --- Rc-based injection (verifying the seam without thread_local) --------
    // This pattern is also valid for callers that want closure injection.
    // We test it here as documentation.

    #[test]
    fn injectable_closure_via_wrapper() {
        // When tests need a closure (not a fn pointer), wrap ShellDeepMemory
        // in a thin struct. This test verifies the pattern compiles + works.
        struct ClosureBackend<F: Fn(&str, &[&str]) -> Result<String, String>> {
            cmd: String,
            f: F,
        }
        impl<F: Fn(&str, &[&str]) -> Result<String, String>> DeepMemory for ClosureBackend<F> {
            fn wake_up(&self) -> Result<String, DeepMemoryError> {
                (self.f)(&self.cmd, &["wake-up"]).map_err(DeepMemoryError::InvokeError)
            }
            fn search(&self, q: &str) -> Result<String, DeepMemoryError> {
                (self.f)(&self.cmd, &["search", q]).map_err(DeepMemoryError::InvokeError)
            }
            fn mine(&self, _p: &Path, _m: &str) -> Result<(), DeepMemoryError> {
                Ok(())
            }
            fn status(&self) -> DeepMemoryStatus {
                DeepMemoryStatus::Configured {
                    cmd: self.cmd.clone(),
                }
            }
        }

        let captured = Rc::new(RefCell::new(Vec::<String>::new()));
        let cap2 = Rc::clone(&captured);
        let b = ClosureBackend {
            cmd: "echo-tool".into(),
            f: move |_prog: &str, args: &[&str]| {
                cap2.borrow_mut().push(args.join(" "));
                Ok("ok".into())
            },
        };
        b.wake_up().unwrap();
        b.search("test query").unwrap();
        assert_eq!(*captured.borrow(), vec!["wake-up", "search test query"]);
    }
}
