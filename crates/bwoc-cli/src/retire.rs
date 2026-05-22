//! `bwoc retire <name>` — the complement to `bwoc new`. Phase 3 vaya
//! starter: remove an agent's registry entry and optionally delete its
//! directory. Full Phase 3 vaya (worktree cleanup, branch release,
//! memory pruning, interconnect deregistration) lands later.
//!
//! Resolution: the agent is looked up in the enclosing workspace's
//! `.bwoc/agents.toml` by `id` (the `agent-<name>` form). Failing that,
//! by `name` alone (so `bwoc retire foo` matches the entry whose id is
//! `agent-foo`).
//!
//! Safety: interactive confirmation in TTY (any non-`y`/`yes` aborts).
//! `--yes` skips the confirmation for scripts. `--keep-files` keeps
//! the agent directory on disk and only removes the registry entry.

use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

use bwoc_core::workspace::AgentsRegistry;

pub struct RetireArgs {
    pub name: String,
    pub workspace: Option<PathBuf>,
    pub yes: bool,
    /// Preserve the entire agent directory; remove only the registry entry.
    pub keep_files: bool,
    /// Preserve just `memories/` (and the parent dir scaffold); remove
    /// everything else. Lets users retire an agent while keeping the
    /// knowledge it accumulated. Mutually exclusive with `keep_files`.
    pub keep_memory: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum RetireError {
    #[error(
        "no workspace found (no .bwoc/workspace.toml in cwd or ancestors). \
             Pass --workspace, set BWOC_WORKSPACE, or run `bwoc init` first."
    )]
    NoWorkspace,
    #[error("no agent named '{name}' in workspace {workspace}")]
    NotFound { name: String, workspace: PathBuf },
    #[error("aborted by user")]
    Aborted,
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("workspace error: {0}")]
    Workspace(#[from] bwoc_core::workspace::WorkspaceError),
}

pub fn run(args: RetireArgs) -> i32 {
    match retire(args) {
        Ok(()) => 0,
        Err(RetireError::Aborted) => {
            eprintln!("bwoc retire: aborted — nothing changed");
            // Aborted is a clean user decision, not an error.
            2
        }
        Err(e) => {
            eprintln!("bwoc retire: {e}");
            match e {
                RetireError::NoWorkspace | RetireError::NotFound { .. } => 2,
                _ => 1,
            }
        }
    }
}

fn retire(args: RetireArgs) -> Result<(), RetireError> {
    let workspace = resolve_workspace(args.workspace).ok_or(RetireError::NoWorkspace)?;
    let mut registry = AgentsRegistry::load(&workspace)?;

    // Match by id first ("agent-foo"), then by name ("foo" → agent-foo).
    let lookup_id = if args.name.starts_with("agent-") {
        args.name.clone()
    } else {
        format!("agent-{}", args.name)
    };
    let idx = registry
        .agents
        .iter()
        .position(|a| a.id == lookup_id)
        .ok_or_else(|| RetireError::NotFound {
            name: args.name.clone(),
            workspace: workspace.clone(),
        })?;

    let entry = registry.agents[idx].clone();
    let agent_path = workspace.join(&entry.path);

    // Confirmation.
    println!();
    println!("About to retire agent:");
    println!("  id:       {}", entry.id);
    println!("  path:     {} (relative to workspace)", entry.path);
    println!("  backend:  {}", entry.backend);
    println!("  status:   {}", entry.status);
    println!();
    if args.keep_files {
        println!("Keeping files on disk; removing only the registry entry.");
    } else if args.keep_memory {
        println!(
            "Keeping just memories/; removing everything else under: {}",
            agent_path.display()
        );
    } else {
        println!("This will DELETE the directory: {}", agent_path.display());
    }
    println!();

    if !args.yes {
        if !io::stdin().is_terminal() {
            // Non-TTY without --yes: refuse to delete silently.
            return Err(RetireError::Aborted);
        }
        let mut stdout = io::stdout();
        write!(stdout, "Proceed? [y/N]: ")?;
        stdout.flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let answer = line.trim().to_ascii_lowercase();
        if answer != "y" && answer != "yes" {
            return Err(RetireError::Aborted);
        }
    }

    // 1. File handling:
    //    --keep-files   → leave dir intact
    //    --keep-memory  → remove all top-level entries EXCEPT memories/
    //    (default)      → remove the whole agent dir
    if !args.keep_files && agent_path.exists() {
        if args.keep_memory {
            remove_all_except_memories(&agent_path)?;
        } else {
            fs::remove_dir_all(&agent_path)?;
        }
    }

    // 2. Remove from registry.
    registry.agents.remove(idx);
    registry.save(&workspace)?;

    println!();
    println!("Retired: {}", entry.id);
    if args.keep_files {
        println!("  Files kept at: {}", agent_path.display());
    } else if args.keep_memory {
        println!(
            "  Memories preserved at: {}",
            agent_path.join("memories").display()
        );
        println!("  Other files removed under: {}", agent_path.display());
    } else {
        println!("  Files removed: {}", agent_path.display());
    }
    println!(
        "  Registry updated: {}/.bwoc/agents.toml",
        workspace.display()
    );
    println!();
    Ok(())
}

