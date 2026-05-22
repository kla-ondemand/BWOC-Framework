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
mod completion;
mod dashboard;
mod doctor;
mod help;
mod i18n;
mod init;
mod new;
mod retire;
mod spawn;
mod status;
mod user_home;
mod util;
mod workspace;

#[derive(Parser, Debug)]
#[command(
    name = "bwoc",
    version,
    about = "BWOC — Buddhist Way of Coding agent framework CLI.",
    long_about = "Phase 1 v2.0. See modules/agent-template/docs/en/PHILOSOPHY.en.md §0.1 The Arc.",
    // We provide our own `help` subcommand (topical guides). Disable clap's
    // auto-generated one to avoid the duplicate-name conflict.
    disable_help_subcommand = true
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
    /// Diagnose environment + workspace; with `--auto`, fix safe issues in place.
    Doctor(DoctorArgs),
    /// Retire an agent — remove it from the workspace's registry (vaya).
    Retire(RetireArgs),
    /// Show per-agent health + identity snapshot (read-only).
    Status(StatusArgs),
    /// Topic-specific help (backends, workspace, manifest, arc, getting-started).
    Help(HelpArgs),
    /// Emit a shell completion script (bash, zsh, fish, powershell, elvish).
    Completion(CompletionArgs),
    /// Launch the interactive TUI dashboard (agents list with navigation; refresh with `r`).
    Dashboard(DashboardArgs),
}

#[derive(Args, Debug)]
struct DashboardArgs {
    /// Workspace root. Resolution: --workspace > BWOC_WORKSPACE env > ancestor walk > cwd.
    #[arg(long = "workspace")]
    workspace: Option<PathBuf>,
}

impl From<DashboardArgs> for dashboard::DashboardArgs {
    fn from(a: DashboardArgs) -> Self {
        Self {
            workspace: a.workspace,
        }
    }
}

#[derive(Args, Debug)]
struct CompletionArgs {
    /// Target shell. Pipe the output to your shell's completion install path.
    #[arg(value_enum)]
    shell: clap_complete::Shell,
}

impl From<CompletionArgs> for completion::CompletionArgs {
    fn from(a: CompletionArgs) -> Self {
        Self { shell: a.shell }
    }
}

#[derive(Args, Debug)]
struct HelpArgs {
    /// Topic name. Run `bwoc help` (no arg) to list available topics.
    topic: Option<String>,
}

impl From<HelpArgs> for help::HelpArgs {
    fn from(a: HelpArgs) -> Self {
        Self { topic: a.topic }
    }
}

#[derive(Args, Debug)]
struct StatusArgs {
    /// Agent name. Matches by id ("agent-foo") or bare name ("foo"). If omitted, shows all agents.
    name: Option<String>,
    /// Workspace root. Resolution: --workspace > BWOC_WORKSPACE env > ancestor walk > cwd.
    #[arg(long = "workspace")]
    workspace: Option<PathBuf>,
    /// Emit JSON to stdout instead of the human-readable layout.
    #[arg(long)]
    json: bool,
}

impl From<StatusArgs> for status::StatusArgs {
    fn from(a: StatusArgs) -> Self {
        Self {
            name: a.name,
            workspace: a.workspace,
            json: a.json,
        }
    }
}

#[derive(Args, Debug)]
struct RetireArgs {
    /// Name of the agent to retire. Matches by id ("agent-foo") or bare name ("foo").
    name: String,
    /// Workspace root. Resolution: --workspace > BWOC_WORKSPACE env > ancestor walk > cwd.
    #[arg(long = "workspace")]
    workspace: Option<PathBuf>,
    /// Skip the interactive confirmation. Required for non-TTY (scripted) use.
    #[arg(long)]
    yes: bool,
    /// Keep the agent directory on disk; only remove the registry entry.
    #[arg(long = "keep-files")]
    keep_files: bool,
}

impl From<RetireArgs> for retire::RetireArgs {
    fn from(a: RetireArgs) -> Self {
        Self {
            name: a.name,
            workspace: a.workspace,
            yes: a.yes,
            keep_files: a.keep_files,
        }
    }
}

#[derive(Args, Debug)]
struct DoctorArgs {
    /// Workspace root to diagnose. Defaults: BWOC_WORKSPACE env > ancestor walk > cwd.
    #[arg(long = "workspace")]
    path: Option<PathBuf>,
    /// Attempt to fix safe issues automatically (missing dirs, missing symlinks).
    #[arg(long)]
    auto: bool,
}

impl From<DoctorArgs> for doctor::DoctorArgs {
    fn from(a: DoctorArgs) -> Self {
        Self {
            path: a.path,
            auto: a.auto,
        }
    }
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
    /// Find inconsistencies (phantom registry entries, orphan dirs); --apply to fix safe ones.
    Prune {
        /// Workspace root path. Defaults to current directory.
        path: Option<PathBuf>,
        /// Apply the safe removals (phantom entries). Orphan dirs are never auto-removed.
        #[arg(long)]
        apply: bool,
    },
}

#[derive(Args, Debug)]
struct ListArgs {
    /// Workspace root path. Resolution: --workspace > BWOC_WORKSPACE env > ancestor walk > cwd.
    #[arg(long = "workspace")]
    path: Option<PathBuf>,
    /// Emit JSON to stdout instead of the human-readable table.
    #[arg(long)]
    json: bool,
}

impl ListArgs {
    fn into_runtime(self, lang: String) -> workspace::ListArgs {
        workspace::ListArgs {
            path: self.path,
            lang,
            json: self.json,
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
                WorkspaceCommand::Prune { path, apply } => {
                    workspace::run_prune(workspace::PruneArgs { path, apply })
                }
            };
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Some(Commands::List(args)) => {
            let code = workspace::run_list(args.into_runtime(lang.clone()));
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Some(Commands::Doctor(args)) => {
            let code = doctor::run(args.into());
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Some(Commands::Retire(args)) => {
            let code = retire::run(args.into());
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Some(Commands::Status(args)) => {
            let code = status::run(args.into());
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Some(Commands::Help(args)) => {
            let code = help::run(args.into());
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Some(Commands::Completion(args)) => {
            let code = completion::run::<Cli>(args.into());
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Some(Commands::Dashboard(args)) => {
            let code = dashboard::run(args.into());
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
