//! `bwoc status [name]` — per-agent health + identity snapshot.
//!
//! Phase 2 ṭhiti starter. Without a name, prints a per-agent table
//! (id · role · backend · model · health). With a name, prints full
//! detail for that one agent — manifest fields, dir contents, health
//! probes, dates.
//!
//! Read-only — no process supervision (that needs Phase 2 control
//! socket). Health probes mirror `bwoc doctor`'s per-agent checks:
//! does the dir exist? AGENTS.md present? all 4 backend symlinks?
//!
//! Workspace resolution: explicit > BWOC_WORKSPACE > ancestor walk
//! > exit 2 (same chain as list / info / validate).

use std::path::{Path, PathBuf};

use bwoc_core::manifest::Manifest;
use bwoc_core::workspace::{AgentEntry, AgentsRegistry};

pub struct StatusArgs {
    pub name: Option<String>,
    pub workspace: Option<PathBuf>,
    pub json: bool,
}

const BACKEND_SYMLINKS: &[&str] = &["CLAUDE.md", "GEMINI.md", "CODEX.md", "KIMI.md"];

pub fn run(args: StatusArgs) -> i32 {
    let Some(root) = resolve_workspace(args.workspace) else {
        eprintln!(
            "bwoc status: no workspace found (no .bwoc/workspace.toml in cwd or ancestors). \
             Pass --workspace, set BWOC_WORKSPACE, or run `bwoc init` first."
        );
        return 2;
    };

    let registry = match AgentsRegistry::load(&root) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("bwoc status: failed to read agents.toml: {e}");
            return 1;
        }
    };

    // JSON branch — single shape for both "all" and "one" cases.
    if args.json {
        return emit_json(&root, &registry, args.name.as_deref());
    }

    match args.name {
        Some(name) => print_one(&root, &registry, &name),
        None => print_all(&root, &registry),
    }
}

fn emit_json(root: &Path, registry: &AgentsRegistry, name: Option<&str>) -> i32 {
    let agents: Vec<serde_json::Value> = registry
        .agents
        .iter()
        .filter(|a| {
            name.is_none_or(|n| {
                let lookup = if n.starts_with("agent-") {
                    n.to_string()
                } else {
                    format!("agent-{n}")
                };
                a.id == lookup
            })
        })
        .map(|a| {
            let health = probe(root, a);
            let (health_str, health_detail) = match &health {
                Health::Ok => ("ok", None),
                Health::Warn(m) => ("warn", Some(m.as_str())),
                Health::Fail(m) => ("fail", Some(m.as_str())),
            };
            let primary_model = read_primary_model(root, a);
            serde_json::json!({
                "id": a.id,
                "path": a.path,
                "backend": a.backend,
                "status": a.status,
                "incarnated": a.incarnated,
                "primary_model": primary_model,
                "health": health_str,
                "health_detail": health_detail,
            })
        })
        .collect();

    if name.is_some() && agents.is_empty() {
        eprintln!(
            "bwoc status: no agent named '{}' in workspace {}",
            name.unwrap_or(""),
            root.display()
        );
        return 2;
    }

    let value = serde_json::json!({
        "workspace": root.display().to_string(),
        "agents": agents,
    });
    match serde_json::to_string_pretty(&value) {
        Ok(s) => {
            println!("{s}");
            0
        }
        Err(e) => {
            eprintln!("bwoc status: failed to serialize JSON: {e}");
            1
        }
    }
}

fn print_all(root: &Path, registry: &AgentsRegistry) -> i32 {
    println!();
    println!("Workspace: {}", root.display());
    println!();
    if registry.agents.is_empty() {
        println!("(no agents registered — `bwoc new <name>` to incarnate the first)");
        println!();
        return 0;
    }

    println!(
        "{:<24} {:<8} {:<10} {:<24} PATH",
        "ID", "HEALTH", "BACKEND", "MODEL"
    );
    println!(
        "{:<24} {:<8} {:<10} {:<24} {}",
        "─".repeat(24),
        "─".repeat(8),
        "─".repeat(10),
        "─".repeat(24),
        "─".repeat(20),
    );
    let mut unhealthy = 0u32;
    for a in &registry.agents {
        let health = probe(root, a);
        let model = read_primary_model(root, a).unwrap_or_else(|| "—".to_string());
        let mark = match health {
            Health::Ok => "✓",
            Health::Warn(_) => "⚠",
            Health::Fail(_) => "✗",
        };
        if !matches!(health, Health::Ok) {
            unhealthy += 1;
        }
        println!(
            "{:<24} {:<8} {:<10} {:<24} {}",
            a.id, mark, a.backend, model, a.path
        );
    }
    println!();
    if unhealthy > 0 {
        println!(
            "{unhealthy} agent(s) need attention. Run `bwoc status <name>` for details, or `bwoc doctor` to scan + auto-fix safe issues."
        );
    } else {
        println!("All agents healthy. Run `bwoc status <name>` for per-agent detail.");
    }
    println!();
    0
}

