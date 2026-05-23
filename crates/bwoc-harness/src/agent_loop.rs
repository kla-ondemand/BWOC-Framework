//! The core agentic turn loop.
//!
//! Each turn:
//!   1. Build messages (system prompt + history + tool schemas).
//!   2. Call the provider (`stream=false` first; streaming path also present).
//!   3. Accumulate `tool_calls` from the response.
//!   4. Dispatch each tool call → capture result.
//!   5. Append `assistant(tool_calls)` + `tool` result messages to history.
//!   6. Repeat.
//!
//! Stop conditions:
//!   - No `tool_calls` in the response (model returned final answer).
//!   - Reached `max_iterations`.
//!   - External cancel signal (future P3 queue integration — stub for now).
//!
//! Context compaction (summarise / truncate) is P4 — not implemented here.

use std::sync::Arc;

use crate::error::{HarnessError, HarnessResult};
use crate::provider::{ChatMessage, ProviderClient, ToolCall};
use crate::tools::registry::dispatch;
use crate::tools::{ToolContext, ToolRegistry};

/// Configuration for a single agent run.
#[derive(Debug, Clone)]
pub struct LoopConfig {
    /// Model identifier (e.g. `"gemma4"`, `"qwen2.5-coder:7b"`).
    pub model: String,
    /// Maximum number of turns before giving up.
    pub max_iterations: u32,
    /// Whether to use streaming mode (SSE) for token deltas.
    /// `false` = use the blocking complete() path (simpler, spike-proven).
    pub stream: bool,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            model: "gemma4".to_string(),
            max_iterations: 20,
            stream: false,
        }
    }
}

/// Result of a completed agent run.
#[derive(Debug)]
pub struct LoopResult {
    /// Final text response from the model (content of the last assistant message).
    pub final_response: String,
    /// Number of turns taken.
    pub turns: u32,
    /// All messages exchanged (for debug / memory purposes).
    pub history: Vec<ChatMessage>,
}

/// Run the agentic loop.
///
/// # Arguments
/// - `provider` — injectable provider client (real or mock).
/// - `registry` — tool registry.
/// - `ctx` — working directory context for tool execution.
/// - `config` — loop configuration.
/// - `system_prompt` — the agent's system prompt (loaded from `AGENTS.md`).
/// - `initial_messages` — the first user message(s).
pub async fn run_loop(
    provider: Arc<dyn ProviderClient>,
    registry: Arc<ToolRegistry>,
    ctx: ToolContext,
    config: LoopConfig,
    system_prompt: String,
    initial_messages: Vec<ChatMessage>,
) -> HarnessResult<LoopResult> {
    let tools = registry.tool_schemas();

    // Build the initial message history.
    let mut history: Vec<ChatMessage> = Vec::new();
    history.push(ChatMessage::system(&system_prompt));
    history.extend(initial_messages);

    let mut turns = 0u32;

    loop {
        turns += 1;
        if turns > config.max_iterations {
            return Err(HarnessError::MaxIterations(config.max_iterations));
        }

        // ── Turn: call the provider ──────────────────────────────────────
        let completion = if config.stream {
            // Stream the response and accumulate into a ChatCompletion-like result.
            stream_and_accumulate(&*provider, history.clone(), tools.clone(), &config.model).await?
        } else {
            // Blocking complete.
            let resp = provider
                .complete(history.clone(), tools.clone(), &config.model)
                .await?;
            let choice = resp.choices.into_iter().next().ok_or_else(|| {
                HarnessError::Provider("provider returned empty choices".to_string())
            })?;
            choice.message
        };

        // ── Check for tool calls ─────────────────────────────────────────
        let tool_calls = completion.tool_calls.clone().unwrap_or_default();

        if tool_calls.is_empty() {
            // No tool calls → model has given its final answer.
            let final_response = completion.content.clone().unwrap_or_default();
            history.push(completion);
            return Ok(LoopResult {
                final_response,
                turns,
                history,
            });
        }

        // ── Dispatch tools ───────────────────────────────────────────────
        // Append the assistant message (with tool_calls) first, then the
        // results — this is required by the OpenAI spec.
        history.push(completion);

        let results = execute_tool_calls(&tool_calls, &registry, &ctx).await;

        for result in results {
            history.push(ChatMessage::tool_result(result.call_id, result.content));
        }
        // Continue to next turn.
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Dispatch all tool calls in a turn concurrently, preserving order.
async fn execute_tool_calls(
    calls: &[ToolCall],
    registry: &ToolRegistry,
    ctx: &ToolContext,
) -> Vec<ToolCallResult> {
    // P1: sequential execution (concurrent tool execution is P3).
    let mut results = Vec::with_capacity(calls.len());
    for call in calls {
        let content = dispatch(registry, &call.function.name, &call.function.arguments, ctx).await;
        results.push(ToolCallResult {
            call_id: call.id.clone(),
            tool_name: call.function.name.clone(),
            content,
        });
    }
    results
}

struct ToolCallResult {
    call_id: String,
    #[allow(dead_code)]
    tool_name: String,
    content: String,
}

/// Stream a response and accumulate content + tool_calls into a single
/// [`ChatMessage`] as if it were a non-streaming completion.
async fn stream_and_accumulate(
    provider: &dyn ProviderClient,
    messages: Vec<ChatMessage>,
    tools: Vec<crate::provider::Tool>,
    model: &str,
) -> HarnessResult<ChatMessage> {
    use futures_util::StreamExt;

    let mut stream = provider.stream(messages, tools, model).await?;

    let mut content_buf = String::new();
    // tool_calls accumulation: index → (id, type, name, args_buf)
    let mut tool_calls_acc: std::collections::HashMap<u32, ToolCallAccumulator> =
        std::collections::HashMap::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        for delta_choice in chunk.choices {
            let delta = delta_choice.delta;

            if let Some(content) = delta.content {
                content_buf.push_str(&content);
            }

            if let Some(tc_deltas) = delta.tool_calls {
                for tc_delta in tc_deltas {
                    let acc = tool_calls_acc.entry(tc_delta.index).or_default();
                    if let Some(id) = tc_delta.id {
                        acc.id = id;
                    }
                    if let Some(kind) = tc_delta.r#type {
                        acc.kind = kind;
                    }
                    if let Some(func) = tc_delta.function {
                        if let Some(name) = func.name {
                            acc.name = name;
                        }
                        if let Some(args) = func.arguments {
                            acc.args_buf.push_str(&args);
                        }
                    }
                }
            }
        }
    }

    // Assemble tool calls if any were accumulated.
    let tool_calls: Vec<ToolCall> = if tool_calls_acc.is_empty() {
        vec![]
    } else {
        let mut sorted: Vec<_> = tool_calls_acc.into_iter().collect();
        sorted.sort_by_key(|(idx, _)| *idx);
        sorted
            .into_iter()
            .map(|(_, acc)| ToolCall {
                id: acc.id,
                kind: acc.kind,
                function: crate::provider::FunctionCall {
                    name: acc.name,
                    arguments: acc.args_buf,
                },
            })
            .collect()
    };

    Ok(ChatMessage::assistant(
        if content_buf.is_empty() {
            None
        } else {
            Some(content_buf)
        },
        if tool_calls.is_empty() {
            None
        } else {
            Some(tool_calls)
        },
    ))
}

