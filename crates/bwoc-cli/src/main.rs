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
mod chat;
mod check;
mod completion;
mod dashboard;
mod doctor;
mod help;
mod i18n;
mod inbox;
mod init;
mod livecheck;
mod log;
mod memory;
mod new;
mod ping;
mod retire;
mod send;
mod spawn;
mod start;
mod status;
mod stop;
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
        /// Emit JSON to stdout instead of the human-readable report.
        #[arg(long)]
        json: bool,
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
    /// Diagnose environment + workspace; with `--auto`, fix safe issues in place. `--json` for structured output.
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
    /// Pause an agent — set status = "stopped" without removing files.
    Stop(StopArgs),
    /// Reactivate a stopped agent — set status = "active".
    Start(StartArgs),
    /// Ping a `bwoc-agent --serve`'d agent over its Unix socket (PING → PONG).
    Ping(PingArgs),
    /// Append a message to an agent's inbox (`.bwoc/inbox.jsonl`).
    Send(SendArgs),
    /// Chat with an agent — exec backend CLI with manifest-driven model.
    Chat(ChatArgs),
    /// Read messages from an agent's inbox (`.bwoc/inbox.jsonl`).
    Inbox(InboxArgs),
    /// Tail an agent's daemon log (`.bwoc/agent.log`) — daemon stderr.
    Log(LogArgs),
    /// Read workspace-level memory (`.bwoc/memory/`).
    #[command(subcommand)]
    Memory(MemoryAction),
}

#[derive(clap::Subcommand, Debug)]
enum MemoryAction {
    /// List user-authored memory entries in `.bwoc/memory/`.
    List {
        /// Workspace root. Resolution: --workspace > BWOC_WORKSPACE env > ancestor walk > cwd.
        #[arg(long = "workspace")]
        workspace: Option<PathBuf>,
        /// Emit JSON instead of the human table.
        #[arg(long)]
        json: bool,
    },
    /// Print one memory entry's contents to stdout, or `--all` for every entry concatenated.
    Show {
        /// Entry name (with or without `.md` extension). Omit when using `--all`.
        name: Option<String>,
        /// Print every entry concatenated (alphabetical), each with a `# === <name> ===` header.
        /// Mutually exclusive with `<name>`.
        #[arg(long, conflicts_with = "name")]
        all: bool,
        /// Workspace root. Resolution chain same as `memory list`.
        #[arg(long = "workspace")]
        workspace: Option<PathBuf>,
        /// Emit JSON (only meaningful with `--all`; single-entry `show` is plain content).
        #[arg(long)]
        json: bool,
    },
    /// Write a memory entry. Reads from `--file` or stdin.
    Put {
        /// Entry name (with or without `.md` extension).
        name: String,
        /// Source file. If omitted, content is read from stdin until EOF.
        #[arg(long)]
        file: Option<PathBuf>,
        /// Overwrite an existing entry. Refuses without this flag.
        #[arg(long)]
        force: bool,
        /// Workspace root. Resolution chain same as `memory list`.
        #[arg(long = "workspace")]
        workspace: Option<PathBuf>,
    },
    /// Substring search across memory entries (case-insensitive).
    Search {
        /// Substring to look for in any entry's content.
        query: String,
        /// Workspace root. Resolution chain same as `memory list`.
        #[arg(long = "workspace")]
        workspace: Option<PathBuf>,
        /// Emit JSON instead of the human-readable grep-style output.
        #[arg(long)]
        json: bool,
    },
    /// Delete a memory entry. Prompts on TTY unless `--yes` is given.
    Rm {
        /// Entry name (with or without `.md` extension).
        name: String,
        /// Skip the TTY confirmation prompt.
        #[arg(long, short)]
        yes: bool,
        /// Workspace root. Resolution chain same as `memory list`.
        #[arg(long = "workspace")]
        workspace: Option<PathBuf>,
    },
}

