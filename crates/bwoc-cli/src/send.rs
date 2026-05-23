//! `bwoc send <to> <message>` — Phase 3 sammā-vācā Phase 0.
//!
//! User → agent inbox communication. Appends a JSON line to
//! `<agent>/.bwoc/inbox.jsonl`. Each line is one message:
//!
//!   {"ts":"...","messageId":"msg-...","from":"user","to":"<agent-id>",
//!    "message":"...","replyTo":"msg-..."?}
//!
//! `messageId` is always present (generated here). `replyTo` is present
//! only when the caller passes `--reply-to` — typically the Stop hook
//! at `modules/agent-template/.claude/hooks/inbox-auto-reply.sh`.
//!
//! Agent → agent messaging (the full sammā-vācā channel with
//! Sāraṇīyadhamma 6 + Kalyāṇamitta 7 trust scoring) lands later.
//! For now this gives users a way to leave instructions for an agent
//! that's offline or paused, and establishes the JSONL inbox format
//! so the future daemon can read from a stable file shape.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bwoc_core::workspace::AgentsRegistry;

pub struct SendArgs {
    pub to: String,
    pub message: String,
    /// Sender identity. `None` → write `from: "user"` (legacy default,
    /// human operator). `Some(name)` → resolve to an agent in the
    /// workspace registry and write `from: <agentId>`. See
    /// `modules/agent-template/interconnect/messaging.md` §"CLI Surface".
    pub from: Option<String>,
    /// When set, this envelope is a reply to a prior message. The value
    /// is the prior envelope's `messageId`. Stamped into the envelope as
    /// `replyTo` so recipients can thread, and used by the auto-reply
    /// hook to close a request/response loop. See messaging.md §Wakeup.
    pub reply_to: Option<String>,
    /// Skip the best-effort tmux send-keys wakeup. CI/daemons set this
    /// so non-interactive callers don't side-effect into a TUI session.
    pub no_wakeup: bool,
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
    #[error(
        "no sender agent named '{name}' in workspace {workspace} (--from must reference a registered agent)"
    )]
    SenderNotFound { name: String, workspace: PathBuf },
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
                SendError::NoWorkspace
                | SendError::NotFound { .. }
                | SendError::SenderNotFound { .. }
                | SendError::EmptyMessage => 2,
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

    let lookup_id = canonicalize(&args.to);
    let entry = registry
        .agents
        .iter()
        .find(|a| a.id == lookup_id)
        .ok_or_else(|| SendError::NotFound {
            name: args.to.clone(),
            workspace: workspace.clone(),
        })?;

    // Resolve sender identity. None → "user" (default, legacy behavior).
    // Some(name) → must match an agent in the same workspace registry.
    // Trust verification happens at the recipient daemon (trust step 4)
    // by reading the sender's manifest — we only enforce existence here.
    let from = match args.from.as_deref() {
        None => "user".to_string(),
        Some(name) => {
            let sender_id = canonicalize(name);
            let sender = registry
                .agents
                .iter()
                .find(|a| a.id == sender_id)
                .ok_or_else(|| SendError::SenderNotFound {
                    name: name.to_string(),
                    workspace: workspace.clone(),
                })?;
            sender.id.clone()
        }
    };

    let agent_path = workspace.join(&entry.path);
    let bwoc_dir = agent_path.join(".bwoc");
    std::fs::create_dir_all(&bwoc_dir)?;
    let inbox_path = bwoc_dir.join("inbox.jsonl");

    let ts = crate::util::utc_now_iso8601();
    let message_id = generate_message_id(&ts);
    let mut envelope = serde_json::Map::new();
    envelope.insert("ts".into(), ts.clone().into());
    envelope.insert("messageId".into(), message_id.clone().into());
    envelope.insert("from".into(), from.clone().into());
    envelope.insert("to".into(), entry.id.clone().into());
    envelope.insert("message".into(), args.message.clone().into());
    if let Some(rt) = args.reply_to.as_deref() {
        envelope.insert("replyTo".into(), rt.into());
    }
    let line = serde_json::to_string(&serde_json::Value::Object(envelope))?;

    // Append-only — multiple `bwoc send` calls just stack lines. The
    // agent's daemon (when it reads inbox) is responsible for tracking
    // which messages have been consumed (probably via a sibling
    // `inbox.cursor` file once we add daemon-side reads).
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&inbox_path)?;
    writeln!(f, "{line}")?;

    // Best-effort tmux wakeup — see notify_tmux for the convention and
    // the silent skip rules. Suppressed for CI/daemons via --no-wakeup
    // or BWOC_DISABLE_TMUX_WAKEUP (the latter keeps tests quiet).
    if !args.no_wakeup && std::env::var("BWOC_DISABLE_TMUX_WAKEUP").is_err() {
        notify_tmux(&entry.id, &from, &message_id, &args.message);
    }

    let reply_suffix = match args.reply_to.as_deref() {
        Some(rt) => format!(", reply to {rt}"),
        None => String::new(),
    };
    println!();
    println!(
        "Sent to {} (from {from}) [id {message_id}{reply_suffix}]: {}",
        entry.id, args.message,
    );
    println!("  Inbox: {} (appended at {ts})", inbox_path.display());
    println!();
    Ok(())
}

