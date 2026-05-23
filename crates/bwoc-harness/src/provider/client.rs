//! OpenAI-compatible HTTP client.
//!
//! [`ProviderClient`] is the injectable trait — tests swap in a mock.
//! [`OllamaClient`] is the real implementation backed by `reqwest`.
//!
//! ## Retry classification
//!
//! HTTP errors are split into two buckets:
//!
//! - **Transient** (retry-safe): connection errors, 5xx responses, request
//!   timeouts.  Callers with exponential-backoff retry loops use
//!   [`HarnessError::is_transient`] to gate retries.
//! - **Fatal** (fail-fast): 404 (`ModelNotFound`), other 4xx, JSON parse
//!   failures.  Retrying these is pointless and misleading.
//!
//! The retry loop itself lives in `agent_loop` — the provider just classifies.

use std::pin::Pin;

use async_trait::async_trait;
use futures_util::Stream;
use reqwest::Client;
use serde_json::{Value, json};

use super::types::{ChatCompletion, ChatMessage, StreamChunk, Tool};
use crate::error::HarnessError;

// ---------------------------------------------------------------------------
// Trait (injectable / mockable)
// ---------------------------------------------------------------------------

/// The interface the agent loop uses to call the model.
///
/// Implementors: [`OllamaClient`] (real HTTP) + any mock in tests.
#[async_trait]
pub trait ProviderClient: Send + Sync {
    /// Blocking (stream=false) completion.  Returns the full response.
    async fn complete(
        &self,
        messages: Vec<ChatMessage>,
        tools: Vec<Tool>,
        model: &str,
    ) -> Result<ChatCompletion, HarnessError>;

    /// Streaming (stream=true) completion.  Returns an SSE chunk stream.
    async fn stream(
        &self,
        messages: Vec<ChatMessage>,
        tools: Vec<Tool>,
        model: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, HarnessError>> + Send>>, HarnessError>;

    /// Validate that `model` is available at this endpoint.
    /// Returns `Ok(())` if found, `Err(HarnessError::ModelNotFound)` otherwise.
    async fn validate_model(&self, model: &str) -> Result<(), HarnessError>;
}

// ---------------------------------------------------------------------------
// Real implementation
// ---------------------------------------------------------------------------

/// Real HTTP client speaking the OpenAI-compat API.
///
/// Default endpoint: `http://localhost:11434/v1` (Ollama).
#[derive(Debug, Clone)]
pub struct OllamaClient {
    pub base_url: String,
    client: Client,
}

impl OllamaClient {
    /// Create a client with an explicit base URL.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: Client::new(),
        }
    }

    /// Create a client pointing at the default Ollama endpoint.
    pub fn default_endpoint() -> Self {
        Self::new("http://localhost:11434/v1")
    }

    fn completions_url(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }

    fn models_url(&self) -> String {
        format!("{}/models", self.base_url)
    }
}

