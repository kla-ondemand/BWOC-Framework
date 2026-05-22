//! `bwoc-agent` — minimal runtime shipped with each incarnated BWOC agent.
//!
//! Two modes:
//!   - **default** (no args): print the liveness banner from
//!     `config.manifest.json` in cwd and exit. Phase 1 v2.0 DoD.
//!   - **--serve**: write `<cwd>/.bwoc/agent.pid` and block until
//!     SIGTERM / SIGINT. This is the first foundation step toward
//!     Phase 2's control socket — `bwoc status` can detect a running
//!     agent via the PID file + signal-0 liveness test even before
//!     the full IPC protocol lands.

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use bwoc_core::manifest::Manifest;

mod i18n;

fn main() -> ExitCode {
    let serve = std::env::args().any(|a| a == "--serve");
    let lang = i18n::resolve_lang();
    let bundle = i18n::bundle_for(&lang);

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let manifest_path = cwd.join("config.manifest.json");

    if !manifest_path.exists() {
        let cwd_display = cwd.display().to_string();
        eprintln!(
            "{}",
            i18n::t_with(&bundle, "error-missing-manifest", &[("cwd", &cwd_display)])
        );
        return ExitCode::from(2);
    }

    let manifest = match Manifest::load_from_path(&manifest_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!(
                "bwoc-agent: failed to load manifest at {}: {e}",
                manifest_path.display()
            );
            return ExitCode::from(1);
        }
    };

    println!("{}", liveness_banner(&manifest, &bundle));

    if serve {
        return serve_loop(&cwd);
    }
    ExitCode::SUCCESS
}

/// `--serve` mode: write a PID file at `.bwoc/agent.pid`, open a Unix
/// domain socket at `.bwoc/agent.sock`, and accept simple line-based
/// requests until SIGTERM / SIGINT. Removes both files on exit.
///
/// Phase 0 IPC protocol — line-based, one request per connection:
///   `PING\n`       → `PONG\n`
///   anything else  → `ERR unknown command\n`
///
/// Future commands (STATUS / LOG / SEND / STOP) will slot in here as
/// they're spec'd. Keeping it line-text instead of binary so it's
/// debuggable with `nc -U`.
fn serve_loop(cwd: &std::path::Path) -> ExitCode {
    use std::io::ErrorKind;
    use std::os::unix::net::UnixListener;

    let bwoc_dir = cwd.join(".bwoc");
    if let Err(e) = std::fs::create_dir_all(&bwoc_dir) {
        eprintln!("bwoc-agent --serve: failed to create .bwoc/: {e}");
        return ExitCode::from(1);
    }
    let pid_path = bwoc_dir.join("agent.pid");
    let sock_path = bwoc_dir.join("agent.sock");

    // If a previous run left a socket behind, remove it (the pid file is
    // handled by the doctor stale-sweep separately).
    let _ = std::fs::remove_file(&sock_path);

    let pid = std::process::id();
    if let Err(e) = std::fs::write(&pid_path, format!("{pid}\n")) {
        eprintln!(
            "bwoc-agent --serve: failed to write {}: {e}",
            pid_path.display()
        );
        return ExitCode::from(1);
    }
    let listener = match UnixListener::bind(&sock_path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!(
                "bwoc-agent --serve: failed to bind {}: {e}",
                sock_path.display()
            );
            let _ = std::fs::remove_file(&pid_path);
            return ExitCode::from(1);
        }
    };
    if let Err(e) = listener.set_nonblocking(true) {
        eprintln!("bwoc-agent --serve: failed to set non-blocking: {e}");
        let _ = std::fs::remove_file(&pid_path);
        let _ = std::fs::remove_file(&sock_path);
        return ExitCode::from(1);
    }

    eprintln!("bwoc-agent --serve: pid {pid} → {}", pid_path.display());
    eprintln!("bwoc-agent --serve: socket → {}", sock_path.display());
    eprintln!("bwoc-agent --serve: blocking on SIGTERM / SIGINT (Ctrl-C)");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    if let Err(e) = ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }) {
        eprintln!("bwoc-agent --serve: failed to install signal handler: {e}");
        let _ = std::fs::remove_file(&pid_path);
        let _ = std::fs::remove_file(&sock_path);
        return ExitCode::from(1);
    }

    // Single-threaded accept loop with poll. Each accept is non-blocking
    // and yields control quickly so the signal check stays responsive.
    while running.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _addr)) => handle_client(stream, &running),
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                eprintln!("bwoc-agent --serve: accept error: {e}");
                break;
            }
        }
    }

    // Graceful exit — remove PID file + socket.
    if let Err(e) = std::fs::remove_file(&pid_path) {
        eprintln!(
            "bwoc-agent --serve: warning — failed to remove {}: {e}",
            pid_path.display()
        );
    }
    if let Err(e) = std::fs::remove_file(&sock_path) {
        eprintln!(
            "bwoc-agent --serve: warning — failed to remove {}: {e}",
            sock_path.display()
        );
    }
    eprintln!("bwoc-agent --serve: stopped cleanly");
    ExitCode::SUCCESS
}

