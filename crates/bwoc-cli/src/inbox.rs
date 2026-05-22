//! `bwoc inbox <agent>` — read companion to `bwoc send`.
//!
//! Reads `<agent>/.bwoc/inbox.jsonl` (JSON-lines format spec'd by
//! `send.rs`), parses each line as one envelope, and prints them in
//! send order. Malformed lines emit a warning to stderr but don't
//! abort — the rest of the inbox is still readable.

use std::io::BufRead;
use std::path::PathBuf;

use bwoc_core::workspace::AgentsRegistry;

pub struct InboxArgs {
    /// Empty when `--all`. The CLI shim enforces "exactly one of agent | all".
    pub agent: String,
    pub workspace: Option<PathBuf>,
    pub json: bool,
    pub limit: Option<usize>,
    /// Tail mode: print historical messages then block, printing new
    /// envelopes as they arrive. Exit with Ctrl-C.
    pub watch: bool,
    /// Truncate the inbox after printing (acknowledge messages). TTY
    /// prompts unless `yes` is also set.
    pub clear: bool,
    /// Skip the interactive confirmation for `clear`. Required for non-TTY.
    pub yes: bool,
    /// Print just the message count (one integer) instead of envelopes.
    /// Useful for shell scripts:
    ///   `if [ $(bwoc inbox alpha --count) -gt 0 ]; then ...`
    /// With `--json`, emits `{"count": N}`.
    pub count: bool,
    /// Print every agent's inbox concatenated, each preceded by a
    /// `=== <agent-id> (N message(s)) ===` header. Mutually exclusive
    /// with `agent`. `--clear` and `--watch` are refused with `--all`
    /// (no plausible safe semantics: mass-clear is too destructive;
    /// mass-watch would interleave updates from many agents).
    pub all: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum InboxError {
    #[error(
        "no workspace found (no .bwoc/workspace.toml in cwd or ancestors). \
         Pass --workspace, set BWOC_WORKSPACE, or run `bwoc init` first."
    )]
    NoWorkspace,
    #[error("no agent named '{name}' in workspace {workspace}")]
    NotFound { name: String, workspace: PathBuf },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("workspace error: {0}")]
    Workspace(#[from] bwoc_core::workspace::WorkspaceError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn run(args: InboxArgs) -> i32 {
    match inbox(args) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("bwoc inbox: {e}");
            match e {
                InboxError::NoWorkspace | InboxError::NotFound { .. } => 2,
                _ => 1,
            }
        }
    }
}

fn inbox(args: InboxArgs) -> Result<(), InboxError> {
    let workspace = resolve_workspace(args.workspace).ok_or(InboxError::NoWorkspace)?;
    let registry = AgentsRegistry::load(&workspace)?;

    if args.all {
        if args.clear || args.watch {
            eprintln!(
                "bwoc inbox --all: --clear and --watch are not supported with --all \
                 (use `bwoc inbox <name> --clear` per agent, or `bwoc list --inbox-pending` \
                 to find candidates)."
            );
            return Ok(());
        }
        return inbox_all(&workspace, &registry, args.json, args.limit);
    }

    let lookup_id = if args.agent.starts_with("agent-") {
        args.agent.clone()
    } else {
        format!("agent-{}", args.agent)
    };
    let entry = registry
        .agents
        .iter()
        .find(|a| a.id == lookup_id)
        .ok_or_else(|| InboxError::NotFound {
            name: args.agent.clone(),
            workspace: workspace.clone(),
        })?;

    let inbox_path = workspace.join(&entry.path).join(".bwoc/inbox.jsonl");
    let messages = read_messages(&inbox_path)?;

    // --count short-circuit: just the integer, before --limit/--json
    // shape any view. Filters are no-ops on a count anyway.
    if args.count {
        if args.json {
            let value = serde_json::json!({ "count": messages.len() });
            println!("{}", serde_json::to_string(&value)?);
        } else {
            println!("{}", messages.len());
        }
        return Ok(());
    }

    // Apply --limit (last N).
    let view: Vec<&serde_json::Value> = if let Some(n) = args.limit {
        messages.iter().rev().take(n).rev().collect()
    } else {
        messages.iter().collect()
    };

    if args.json && !args.watch {
        // Snapshot mode — pretty-printed full envelope set.
        let value = serde_json::json!({
            "agent": entry.id,
            "inbox": inbox_path.display().to_string(),
            "total": messages.len(),
            "shown": view.len(),
            "messages": view,
        });
        println!("{}", serde_json::to_string_pretty(&value)?);
        return Ok(());
    }
    if args.json && args.watch {
        // Streaming mode — emit historical envelopes (last `view`) as
        // compact JSON lines, then block tailing for new ones (also
        // emitted as JSON lines). Consumers can `jq -c '.'` per line.
        for m in &view {
            println!("{}", serde_json::to_string(m)?);
        }
        return watch_inbox_json(&inbox_path);
    }

    println!();
    println!("Inbox for {}: {} message(s)", entry.id, messages.len());
    if let Some(n) = args.limit {
        if view.len() < messages.len() {
            println!("(showing last {} of {})", view.len(), messages.len());
        } else {
            let _ = n; // limit larger than count; suppress redundant note
        }
    }
    println!("Source: {}", inbox_path.display());
    println!();
    if view.is_empty() {
        println!("(no messages)");
        println!();
        return Ok(());
    }
    for (i, m) in view.iter().enumerate() {
        print_envelope(i + 1, m);
    }

    // --clear is incompatible with --watch (one drains, the other listens
    // forever — combining them is almost certainly a mistake). Reject
    // explicitly so the user picks one.
    if args.clear && args.watch {
        eprintln!("bwoc inbox: --clear and --watch can't be combined (clear drains; watch waits)");
        return Ok(());
    }

    if args.clear {
        clear_inbox(&inbox_path, messages.len(), args.yes)?;
    }

    if args.watch {
        watch_inbox(&inbox_path, messages.len())?;
    }
    Ok(())
}

