//! `bwoc send <to> <message>` — Phase 3 sammā-vācā Phase 0.
//!
//! User → agent inbox communication. Appends a JSON line to
//! `<agent>/.bwoc/inbox.jsonl`. Each line is one message:
//!
//!   {"ts": "<ISO 8601 UTC>", "from": "user", "to": "<agent-id>", "message": "..."}
//!
//! Agent → agent messaging (the full sammā-vācā channel with
//! Sāraṇīyadhamma 6 + Kalyāṇamitta 7 trust scoring) lands later.
//! For now this gives users a way to leave instructions for an agent
//! that's offline or paused, and establishes the JSONL inbox format
//! so the future daemon can read from a stable file shape.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use bwoc_core::workspace::AgentsRegistry;

pub struct SendArgs {
    pub to: String,
    pub message: String,
    pub workspace: Option<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error(
        "no workspace found (no .bwoc/workspace.toml in cwd or ancestors). \
         Pass --workspace, set BWOC_WORKSPACE, or run `bwoc init` first."
    )]
    NoWorkspace,
    #[error("no agent named '{name}' in workspace {workspace}")]
    NotFound { name: String, workspace: PathBuf },
    #[error("empty message — pass non-empty text after the agent name")]
    EmptyMessage,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("workspace error: {0}")]
    Workspace(#[from] bwoc_core::workspace::WorkspaceError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn run(args: SendArgs) -> i32 {
    match send(args) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("bwoc send: {e}");
            match e {
                SendError::NoWorkspace | SendError::NotFound { .. } | SendError::EmptyMessage => 2,
                _ => 1,
            }
        }
    }
}

fn send(args: SendArgs) -> Result<(), SendError> {
    if args.message.trim().is_empty() {
        return Err(SendError::EmptyMessage);
    }
    let workspace = resolve_workspace(args.workspace).ok_or(SendError::NoWorkspace)?;
    let registry = AgentsRegistry::load(&workspace)?;

    let lookup_id = if args.to.starts_with("agent-") {
        args.to.clone()
    } else {
        format!("agent-{}", args.to)
    };
    let entry = registry
        .agents
        .iter()
        .find(|a| a.id == lookup_id)
        .ok_or_else(|| SendError::NotFound {
            name: args.to.clone(),
            workspace: workspace.clone(),
        })?;

    let agent_path = workspace.join(&entry.path);
    let bwoc_dir = agent_path.join(".bwoc");
    std::fs::create_dir_all(&bwoc_dir)?;
    let inbox_path = bwoc_dir.join("inbox.jsonl");

    let ts = crate::util::utc_now_iso8601();
    let envelope = serde_json::json!({
        "ts": ts,
        "from": "user",
        "to": entry.id,
        "message": args.message,
    });
    let line = serde_json::to_string(&envelope)?;

    // Append-only — multiple `bwoc send` calls just stack lines. The
    // agent's daemon (when it reads inbox) is responsible for tracking
    // which messages have been consumed (probably via a sibling
    // `inbox.cursor` file once we add daemon-side reads).
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&inbox_path)?;
    writeln!(f, "{line}")?;

    println!();
    println!("Sent to {}: {}", entry.id, args.message);
    println!("  Inbox: {} (appended at {ts})", inbox_path.display());
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

    fn setup(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("bwoc-send-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".bwoc")).unwrap();
        fs::create_dir_all(root.join("agents/agent-alpha")).unwrap();
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
    fn send_appends_a_jsonl_envelope() {
        let root = setup("ok");
        send(SendArgs {
            to: "alpha".into(),
            message: "hello".into(),
            workspace: Some(root.clone()),
        })
        .unwrap();
        let line =
            std::fs::read_to_string(root.join("agents/agent-alpha/.bwoc/inbox.jsonl")).unwrap();
        let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["to"], "agent-alpha");
        assert_eq!(v["from"], "user");
        assert_eq!(v["message"], "hello");
        assert!(v["ts"].is_string());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn send_appends_multiple_lines() {
        let root = setup("multi");
        for msg in ["one", "two", "three"] {
            send(SendArgs {
                to: "alpha".into(),
                message: msg.into(),
                workspace: Some(root.clone()),
            })
            .unwrap();
        }
        let content =
            std::fs::read_to_string(root.join("agents/agent-alpha/.bwoc/inbox.jsonl")).unwrap();
        assert_eq!(content.lines().count(), 3);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn send_rejects_empty_message() {
        let root = setup("empty");
        let err = send(SendArgs {
            to: "alpha".into(),
            message: "   ".into(),
            workspace: Some(root.clone()),
        });
        assert!(matches!(err, Err(SendError::EmptyMessage)));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn send_fails_for_unknown_agent() {
        let root = setup("nf");
        let err = send(SendArgs {
            to: "zzz".into(),
            message: "x".into(),
            workspace: Some(root.clone()),
        });
        assert!(matches!(err, Err(SendError::NotFound { .. })));
        let _ = fs::remove_dir_all(&root);
    }
}
