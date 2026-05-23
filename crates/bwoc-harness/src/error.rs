//! Harness error types.

use thiserror::Error;

/// Top-level error type for `bwoc-harness`.
#[derive(Debug, Error)]
pub enum HarnessError {
    /// HTTP/provider error (non-404 failure, parse error, network error).
    #[error("provider error: {0}")]
    Provider(String),

    /// The requested model was not found at the endpoint (HTTP 404 or absent
    /// from the models list).  The spike confirmed Ollama returns 404 for
    /// wrong model tags — surface this clearly rather than letting it
    /// manifest as a mysterious JSON parse failure.
    #[error("model not found: `{0}` — check the model tag with `ollama list`")]
    ModelNotFound(String),

    /// A tool invocation failed.
    #[error("tool `{tool}` failed: {reason}")]
    ToolExecution { tool: String, reason: String },

    /// A file-path was rejected by the path confinement check.
    #[error("path `{0}` is outside the allowed working directory")]
    PathEscape(String),

    /// The agent loop hit the maximum iteration limit.
    #[error("agent loop exceeded max iterations ({0})")]
    MaxIterations(u32),

    /// I/O error from the filesystem tools or config loading.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization / deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Catch-all for unexpected conditions.
    #[error("{0}")]
    Other(String),
}

/// Convenience alias.
pub type HarnessResult<T> = Result<T, HarnessError>;