/// Truncate the inbox file after the user confirms. The daemon's
/// `check_inbox_for_new` already handles file truncation gracefully —
/// it resets its cursor on shrink, so a live daemon won't crash or
/// re-announce pre-clear messages.
fn clear_inbox(inbox_path: &std::path::Path, count: usize, yes: bool) -> Result<(), InboxError> {
    use std::io::{IsTerminal, Write};

    if count == 0 {
        println!("(nothing to clear)");
        return Ok(());
    }
    if !yes {
        if !std::io::stdin().is_terminal() {
            eprintln!("bwoc inbox --clear: non-TTY without --yes — aborted");
            return Ok(());
        }
        print!(
            "Delete {count} message(s) from {}? [y/N]: ",
            inbox_path.display()
        );
        std::io::stdout().flush()?;
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        let answer = line.trim().to_ascii_lowercase();
        if answer != "y" && answer != "yes" {
            println!("(aborted — nothing changed)");
            return Ok(());
        }
    }
    // Truncate to zero by re-opening with truncate=true. Simpler than
    // remove+create — preserves the file inode so a watching daemon's
    // open handle (if any) stays valid in the truncation case.
    std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(inbox_path)?;
    println!("Cleared {count} message(s) from {}.", inbox_path.display());
    Ok(())
}

/// Print one envelope as a numbered block (one ts/from header, then
/// indented message body — multi-line aware).
fn print_envelope(idx: usize, m: &serde_json::Value) {
    let ts = m.get("ts").and_then(|v| v.as_str()).unwrap_or("—");
    let from = m.get("from").and_then(|v| v.as_str()).unwrap_or("—");
    let msg = m
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("(no message)");
    println!("  [{idx}]  {ts}  ←  {from}");
    for line in msg.lines() {
        println!("        {line}");
    }
    println!();
}