/// Walk the agent directory and remove everything except `memories/`.
/// Idempotent — missing memories/ is fine (the parent dir just gets
/// stripped clean). After this returns, `<agent_path>/memories/` is
/// the only thing left, if it existed.
fn remove_all_except_memories(agent_path: &std::path::Path) -> io::Result<()> {
    let read = fs::read_dir(agent_path)?;
    for entry in read.flatten() {
        let name = entry.file_name();
        if name == "memories" {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

/// Resolve the workspace root: explicit > BWOC_WORKSPACE env > ancestor
/// walk from cwd > None. Mirror of the chain in workspace.rs and
/// doctor.rs (kept private to avoid premature extraction).
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
    use bwoc_core::workspace::{AgentEntry, Workspace, WorkspaceDefaults, WorkspaceMeta};

    fn setup_workspace(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("bwoc-retire-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".bwoc")).unwrap();
        fs::create_dir_all(root.join("agents")).unwrap();
        let ws = Workspace {
            workspace: WorkspaceMeta {
                name: label.to_string(),
                version: "0.1.0".to_string(),
                created: "2026-05-22T00:00:00Z".to_string(),
            },
            defaults: WorkspaceDefaults::default(),
        };
        ws.save(&root).unwrap();
        let mut reg = AgentsRegistry::default();
        reg.agents.push(AgentEntry {
            id: "agent-alpha".into(),
            path: "agents/agent-alpha".into(),
            backend: "claude".into(),
            incarnated: "2026-05-22T00:00:00Z".into(),
            status: "active".into(),
        });
        reg.save(&root).unwrap();
        fs::create_dir_all(root.join("agents/agent-alpha")).unwrap();
        fs::write(root.join("agents/agent-alpha/AGENTS.md"), "stub").unwrap();
        root
    }

    #[test]
    fn retire_removes_entry_and_files() {
        let root = setup_workspace("removes");
        let args = RetireArgs {
            name: "alpha".into(),
            workspace: Some(root.clone()),
            yes: true,
            keep_files: false,
        };
        assert!(retire(args).is_ok());

        let reg = AgentsRegistry::load(&root).unwrap();
        assert!(reg.agents.is_empty(), "registry should be empty");
        assert!(
            !root.join("agents/agent-alpha").exists(),
            "agent dir should be deleted"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn retire_keeps_files_when_flagged() {
        let root = setup_workspace("keep");
        let args = RetireArgs {
            name: "alpha".into(),
            workspace: Some(root.clone()),
            yes: true,
            keep_files: true,
        };
        assert!(retire(args).is_ok());

        let reg = AgentsRegistry::load(&root).unwrap();
        assert!(reg.agents.is_empty());
        assert!(
            root.join("agents/agent-alpha/AGENTS.md").exists(),
            "files should be kept"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn retire_fails_for_unknown_name() {
        let root = setup_workspace("unknown");
        let args = RetireArgs {
            name: "nonexistent".into(),
            workspace: Some(root.clone()),
            yes: true,
            keep_files: false,
        };
        match retire(args) {
            Err(RetireError::NotFound { name, .. }) => assert_eq!(name, "nonexistent"),
            other => panic!("expected NotFound, got {other:?}"),
        }
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn retire_matches_full_id_or_bare_name() {
        let root = setup_workspace("idmatch");
        // The agent id is "agent-alpha"; both "alpha" and "agent-alpha" must work.
        for name in ["alpha", "agent-alpha"] {
            // Re-set up between runs.
            let _ = fs::remove_dir_all(&root);
            let r = setup_workspace("idmatch");
            let args = RetireArgs {
                name: name.into(),
                workspace: Some(r.clone()),
                yes: true,
                keep_files: false,
            };
            assert!(retire(args).is_ok(), "should match name={name}");
            let _ = fs::remove_dir_all(&r);
        }
    }
}