#[async_trait]
impl ProviderClient for OllamaClient {
    async fn complete(
        &self,
        messages: Vec<ChatMessage>,
        tools: Vec<Tool>,
        model: &str,
    ) -> Result<ChatCompletion, HarnessError> {
        let body = build_request_body(messages, tools, model, false);

        let resp = self
            .client
            .post(self.completions_url())
            .json(&body)
            .send()
            .await
            .map_err(|e| HarnessError::TransientProvider(format!("HTTP request failed: {e}")))?;

        let status = resp.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(HarnessError::ModelNotFound(model.to_string()));
        }
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            // 5xx = transient; 4xx = fatal.
            return Err(classify_http_error(status.as_u16(), &text));
        }

        resp.json::<ChatCompletion>()
            .await
            .map_err(|e| HarnessError::Provider(format!("Failed to parse response: {e}")))
    }

    async fn stream(
        &self,
        messages: Vec<ChatMessage>,
        tools: Vec<Tool>,
        model: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, HarnessError>> + Send>>, HarnessError>
    {
        use bytes::Bytes;
        use futures_util::{StreamExt, TryStreamExt};

        let body = build_request_body(messages, tools, model, true);

        let resp = self
            .client
            .post(self.completions_url())
            .json(&body)
            .send()
            .await
            .map_err(|e| HarnessError::TransientProvider(format!("HTTP request failed: {e}")))?;

        let status = resp.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(HarnessError::ModelNotFound(model.to_string()));
        }
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(classify_http_error(status.as_u16(), &text));
        }

        // Parse SSE: each line starting with "data: " is a JSON chunk.
        // "[DONE]" signals end of stream.
        let byte_stream = resp.bytes_stream();
        let stream = byte_stream
            .map_err(|e| HarnessError::Provider(format!("Stream error: {e}")))
            .flat_map(|chunk_result: Result<Bytes, HarnessError>| {
                let lines: Vec<Result<StreamChunk, HarnessError>> = match chunk_result {
                    Err(e) => vec![Err(e)],
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        text.lines()
                            .filter_map(|line| {
                                let line = line.trim();
                                if let Some(data) = line.strip_prefix("data: ") {
                                    if data == "[DONE]" {
                                        return None; // end-of-stream sentinel
                                    }
                                    Some(serde_json::from_str::<StreamChunk>(data).map_err(|e| {
                                        HarnessError::Provider(format!(
                                            "SSE parse error on `{data}`: {e}"
                                        ))
                                    }))
                                } else {
                                    None
                                }
                            })
                            .collect()
                    }
                };
                futures_util::stream::iter(lines)
            });

        Ok(Box::pin(stream))
    }

    async fn validate_model(&self, model: &str) -> Result<(), HarnessError> {
        // GET /v1/models returns a list; check the model is present.
        let resp = self
            .client
            .get(self.models_url())
            .send()
            .await
            .map_err(|e| HarnessError::Provider(format!("Model list request failed: {e}")))?;

        if !resp.status().is_success() {
            // If the endpoint doesn't implement /models, fall through and
            // let the first completion call surface the 404.
            return Ok(());
        }

        let body: Value = resp
            .json()
            .await
            .map_err(|e| HarnessError::Provider(format!("Failed to parse models list: {e}")))?;

        let found = body["data"]
            .as_array()
            .map(|arr| arr.iter().any(|m| m["id"].as_str() == Some(model)))
            .unwrap_or(false);

        if found {
            Ok(())
        } else {
            Err(HarnessError::ModelNotFound(model.to_string()))
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_request_body(
    messages: Vec<ChatMessage>,
    tools: Vec<Tool>,
    model: &str,
    stream: bool,
) -> Value {
    let mut body = json!({
        "model": model,
        "messages": messages,
        "stream": stream,
    });

    if !tools.is_empty() {
        body["tools"] = serde_json::to_value(tools).unwrap_or(Value::Array(vec![]));
    }

    body
}

// ---------------------------------------------------------------------------
// HTTP error classification helper
// ---------------------------------------------------------------------------

/// Classify an HTTP error as transient (5xx) or fatal (4xx).
///
/// - **5xx** — server-side error, may be transient: retry with backoff.
/// - **4xx** (non-404) — client-side error (bad request, auth failure, rate
///   limit exceeded with no retry-after, etc.) — fail fast.
///
/// 404 is handled before this function is called and maps to
/// [`HarnessError::ModelNotFound`].
pub(crate) fn classify_http_error(status: u16, body: &str) -> HarnessError {
    if status >= 500 {
        HarnessError::TransientProvider(format!("HTTP {status}: {body}"))
    } else {
        HarnessError::Provider(format!("HTTP {status}: {body}"))
    }
}

// ---------------------------------------------------------------------------
// async_trait re-export helper — keep the dep inside this crate
// ---------------------------------------------------------------------------
// We use async_trait from the futures ecosystem; declare it in Cargo.toml.
// The attribute is applied above — this comment is a reminder, not code.
