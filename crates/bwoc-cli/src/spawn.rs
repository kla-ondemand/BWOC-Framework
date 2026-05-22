//! `bwoc spawn` — exec the configured LLM backend CLI in an agent's directory.
//!
//! Minimal Phase 1 v2.0 implementation: requires explicit `--path` (workspace
//! discovery and `agents.toml` lookup defer to Phase 2). Validates the path is
//! a BWOC agent (has `AGENTS.md`) before spawning. Propagates the backend's
//! exit code.

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::i18n;

/// Which backend CLI to invoke. Maps 1:1 to the program name on PATH.
#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Backend {
    Claude,
    Gemini,
    Codex,
    Kimi,
}

impl Backend {
    pub fn cli_name(self) -> &'static str {
        match self {
            Backend::Claude => "claude",
            Backend::Gemini => "gemini",
            Backend::Codex => "codex",
            Backend::Kimi => "kimi",
        }
    }

    /// Curated catalog of common LLM model identifiers per backend, surfaced
    /// in the `bwoc new` interactive picker. First entry is the recommended
    /// default. Free-text input is still accepted for unlisted models — this
    /// is a convenience, not a whitelist. Update as backends release models.
    pub fn models(self) -> &'static [&'static str] {
        match self {
            Backend::Claude => &["claude-opus-4-7", "claude-sonnet-4-6", "claude-haiku-4-5"],
            Backend::Gemini => &[
                "gemini-2.5-pro",
                "gemini-2.5-flash",
                "gemini-2.5-flash-lite",
            ],
            Backend::Codex => &["gpt-5", "gpt-5-mini", "o1"],
            Backend::Kimi => &["kimi-k2", "kimi-k1.5"],
        }
    }
}

pub struct SpawnArgs {
    pub path: Option<PathBuf>,
    pub backend: Backend,
    pub extra: Vec<OsString>,
    pub lang: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SpawnError {
    #[error("agent path does not exist: {0}")]
    PathMissing(PathBuf),
    #[error("not a BWOC agent: {0} (no AGENTS.md found)")]
    NotAnAgent(PathBuf),
    #[error("backend CLI '{backend}' not found on PATH; install it or pick another --backend")]
    BackendNotFound { backend: &'static str },
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

/// Entry point — returns the process exit code.
pub fn run(args: SpawnArgs) -> i32 {
    match spawn(args) {
        Ok(code) => code,
        Err(
            e @ (SpawnError::PathMissing(_)
            | SpawnError::NotAnAgent(_)
            | SpawnError::BackendNotFound { .. }),
        ) => {
            eprintln!("bwoc spawn: {e}");
            2
        }
        Err(e) => {
            eprintln!("bwoc spawn: {e}");
            1
        }
    }
}

pub fn spawn(args: SpawnArgs) -> Result<i32, SpawnError> {
    let bundle = i18n::bundle_for(&args.lang);
    let path = args
        .path
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    validate_agent_path(&path)?;

    let backend = args.backend.cli_name();
    let path_display = path.display().to_string();
    eprintln!(
        "{}",
        i18n::t_with(
            &bundle,
            "spawn-exec-status",
            &[("backend", backend), ("path", &path_display)],
        )
    );

    let status = Command::new(backend)
        .current_dir(&path)
        .args(&args.extra)
        .status()
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                SpawnError::BackendNotFound { backend }
            } else {
                SpawnError::Io(e)
            }
        })?;

    Ok(status.code().unwrap_or(1))
}

fn validate_agent_path(path: &Path) -> Result<(), SpawnError> {
    if !path.is_dir() {
        return Err(SpawnError::PathMissing(path.to_path_buf()));
    }
    let agents_md = path.join("AGENTS.md");
    if !agents_md.exists() {
        return Err(SpawnError::NotAnAgent(path.to_path_buf()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_cli_names() {
        assert_eq!(Backend::Claude.cli_name(), "claude");
        assert_eq!(Backend::Gemini.cli_name(), "gemini");
        assert_eq!(Backend::Codex.cli_name(), "codex");
        assert_eq!(Backend::Kimi.cli_name(), "kimi");
    }

    #[test]
    fn validate_rejects_missing_path() {
        assert!(matches!(
            validate_agent_path(Path::new("/nonexistent/path/xyz123")),
            Err(SpawnError::PathMissing(_))
        ));
    }

    #[test]
    fn validate_rejects_non_agent_dir() {
        // /tmp exists but has no AGENTS.md → NotAnAgent
        let p = Path::new("/tmp");
        // Only run if /tmp/AGENTS.md doesn't exist (which is the realistic case).
        if !p.join("AGENTS.md").exists() {
            assert!(matches!(
                validate_agent_path(p),
                Err(SpawnError::NotAnAgent(_))
            ));
        }
    }

    #[test]
    fn validate_accepts_agent_template() {
        let template =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../modules/agent-template");
        assert!(validate_agent_path(&template).is_ok());
    }
}
