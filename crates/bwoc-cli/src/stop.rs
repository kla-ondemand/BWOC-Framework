//! `bwoc stop <name>` — pause an agent without deleting it.
//!
//! Sets `status = "stopped"` in the workspace's `.bwoc/agents.toml` for
//! the given agent. Files on disk stay intact. The counterpart of
//! `bwoc retire` (which removes the entry + optionally the directory);
//! this is the lighter "pause / deactivate" operation.
//!
//! When Phase 2's control socket lands, this verb will also signal a
//! running `bwoc-agent` process to stop gracefully. For now it's a
//! registry-only mutation.
//!
//! Lookup: matches by full id (`agent-foo`) or bare name (`foo`).
//! TTY confirmation; `--yes` to skip for scripts.

use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

use bwoc_core::workspace::AgentsRegistry;

pub struct StopArgs {
    pub name: String,
    pub workspace: Option<PathBuf>,
    pub yes: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum StopError {
    #[error(
        "no workspace found (no .bwoc/workspace.toml in cwd or ancestors). \
         Pass --workspace, set BWOC_WORKSPACE, or run `bwoc init` first."
    )]
    NoWorkspace,
    #[error("no agent named '{name}' in workspace {workspace}")]
    NotFound { name: String, workspace: PathBuf },
    #[error("agent '{name}' is already stopped")]
    AlreadyStopped { name: String },
    #[error("aborted by user")]
    Aborted,
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("workspace error: {0}")]
    Workspace(#[from] bwoc_core::workspace::WorkspaceError),
}

pub fn run(args: StopArgs) -> i32 {
    match stop(args) {
        Ok(()) => 0,
        Err(StopError::Aborted) => {
            eprintln!("bwoc stop: aborted — nothing changed");
            2
        }
        Err(e) => {
            eprintln!("bwoc stop: {e}");
            match e {
                StopError::NoWorkspace
                | StopError::NotFound { .. }
                | StopError::AlreadyStopped { .. } => 2,
                _ => 1,
            }
        }
    }
}

fn stop(args: StopArgs) -> Result<(), StopError> {
    let workspace = resolve_workspace(args.workspace).ok_or(StopError::NoWorkspace)?;
    let mut registry = AgentsRegistry::load(&workspace)?;

    let lookup_id = if args.name.starts_with("agent-") {
        args.name.clone()
    } else {
        format!("agent-{}", args.name)
    };
    let idx = registry
        .agents
        .iter()
        .position(|a| a.id == lookup_id)
        .ok_or_else(|| StopError::NotFound {
            name: args.name.clone(),
            workspace: workspace.clone(),
        })?;

    if registry.agents[idx].status == "stopped" {
        return Err(StopError::AlreadyStopped {
            name: args.name.clone(),
        });
    }

    let entry = registry.agents[idx].clone();
    println!();
    println!("About to stop agent:");
    println!("  id:       {}", entry.id);
    println!("  path:     {}", entry.path);
    println!("  backend:  {}", entry.backend);
    println!("  status:   {} → stopped", entry.status);
    println!();
    println!("Files stay on disk. Use `bwoc retire` to remove entirely.");
    println!();

    if !args.yes {
        if !io::stdin().is_terminal() {
            return Err(StopError::Aborted);
        }
        let mut stdout = io::stdout();
        write!(stdout, "Proceed? [y/N]: ")?;
        stdout.flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let answer = line.trim().to_ascii_lowercase();
        if answer != "y" && answer != "yes" {
            return Err(StopError::Aborted);
        }
    }

    registry.agents[idx].status = "stopped".to_string();
    registry.save(&workspace)?;

    println!();
    println!("Stopped: {}", entry.id);
    println!(
        "  Registry updated: {}/.bwoc/agents.toml",
        workspace.display()
    );
    println!(
        "  Files preserved at: {}",
        workspace.join(&entry.path).display()
    );
    println!();
    Ok(())
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

    fn setup_workspace(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("bwoc-stop-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".bwoc")).unwrap();
        fs::create_dir_all(root.join("agents")).unwrap();
        Workspace {
            workspace: WorkspaceMeta {
                name: label.to_string(),
                version: "0.1.0".to_string(),
                created: "2026-05-22T00:00:00Z".to_string(),
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

    #[test]
    fn stop_sets_status_to_stopped() {
        let root = setup_workspace("ok");
        assert!(
            stop(StopArgs {
                name: "alpha".into(),
                workspace: Some(root.clone()),
                yes: true,
            })
            .is_ok()
        );
        let reg = AgentsRegistry::load(&root).unwrap();
        assert_eq!(reg.agents[0].status, "stopped");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn stop_refuses_already_stopped() {
        let root = setup_workspace("already");
        stop(StopArgs {
            name: "alpha".into(),
            workspace: Some(root.clone()),
            yes: true,
        })
        .unwrap();
        let err = stop(StopArgs {
            name: "alpha".into(),
            workspace: Some(root.clone()),
            yes: true,
        });
        assert!(matches!(err, Err(StopError::AlreadyStopped { .. })));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn stop_fails_for_unknown_name() {
        let root = setup_workspace("missing");
        let err = stop(StopArgs {
            name: "zzz".into(),
            workspace: Some(root.clone()),
            yes: true,
        });
        assert!(matches!(err, Err(StopError::NotFound { .. })));
        let _ = fs::remove_dir_all(&root);
    }
}
