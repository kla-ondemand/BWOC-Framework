//! Shared liveness + inbox helpers.
//!
//! Consolidates what previously lived as 5 near-identical copies across
//! `status.rs`, `doctor.rs`, `workspace.rs`, `dashboard.rs`, and `start.rs`:
//!
//!   - `signal_zero_alive(pid)` — Unix signal-0 liveness probe
//!   - `running_pid(root, agent)` — PID file + signal-0
//!   - `query_uptime(root, agent)` — STATUS over the agent's Unix socket
//!   - `format_uptime(secs)` — short "42s" / "5m12s" / "3h07m" / "2d04h"
//!   - `inbox_count(root, agent)` — count of complete envelopes in inbox.jsonl
//!
//! All helpers are read-only and side-effect free (except `query_uptime`
//! which opens a Unix socket — bounded by a 300ms timeout). No new
//! dependencies — uses `libc` (already in bwoc-cli deps).

use std::path::Path;

use bwoc_core::workspace::AgentEntry;

/// Send signal 0 to `pid` — true iff the process exists and is signal-
/// reachable from this process. The standard Unix liveness test.
/// Conservatively false on Windows until Phase 2 named-pipe supervision.
#[cfg(unix)]
pub fn signal_zero_alive(pid: u32) -> bool {
    // SAFETY: kill(pid, 0) is a syscall with no side effects beyond
    // returning an errno; pid is u32 cast to libc::pid_t, signal=0.
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}

#[cfg(not(unix))]
pub fn signal_zero_alive(_pid: u32) -> bool {
    false
}

/// Read `<agent>/.bwoc/agent.pid` and return the pid iff it's alive.
/// Stale pid files (file present but process gone) return None — `bwoc
/// doctor --auto` sweeps them separately.
pub fn running_pid(root: &Path, a: &AgentEntry) -> Option<u32> {
    let pid_path = root.join(&a.path).join(".bwoc/agent.pid");
    let raw = std::fs::read_to_string(&pid_path).ok()?;
    let pid: u32 = raw.trim().parse().ok()?;
    if signal_zero_alive(pid) {
        Some(pid)
    } else {
        None
    }
}

/// Query the agent daemon's STATUS command via Unix socket. Returns
/// Some(uptime_secs) on a valid reply, None when the socket is missing
/// or the daemon doesn't answer. Bounded by 300ms timeout so a hung
/// daemon can't slow callers.
#[cfg(unix)]
pub fn query_uptime(root: &Path, a: &AgentEntry) -> Option<u64> {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixStream;
    use std::time::Duration;

    let sock_path = root.join(&a.path).join(".bwoc/agent.sock");
    if !sock_path.exists() {
        return None;
    }
    let mut stream = UnixStream::connect(&sock_path).ok()?;
    let _ = stream.set_read_timeout(Some(Duration::from_millis(300)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(300)));
    stream.write_all(b"STATUS\n").ok()?;
    let mut line = String::new();
    BufReader::new(&stream).read_line(&mut line).ok()?;
    // Expected reply: `OK uptime_secs=<N> pid=<N>\n`
    for token in line.split_whitespace() {
        if let Some(rest) = token.strip_prefix("uptime_secs=") {
            return rest.parse().ok();
        }
    }
    None
}

#[cfg(not(unix))]
pub fn query_uptime(_root: &Path, _a: &AgentEntry) -> Option<u64> {
    None
}

/// Format seconds as a short human term:
///   `< 1m`  → "42s"
///   `< 1h`  → "5m12s"
///   `< 1d`  → "3h07m"
///   `>= 1d` → "2d04h"
///
/// Keeps status lines one column-length even at long uptimes.
pub fn format_uptime(secs: u64) -> String {
    let (d, rem) = (secs / 86400, secs % 86400);
    let (h, rem) = (rem / 3600, rem % 3600);
    let (m, s) = (rem / 60, rem % 60);
    if d > 0 {
        format!("{d}d{h:02}h")
    } else if h > 0 {
        format!("{h}h{m:02}m")
    } else if m > 0 {
        format!("{m}m{s:02}s")
    } else {
        format!("{s}s")
    }
}

/// Count complete envelope lines in `<agent>/.bwoc/inbox.jsonl`. Returns
/// 0 when the file is missing or unreadable — same shape as a real
/// empty inbox, which keeps callers simple.
pub fn inbox_count(root: &Path, a: &AgentEntry) -> usize {
    let path = root.join(&a.path).join(".bwoc/inbox.jsonl");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return 0;
    };
    content.lines().filter(|l| !l.trim().is_empty()).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_uptime_under_a_minute() {
        assert_eq!(format_uptime(0), "0s");
        assert_eq!(format_uptime(42), "42s");
        assert_eq!(format_uptime(59), "59s");
    }

    #[test]
    fn format_uptime_minutes() {
        assert_eq!(format_uptime(60), "1m00s");
        assert_eq!(format_uptime(312), "5m12s");
        assert_eq!(format_uptime(3599), "59m59s");
    }

    #[test]
    fn format_uptime_hours() {
        assert_eq!(format_uptime(3600), "1h00m");
        assert_eq!(format_uptime(11220), "3h07m");
        assert_eq!(format_uptime(86399), "23h59m");
    }

    #[test]
    fn format_uptime_days() {
        assert_eq!(format_uptime(86400), "1d00h");
        assert_eq!(format_uptime(187200), "2d04h");
    }

    #[test]
    fn signal_zero_on_self_returns_true() {
        let pid = std::process::id();
        assert!(signal_zero_alive(pid));
    }

    #[test]
    fn signal_zero_on_unlikely_pid_returns_false() {
        // PID 1 is init/launchd — alive but not signalable by non-root
        // (so kill returns -1 with EPERM, not 0). Test with a clearly
        // unused high PID instead — system limit is typically <100000.
        assert!(!signal_zero_alive(999_999));
    }
}