#[cfg(unix)]
fn handle_client(mut stream: std::os::unix::net::UnixStream, running: &Arc<AtomicBool>) {
    use std::io::{BufRead, BufReader, Write};
    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return;
    }
    let cmd = line.trim();
    let response: &[u8] = match cmd {
        "PING" => b"PONG\n",
        "STOP" => {
            // Mark for shutdown; the accept loop will see this on its
            // next iteration (within ~100ms) and exit cleanly. Reply
            // BEFORE flipping the flag so the client always reads our
            // response — otherwise the loop might race-clean the socket
            // before write_all returns.
            running.store(false, Ordering::SeqCst);
            b"OK shutting down\n"
        }
        _ => b"ERR unknown command\n",
    };
    let _ = stream.write_all(response);
}

/// Pure-data formatter for the liveness output. Kept separate from `main` so
/// it can be unit-tested without needing a real manifest on disk.
fn liveness_banner(
    m: &Manifest,
    bundle: &fluent_bundle::FluentBundle<fluent_bundle::FluentResource>,
) -> String {
    let mut lines = Vec::with_capacity(8);
    lines.push(i18n::t_with(
        bundle,
        "liveness-alive",
        &[("agent_id", m.agent_id.as_str())],
    ));
    lines.push(i18n::t_with(
        bundle,
        "liveness-role",
        &[("role", m.agent_role.as_str())],
    ));
    lines.push(i18n::t_with(
        bundle,
        "liveness-model",
        &[("model", m.primary_model.as_str())],
    ));
    if let Some(ref fb) = m.fallback_model {
        lines.push(i18n::t_with(
            bundle,
            "liveness-fallback",
            &[("fallback", fb.as_str())],
        ));
    }
    lines.push(i18n::t_with(
        bundle,
        "liveness-memory",
        &[("memory_path", m.memory_path.as_str())],
    ));
    lines.push(i18n::t_with(
        bundle,
        "liveness-version",
        &[("version", m.version.as_str())],
    ));
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Manifest {
        Manifest {
            name: "demo".into(),
            agent_id: "agent-demo".into(),
            agent_role: "demo role".into(),
            primary_model: "model-x".into(),
            fallback_model: Some("model-y".into()),
            memory_path: "memories/".into(),
            sessions_path: None,
            deep_memory_cmd: None,
            lint_cmd: "true".into(),
            format_cmd: "true".into(),
            test_cmd: "true".into(),
            build_cmd: "true".into(),
            worktree_base: None,
            version: "2.0".into(),
        }
    }

    #[test]
    fn banner_shows_required_fields_en() {
        let bundle = i18n::bundle_for("en");
        let b = liveness_banner(&sample(), &bundle);
        assert!(b.contains("I am alive: agent-demo"));
        assert!(b.contains("demo role"));
        assert!(b.contains("model-x"));
        assert!(b.contains("model-y"));
        assert!(b.contains("memories/"));
        assert!(b.contains("2.0"));
    }

    #[test]
    fn banner_shows_required_fields_th() {
        let bundle = i18n::bundle_for("th");
        let b = liveness_banner(&sample(), &bundle);
        assert!(b.contains("ฉันยังมีชีวิตอยู่: agent-demo"));
        assert!(b.contains("demo role"));
        assert!(b.contains("model-x"));
    }

    #[test]
    fn banner_omits_optional_fallback_when_none() {
        let bundle = i18n::bundle_for("en");
        let mut m = sample();
        m.fallback_model = None;
        let b = liveness_banner(&m, &bundle);
        assert!(b.contains("I am alive:"));
        assert!(!b.contains("fallback:"));
    }
}
