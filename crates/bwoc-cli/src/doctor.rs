//! `bwoc doctor` — environment + workspace diagnostic with optional auto-fix.
//!
//! Runs a set of checks, reports each as PASS / WARN / FAIL / FIXED, and
//! exits 0 if no FAILs remain after the (optional) auto-fix pass.
//!
//! Checks are intentionally narrow and predictable. Auto-fix only touches
//! things with one obvious correct answer (missing dirs, missing
//! symlinks). Anything ambiguous — malformed config, missing AGENTS.md,
//! missing backend CLI — is reported, never silently rewritten.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use bwoc_core::workspace::{AgentEntry, AgentsRegistry, Workspace};

use crate::user_home;

pub struct DoctorArgs {
    pub path: Option<PathBuf>,
    pub auto: bool,
}

#[derive(Debug, Clone)]
enum Status {
    Pass,
    Warn(String),
    Fail(String),
    Fixed(String),
}

struct CheckResult {
    name: String,
    status: Status,
}

/// Standard scaffold dirs created by `bwoc init`. Mirror of
/// `init.rs::WORKSPACE_EXTRAS`. Kept local to avoid cross-module
/// re-export ceremony.
const WORKSPACE_EXTRAS: &[&str] = &["projects", "notes"];
/// Backend symlink files inside each incarnated agent. Mirror of
/// `new.rs::create_symlinks`.
const BACKEND_SYMLINKS: &[&str] = &["CLAUDE.md", "GEMINI.md", "CODEX.md", "KIMI.md"];

pub fn run(args: DoctorArgs) -> i32 {
    let mut results: Vec<CheckResult> = Vec::new();

    // 1. ~/.bwoc/ directory.
    results.push(check_user_home(args.auto));

    // 2. Backend CLIs on PATH (informational).
    results.push(check_backends());

    // 3. Workspace-level checks if we're inside one.
    let ws_root = resolve_workspace_root(args.path);
    match ws_root {
        Some(root) => {
            results.push(check_workspace_toml(&root));
            results.push(check_agents_toml(&root));
            results.push(check_scaffold_dirs(&root, args.auto));
            // Per-agent symlinks (only if registry parsed).
            if AgentsRegistry::load(&root).is_ok() {
                for r in check_agent_symlinks(&root, args.auto) {
                    results.push(r);
                }
                // Stale `agent.pid` cleanup (foundation laid by
                // `bwoc-agent --serve`).
                for r in check_stale_pids(&root, args.auto) {
                    results.push(r);
                }
                // Stale `agent.sock` cleanup (sibling of the PID sweep).
                for r in check_stale_sockets(&root, args.auto) {
                    results.push(r);
                }
            }
        }
        None => results.push(CheckResult {
            name: "workspace".into(),
            status: Status::Warn(
                "not inside a BWOC workspace — workspace-level checks skipped. \
                 Pass a path or run `bwoc init` first."
                    .into(),
            ),
        }),
    }

    print_report(&results, args.auto);

    if results.iter().any(|r| matches!(r.status, Status::Fail(_))) {
        2
    } else {
        0
    }
}

// --- individual checks -----------------------------------------------------

fn check_user_home(auto: bool) -> CheckResult {
    match user_home::bwoc_home() {
        Ok(p) if p.is_dir() => CheckResult {
            name: "~/.bwoc/".into(),
            status: Status::Pass,
        },
        Ok(p) if auto => match user_home::ensure_initialized() {
            Ok(_) => CheckResult {
                name: "~/.bwoc/".into(),
                status: Status::Fixed(format!("created {}", p.display())),
            },
            Err(e) => CheckResult {
                name: "~/.bwoc/".into(),
                status: Status::Fail(format!("could not create: {e}")),
            },
        },
        Ok(p) => CheckResult {
            name: "~/.bwoc/".into(),
            status: Status::Fail(format!(
                "missing — rerun any bwoc command or pass --auto to create at {}",
                p.display()
            )),
        },
        Err(e) => CheckResult {
            name: "~/.bwoc/".into(),
            status: Status::Fail(format!("cannot resolve: {e}")),
        },
    }
}