/// Tail mode — block reading new envelopes from `inbox_path` past the
/// last known count. Exit with Ctrl-C (default SIGINT terminates the
/// process; no graceful state to flush since this is read-only).
/// Print every agent's inbox concatenated. Per-agent header is
/// `=== <agent-id> (N message(s)) ===`. Empty inboxes still get a
/// header (so the user knows the agent was inspected — silent skipping
/// is more confusing than a "0 message(s)" line).
///
/// `--json` shape:
///   {
///     "workspace": "<path>",
///     "agents": [
///       { "agent": "agent-foo", "inbox": "<path>", "total": N,
///         "shown": M, "messages": [...] },
///       ...
///     ]
///   }
fn inbox_all(
    workspace: &std::path::Path,
    registry: &AgentsRegistry,
    json: bool,
    limit: Option<usize>,
) -> Result<(), InboxError> {
    if registry.agents.is_empty() {
        if json {
            let value = serde_json::json!({
                "workspace": workspace.display().to_string(),
                "agents": [],
            });
            println!("{}", serde_json::to_string_pretty(&value)?);
        } else {
            println!(
                "bwoc inbox --all: no agents registered in {}.",
                workspace.display()
            );
        }
        return Ok(());
    }

    if json {
        let mut per_agent: Vec<serde_json::Value> = Vec::with_capacity(registry.agents.len());
        for entry in &registry.agents {
            let inbox_path = workspace.join(&entry.path).join(".bwoc/inbox.jsonl");
            let messages = read_messages(&inbox_path)?;
            let view: Vec<&serde_json::Value> = if let Some(n) = limit {
                messages.iter().rev().take(n).rev().collect()
            } else {
                messages.iter().collect()
            };
            per_agent.push(serde_json::json!({
                "agent": entry.id,
                "inbox": inbox_path.display().to_string(),
                "total": messages.len(),
                "shown": view.len(),
                "messages": view,
            }));
        }
        let value = serde_json::json!({
            "workspace": workspace.display().to_string(),
            "agents": per_agent,
        });
        println!("{}", serde_json::to_string_pretty(&value)?);
        return Ok(());
    }

    // Human mode. One section per agent.
    for entry in &registry.agents {
        let inbox_path = workspace.join(&entry.path).join(".bwoc/inbox.jsonl");
        let messages = read_messages(&inbox_path)?;
        let view: Vec<&serde_json::Value> = if let Some(n) = limit {
            messages.iter().rev().take(n).rev().collect()
        } else {
            messages.iter().collect()
        };
        println!();
        println!("=== {} ({} message(s)) ===", entry.id, messages.len());
        if messages.is_empty() {
            continue;
        }
        for v in &view {
            let ts = v.get("ts").and_then(|x| x.as_str()).unwrap_or("?");
            let from = v.get("from").and_then(|x| x.as_str()).unwrap_or("?");
            let msg = v.get("message").and_then(|x| x.as_str()).unwrap_or("?");
            println!("[{ts}] {from}: {msg}");
        }
    }
    println!();
    Ok(())
}

/// JSON streaming tail. Same poll loop as `watch_inbox`, but each new
/// envelope is printed as a single compact-JSON line — no header, no
/// numbering, no decoration. Designed for `bwoc inbox alpha --watch
/// --json | jq -c '.'` or piping into a log aggregator.
fn watch_inbox_json(inbox_path: &std::path::Path) -> Result<(), InboxError> {
    use std::time::Duration;
    let mut offset: u64 = std::fs::metadata(inbox_path).map(|m| m.len()).unwrap_or(0);
    loop {
        let Ok(meta) = std::fs::metadata(inbox_path) else {
            std::thread::sleep(Duration::from_millis(300));
            continue;
        };
        let size = meta.len();
        if size < offset {
            offset = size;
            std::thread::sleep(Duration::from_millis(300));
            continue;
        }
        if size == offset {
            std::thread::sleep(Duration::from_millis(300));
            continue;
        }
        use std::io::{Read, Seek, SeekFrom};
        let mut file = std::fs::File::open(inbox_path)?;
        file.seek(SeekFrom::Start(offset))?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        let mut consumed: u64 = 0;
        for line in buf.split_inclusive('\n') {
            if !line.ends_with('\n') {
                break;
            }
            let trimmed = line.trim();
            consumed += line.len() as u64;
            if trimmed.is_empty() {
                continue;
            }
            // Pass through verbatim if it parses as JSON; otherwise emit
            // a one-shot error envelope so consumers see something rather
            // than silently dropping malformed lines.
            match serde_json::from_str::<serde_json::Value>(trimmed) {
                Ok(v) => println!("{}", serde_json::to_string(&v)?),
                Err(e) => {
                    let err = serde_json::json!({
                        "error": "malformed_envelope",
                        "detail": e.to_string(),
                        "raw": trimmed,
                    });
                    println!("{}", serde_json::to_string(&err)?);
                }
            }
        }
        offset += consumed;
    }
}

fn watch_inbox(inbox_path: &std::path::Path, mut idx: usize) -> Result<(), InboxError> {
    use std::time::Duration;

    println!("(watching for new messages — Ctrl-C to stop)");
    println!();

    // Track byte offset rather than line count — robust to lines being
    // partially flushed. Starts at current EOF so we don't re-print the
    // historical view.
    let mut offset: u64 = std::fs::metadata(inbox_path).map(|m| m.len()).unwrap_or(0);

    loop {
        let Ok(meta) = std::fs::metadata(inbox_path) else {
            // File might be missing if no one has sent yet — keep polling.
            std::thread::sleep(Duration::from_millis(300));
            continue;
        };
        let size = meta.len();
        if size < offset {
            // Truncation — reset.
            eprintln!("(inbox truncated; resuming from new EOF)");
            offset = size;
            std::thread::sleep(Duration::from_millis(300));
            continue;
        }
        if size == offset {
            std::thread::sleep(Duration::from_millis(300));
            continue;
        }

        // Read past-offset and print complete lines.
        use std::io::{Read, Seek, SeekFrom};
        let mut file = std::fs::File::open(inbox_path)?;
        file.seek(SeekFrom::Start(offset))?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        let mut consumed: u64 = 0;
        for line in buf.split_inclusive('\n') {
            if !line.ends_with('\n') {
                break; // partial — wait for the rest
            }
            let trimmed = line.trim();
            consumed += line.len() as u64;
            if trimmed.is_empty() {
                continue;
            }
            match serde_json::from_str::<serde_json::Value>(trimmed) {
                Ok(v) => {
                    idx += 1;
                    print_envelope(idx, &v);
                }
                Err(e) => {
                    eprintln!("bwoc inbox: warning — malformed JSON skipped ({e})");
                }
            }
        }
        offset += consumed;
    }
}

