//! Sandbox — OS-level execution confinement for tool calls.
//!
//! P2 component. Confines all tool effects to the agent worktree using
//! platform-native mechanisms:
//!   - macOS: `sandbox-exec` profile
//!   - Linux: landlock + seccomp
//!
//! Also wraps `run_command` with an env scrub (strip secrets from child env)
//! and an arg-scan deny list (`curl|sh`, `sudo`, force-push, etc.).
//!
//! The P1 path-confinement in `tools::ToolContext::resolve_path` is the
//! minimal baseline that keeps P1 from being wildly unsafe.  This module
//! adds the OS layer on top.
//!
//! TODO: P2 — implement platform sandbox, arg scan, env scrub.