fn check_backends() -> CheckResult {
    let backends = ["claude", "gemini", "codex", "kimi"];
    let mut available = Vec::new();
    for b in backends {
        if which(b).is_some() {
            available.push(b);
        }
    }
    let name = "backends on PATH".to_string();
    if available.is_empty() {
        CheckResult {
            name,
            status: Status::Warn(
                "no backend CLI on PATH (claude/gemini/codex/kimi). `bwoc spawn` will fail.".into(),
            ),
        }
    } else {
        CheckResult {
            name,
            status: Status::Pass,
        }
    }
}

fn check_workspace_toml(root: &Path) -> CheckResult {
    match Workspace::load(root) {
        Ok(_) => CheckResult {
            name: ".bwoc/workspace.toml".into(),
            status: Status::Pass,
        },
        Err(e) => CheckResult {
            name: ".bwoc/workspace.toml".into(),
            status: Status::Fail(format!("parse error: {e}")),
        },
    }
}

fn check_agents_toml(root: &Path) -> CheckResult {
    match AgentsRegistry::load(root) {
        Ok(_) => CheckResult {
            name: ".bwoc/agents.toml".into(),
            status: Status::Pass,
        },
        Err(e) => CheckResult {
            name: ".bwoc/agents.toml".into(),
            status: Status::Fail(format!("parse error: {e}")),
        },
    }
}

fn check_scaffold_dirs(root: &Path, auto: bool) -> CheckResult {
    let ws = Workspace::load(root).ok();
    let agents_dir = ws
        .map(|w| w.defaults.agents_dir)
        .unwrap_or_else(|| "agents".into());
    let mut to_check = vec![agents_dir.as_str()];
    to_check.extend(WORKSPACE_EXTRAS.iter().copied());

    let missing: Vec<&str> = to_check
        .into_iter()
        .filter(|d| !root.join(d).is_dir())
        .collect();

    let name = "scaffold dirs".into();
    if missing.is_empty() {
        return CheckResult {
            name,
            status: Status::Pass,
        };
    }
    if auto {
        let mut created = Vec::new();
        for d in &missing {
            if fs::create_dir_all(root.join(d)).is_ok() {
                created.push(*d);
            }
        }
        return CheckResult {
            name,
            status: Status::Fixed(format!("created: {}", created.join(", "))),
        };
    }
    CheckResult {
        name,
        status: Status::Fail(format!(
            "missing: {} (rerun with --auto to create)",
            missing.join(", ")
        )),
    }
}

/// Detect `<agent>/.bwoc/agent.pid` files whose process is no longer
/// alive (signal-0 fails). With --auto, remove the stale file.
///
/// Mirrors the liveness check in `status.rs::signal_zero_alive`;
/// duplicated to keep the doctor module self-contained (extract to a
/// shared helper when a third caller appears).
fn check_stale_pids(root: &Path, auto: bool) -> Vec<CheckResult> {
    let Ok(registry) = AgentsRegistry::load(root) else {
        return vec![];
    };
    let mut out = Vec::new();
    for a in &registry.agents {
        let pid_path = root.join(&a.path).join(".bwoc/agent.pid");
        if !pid_path.is_file() {
            continue; // Not a daemon-mode agent; nothing to clean.
        }
        let raw = match std::fs::read_to_string(&pid_path) {
            Ok(s) => s,
            Err(e) => {
                out.push(CheckResult {
                    name: format!("agent pid: {}", a.id),
                    status: Status::Warn(format!("could not read {}: {e}", pid_path.display())),
                });
                continue;
            }
        };
        let pid: u32 = match raw.trim().parse() {
            Ok(n) => n,
            Err(_) => {
                // Malformed PID file — auto-fix would remove it.
                if auto {
                    let _ = std::fs::remove_file(&pid_path);
                    out.push(CheckResult {
                        name: format!("agent pid: {}", a.id),
                        status: Status::Fixed(format!(
                            "removed malformed PID file: {}",
                            pid_path.display()
                        )),
                    });
                } else {
                    out.push(CheckResult {
                        name: format!("agent pid: {}", a.id),
                        status: Status::Fail(format!(
                            "malformed PID file at {} (rerun with --auto to remove)",
                            pid_path.display()
                        )),
                    });
                }
                continue;
            }
        };
        if signal_zero_alive(pid) {
            // Process exists — not stale, skip silently.
            continue;
        }
        // Stale.
        if auto {
            if std::fs::remove_file(&pid_path).is_ok() {
                out.push(CheckResult {
                    name: format!("agent pid: {}", a.id),
                    status: Status::Fixed(format!("removed stale PID file (pid {pid} not alive)")),
                });
            } else {
                out.push(CheckResult {
                    name: format!("agent pid: {}", a.id),
                    status: Status::Fail(format!(
                        "stale PID {pid} but couldn't remove {}",
                        pid_path.display()
                    )),
                });
            }
        } else {
            out.push(CheckResult {
                name: format!("agent pid: {}", a.id),
                status: Status::Fail(format!(
                    "stale PID file at {} (pid {pid} not alive; rerun with --auto to remove)",
                    pid_path.display()
                )),
            });
        }
    }
    out
}

