//! Startup banner for `bwoc` when invoked with no subcommand.
//!
//! ANSI Shadow wordmark + version line + sections listing available
//! commands, backends, and locales. Colors only fire on a real TTY
//! (detected via `std::io::IsTerminal`) so piped/CI output stays clean.
//!
//! Lean by design — no `crossterm`/`owo-colors`/`ratatui` dep. If we
//! ever grow to a full interactive TUI (panels, keybindings), promote
//! at that point.

use std::io::{self, IsTerminal};

const VERSION: &str = env!("CARGO_PKG_VERSION");

const TAGLINE: &str = "Buddhist Way of Coding — an engineering framework for AI coding agents.";
const REPO_URL: &str = "https://github.com/bemindlabs/BWOC-Framework";

// ANSI Shadow font for "BWOC". 6 rows tall, ~36 cols wide.
const WORDMARK: &str = "\
 ██████╗ ██╗    ██╗ ██████╗  ██████╗\n\
 ██╔══██╗██║    ██║██╔═══██╗██╔════╝\n\
 ██████╔╝██║ █╗ ██║██║   ██║██║     \n\
 ██╔══██╗██║███╗██║██║   ██║██║     \n\
 ██████╔╝╚███╔███╔╝╚██████╔╝╚██████╗\n\
 ╚═════╝  ╚══╝╚══╝  ╚═════╝  ╚═════╝";

const SUBTITLE: &str = "                            Framework";

const COMMANDS: &[(&str, &str)] = &[
    ("init", "Create a BWOC workspace at the given path"),
    ("new", "Incarnate a new agent from the template"),
    ("check", "Audit an agent for backend neutrality"),
    (
        "spawn",
        "Run the configured LLM backend in an agent's directory",
    ),
    ("list", "List agents registered in the workspace"),
    ("workspace", "Inspect the workspace (info, validate, prune)"),
    (
        "doctor",
        "Diagnose environment + workspace, --auto to fix safe issues",
    ),
    ("retire", "Retire an agent — remove from registry (vaya)"),
    ("status", "Per-agent health + identity snapshot (read-only)"),
    (
        "help",
        "Topic-specific help (backends, workspace, manifest, arc, getting-started)",
    ),
    (
        "completion",
        "Emit a shell completion script (bash, zsh, fish, powershell, elvish)",
    ),
    (
        "dashboard",
        "Interactive TUI — agents list with navigation + refresh",
    ),
    (
        "stop",
        "Pause an agent — set status = stopped without removing files",
    ),
    ("start", "Reactivate a stopped agent — set status = active"),
    (
        "ping",
        "Ping a --serve'd agent over its Unix socket (PING → PONG)",
    ),
];

const BACKENDS: &str = "claude · gemini · codex · kimi";
const LOCALES: &str = "en · th";

/// Print the banner to stdout. Honours TTY/non-TTY for color output.
pub fn print() {
    let c = if io::stdout().is_terminal() {
        Colors::ansi()
    } else {
        Colors::none()
    };

    println!();
    println!("{}{WORDMARK}{}", c.bold_yellow, c.reset);
    println!("{}{SUBTITLE}{}", c.bold, c.reset);
    println!();
    println!(
        "  {}BWOC Framework{} v{VERSION}  —  {TAGLINE}",
        c.bold, c.reset
    );
    println!();
    println!("{}Available Commands:{}", c.bold_cyan, c.reset);
    for (name, desc) in COMMANDS {
        println!("  {}{name:<10}{}  {desc}", c.yellow, c.reset);
    }
    println!();
    println!("{}Backends:{}  {BACKENDS}", c.bold_cyan, c.reset);
    println!("{}Locales:{}   {LOCALES}", c.bold_cyan, c.reset);
    println!();
    println!(
        "{}Hint:{} run `bwoc <command> --help` for details.",
        c.dim, c.reset
    );
    println!("{}Repo:{} {REPO_URL}{}", c.dim, c.dim, c.reset);
    println!();
}

/// Bundle of ANSI escapes. `none()` returns empty strings so the same
/// `println!` calls work uncolored in pipes and CI.
struct Colors {
    bold: &'static str,
    dim: &'static str,
    bold_yellow: &'static str,
    bold_cyan: &'static str,
    yellow: &'static str,
    reset: &'static str,
}

impl Colors {
    fn ansi() -> Self {
        Self {
            bold: "\x1b[1m",
            dim: "\x1b[2m",
            bold_yellow: "\x1b[1;33m",
            bold_cyan: "\x1b[1;36m",
            yellow: "\x1b[33m",
            reset: "\x1b[0m",
        }
    }
    fn none() -> Self {
        Self {
            bold: "",
            dim: "",
            bold_yellow: "",
            bold_cyan: "",
            yellow: "",
            reset: "",
        }
    }
}
