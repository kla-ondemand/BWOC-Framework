//! `bwoc-harness` — BWOC self-hosted agentic harness.
//!
//! Provides an OpenAI-compatible agentic loop that lets self-hosted models
//! (Ollama, vLLM, LM Studio, llama.cpp server) act as full BWOC coding agents.
//!
//! # Phasing
//! - **P1 (this crate):** core loop + tool set + streaming, single task,
//!   inside a worktree.  Dev-only (no safety guardrails).
//! - **P2:** safety — guardrails + sandbox + permission system.
//! - **P3:** task queue + telemetry + tool authentication.
//! - **P4:** eval framework + hardening.
//! - **P5:** backend wiring (`bwoc spawn --backend ollama`).
//!
//! # Architecture
//! See `notes/2026-05-23_ollama-agentic-harness-design.md` for the full
//! design rationale and phasing decisions.

pub mod agent_loop;
pub mod error;
pub mod eval;
pub mod policy;
pub mod provider;
pub mod queue;
pub mod sandbox;
pub mod telemetry;
pub mod tools;
