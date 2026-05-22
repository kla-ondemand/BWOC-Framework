# How-To: Workspace Layout

## Goal

Understand where every directory in a BWOC workspace fits, what files live there, and what's enforced vs convention.

## Prerequisites

- A workspace created by `bwoc init` (see [`first-agent.md`](first-agent.md))

## The shape

After `bwoc init`, a workspace looks like this:

```
my-workspace/
├── .bwoc/                          # workspace config (do not delete)
│   ├── workspace.toml              # name, version, defaults
│   └── agents.toml                 # registry of incarnated agents
├── agents/                         # incarnated agents land here (per agents_dir)
│   └── README.md                   # what agents/ is for
├── projects/                       # your work — apps, repos, libraries
│   └── README.md                   # convention reminder
└── notes/                          # YYYY-MM-DD_<title>.md implementation logs
    └── README.md                   # naming convention
```

After incarnating one agent (`bwoc new alpha`), `agents/` looks like:

```
agents/
├── README.md
└── agent-alpha/
    ├── AGENTS.md                   # backend-neutral instruction set
    ├── CLAUDE.md → AGENTS.md       # backend symlinks (4 total)
    ├── GEMINI.md → AGENTS.md
    ├── CODEX.md  → AGENTS.md
    ├── KIMI.md   → AGENTS.md
    ├── config.manifest.json        # resolved manifest (machine-readable)
    ├── conventions.md              # naming + placeholder rules
    ├── neutrality.md               # backend-neutrality contract
    ├── persona/                    # identity, domains, boundaries
    ├── memories/                   # per-agent memory store
    ├── interconnect/               # capabilities, trust scoring (Phase 3)
    ├── mindsets/                   # SPEC.md
    └── skills/                     # SPEC.md
```

## What's enforced

The CLI **requires** these:

- `.bwoc/workspace.toml` — workspace marker; `bwoc init` creates it, `bwoc workspace validate` checks it
- `.bwoc/agents.toml` — agents registry; `bwoc new` appends, `bwoc retire` removes
- `agents/` (or `defaults.agents_dir`) — where new agents land

The CLI **suggests** these (auto-created by `bwoc init`, but you can rm/customize):

- `projects/` — your work
- `notes/` — implementation logs
- `README.md` in each scaffold dir

The CLI **never touches** these:

- Anything outside `.bwoc/` and `agents/` — your `projects/`, `notes/`, and any custom dirs are yours
- Files in an agent dir after incarnation — once `bwoc new` finishes, the agent's tree is yours to edit

## Central memory: `~/.bwoc/`

Independent of any workspace:

```
~/.bwoc/
├── config.toml                     # per-user config (yours to edit)
├── template/                       # optional — extracted agent template cache
└── memory/                         # central per-user memory (Phase 2+)
```

`bwoc` auto-creates `~/.bwoc/` + an empty `config.toml` on first invocation. The other entries are populated on demand by specific commands.

## Validate the layout

```bash
bwoc workspace validate
```

Runs five rules from `docs/en/WORKSPACE.en.md` §Validation:

1. `.bwoc/` exists
2. `workspace.toml` parses + has required fields
3. `version` is parseable SemVer (`X.Y.Z`)
4. `agents.toml` parses
5. `agents_dir` exists

Exit 0 = complete; exit 2 = violations.

```bash
bwoc doctor
```

A richer check that also looks at `~/.bwoc/`, backend CLIs on PATH, scaffold dir READMEs, and per-agent backend symlinks. With `--auto`, fixes the safe issues (missing dirs, missing symlinks).

## What's next

- [`docs/en/WORKSPACE.en.md`](../../docs/en/WORKSPACE.en.md) — formal spec
- [`docs/en/NAMING.en.md`](../../docs/en/NAMING.en.md) — file naming convention across the framework
- [`diagnose-and-fix.md`](diagnose-and-fix.md) — how `bwoc doctor` repairs common issues
