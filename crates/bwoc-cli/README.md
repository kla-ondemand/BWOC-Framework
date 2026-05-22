# bwoc-cli

The `bwoc` command-line tool — incarnate, check, spawn, and control [BWOC](../../README.md) agents.

Native single binary for **macOS · Linux · Windows**. Localized output (**TH · EN** shipping at launch; any future language is a folder drop).

## Install

**One command** (from a clone of the framework repo):

```bash
./scripts/install.sh
```

Or equivalently:

```bash
cargo install --path crates/bwoc-cli --locked
```

Both install the `bwoc` binary to `~/.cargo/bin/bwoc`. Requires a [Rust toolchain](https://rustup.rs/) on PATH.

## Usage

```bash
bwoc --help                          # show command surface + flags
bwoc --lang th                       # localized output (Thai)
bwoc --lang en                       # localized output (English)
```

### Language selection

Precedence: `--lang <code>` flag → `BWOC_LANG` env var → `$LANG` env var → `en` fallback.

```bash
BWOC_LANG=th bwoc                    # via env
LANG=th_TH.UTF-8 bwoc                # via POSIX locale
```

## Command surface (Phase 1 v2.0)

| Command | Arc phase | Status |
|---|---|---|
| `bwoc init [path]` | uppāda | scaffolding — implementation follows |
| `bwoc workspace info [path]` | — | scaffolding |
| `bwoc workspace validate [path]` | — | scaffolding |
| `bwoc new <name>` | uppāda | scaffolding — implementation follows |
| `bwoc check [path]` | uppāda | scaffolding — implementation follows |
| `bwoc spawn <name>` | uppāda → ṭhiti | scaffolding — minimal `exec` follows |
| `bwoc list` | ṭhiti | Phase 1 v2.0 (lists workspace `agents.toml`) |
| `bwoc status` / `log` / `send` | ṭhiti | Phase 2 |
| `bwoc stop` / `retire` | vaya | Phase 2 / 3 |

Arc phases are named per [`PHILOSOPHY.en.md` §0.1](../../modules/agent-template/docs/en/PHILOSOPHY.en.md#01-the-arc--uppāda--ṭhiti--vaya).

## Workspace flag

All operational commands accept `--workspace <path>` (or read `BWOC_WORKSPACE` env), falling back to the nearest ancestor of `cwd` that contains a `.bwoc/` marker, then to `cwd`, then refuse to run. Operational commands **validate the workspace first** and exit `2` with an actionable message if it is incomplete. See [`WORKSPACE.en.md`](../../docs/en/WORKSPACE.en.md).

## Adding a new locale

```bash
mkdir crates/bwoc-cli/locales/<lang>
# Copy keys from en/cli.ftl and translate values
```

No code change required.

## Status

**Phase 1 v2.0 — scaffold.** The `--lang` flag and locale loader work; command implementations land in follow-up iterations.

## License

[MIT](../../LICENSE).
