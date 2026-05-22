//! `bwoc help [topic]` — topic-specific help embedded in the binary.
//!
//! `--help` (clap-generated) covers per-flag mechanics. This complements
//! it: topical guides for concepts that span multiple commands —
//! backends, workspace layout, manifest fields, the arc, and a quick
//! getting-started cheatsheet.
//!
//! `bwoc help` with no topic lists what's available.

use std::io::{self, IsTerminal};

pub struct HelpArgs {
    pub topic: Option<String>,
}

const TOPICS: &[(&str, &str, &str)] = &[
    (
        "getting-started",
        "5-step quickstart from `bwoc init` to first spawn",
        GETTING_STARTED,
    ),
    (
        "backends",
        "The 4 declared backends and how to switch between them",
        BACKENDS,
    ),
    (
        "workspace",
        "Workspace layout: .bwoc/, agents/, projects/, notes/",
        WORKSPACE,
    ),
    ("manifest", "config.manifest.json field reference", MANIFEST),
    (
        "arc",
        "uppāda · ṭhiti · vaya — how commands map to the agent lifecycle",
        ARC,
    ),
];

pub fn run(args: HelpArgs) -> i32 {
    let topic = args.topic.as_deref();
    let colored = io::stdout().is_terminal();
    let c = Colors::for_tty(colored);

    let Some(name) = topic else {
        print_index(&c);
        return 0;
    };

    let Some((_, _, body)) = TOPICS.iter().find(|(t, _, _)| *t == name) else {
        eprintln!("bwoc help: unknown topic '{name}'");
        eprintln!();
        print_index(&c);
        return 2;
    };

    println!();
    println!("{}# {name}{}", c.bold_cyan, c.reset);
    println!();
    print!("{body}");
    println!();
    0
}

fn print_index(c: &Colors) {
    println!();
    println!("{}Available help topics:{}", c.bold_cyan, c.reset);
    println!();
    for (name, summary, _) in TOPICS {
        println!("  {}{name:<20}{}  {summary}", c.yellow, c.reset);
    }
    println!();
    println!(
        "{}Usage:{} bwoc help <topic>     (e.g. `bwoc help getting-started`)",
        c.dim, c.reset
    );
    println!(
        "{}      {} bwoc --help            for per-command flags",
        c.dim, c.reset
    );
    println!();
}

struct Colors {
    bold_cyan: &'static str,
    yellow: &'static str,
    dim: &'static str,
    reset: &'static str,
}

impl Colors {
    fn for_tty(on: bool) -> Self {
        if on {
            Self {
                bold_cyan: "\x1b[1;36m",
                yellow: "\x1b[33m",
                dim: "\x1b[2m",
                reset: "\x1b[0m",
            }
        } else {
            Self {
                bold_cyan: "",
                yellow: "",
                dim: "",
                reset: "",
            }
        }
    }
}

// --- topic content (markdown-flavored plain text) -------------------------

const GETTING_STARTED: &str = "\
1. Initialize a workspace
   $ mkdir my-workspace && cd my-workspace
   $ bwoc init

2. Incarnate your first agent (interactive — press Enter to accept defaults)
   $ bwoc new alpha
     agentRole       picker (default: code reviewer)
     primaryModel    picker per --backend (default: first model in catalog)
     lint/format/test/build cmds  stack-detected defaults

3. Verify backend neutrality
   $ bwoc check agents/agent-alpha

4. See what's registered + health
   $ bwoc list
   $ bwoc status

5. Spawn the configured backend in the agent's directory
   $ bwoc spawn --path agents/agent-alpha --backend claude

See also:
  bwoc help workspace    — what each directory means
  bwoc help backends     — switching between claude/gemini/codex/kimi
  bwoc help arc          — uppāda · ṭhiti · vaya mapping
  examples/howto/        — full walkthroughs
";

const BACKENDS: &str = "\
BWOC supports 4 declared backends (Samānattatā — equal treatment, no lock-in):

  | Backend  | CLI binary | Common models                                          |
  |----------|------------|--------------------------------------------------------|
  | Claude   | claude     | claude-opus-4-7, claude-sonnet-4-6, claude-haiku-4-5   |
  | Gemini   | gemini     | gemini-2.5-pro, gemini-2.5-flash, gemini-2.5-flash-lite|
  | Codex    | codex      | gpt-5, gpt-5-mini, o1                                  |
  | Kimi     | kimi       | kimi-k2, kimi-k1.5                                     |

Each agent picks ONE backend at incarnation, recorded in its
config.manifest.json (primaryModel + optional fallbackModel) and in
.bwoc/agents.toml.

All 4 read the SAME AGENTS.md via symlinks (CLAUDE.md / GEMINI.md /
CODEX.md / KIMI.md all → AGENTS.md). If your agent's instructions
assume a specific backend, `bwoc check` flags it as a neutrality
violation.

Three ways to set the backend:
  - At incarnation:   bwoc new my-agent --backend gemini
  - Manifest edit:    edit agents/<name>/config.manifest.json then update
                      .bwoc/agents.toml's `backend = \"...\"`
  - Per spawn:        bwoc spawn --path agents/<name> --backend kimi
                      (overrides for one session — useful for cross-
                       backend testing)

See: bwoc help manifest  — full manifest field reference
     examples/howto/configure-backends.md
";

const WORKSPACE: &str = "\
A BWOC workspace is any directory containing `.bwoc/workspace.toml`.

After `bwoc init`:

  my-workspace/
  ├── .bwoc/                  workspace config (do not delete)
  │   ├── workspace.toml      name, version, defaults
  │   └── agents.toml         registry of incarnated agents
  ├── agents/                 incarnated agents land here (per agents_dir)
  │   └── README.md           what agents/ is for
  ├── projects/               your work — apps, repos, libraries
  │   └── README.md           convention reminder
  └── notes/                  YYYY-MM-DD_<title>.md implementation logs
      └── README.md           naming convention