impl MemoryAction {
    fn into_runtime(self) -> memory::MemoryArgs {
        match self {
            MemoryAction::List { workspace, json } => memory::MemoryArgs {
                action: memory::MemoryAction::List,
                workspace,
                json,
            },
            MemoryAction::Show {
                name,
                all,
                workspace,
                json,
            } => {
                // If neither `<name>` nor `--all` was provided, pass an
                // empty Show("") through. memory::show() detects this and
                // emits a helpful error. This keeps into_runtime
                // infallible (returns MemoryArgs, not Result).
                let action = if all {
                    memory::MemoryAction::ShowAll
                } else {
                    memory::MemoryAction::Show(name.unwrap_or_default())
                };
                memory::MemoryArgs {
                    action,
                    workspace,
                    json,
                }
            }
            MemoryAction::Put {
                name,
                file,
                force,
                workspace,
            } => {
                let source = match file {
                    Some(p) => memory::PutSource::FilePath(p),
                    None => memory::PutSource::Stdin,
                };
                memory::MemoryArgs {
                    action: memory::MemoryAction::Put {
                        name,
                        source,
                        force,
                    },
                    workspace,
                    json: false,
                }
            }
            MemoryAction::Search {
                query,
                workspace,
                json,
            } => memory::MemoryArgs {
                action: memory::MemoryAction::Search(query),
                workspace,
                json,
            },
            MemoryAction::Rm {
                name,
                yes,
                workspace,
            } => memory::MemoryArgs {
                action: memory::MemoryAction::Remove { name, yes },
                workspace,
                json: false,
            },
        }
    }
}

#[derive(Args, Debug)]
struct LogArgs {
    /// Agent name. Matches by id ("agent-foo") or bare name ("foo").
    agent: String,
    /// Workspace root. Resolution: --workspace > BWOC_WORKSPACE env > ancestor walk > cwd.
    #[arg(long = "workspace")]
    workspace: Option<PathBuf>,
    /// Block + stream new lines as they arrive (Ctrl-C to stop).
    #[arg(short = 'f', long)]
    follow: bool,
    /// Number of trailing lines to print before --follow blocks (or as the whole output).
    #[arg(short = 'n', long, default_value_t = 50)]
    lines: usize,
    /// Truncate the log file before printing. Useful when starting fresh observation.
    #[arg(long)]
    clear: bool,
}

impl From<LogArgs> for log::LogArgs {
    fn from(a: LogArgs) -> Self {
        Self {
            agent: a.agent,
            workspace: a.workspace,
            follow: a.follow,
            lines: a.lines,
            clear: a.clear,
        }
    }
}

#[derive(Args, Debug)]
struct InboxArgs {
    /// Agent name. Matches by id ("agent-foo") or bare name ("foo").
    agent: String,
    /// Workspace root. Resolution: --workspace > BWOC_WORKSPACE env > ancestor walk > cwd.
    #[arg(long = "workspace")]
    workspace: Option<PathBuf>,
    /// Emit JSON instead of the human-readable layout.
    #[arg(long)]
    json: bool,
    /// Show only the last N messages.
    #[arg(long)]
    limit: Option<usize>,
    /// Tail mode — block and print new envelopes as they arrive (Ctrl-C to stop).
    #[arg(long)]
    watch: bool,
    /// Truncate the inbox after printing (acknowledge / delete all messages).
    #[arg(long)]
    clear: bool,
    /// Skip the interactive confirmation for `--clear`. Required for non-TTY.
    #[arg(long)]
    yes: bool,
}

impl From<InboxArgs> for inbox::InboxArgs {
    fn from(a: InboxArgs) -> Self {
        Self {
            agent: a.agent,
            workspace: a.workspace,
            json: a.json,
            limit: a.limit,
            watch: a.watch,
            clear: a.clear,
            yes: a.yes,
        }
    }
}

#[derive(Args, Debug)]
struct ChatArgs {
    /// Agent name. Matches by id ("agent-foo") or bare name ("foo").
    name: String,
    /// Workspace root. Resolution: --workspace > BWOC_WORKSPACE env > ancestor walk > cwd.
    #[arg(long = "workspace")]
    workspace: Option<PathBuf>,
    /// Run inside a new tmux window instead of exec'ing in this shell. Requires $TMUX.
    #[arg(long)]
    tmux: bool,
}

impl ChatArgs {
    fn into_runtime(self, lang: String) -> chat::ChatArgs {
        chat::ChatArgs {
            name: self.name,
            workspace: self.workspace,
            lang,
            tmux: self.tmux,
        }
    }
}

