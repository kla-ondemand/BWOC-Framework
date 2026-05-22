//! `bwoc start <name>` — reactivate a previously stopped agent.
//!
//! Sets `status` from `"stopped"` back to `"active"` in the workspace's
//! `.bwoc/agents.toml`. The counterpart of `bwoc stop`.
//!
//! Mirror of `stop.rs` (deliberately duplicated for now — two small
//! siblings with no API drift beat a premature "set-state" abstraction).
//! If/when a third state verb appears, fold into `crate::state` or
//! similar.

use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

use bwoc_core::workspace::AgentsRegistry;

pub struct StartArgs {
    pub name: String,
    pub workspace: Option<PathBuf>,
    pub yes: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum StartError {
    #[error(
        "no workspace found (no .bwoc/workspace.toml in cwd or ancestors). \
         Pass --workspace, set BWOC_WORKSPACE, or run `bwoc init` first."
    )]
    NoWorkspace,
    #[error("no agent named '{name}' in workspace {workspace}")]
    NotFound { name: String, workspace: PathBuf },
    #[error("agent '{name}' is already active")]
    AlreadyActive { name: String },
    #[error("aborted by user")]
    Aborted,
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("workspace error: {0}")]
    Workspace(#[from] bwoc_core::workspace::WorkspaceError),
}

pub fn run(args: StartArgs) -> i32 {
    match start(args) {
        Ok(()) => 0,
        Err(StartError::Aborted) => {
            eprintln!("bwoc start: aborted — nothing changed");
            2
        }
        Err(e) => {
            eprintln!("bwoc start: {e}");
            match e {
                StartError::NoWorkspace
                | StartError::NotFound { .. }
                | StartError::AlreadyActive { .. } => 2,
                _ => 1,
            }
        }
    }
}

fn start(args: StartArgs) -> Result<(), StartError> {
    let workspace = resolve_workspace(args.workspace).ok_or(StartError::NoWorkspace)?;
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
        .ok_or_else(|| StartError::NotFound {
            name: args.name.clone(),
            workspace: workspace.clone(),
        })?;

    if registry.agents[idx].status == "active" {
        return Err(StartError::AlreadyActive {
            name: args.name.clone(),
        });
    }

    let entry = registry.agents[idx].clone();
    println!();
    println!("About to start agent:");
    println!("  id:       {}", entry.id);
    println!("  path:     {}", entry.path);
    println!("  backend:  {}", entry.backend);
    println!("  status:   {} → active", entry.status);
    println!();

    if !args.yes {
        if !io::stdin().is_terminal() {
            return Err(StartError::Aborted);
        }
        let mut stdout = io::stdout();
        write!(stdout, "Proceed? [y/N]: ")?;
        stdout.flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let answer = line.trim().to_ascii_lowercase();
        if answer != "y" && answer != "yes" {
            return Err(StartError::Aborted);
        }
    }

    registry.agents[idx].status = "active".to_string();
    registry.save(&workspace)?;

    println!();
    println!("Started: {}", entry.id);
    println!(
        "  Registry updated: {}/.bwoc/agents.toml",
        workspace.display()
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

    fn setup_workspace(label: &str, initial_status: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("bwoc-start-{label}-{}", std::process::id()));
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
            status: initial_status.into(),
        });
        reg.save(&root).unwrap();
        root
    }

    #[test]
    fn start_sets_status_to_active() {
        let root = setup_workspace("ok", "stopped");
        assert!(
            start(StartArgs {
                name: "alpha".into(),
                workspace: Some(root.clone()),
                yes: true,
            })
            .is_ok()
        );
        let reg = AgentsRegistry::load(&root).unwrap();
        assert_eq!(reg.agents[0].status, "active");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn start_refuses_already_active() {
        let root = setup_workspace("already", "active");
        let err = start(StartArgs {
            name: "alpha".into(),
            workspace: Some(root.clone()),
            yes: true,
        });
        assert!(matches!(err, Err(StartError::AlreadyActive { .. })));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn start_fails_for_unknown_name() {
        let root = setup_workspace("missing", "stopped");
        let err = start(StartArgs {
            name: "zzz".into(),
            workspace: Some(root.clone()),
            yes: true,
        });
        assert!(matches!(err, Err(StartError::NotFound { .. })));
        let _ = fs::remove_dir_all(&root);
    }
}
