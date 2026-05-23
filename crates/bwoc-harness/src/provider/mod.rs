//! OpenAI-compatible provider client.
//!
//! Speaks `POST /v1/chat/completions` with `tools` + `tool_calls`.
//! Supports `stream=false` and `stream=true` (SSE token deltas).
//!
//! The [`ProviderClient`] trait makes the HTTP transport injectable so unit
//! tests do NOT require a live endpoint.  Any test that hits a real Ollama
//! endpoint must be `#[ignore]`d.

pub mod client;
pub mod types;

pub use client::{OllamaClient, ProviderClient};
pub use types::{
    ChatCompletion, ChatMessage, Choice, Delta, FinishReason, Function, FunctionCall, Role,
    StreamChunk, StreamDelta, Tool, ToolCall, ToolCallResult, Usage,
};