#[derive(Args, Debug)]
struct SendArgs {
    /// Recipient agent. Matches by id ("agent-foo") or bare name ("foo").
    to: String,
    /// Message text (everything after the agent name; quote multi-word).
    message: String,
    /// Workspace root. Resolution: --workspace > BWOC_WORKSPACE env > ancestor walk > cwd.
    #[arg(long = "workspace")]
    workspace: Option<PathBuf>,
}

impl From<SendArgs> for send::SendArgs {
    fn from(a: SendArgs) -> Self {
        Self {
            to: a.to,
            message: a.message,
            workspace: a.workspace,
        }
    }
}

#[derive(Args, Debug)]
struct PingArgs {
    /// Agent name. Matches by id ("agent-foo") or bare name ("foo").
    name: String,
    /// Workspace root. Resolution: --workspace > BWOC_WORKSPACE env > ancestor walk > cwd.
    #[arg(long = "workspace")]
    workspace: Option<PathBuf>,
}

impl From<PingArgs> for ping::PingArgs {
    fn from(a: PingArgs) -> Self {
        Self {
            name: a.name,
            workspace: a.workspace,
        }
    }
}

#[derive(Args, Debug)]
#[command(group(clap::ArgGroup::new("target").required(true).args(["name", "all"])))]
struct StartArgs {
    /// Name of the agent. Matches by id ("agent-foo") or bare name ("foo").
    /// Mutually exclusive with `--all` (clap group enforces).
    name: Option<String>,
    /// Start every stopped agent in the workspace. Honors `--yes` + `--no-daemon`.
    /// Mutually exclusive with `name`.
    #[arg(long)]
    all: bool,
    /// Workspace root. Resolution: --workspace > BWOC_WORKSPACE env > ancestor walk > cwd.
    #[arg(long = "workspace")]
    workspace: Option<PathBuf>,
    /// Skip the interactive confirmation. Required for non-TTY (scripted) use.
    #[arg(long)]
    yes: bool,
    /// Only flip registry status; do not spawn `bwoc-agent --serve`.
    #[arg(long = "no-daemon")]
    no_daemon: bool,
}

impl From<StartArgs> for start::StartArgs {
    fn from(a: StartArgs) -> Self {
        Self {
            name: a.name.unwrap_or_default(), // clap group ensures one of (name, all)
            workspace: a.workspace,
            yes: a.yes,
            no_daemon: a.no_daemon,
            all: a.all,
        }
    }
}

#[derive(Args, Debug)]
#[command(group(clap::ArgGroup::new("target").required(true).args(["name", "all"])))]
struct StopArgs {
    /// Name of the agent. Matches by id ("agent-foo") or bare name ("foo").
    /// Mutually exclusive with `--all` (clap group enforces).
    name: Option<String>,
    /// Stop every non-stopped agent in the workspace. Honors `--yes`.
    /// Mutually exclusive with `name`.
    #[arg(long)]
    all: bool,
    /// Workspace root. Resolution: --workspace > BWOC_WORKSPACE env > ancestor walk > cwd.
    #[arg(long = "workspace")]
    workspace: Option<PathBuf>,
    /// Skip the interactive confirmation. Required for non-TTY (scripted) use.
    #[arg(long)]
    yes: bool,
}

impl From<StopArgs> for stop::StopArgs {
    fn from(a: StopArgs) -> Self {
        Self {
            name: a.name.unwrap_or_default(), // clap group ensures one of (name, all)
            workspace: a.workspace,
            yes: a.yes,
            all: a.all,
        }
    }
}

#[derive(Args, Debug)]
struct DashboardArgs {
    /// Workspace root. Resolution: --workspace > BWOC_WORKSPACE env > ancestor walk > cwd.
    #[arg(long = "workspace")]
    workspace: Option<PathBuf>,
}