/// Read the inbox file line-by-line. Malformed lines warn-and-skip;
/// missing file is treated as an empty inbox.
fn read_messages(path: &std::path::Path) -> Result<Vec<serde_json::Value>, InboxError> {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    let reader = std::io::BufReader::new(file);
    let mut out = Vec::new();
    for (lineno, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<serde_json::Value>(&line) {
            Ok(v) => out.push(v),
            Err(e) => {
                eprintln!(
                    "bwoc inbox: warning — line {} is malformed JSON, skipped ({e})",
                    lineno + 1
                );
            }
        }
    }
    Ok(out)
}

fn resolve_workspace(explicit: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(p) = explicit {
        return Some(p);
    }
    if let Ok(env_path) = std::env::var("BWOC_WORKSPACE") {
        if !env_path.is_empty() {
            return Some(PathBuf::from(env_path));
        }
    }
    let mut cur = std::env::current_dir().ok()?;
    loop {
        if cur.join(".bwoc/workspace.toml").is_file() {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bwoc_core::workspace::{
        AgentEntry, AgentsRegistry, Workspace, WorkspaceDefaults, WorkspaceMeta,
    };
    use std::fs;
    use std::io::Write;

    fn setup(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("bwoc-inbox-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".bwoc")).unwrap();
        fs::create_dir_all(root.join("agents/agent-alpha/.bwoc")).unwrap();
        Workspace {
            workspace: WorkspaceMeta {
                name: label.into(),
                version: "0.1.0".into(),
                created: "2026-05-22T00:00:00Z".into(),
            },
            defaults: WorkspaceDefaults::default(),
        }
        .save(&root)
        .unwrap();
        let mut reg = AgentsRegistry::default();
        reg.agents.push(AgentEntry {
            id: "agent-alpha".into(),
            path: "agents/agent-alpha".into(),
            backend: "claude".into(),
            incarnated: "2026-05-22T00:00:00Z".into(),
            status: "active".into(),
        });
        reg.save(&root).unwrap();
        root
    }

    fn write_inbox(root: &std::path::Path, lines: &[&str]) {
        let p = root.join("agents/agent-alpha/.bwoc/inbox.jsonl");
        let mut f = fs::File::create(p).unwrap();
        for l in lines {
            writeln!(f, "{l}").unwrap();
        }
    }

    #[test]
    fn read_skips_malformed_lines_but_keeps_valid_ones() {
        let root = setup("malformed");
        write_inbox(
            &root,
            &[
                r#"{"ts":"t1","from":"user","to":"agent-alpha","message":"hello"}"#,
                "this is not json",
                r#"{"ts":"t2","from":"user","to":"agent-alpha","message":"world"}"#,
            ],
        );
        let msgs = read_messages(&root.join("agents/agent-alpha/.bwoc/inbox.jsonl")).unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0]["message"], "hello");
        assert_eq!(msgs[1]["message"], "world");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_inbox_is_empty_not_error() {
        let root = setup("missing");
        let msgs = read_messages(&root.join("agents/agent-alpha/.bwoc/inbox.jsonl")).unwrap();
        assert!(msgs.is_empty());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn limit_returns_last_n() {
        // Direct slice-test of the limit logic (the run loop does it
        // identically). 5 messages, limit 2 → last 2.
        let msgs: Vec<serde_json::Value> = (0..5)
            .map(|i| serde_json::json!({ "message": format!("m{i}") }))
            .collect();
        let view: Vec<&serde_json::Value> = msgs.iter().rev().take(2).rev().collect();
        assert_eq!(view.len(), 2);
        assert_eq!(view[0]["message"], "m3");
        assert_eq!(view[1]["message"], "m4");
    }
}
