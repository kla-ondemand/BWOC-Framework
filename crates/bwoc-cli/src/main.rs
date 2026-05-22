//! `bwoc` — the BWOC framework CLI.
//!
//! Phase 1 v2.0. Implemented subcommands so far: `check`, `new`, `spawn`.
//! Others land in follow-up fires of the loop. See `crates/bwoc-cli/README.md`
//! for the full surface and per-command status.

use clap::{Args, Parser, Subcommand};
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

mod banner;
mod check;
mod i18n;
mod init;
mod new;
mod spawn;
mod user_home;
mod util;
mod workspace;

#[derive(Parser, Debug)]
#[command(
    name = "bwoc",
    version,
    about = "BWOC — Buddhist Way of Coding agent framework CLI.",
    long_about = "Phase 1 v2.0. See modules/agent-template/docs/en/PHILOSOPHY.en.md §0.1 The Arc."
)]
struct Cli {
    /// Language for CLI output. Phase 1 ships with `en` and `th`.
    /// Precedence: --lang flag > BWOC_LANG env > $LANG > en fallback.
    #[arg(long, global = true)]
    lang: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Verify backend neutrality of an agent or the template (read-only audit).
    Check {
        /// Path to the agent or template to audit. Defaults to current directory.
        path: Option<PathBuf>,
    },
    /// Incarnate a new agent from the template (uppāda).
    New(Box<NewArgs>),
    /// Exec the configured LLM backend CLI in an agent's directory (uppāda → ṭhiti).
    Spawn(SpawnArgs),
    /// Initialize a BWOC workspace at the given path (uppāda).
    Init(InitArgs),
    /// Inspect a BWOC workspace.
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommand,
    },
    /// List agents registered in the enclosing workspace's agents.toml.
    List(ListArgs),
}

#[derive(Subcommand, Debug)]
enum WorkspaceCommand {
    /// Show resolved workspace path, config, and agent count.
    Info {
        /// Workspace root path. Defaults to current directory.
        path: Option<PathBuf>,
    },
    /// Run validation rules; exit 0 if complete, 2 if violations.
    Validate {
        /// Workspace root path. Defaults to current directory.
        path: Option<PathBuf>,
    },
}

#[derive(Args, Debug)]
struct ListArgs {
    /// Workspace root path. Resolution: --workspace > BWOC_WORKSPACE env > ancestor walk > cwd.
    #[arg(long = "workspace")]
    path: Option<PathBuf>,
}

impl ListArgs {
    fn into_runtime(self, lang: String) -> workspace::ListArgs {
        workspace::ListArgs {
            path: self.path,
            lang,
        }
    }
}

#[derive(Args, Debug)]
struct InitArgs {
    /// Path to initialize as a workspace. Defaults to current directory.
    path: Option<PathBuf>,
    /// Overwrite an existing workspace.toml.
    #[arg(long)]
    force: bool,
}

impl InitArgs {
    fn into_runtime(self, lang: String) -> init::InitArgs {
        init::InitArgs {
            path: self.path,
            force: self.force,
            lang,
        }
    }
}

#[derive(Args, Debug)]
struct SpawnArgs {
    /// Path to the agent directory. Defaults to current directory.
    #[arg(long)]
    path: Option<PathBuf>,
    /// LLM backend CLI to invoke.
    #[arg(long, value_enum, default_value_t = spawn::Backend::Claude)]
    backend: spawn::Backend,
    /// Extra arguments passed verbatim to the backend CLI (after `--`).
    #[arg(last = true)]
    extra: Vec<OsString>,
}

impl SpawnArgs {
    fn into_runtime(self, lang: String) -> spawn::SpawnArgs {
        spawn::SpawnArgs {
            path: self.path,
            backend: self.backend,
            extra: self.extra,
            lang,
        }
    }
}

