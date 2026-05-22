//! `bwoc memory list|show <name>` — workspace-level memory access.
//!
//! Reads `.bwoc/memory/` (scaffolded by `bwoc init`). Per the
//! WORKSPACE.en.md spec §"Central Memory", this is the per-workspace
//! tier — knowledge shared across all agents in the workspace.
//!
//! Two read commands ship now:
//!   - `bwoc memory list`            — names + sizes of *.md files
//!   - `bwoc memory show <name>`     — print one file's contents
//!
//! Write (`put`) is deliberately deferred — the directory is plain
//! Markdown, so `cat > .bwoc/memory/<name>.md` from any shell works
//! today. The CLI exists for *reading* (which agents need at runtime)
//! and for keeping users from having to remember the path layout.
//!
//! README.md inside the dir is intentionally exempted from `list`
//! output — it's slot-level documentation, not a memory entry.

use std::path::{Path, PathBuf};

pub struct MemoryArgs {
    pub action: MemoryAction,
    pub workspace: Option<PathBuf>,
    pub json: bool,
}

pub enum MemoryAction {
    List,
    Show(String),
}

pub fn run(args: MemoryArgs) -> i32 {
    let Some(workspace) = resolve_workspace(args.workspace) else {
        eprintln!(
            "bwoc memory: no workspace found (no .bwoc/workspace.toml in cwd or ancestors). \
             Pass --workspace, set BWOC_WORKSPACE, or run `bwoc init` first."
        );
        return 2;
    };
    let memory_dir = workspace.join(".bwoc/memory");
    if !memory_dir.is_dir() {
        eprintln!(
            "bwoc memory: {} doesn't exist. Run `bwoc init` again — older workspaces \
             may pre-date the memory scaffold; the directory can be created manually.",
            memory_dir.display()
        );
        return 2;
    }

    match args.action {
        MemoryAction::List => list(&memory_dir, args.json),
        MemoryAction::Show(name) => show(&memory_dir, &name),
    }
}

/// List user-authored memory entries. Skips `README.md` (the slot doc
/// scaffolded by `bwoc init`); only `.md` files counted.
fn list(memory_dir: &Path, json: bool) -> i32 {
    let mut entries: Vec<(String, u64)> = Vec::new();
    let Ok(read) = std::fs::read_dir(memory_dir) else {
        eprintln!("bwoc memory: failed to read {}", memory_dir.display());
        return 1;
    };
    for entry in read.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".md") || name == "README.md" {
            continue;
        }
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        entries.push((name, size));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    if json {
        let value = serde_json::json!({
            "workspace_memory_dir": memory_dir.display().to_string(),
            "entries": entries
                .iter()
                .map(|(n, s)| serde_json::json!({ "name": n, "size_bytes": s }))
                .collect::<Vec<_>>(),
        });
        match serde_json::to_string_pretty(&value) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("bwoc memory list --json: serialize failed: {e}");
                return 1;
            }
        }
        return 0;
    }

    println!();
    println!("Workspace memory: {}", memory_dir.display());
    println!();
    if entries.is_empty() {
        println!(
            "(no entries — drop .md files in {} to populate)",
            memory_dir.display()
        );
        println!();
        return 0;
    }
    println!("{:<40} SIZE", "NAME");
    println!("{} {}", "─".repeat(40), "─".repeat(10));
    for (name, size) in &entries {
        println!("{name:<40} {size} bytes");
    }
    println!();
    println!("Use `bwoc memory show <name>` to read one.");
    println!();
    0
}

/// Print one memory file's contents. `<name>` may be given with or
/// without the `.md` extension; we normalize.
fn show(memory_dir: &Path, name: &str) -> i32 {
    let filename = if name.ends_with(".md") {
        name.to_string()
    } else {
        format!("{name}.md")
    };
    // Refuse traversal — memory entries are flat files only.
    if filename.contains('/') || filename.contains('\\') || filename.starts_with('.') {
        eprintln!(
            "bwoc memory show: invalid name '{name}' — must be a flat *.md filename, \
             no path separators, no dot-prefix."
        );
        return 2;
    }
    let path = memory_dir.join(&filename);
    if !path.is_file() {
        eprintln!(
            "bwoc memory show: no entry named '{filename}' in {}. \
             Try `bwoc memory list`.",
            memory_dir.display()
        );
        return 2;
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            print!("{content}");
            // Ensure trailing newline so terminal prompt lands on a new line
            // even if the file doesn't end with one.
            if !content.ends_with('\n') {
                println!();
            }
            0
        }
        Err(e) => {
            eprintln!("bwoc memory show: failed to read {}: {e}", path.display());
            1
        }
    }
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