impl DashboardArgs {
    fn into_runtime(self, lang: String) -> dashboard::DashboardArgs {
        dashboard::DashboardArgs {
            workspace: self.workspace,
            lang,
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
    /// Emit structured JSON instead of the human-readable list.
    #[arg(long)]
    json: bool,
}

impl From<DoctorArgs> for doctor::DoctorArgs {
    fn from(a: DoctorArgs) -> Self {
        Self {
            path: a.path,
            auto: a.auto,
            json: a.json,
        }
    }
}

#[derive(Subcommand, Debug)]
enum WorkspaceCommand {
    /// Show resolved workspace path, config, and agent count.
    Info {
        /// Workspace root path. Defaults to current directory.
        path: Option<PathBuf>,
        /// Emit JSON instead of the human-readable layout.
        #[arg(long)]
        json: bool,
    },
    /// Run validation rules; exit 0 if complete, 2 if violations.
    Validate {
        /// Workspace root path. Defaults to current directory.
        path: Option<PathBuf>,
        /// Emit JSON instead of the human-readable report.
        #[arg(long)]
        json: bool,
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
    /// Filter by status (exact match). Common values: active, stopped, retired.
    #[arg(long)]
    status: Option<String>,
    /// Filter by backend (exact match).
    #[arg(long, value_enum)]
    backend: Option<spawn::Backend>,
    /// Filter to agents whose daemon is actually running (PID file + signal-0).
    #[arg(long)]
    running: bool,
    /// Filter to agents with at least one pending inbox envelope.
    #[arg(long = "inbox-pending")]
    inbox_pending: bool,
    /// Sort key. Default: registry insertion order. One of: id, inbox, incarnated, backend.
    #[arg(long)]
    sort: Option<String>,
    /// Print just the count of matching agents (one integer) instead of the table.
    /// With `--json`, emits `{"count": N}`.
    #[arg(long)]
    count: bool,
    /// Print bare agent ids, one per line. Combine with filters for `for $name in ...` loops.
    /// With `--json`, emits `{"names": [...]}`. `--count` wins if both are set.
    #[arg(long = "names-only")]
    names_only: bool,
}

impl ListArgs {
    fn into_runtime(self, lang: String) -> workspace::ListArgs {
        workspace::ListArgs {
            path: self.path,
            lang,
            json: self.json,
            status_filter: self.status,
            backend_filter: self.backend.map(|b| b.cli_name().to_string()),
            running_only: self.running,
            inbox_pending_only: self.inbox_pending,
            sort: self.sort,
            count_only: self.count,
            names_only: self.names_only,
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
    /// Persona scope: one-line "this agent does X". Fills `{{scopeDescription}}`.
    #[arg(long)]
    scope: Option<String>,
    /// Persona anti-scope: one-line "this agent does NOT do Y".
    #[arg(long = "out-of-scope")]
    out_of_scope: Option<String>,
    /// Initial mindsets to seed — comma-separated kebab-case names (e.g.
    /// "verify-before-act,right-amount"). One stub `.md` per name.
    #[arg(long)]
    mindsets: Option<String>,
    /// Initial skills to seed — comma-separated kebab-case names. One stub
    /// `.md` per name.
    #[arg(long)]
    skills: Option<String>,
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
            scope: self.scope,
            out_of_scope: self.out_of_scope,
            mindsets: self.mindsets,
            skills: self.skills,
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
        Some(Commands::Check { path, json }) => {
            let target = path.unwrap_or_else(|| PathBuf::from("."));
            let code = check::run(&target, &lang, json);
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
                WorkspaceCommand::Info { path, json } => workspace::run_info(workspace::InfoArgs {
                    path,
                    lang: lang.clone(),
                    json,
                }),
                WorkspaceCommand::Validate { path, json } => {
                    workspace::run_validate(workspace::ValidateArgs {
                        path,
                        lang: lang.clone(),
                        json,
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
            let code = dashboard::run(args.into_runtime(lang.clone()));
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Some(Commands::Stop(args)) => {
            let code = stop::run(args.into());
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Some(Commands::Start(args)) => {
            let code = start::run(args.into());
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Some(Commands::Ping(args)) => {
            let code = ping::run(args.into());
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Some(Commands::Send(args)) => {
            let code = send::run(args.into());
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Some(Commands::Chat(args)) => {
            let code = chat::run(args.into_runtime(lang.clone()));
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Some(Commands::Inbox(args)) => {
            let code = inbox::run(args.into());
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Some(Commands::Log(args)) => {
            let code = log::run(args.into());
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Some(Commands::Memory(action)) => {
            let code = memory::run(action.into_runtime());
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
