//! Task queue — async, bounded, cancellable work queue.
//!
//! P3 component. Pulls claimable tasks from the Saṅgha shared task list
//! (`bwoc-core::team`) and from local submissions.  Enforces one in-flight
//! task per worktree.  Feeds the agentic loop via [`agent_loop::run_loop`].
//!
//! Also wires `bwoc_task` (claim/complete) and `bwoc_send` (inter-agent
//! messaging) as tools once the queue is running.
//!
//! TODO: P3 — implement bounded queue, Saṅgha integration, cancel signal.