What's enforced (CLI requires):
  - .bwoc/workspace.toml      bwoc init creates; bwoc workspace validate checks
  - .bwoc/agents.toml         bwoc new appends; bwoc retire removes
  - agents/ or defaults.agents_dir  where new agents land

What's suggested (auto-created by bwoc init, you can rm or customize):
  - projects/                 your work
  - notes/                    implementation logs
  - README.md in each scaffold dir

What the CLI never touches:
  - Anything outside .bwoc/ and agents/ — your projects/ and notes/
    are yours
  - Files in an agent dir after incarnation — once bwoc new finishes,
    the agent's tree is yours to edit

Central per-user state (independent of any workspace):
  ~/.bwoc/
  ├── config.toml             per-user config (yours to edit)
  ├── template/               optional — extracted agent template cache
  └── memory/                 central per-user memory (Phase 2+)

Workspace resolution chain (for any cmd that needs to find one):
  --workspace <path>          explicit
  BWOC_WORKSPACE              env var
  ancestor walk for .bwoc/    from cwd upward
  cwd self-check              last resort
  exit 2 with hint            if none found

Validate the layout:
  bwoc workspace validate     — 5 spec rules (exit 0 clean, 2 violations)
  bwoc doctor                 — richer; with --auto, fixes safe issues
  bwoc workspace prune        — find drift between registry and disk
";

const MANIFEST: &str = "\
Every incarnated agent has `config.manifest.json` at its root.
Schema (resolved by `bwoc new`, written verbatim — no placeholders):

  Required (prompted if missing):
    name              kebab-case agent name (e.g. \"alpha\")
    agentId           always \"agent-{name}\" (auto-derived)
    agentRole         one-or-two-word role description
    primaryModel      LLM model identifier (e.g. \"claude-opus-4-7\")
    lintCmd           shell command for lint
    formatCmd         shell command for format check
    testCmd           shell command for tests
    buildCmd          shell command for build
    memoryPath        relative path to per-agent memory dir
                      (default: \"memories/\")
    version           manifest schema version (auto: \"2.0\")

  Optional (no prompt; pass --flag if you want them):
    fallbackModel     fallback LLM model identifier
    sessionsPath      session data dir for Tier 2 memory mining
    deepMemoryCmd     Tier 2 memory CLI command
    worktreeBase      base path for spawned worktrees

  Auto-managed (not prompted, not exposed as flag):
    requiredConfig    field descriptions (copied from the template
                      manifest so `bwoc new` can use them for prompts)

Where the values come from:
  - --flag                          highest precedence
  - interactive picker / default    if TTY and flag missing
  - non-TTY without flag            fail-fast with MissingFields error

See: bwoc new --help    — every flag listed
     docs/en/INCARNATION.en.md §\"Setting the Manifest\"
";

const ARC: &str = "\
BWOC models an agent's life as three phases (AN 3.47 Saṅkhata Sutta):

  uppāda    arising      — an agent is created
  ṭhiti     persisting   — an agent operates
  vaya      ceasing      — an agent ends

The CLI maps directly:

  Phase 1 v2.0 (uppāda, shipped):
    bwoc init <path>          establish the workspace
    bwoc new <name>            incarnate an agent
    bwoc check <path>          audit backend neutrality
    bwoc spawn --path <p>      bring the agent to life (exec the backend)

  Phase 2 (ṭhiti, in progress):
    bwoc status [name]         per-agent health snapshot (shipped)
    bwoc list                  registry view (shipped)
    bwoc workspace info        workspace overview (shipped)
    bwoc workspace validate    enforce layout rules (shipped)
    bwoc workspace prune       reconcile registry vs disk (shipped)
    bwoc doctor                env + workspace diagnostic (shipped)
    --- not yet ---
    bwoc-agent control socket  long-running process supervision
    bwoc log / send / stop     interact with a live agent

  Phase 3 (vaya, partially shipped):
    bwoc retire <name>         remove from registry (+ optional files) — shipped
    --- not yet ---
    bwoc stop <name>           graceful signal-escalation shutdown
    full vaya cleanup          worktree, branch, memory, interconnect

  Phase 4 (adoption — beyond code):
    Reference agents, fleet patterns, ecosystem growth.

Every command's purpose, in one phrase: \"which phase of the arc
does this push the agent into?\". Commands that don't fit are
infrastructure (init, doctor) — they configure the environment,
not the agent.

See: docs/en/ROADMAP.en.md         — full phase definitions
     docs/en/PHILOSOPHY.en.md §0.1  — the arc in detail
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_topic_has_nonempty_content() {
        for (name, summary, body) in TOPICS {
            assert!(!name.is_empty(), "topic name empty");
            assert!(!summary.is_empty(), "{name} summary empty");
            assert!(
                body.len() > 100,
                "{name} body too short ({} chars)",
                body.len()
            );
        }
    }

    #[test]
    fn topic_names_are_stable_slugs() {
        for (name, _, _) in TOPICS {
            for c in name.chars() {
                assert!(
                    c.is_ascii_lowercase() || c == '-',
                    "topic name '{name}' has invalid char '{c}'"
                );
            }
        }
    }

    #[test]
    fn unknown_topic_returns_exit_2() {
        let args = HelpArgs {
            topic: Some("nonexistent-zzz".to_string()),
        };
        // Can't easily assert exit code without capturing stdout/stderr,
        // but we can at least invoke without panicking.
        let code = run(args);
        assert_eq!(code, 2);
    }
}