#[derive(Default)]
struct ToolCallAccumulator {
    id: String,
    kind: String,
    name: String,
    args_buf: String,
}

// ---------------------------------------------------------------------------
// Tests (all offline — no real provider)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::HarnessError;
    use crate::provider::types::FunctionCall;
    use crate::provider::{
        ChatCompletion, ChatMessage, Choice, FinishReason, ProviderClient, StreamChunk, Tool,
        ToolCall,
    };
    use crate::tools::registry::default_registry;
    use async_trait::async_trait;
    use futures_util::Stream;
    use std::pin::Pin;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // ── Mock provider ────────────────────────────────────────────────────────

    /// A mock provider that returns pre-configured responses in sequence.
    struct MockProvider {
        responses: Mutex<Vec<ChatCompletion>>,
    }

    impl MockProvider {
        fn new(responses: Vec<ChatCompletion>) -> Self {
            Self {
                responses: Mutex::new(responses),
            }
        }
    }

    #[async_trait]
    impl ProviderClient for MockProvider {
        async fn complete(
            &self,
            _messages: Vec<ChatMessage>,
            _tools: Vec<Tool>,
            _model: &str,
        ) -> Result<ChatCompletion, HarnessError> {
            let mut lock = self.responses.lock().unwrap();
            if lock.is_empty() {
                return Err(HarnessError::Provider("mock exhausted".to_string()));
            }
            Ok(lock.remove(0))
        }

        async fn stream(
            &self,
            _messages: Vec<ChatMessage>,
            _tools: Vec<Tool>,
            _model: &str,
        ) -> Result<
            Pin<Box<dyn Stream<Item = Result<StreamChunk, HarnessError>> + Send>>,
            HarnessError,
        > {
            // Not used in non-streaming tests.
            Err(HarnessError::Provider(
                "mock: stream not implemented".to_string(),
            ))
        }

        async fn validate_model(&self, _model: &str) -> Result<(), HarnessError> {
            Ok(())
        }
    }

    fn make_final_response(content: &str) -> ChatCompletion {
        ChatCompletion {
            id: "mock-id".to_string(),
            choices: vec![Choice {
                index: 0,
                message: ChatMessage::assistant(Some(content.to_string()), None),
                finish_reason: Some(FinishReason::Stop),
            }],
            usage: None,
        }
    }

    fn make_tool_call_response(tool_name: &str, args: &str) -> ChatCompletion {
        let call = ToolCall {
            id: "call-1".to_string(),
            kind: "function".to_string(),
            function: FunctionCall {
                name: tool_name.to_string(),
                arguments: args.to_string(),
            },
        };
        ChatCompletion {
            id: "mock-id".to_string(),
            choices: vec![Choice {
                index: 0,
                message: ChatMessage::assistant(None, Some(vec![call])),
                finish_reason: Some(FinishReason::ToolCalls),
            }],
            usage: None,
        }
    }

    // ── Tests ────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn loop_immediate_final_answer() {
        let tmp = TempDir::new().unwrap();
        let provider = Arc::new(MockProvider::new(vec![make_final_response(
            "Hello, world!",
        )]));
        let registry = Arc::new(default_registry());
        let ctx = ToolContext::new(tmp.path());
        let config = LoopConfig {
            model: "mock".to_string(),
            max_iterations: 5,
            stream: false,
        };

        let result = run_loop(
            provider,
            registry,
            ctx,
            config,
            "You are a helpful assistant.".to_string(),
            vec![ChatMessage::user("Say hello")],
        )
        .await
        .unwrap();

        assert_eq!(result.final_response, "Hello, world!");
        assert_eq!(result.turns, 1);
    }

    #[tokio::test]
    async fn loop_tool_call_then_final() {
        let tmp = TempDir::new().unwrap();

        // Write a file the model will "read".
        tokio::fs::write(tmp.path().join("note.txt"), "secret content")
            .await
            .unwrap();

        // Turn 1: model calls read_file.
        // Turn 2: model gives final answer after seeing the result.
        let provider = Arc::new(MockProvider::new(vec![
            make_tool_call_response("read_file", r#"{"path": "note.txt"}"#),
            make_final_response("The file contains: secret content"),
        ]));

        let registry = Arc::new(default_registry());
        let ctx = ToolContext::new(tmp.path());
        let config = LoopConfig {
            model: "mock".to_string(),
            max_iterations: 5,
            stream: false,
        };

        let result = run_loop(
            provider,
            registry,
            ctx,
            config,
            "You are a helpful assistant.".to_string(),
            vec![ChatMessage::user("What is in note.txt?")],
        )
        .await
        .unwrap();

        assert!(result.final_response.contains("secret content"));
        assert_eq!(result.turns, 2);
    }

    #[tokio::test]
    async fn loop_max_iterations_error() {
        let tmp = TempDir::new().unwrap();

        // Keep returning tool calls → should hit max_iterations.
        let provider = Arc::new(MockProvider::new(vec![
            make_tool_call_response("list_dir", "{}"),
            make_tool_call_response("list_dir", "{}"),
            make_tool_call_response("list_dir", "{}"),
            make_tool_call_response("list_dir", "{}"),
            make_tool_call_response("list_dir", "{}"),
        ]));

        let registry = Arc::new(default_registry());
        let ctx = ToolContext::new(tmp.path());
        let config = LoopConfig {
            model: "mock".to_string(),
            max_iterations: 3,
            stream: false,
        };

        let err = run_loop(
            provider,
            registry,
            ctx,
            config,
            "system".to_string(),
            vec![ChatMessage::user("loop forever")],
        )
        .await
        .unwrap_err();

        assert!(matches!(err, HarnessError::MaxIterations(3)));
    }

    #[tokio::test]
    async fn loop_write_then_read() {
        let tmp = TempDir::new().unwrap();

        // Turn 1: write_file, Turn 2: read_file, Turn 3: final.
        let provider = Arc::new(MockProvider::new(vec![
            make_tool_call_response("write_file", r#"{"path": "out.txt", "content": "done"}"#),
            make_tool_call_response("read_file", r#"{"path": "out.txt"}"#),
            make_final_response("Wrote and confirmed: done"),
        ]));

        let registry = Arc::new(default_registry());
        let ctx = ToolContext::new(tmp.path());
        let config = LoopConfig {
            model: "mock".to_string(),
            max_iterations: 5,
            stream: false,
        };

        let result = run_loop(
            provider,
            registry,
            ctx,
            config,
            "system".to_string(),
            vec![ChatMessage::user("write a file")],
        )
        .await
        .unwrap();

        assert!(result.final_response.contains("done"));
        assert_eq!(result.turns, 3);
        // Verify the file was actually written.
        let content = tokio::fs::read_to_string(tmp.path().join("out.txt"))
            .await
            .unwrap();
        assert_eq!(content, "done");
    }
}