#[derive(Args, Debug)]
struct NewArgs {
    /// Agent name (kebab-case, e.g. "database-schema").
    name: String,
    /// Target directory for the new agent. Default: ../agent-<name>/ relative to template.
    #[arg(long)]
    target: Option<PathBuf>,
    /// Path to the template directory. Default: auto-detect `modules/agent-template/` from cwd ancestors.
    #[arg(long)]
    template: Option<PathBuf>,
    /// Primary backend recorded in the workspace registry. Default: claude.
    #[arg(long, value_enum, default_value_t = spawn::Backend::Claude)]
    backend: spawn::Backend,
    /// One-line role description. Prompted if missing on a TTY.
    #[arg(long)]
    role: Option<String>,
    /// Primary LLM model identifier. Prompted if missing on a TTY.
    #[arg(long)]
    primary_model: Option<String>,
    /// Fallback LLM model identifier (truly optional).
    #[arg(long)]
    fallback_model: Option<String>,
    /// File-based memory directory. Default: memories/
    #[arg(long, default_value = "memories/")]
    memory_path: String,
    /// Session data directory for Tier 2 mining (truly optional).
    #[arg(long)]
    sessions_path: Option<String>,
    /// Tier 2 memory CLI command (truly optional).
    #[arg(long)]
    deep_memory_cmd: Option<String>,
    /// Lint command for the verification gate. Prompted if missing on a TTY.
    #[arg(long)]
    lint_cmd: Option<String>,
    /// Format command for the verification gate. Prompted if missing on a TTY.
    #[arg(long)]
    format_cmd: Option<String>,
    /// Test command for the verification gate. Prompted if missing on a TTY.
    #[arg(long)]
    test_cmd: Option<String>,
    /// Build command for the verification gate. Prompted if missing on a TTY.
    #[arg(long)]
    build_cmd: Option<String>,
    /// Base directory for worktrees (truly optional). Default: /tmp
    #[arg(long)]
    worktree_base: Option<String>,
}

impl NewArgs {
    fn into_runtime(self, lang: String) -> new::NewArgs {
        new::NewArgs {
            name: self.name,
            target: self.target,
            template: self.template,
            backend: self.backend,
            lang,
            role: self.role,
            primary_model: self.primary_model,
            fallback_model: self.fallback_model,
            memory_path: self.memory_path,
            sessions_path: self.sessions_path,
            deep_memory_cmd: self.deep_memory_cmd,
            lint_cmd: self.lint_cmd,
            format_cmd: self.format_cmd,
            test_cmd: self.test_cmd,
            build_cmd: self.build_cmd,
            worktree_base: self.worktree_base,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let lang = resolve_lang(cli.lang);
    let bundle = i18n::bundle_for(&lang);

    // Best-effort: ensure ~/.bwoc/ exists before any command runs. Failure here
    // (e.g., $HOME unset or read-only filesystem) logs a warning but does not
    // block the command — most commands don't yet read user-level config.
    if let Err(e) = user_home::ensure_initialized() {
        eprintln!("bwoc: warning — could not initialize ~/.bwoc/: {e}");
    }

    match cli.command {
        Some(Commands::Check { path }) => {
            let target = path.unwrap_or_else(|| PathBuf::from("."));
            let code = check::run(&target, &lang);
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Some(Commands::New(args)) => {
            let code = new::run((*args).into_runtime(lang.clone()));
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Some(Commands::Spawn(args)) => {
            let code = spawn::run(args.into_runtime(lang.clone()));
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Some(Commands::Init(args)) => {
            let code = init::run(args.into_runtime(lang.clone()));
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Some(Commands::Workspace { command }) => {
            let code = match command {
                WorkspaceCommand::Info { path } => workspace::run_info(workspace::InfoArgs {
                    path,
                    lang: lang.clone(),
                }),
                WorkspaceCommand::Validate { path } => {
                    workspace::run_validate(workspace::ValidateArgs {
                        path,
                        lang: lang.clone(),
                    })
                }
            };
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Some(Commands::List(args)) => {
            let code = workspace::run_list(args.into_runtime(lang.clone()));
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        None => {
            // No subcommand — print the startup banner. Banner already
            // includes a `bwoc --help` hint at the bottom.
            let _ = &bundle; // banner is lang-agnostic for now
            banner::print();
            ExitCode::SUCCESS
        }
    }
}

fn resolve_lang(flag: Option<String>) -> String {
    flag.or_else(|| std::env::var("BWOC_LANG").ok())
        .or_else(|| std::env::var("LANG").ok().and_then(parse_locale))
        .unwrap_or_else(|| "en".to_string())
}

/// Extract the language tag from a POSIX `LANG`-style value like `th_TH.UTF-8`.
fn parse_locale(raw: String) -> Option<String> {
    let tag = raw.split(['.', '@']).next()?;
    let lang = tag.split('_').next()?;
    if lang.is_empty() {
        None
    } else {
        Some(lang.to_ascii_lowercase())
    }
}
