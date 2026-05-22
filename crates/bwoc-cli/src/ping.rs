//! `bwoc ping <name>` — verify a `bwoc-agent --serve`'d agent is reachable.
//!
//! Phase 0 client for the IPC protocol started in this iter. Opens
//! `<workspace>/<agent>/.bwoc/agent.sock`, writes `PING\n`, reads one
//! line, prints it. Exit code:
//!   0  → got `PONG`
//!   1  → got something else (protocol drift)
//!   2  → workspace / agent / socket not found, or connection refused

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::Duration;

use bwoc_core::workspace::AgentsRegistry;

pub struct PingArgs {
    pub name: String,
    pub workspace: Option<PathBuf>,
}

pub fn run(args: PingArgs) -> i32 {
    let Some(workspace) = resolve_workspace(args.workspace) else {
        eprintln!(
            "bwoc ping: no workspace found. Pass --workspace, set BWOC_WORKSPACE, or run from a workspace dir."
        );
        return 2;
    };
    let registry = match AgentsRegistry::load(&workspace) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("bwoc ping: failed to read agents.toml: {e}");
            return 1;
        }
    };
    let lookup_id = if args.name.starts_with("agent-") {
        args.name.clone()
    } else {
        format!("agent-{}", args.name)
    };
    let Some(entry) = registry.agents.iter().find(|a| a.id == lookup_id) else {
        eprintln!(
            "bwoc ping: no agent named '{}' in workspace {}. Try `bwoc list`.",
            args.name,
            workspace.display()
        );
        return 2;
    };

    let sock_path = workspace.join(&entry.path).join(".bwoc/agent.sock");
    if !sock_path.exists() {
        eprintln!(
            "bwoc ping: agent socket missing at {}. Is `bwoc-agent --serve` running in that dir?",
            sock_path.display()
        );
        return 2;
    }

    #[cfg(unix)]
    {
        use std::os::unix::net::UnixStream;
        let mut stream = match UnixStream::connect(&sock_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "bwoc ping: failed to connect to {}: {e}",
                    sock_path.display()
                );
                return 2;
            }
        };
        // Bound read time so a hung agent doesn't hang the CLI.
        let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));

        if let Err(e) = stream.write_all(b"PING\n") {
            eprintln!("bwoc ping: write failed: {e}");
            return 1;
        }
        let mut reader = BufReader::new(&stream);
        let mut line = String::new();
        if let Err(e) = reader.read_line(&mut line) {
            eprintln!("bwoc ping: read failed: {e}");
            return 1;
        }
        let response = line.trim();
        println!("{} → {}", entry.id, response);
        if response == "PONG" { 0 } else { 1 }
    }
    #[cfg(not(unix))]
    {
        eprintln!("bwoc ping: Unix domain sockets only (Windows support: Phase 2 sub-task)");
        1
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
