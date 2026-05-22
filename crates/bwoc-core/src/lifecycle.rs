//! Agent lifecycle phases — the BWOC arc.
//!
//! Named after AN 3.47 (Saṅkhata Sutta). See
//! `modules/agent-template/docs/en/PHILOSOPHY.en.md` §0.1.

/// The three phases of an agent (or any of its tasks, sessions, worktrees).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LifecyclePhase {
    /// uppāda — arising. Identity created, manifest resolved.
    Uppada,
    /// ṭhiti — persisting-with-change. Operation under discipline.
    Thiti,
    /// vaya — passing-away. Cleanup; no clinging (anattā).
    Vaya,
}