#[cfg(unix)]
fn signal_zero_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}

#[cfg(not(unix))]
fn signal_zero_alive(_pid: u32) -> bool {
    false
}

/// Detect `<agent>/.bwoc/agent.sock` files that no live process owns.
/// A socket is considered stale when ANY of:
///   - sibling `agent.pid` missing  (orphan socket from a crash)
///   - sibling `agent.pid` exists but the pid isn't alive
/// With --auto, remove the stale socket. Live sockets are left alone.
fn check_stale_sockets(root: &Path, auto: bool) -> Vec<CheckResult> {
    let Ok(registry) = AgentsRegistry::load(root) else {
        return vec![];
    };
    let mut out = Vec::new();
    for a in &registry.agents {
        let bwoc = root.join(&a.path).join(".bwoc");
        let sock_path = bwoc.join("agent.sock");
        if !sock_path.exists() {
            continue;
        }
        // Is there a live owning process?
        let pid_path = bwoc.join("agent.pid");
        let owner_alive = std::fs::read_to_string(&pid_path)
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .map(signal_zero_alive)
            .unwrap_or(false);
        if owner_alive {
            continue; // Socket has a live owner; not stale.
        }
        // Stale.
        if auto {
            if std::fs::remove_file(&sock_path).is_ok() {
                out.push(CheckResult {
                    name: format!("agent sock: {}", a.id),
                    status: Status::Fixed(format!("removed stale socket (no live owning process)")),
                });
            } else {
                out.push(CheckResult {
                    name: format!("agent sock: {}", a.id),
                    status: Status::Fail(format!(
                        "stale socket at {} but couldn't remove",
                        sock_path.display()
                    )),
                });
            }
        } else {
            out.push(CheckResult {
                name: format!("agent sock: {}", a.id),
                status: Status::Fail(format!(
                    "stale socket at {} (no live owner; rerun with --auto to remove)",
                    sock_path.display()
                )),
            });
        }
    }
    out
}

fn check_agent_symlinks(root: &Path, auto: bool) -> Vec<CheckResult> {
    let Ok(registry) = AgentsRegistry::load(root) else {
        return vec![];
    };
    registry
        .agents
        .iter()
        .map(|a| check_single_agent(root, a, auto))
        .collect()
}

