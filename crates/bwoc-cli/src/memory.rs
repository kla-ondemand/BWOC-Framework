//! `bwoc memory list|show|put|search` — workspace-level memory access.
//!
//! Reads + writes `.bwoc/memory/` (scaffolded by `bwoc init`). Per the
//! WORKSPACE.en.md spec §"Central Memory", this is the per-workspace
//! tier — knowledge shared across all agents in the workspace.
//!
//! Commands:
//!   - `bwoc memory list`              — names + sizes (table or `--json`)
//!   - `bwoc memory show <name>`       — print one file's contents
//!   - `bwoc memory put <name>`        — write from `--file` or stdin
//!   - `bwoc memory search <query>`    — substring match across entries
//!
//! All commands operate strictly inside `.bwoc/memory/`. Name traversal
//! (`/`, `\`, leading `.`) is refused before any file-system access.
//! Atomic write (stage to `.tmp` → rename) so a failed write never
//! leaves half-state. `put` refuses overwrite without `--force`.
//!
//! README.md inside the dir is intentionally exempted from `list` and
//! `search` — it's slot-level documentation, not a memory entry.

use std::path::{Path, PathBuf};

pub struct MemoryArgs {
    pub action: MemoryAction,
    pub workspace: Option<PathBuf>,
    pub json: bool,
}

pub enum MemoryAction {
    List,
    Show(String),
    /// Write an entry. `source` is the content stream; `force` permits overwrite.
    Put {
        name: String,
        source: PutSource,
        force: bool,
    },
    /// Substring search across all memory entries. Case-insensitive.
    Search(String),
}

/// Where `put` reads the new entry's content from.
pub enum PutSource {
    /// Read from a file on disk. Useful for `bwoc memory put name --file ./scratch.md`.
    FilePath(PathBuf),
    /// Read from stdin until EOF. Useful for here-docs and pipes.
    Stdin,
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
        MemoryAction::Put {
            name,
            source,
            force,
        } => put(&memory_dir, &name, source, force),
        MemoryAction::Search(query) => search(&memory_dir, &query, args.json),
    }
}

/// Substring search across all `.md` entries (excluding README.md). Prints
/// `<name>:<line>:<content>` per match in human mode; structured shape in
/// `--json`. Case-insensitive. Exit 0 even when no matches (an empty
/// search isn't an error — `grep`'s pattern).
fn search(memory_dir: &Path, query: &str, json: bool) -> i32 {
    let needle = query.to_lowercase();
    let Ok(read) = std::fs::read_dir(memory_dir) else {
        eprintln!(
            "bwoc memory search: failed to read {}",
            memory_dir.display()
        );
        return 1;
    };

    let mut hits: Vec<(String, usize, String)> = Vec::new();
    let mut entry_paths: Vec<(String, PathBuf)> = read
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if !name.ends_with(".md") || name == "README.md" {
                return None;
            }
            Some((name, e.path()))
        })
        .collect();
    entry_paths.sort_by(|a, b| a.0.cmp(&b.0));

    for (name, path) in &entry_paths {
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        for (idx, line) in content.lines().enumerate() {
            if line.to_lowercase().contains(&needle) {
                hits.push((name.clone(), idx + 1, line.to_string()));
            }
        }
    }

    if json {
        let value = serde_json::json!({
            "query": query,
            "hits": hits
                .iter()
                .map(|(n, l, s)| serde_json::json!({
                    "name": n,
                    "line": l,
                    "content": s,
                }))
                .collect::<Vec<_>>(),
        });
        match serde_json::to_string_pretty(&value) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("bwoc memory search --json: serialize failed: {e}");
                return 1;
            }
        }
        return 0;
    }

    if hits.is_empty() {
        println!("(no matches for '{query}' in {})", memory_dir.display());
        return 0;
    }
    println!();
    for (name, line, content) in &hits {
        println!("{name}:{line}: {content}");
    }
    println!();
    println!(
        "{} match{} for '{query}'.",
        hits.len(),
        if hits.len() == 1 { "" } else { "es" },
    );
    println!();
    0
}

/// Write a memory entry. Refuses traversal patterns + dot-prefix
/// (same rule as `show`). Refuses overwrite without `--force`. Source
/// is either a file path or stdin; on EOF the content is written to
/// `.bwoc/memory/<name>.md` atomically (write to temp + rename, so a
/// failed write doesn't leave a half-written file).
fn put(memory_dir: &Path, name: &str, source: PutSource, force: bool) -> i32 {
    let filename = if name.ends_with(".md") {
        name.to_string()
    } else {
        format!("{name}.md")
    };
    if filename.contains('/') || filename.contains('\\') || filename.starts_with('.') {
        eprintln!(
            "bwoc memory put: invalid name '{name}' — must be a flat *.md filename, \
             no path separators, no dot-prefix."
        );
        return 2;
    }
    let target = memory_dir.join(&filename);
    if target.exists() && !force {
        eprintln!(
            "bwoc memory put: '{filename}' already exists at {}. \
             Re-run with --force to overwrite.",
            target.display()
        );
        return 2;
    }

    // Read the content.
    let content = match source {
        PutSource::FilePath(p) => match std::fs::read_to_string(&p) {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "bwoc memory put: failed to read source file {}: {e}",
                    p.display()
                );
                return 1;
            }
        },
        PutSource::Stdin => {
            use std::io::Read;
            let mut buf = String::new();
            if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
                eprintln!("bwoc memory put: failed to read stdin: {e}");
                return 1;
            }
            if buf.is_empty() {
                eprintln!(
                    "bwoc memory put: stdin was empty — pipe content in, e.g. \
                     `echo 'team-style: 2-space indent' | bwoc memory put team-style`."
                );
                return 2;
            }
            buf
        }
    };

    // Atomic write: stage to a sibling .tmp, then rename.
    let tmp = target.with_extension("md.tmp");
    if let Err(e) = std::fs::write(&tmp, &content) {
        eprintln!("bwoc memory put: failed to stage {}: {e}", tmp.display());
        return 1;
    }
    if let Err(e) = std::fs::rename(&tmp, &target) {
        eprintln!(
            "bwoc memory put: failed to install {}: {e}",
            target.display()
        );
        let _ = std::fs::remove_file(&tmp);
        return 1;
    }

    println!(
        "Wrote {} ({} byte{}).",
        target.display(),
        content.len(),
        if content.len() == 1 { "" } else { "s" }
    );
    0
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