fn print_one(root: &Path, registry: &AgentsRegistry, name: &str) -> i32 {
    let lookup_id = if name.starts_with("agent-") {
        name.to_string()
    } else {
        format!("agent-{name}")
    };
    let Some(entry) = registry.agents.iter().find(|a| a.id == lookup_id) else {
        eprintln!(
            "bwoc status: no agent named '{name}' in workspace {}. Try `bwoc list` to see what's registered.",
            root.display()
        );
        return 2;
    };

    let agent_path = root.join(&entry.path);
    let health = probe(root, entry);

    println!();
    println!("Agent: {}", entry.id);
    println!("={}", "=".repeat(entry.id.len() + 5));
    println!("  path:        {}", entry.path);
    println!("  backend:     {}", entry.backend);
    println!("  status:      {}", entry.status);
    println!("  incarnated:  {}", entry.incarnated);
    println!();
    match Manifest::load_from_path(&agent_path.join("config.manifest.json")) {
        Ok(m) => {
            println!("  Manifest:");
            println!("    role:           {}", m.agent_role);
            println!("    primaryModel:   {}", m.primary_model);
            if let Some(fb) = &m.fallback_model {
                println!("    fallbackModel:  {fb}");
            }
            println!("    memoryPath:     {}", m.memory_path);
            println!("    lintCmd:        {}", m.lint_cmd);
            println!("    formatCmd:      {}", m.format_cmd);
            println!("    testCmd:        {}", m.test_cmd);
            println!("    buildCmd:       {}", m.build_cmd);
            println!("    version:        {}", m.version);
        }
        Err(e) => println!("  Manifest:    (failed to read: {e})"),
    }
    println!();
    println!("  Health:");
    match &health {
        Health::Ok => println!("    ✓ all probes passed"),
        Health::Warn(msg) => println!("    ⚠ {msg}"),
        Health::Fail(msg) => println!("    ✗ {msg}"),
    }
    println!();
    if matches!(health, Health::Fail(_)) {
        return 2;
    }
    0
}

// --- helpers ---------------------------------------------------------------

#[derive(Debug)]
enum Health {
    Ok,
    Warn(String),
    Fail(String),
}

/// Per-agent health probe mirroring doctor's per-agent checks but
/// returning a single summarised verdict.
fn probe(root: &Path, a: &AgentEntry) -> Health {
    let p = root.join(&a.path);
    if !p.is_dir() {
        return Health::Fail(format!("directory missing: {}", p.display()));
    }
    if !p.join("AGENTS.md").is_file() {
        return Health::Fail(format!("missing AGENTS.md in {}", p.display()));
    }
    let missing: Vec<&str> = BACKEND_SYMLINKS
        .iter()
        .copied()
        .filter(|link| !p.join(link).exists())
        .collect();
    if !missing.is_empty() {
        return Health::Warn(format!(
            "missing backend symlinks: {} (rerun `bwoc doctor --auto`)",
            missing.join(", ")
        ));
    }
    if !p.join("config.manifest.json").is_file() {
        return Health::Warn("config.manifest.json missing".to_string());
    }
    Health::Ok
}

fn read_primary_model(root: &Path, a: &AgentEntry) -> Option<String> {
    let manifest = root.join(&a.path).join("config.manifest.json");
    Manifest::load_from_path(&manifest)
        .ok()
        .map(|m| m.primary_model)
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
    use bwoc_core::workspace::AgentEntry;
    use std::fs;

    #[test]
    fn probe_ok_for_complete_agent() {
        let root = std::env::temp_dir().join(format!("bwoc-status-ok-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let agent = root.join("agents/agent-alpha");
        fs::create_dir_all(&agent).unwrap();
        fs::write(agent.join("AGENTS.md"), "stub").unwrap();
        for link in BACKEND_SYMLINKS {
            fs::write(agent.join(link), "stub").unwrap();
        }
        fs::write(agent.join("config.manifest.json"), "{}").unwrap();
        let entry = AgentEntry {
            id: "agent-alpha".into(),
            path: "agents/agent-alpha".into(),
            backend: "claude".into(),
            incarnated: "2026-05-22".into(),
            status: "active".into(),
        };
        assert!(matches!(probe(&root, &entry), Health::Ok));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn probe_fail_for_missing_dir() {
        let root = std::env::temp_dir().join(format!("bwoc-status-miss-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let entry = AgentEntry {
            id: "agent-alpha".into(),
            path: "agents/agent-alpha".into(),
            backend: "claude".into(),
            incarnated: "2026-05-22".into(),
            status: "active".into(),
        };
        assert!(matches!(probe(&root, &entry), Health::Fail(_)));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn probe_warn_for_missing_symlinks() {
        let root = std::env::temp_dir().join(format!("bwoc-status-warn-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let agent = root.join("agents/agent-alpha");
        fs::create_dir_all(&agent).unwrap();
        fs::write(agent.join("AGENTS.md"), "stub").unwrap();
        // Note: NO symlinks created
        let entry = AgentEntry {
            id: "agent-alpha".into(),
            path: "agents/agent-alpha".into(),
            backend: "claude".into(),
            incarnated: "2026-05-22".into(),
            status: "active".into(),
        };
        assert!(matches!(probe(&root, &entry), Health::Warn(_)));
        let _ = fs::remove_dir_all(&root);
    }
}