fn check_single_agent(root: &Path, a: &AgentEntry, auto: bool) -> CheckResult {
    let agent_path = root.join(&a.path);
    let name = format!("agent: {}", a.id);

    if !agent_path.is_dir() {
        return CheckResult {
            name,
            status: Status::Fail(format!("agent directory missing: {}", agent_path.display())),
        };
    }
    let agents_md = agent_path.join("AGENTS.md");
    if !agents_md.is_file() {
        return CheckResult {
            name,
            status: Status::Fail(format!("missing AGENTS.md in {}", agent_path.display())),
        };
    }

    let mut missing = Vec::new();
    for link in BACKEND_SYMLINKS {
        let lp = agent_path.join(link);
        if !lp.exists() {
            missing.push(*link);
        }
    }
    if missing.is_empty() {
        return CheckResult {
            name,
            status: Status::Pass,
        };
    }
    if auto {
        let mut fixed = Vec::new();
        for link in &missing {
            let target = agent_path.join(link);
            #[cfg(unix)]
            if std::os::unix::fs::symlink("AGENTS.md", &target).is_ok() {
                fixed.push(*link);
            }
        }
        return CheckResult {
            name,
            status: Status::Fixed(format!("recreated symlinks: {}", fixed.join(", "))),
        };
    }
    CheckResult {
        name,
        status: Status::Fail(format!(
            "missing backend symlinks: {} (rerun with --auto to recreate)",
            missing.join(", ")
        )),
    }
}

// --- helpers ---------------------------------------------------------------

/// `which`-equivalent: returns the first PATH entry that contains an
/// executable named `cmd`. Avoids depending on the `which` crate.
fn which(cmd: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(cmd);
        if Command::new(&candidate).arg("--version").output().is_ok() {
            return Some(candidate);
        }
    }
    None
}

/// Workspace resolution similar to workspace.rs::find_workspace_root —
/// explicit > BWOC_WORKSPACE env > ancestor walk from cwd > None.
fn resolve_workspace_root(explicit: Option<PathBuf>) -> Option<PathBuf> {
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

fn print_report(results: &[CheckResult], auto: bool) {
    println!();
    println!("BWOC Doctor");
    println!("===========");
    if auto {
        println!("(running in --auto mode: safe issues will be fixed in place)");
    }
    println!();
    let mut pass = 0u32;
    let mut warn = 0u32;
    let mut fail = 0u32;
    let mut fixed = 0u32;
    for r in results {
        match &r.status {
            Status::Pass => {
                pass += 1;
                println!("  PASS   {}", r.name);
            }
            Status::Warn(msg) => {
                warn += 1;
                println!("  WARN   {} — {msg}", r.name);
            }
            Status::Fail(msg) => {
                fail += 1;
                println!("  FAIL   {} — {msg}", r.name);
            }
            Status::Fixed(msg) => {
                fixed += 1;
                println!("  FIXED  {} — {msg}", r.name);
            }
        }
    }
    println!();
    println!("===========\n{pass} pass · {warn} warn · {fail} fail · {fixed} fixed");
    if fail > 0 {
        println!();
        if auto {
            println!(
                "Some failures couldn't be auto-fixed (need user attention — \
                 malformed config, missing AGENTS.md, missing backend CLI)."
            );
        } else {
            println!("Run `bwoc doctor --auto` to attempt auto-fix on safe issues.");
        }
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_scaffold_dirs_reported_when_no_auto() {
        let base = std::env::temp_dir().join(format!("bwoc-doctor-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join(".bwoc")).unwrap();
        fs::write(
            base.join(".bwoc/workspace.toml"),
            "[workspace]\nname=\"x\"\nversion=\"0.1.0\"\ncreated=\"x\"\n",
        )
        .unwrap();
        let r = check_scaffold_dirs(&base, /*auto=*/ false);
        assert!(matches!(r.status, Status::Fail(_)));
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn missing_scaffold_dirs_fixed_with_auto() {
        let base = std::env::temp_dir().join(format!("bwoc-doctor-fix-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join(".bwoc")).unwrap();
        fs::write(
            base.join(".bwoc/workspace.toml"),
            "[workspace]\nname=\"x\"\nversion=\"0.1.0\"\ncreated=\"x\"\n",
        )
        .unwrap();
        let r = check_scaffold_dirs(&base, /*auto=*/ true);
        assert!(matches!(r.status, Status::Fixed(_)));
        assert!(base.join("agents").is_dir());
        assert!(base.join("projects").is_dir());
        assert!(base.join("notes").is_dir());
        let _ = fs::remove_dir_all(&base);
    }
}
