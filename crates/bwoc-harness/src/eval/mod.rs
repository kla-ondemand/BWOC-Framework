//! Agent eval framework — offline task fixtures + scoring.
//!
//! P4 component. Runs task fixtures (repo-state snapshot + task prompt +
//! rubric) through the harness and scores the result:
//!   - Did all verification gates pass?
//!   - Does the output diff match the expected diff?
//!   - Optional LLM-judge scoring for open-ended tasks.
//!
//! Feeds the Paññā 3 self-improvement triggers wired to `session-metrics.jsonl`
//! (see AGENTS.md §8b and §11): if completion rate < 70% or gate pass rate
//! < 70% after 5+ sessions, the eval framework surfaces the root cause for
//! retrospective.
//!
//! TODO: P4 — implement fixture format, runner, gate scorer, LLM-judge integration.