/// Best-effort tmux send-keys ping that wakes a recipient TUI session.
///
/// Convention: recipient `agent-<x>` → tmux session `<x>`. The marker
/// `[bwoc inbox <msg-id> from <sender>]` prefixes the message body so
/// the Stop hook at `modules/agent-template/.claude/hooks/inbox-auto-reply.sh`
/// can detect a bus-triggered turn and thread its reply via `--reply-to`.
///
/// Silent no-op when:
/// - the recipient is not `agent-*` (topics, user-only flows)
/// - `tmux` binary is missing
/// - no tmux session matches the recipient's bare name
///
/// Two-step send (text → 200ms → Enter) — single-call submission gets
/// dropped by Claude Code's TUI input layer; this is the upstream
/// pattern from `it-app-workspace/bin/agent-send`.
fn notify_tmux(to: &str, from: &str, msg_id: &str, message: &str) {
    let Some(session) = to.strip_prefix("agent-") else {
        return;
    };
    if std::process::Command::new("tmux")
        .args(["has-session", "-t", session])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| !s.success())
        .unwrap_or(true)
    {
        return;
    }
    let notify = format!("[bwoc inbox {msg_id} from {from}] {message}");
    let _ = std::process::Command::new("tmux")
        .args(["send-keys", "-t", session, "--", &notify])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    std::thread::sleep(std::time::Duration::from_millis(200));
    let _ = std::process::Command::new("tmux")
        .args(["send-keys", "-t", session, "Enter"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

/// Build a per-envelope id of the form `msg-<utc-slug>-<5hex>`.
///
/// `utc-slug` is the same instant as `ts` collapsed to `YYYYMMDDTHHMMSSZ`.
/// The 5-hex suffix derives from sub-second nanos so two sends inside
/// the same wallclock second still get distinct ids without pulling in
/// a `rand` dependency (Mattaññutā — minimal deps).
fn generate_message_id(ts: &str) -> String {
    let slug: String = ts.chars().filter(|c| *c != '-' && *c != ':').collect();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let suffix = nanos & 0xF_FFFF;
    format!("msg-{slug}-{suffix:05x}")
}

/// Normalize a user-supplied agent name to its canonical `agent-<name>`
/// form. Idempotent: already-canonical inputs pass through unchanged.
fn canonicalize(name: &str) -> String {
    if name.starts_with("agent-") {
        name.to_string()
    } else {
        format!("agent-{name}")
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
            from: None,
            reply_to: None,
            no_wakeup: true,
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
                from: None,
                reply_to: None,
                no_wakeup: true,
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
            from: None,
            reply_to: None,
            no_wakeup: true,
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
            from: None,
            reply_to: None,
            no_wakeup: true,
            workspace: Some(root.clone()),
        });
        assert!(matches!(err, Err(SendError::NotFound { .. })));
        let _ = fs::remove_dir_all(&root);
    }

    // ---- --from <agent> sender identity (messaging.md) ---------------------

    fn setup_with_two_agents(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("bwoc-send-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".bwoc")).unwrap();
        fs::create_dir_all(root.join("agents/agent-alpha")).unwrap();
        fs::create_dir_all(root.join("agents/agent-beta")).unwrap();
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
        for id in ["agent-alpha", "agent-beta"] {
            reg.agents.push(AgentEntry {
                id: id.into(),
                path: format!("agents/{id}"),
                backend: "claude".into(),
                incarnated: "2026-05-22T00:00:00Z".into(),
                status: "active".into(),
            });
        }
        reg.save(&root).unwrap();
        root
    }

    #[test]
    fn send_from_agent_writes_sender_id_into_envelope() {
        let root = setup_with_two_agents("from-agent");
        send(SendArgs {
            to: "alpha".into(),
            message: "peer message".into(),
            from: Some("beta".into()),
            reply_to: None,
            no_wakeup: true,
            workspace: Some(root.clone()),
        })
        .unwrap();
        let line =
            std::fs::read_to_string(root.join("agents/agent-alpha/.bwoc/inbox.jsonl")).unwrap();
        let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["from"], "agent-beta"); // canonical form, not "beta"
        assert_eq!(v["to"], "agent-alpha");
        assert_eq!(v["message"], "peer message");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn send_from_accepts_already_canonical_sender_id() {
        let root = setup_with_two_agents("from-canonical");
        send(SendArgs {
            to: "alpha".into(),
            message: "x".into(),
            from: Some("agent-beta".into()),
            reply_to: None,
            no_wakeup: true,
            workspace: Some(root.clone()),
        })
        .unwrap();
        let line =
            std::fs::read_to_string(root.join("agents/agent-alpha/.bwoc/inbox.jsonl")).unwrap();
        let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["from"], "agent-beta");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn send_from_unknown_sender_fails_with_sender_not_found() {
        let root = setup_with_two_agents("from-bad");
        let err = send(SendArgs {
            to: "alpha".into(),
            message: "x".into(),
            from: Some("ghost".into()),
            reply_to: None,
            no_wakeup: true,
            workspace: Some(root.clone()),
        });
        assert!(
            matches!(err, Err(SendError::SenderNotFound { .. })),
            "expected SenderNotFound, got {err:?}"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn send_from_none_keeps_legacy_user_default() {
        let root = setup_with_two_agents("from-none");
        send(SendArgs {
            to: "alpha".into(),
            message: "still works".into(),
            from: None,
            reply_to: None,
            no_wakeup: true,
            workspace: Some(root.clone()),
        })
        .unwrap();
        let line =
            std::fs::read_to_string(root.join("agents/agent-alpha/.bwoc/inbox.jsonl")).unwrap();
        let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["from"], "user");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn canonicalize_is_idempotent() {
        assert_eq!(canonicalize("foo"), "agent-foo");
        assert_eq!(canonicalize("agent-foo"), "agent-foo");
        // Edge: a bare hyphen is unusual but still canonicalized
        assert_eq!(canonicalize("a"), "agent-a");
    }

    // ---- messageId + replyTo (messaging.md §Envelope Schema) ---------------

    #[test]
    fn send_stamps_message_id_into_envelope() {
        let root = setup("msgid");
        send(SendArgs {
            to: "alpha".into(),
            message: "hi".into(),
            from: None,
            reply_to: None,
            no_wakeup: true,
            workspace: Some(root.clone()),
        })
        .unwrap();
        let line =
            std::fs::read_to_string(root.join("agents/agent-alpha/.bwoc/inbox.jsonl")).unwrap();
        let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        let msg_id = v["messageId"].as_str().expect("messageId stamped");
        assert!(msg_id.starts_with("msg-"), "format: {msg_id}");
        // shape: msg-YYYYMMDDTHHMMSSZ-XXXXX (5 hex)
        let parts: Vec<&str> = msg_id.splitn(3, '-').collect();
        assert_eq!(parts.len(), 3, "msg-<slug>-<hex>: {msg_id}");
        assert_eq!(parts[2].len(), 5, "5-hex suffix: {msg_id}");
        // replyTo absent when not requested
        assert!(v.get("replyTo").is_none());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn send_with_reply_to_round_trips_field() {
        let root = setup("replyto");
        send(SendArgs {
            to: "alpha".into(),
            message: "ack".into(),
            from: None,
            reply_to: Some("msg-20260523T000000Z-deadb".into()),
            no_wakeup: true,
            workspace: Some(root.clone()),
        })
        .unwrap();
        let line =
            std::fs::read_to_string(root.join("agents/agent-alpha/.bwoc/inbox.jsonl")).unwrap();
        let v: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(v["replyTo"], "msg-20260523T000000Z-deadb");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn generate_message_id_collapses_separators_in_slug() {
        let id = generate_message_id("2026-05-23T14:30:12Z");
        assert!(id.starts_with("msg-20260523T143012Z-"), "got {id}");
        // 5-hex tail
        let tail = id.rsplit('-').next().unwrap();
        assert_eq!(tail.len(), 5);
        assert!(tail.chars().all(|c| c.is_ascii_hexdigit()), "hex: {id}");
    }
}
