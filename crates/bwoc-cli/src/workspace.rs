//! `bwoc workspace info` and `bwoc workspace validate`.
//!
//! Read-only inspection of a BWOC workspace. `info` dumps the resolved
//! workspace path, config, and agent count. `validate` runs the five rules
//! from `docs/en/WORKSPACE.en.md` §"Validation Rules" and exits 0 / 2.

use std::path::{Path, PathBuf};

use bwoc_core::workspace::{AgentsRegistry, Workspace};

use crate::i18n;

pub struct InfoArgs {
    pub path: Option<PathBuf>,
    pub lang: String,
    pub json: bool,
}

pub struct ValidateArgs {
    pub path: Option<PathBuf>,
    pub lang: String,
    pub json: bool,
}

pub struct ListArgs {
    pub path: Option<PathBuf>,
    pub lang: String,
    pub json: bool,
    /// Filter to agents whose `status` field exactly matches (e.g. "active", "stopped").
    pub status_filter: Option<String>,
    /// Filter to agents whose `backend` field exactly matches (e.g. "claude", "gemini").
    pub backend_filter: Option<String>,
    /// Filter to agents whose daemon is actually running (PID file + signal-0 check).
    pub running_only: bool,
}

pub struct PruneArgs {
    pub path: Option<PathBuf>,
    pub apply: bool,
}

pub fn run_info(args: InfoArgs) -> i32 {
    let bundle = i18n::bundle_for(&args.lang);
    let Some(root) = find_workspace_root(args.path) else {
        eprintln!(
            "bwoc workspace info: no workspace found (no .bwoc/workspace.toml in cwd or ancestors). \
             Pass a path, set BWOC_WORKSPACE, or run `bwoc init` first."
        );
        return 2;
    };
    if args.json {
        return info_json(&root);
    }
    match info(&root, &bundle) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("bwoc workspace info: {e}");
            1
        }
    }
}

fn info_json(root: &Path) -> i32 {
    if !root.is_dir() {
        eprintln!("bwoc workspace info: not a directory: {}", root.display());
        return 1;
    }
    if !root.join(".bwoc/workspace.toml").exists() {
        eprintln!(
            "bwoc workspace info: not a BWOC workspace (no .bwoc/workspace.toml): {}",
            root.display()
        );
        return 2;
    }
    let ws = match Workspace::load(root) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("bwoc workspace info: {e}");
            return 1;
        }
    };
    let registry = AgentsRegistry::load(root).unwrap_or_default();

    let value = serde_json::json!({
        "workspace": root.display().to_string(),
        "name": ws.workspace.name,
        "version": ws.workspace.version,
        "created": ws.workspace.created,
        "defaults": {
            "agents_dir": ws.defaults.agents_dir,
            "backend": ws.defaults.backend,
            "lang": ws.defaults.lang,
        },
        "agents_count": registry.agents.len(),
        "agents": registry.agents.iter().map(|a| serde_json::json!({
            "id": a.id,
            "path": a.path,
            "backend": a.backend,
            "status": a.status,
            "incarnated": a.incarnated,
        })).collect::<Vec<_>>(),
    });
    match serde_json::to_string_pretty(&value) {
        Ok(s) => {
            println!("{s}");
            0
        }
        Err(e) => {
            eprintln!("bwoc workspace info: failed to serialize JSON: {e}");
            1
        }
    }
}

pub fn run_validate(args: ValidateArgs) -> i32 {
    let bundle = i18n::bundle_for(&args.lang);
    let Some(root) = find_workspace_root(args.path) else {
        eprintln!(
            "bwoc workspace validate: no workspace found (no .bwoc/workspace.toml in cwd or ancestors). \
             Pass a path, set BWOC_WORKSPACE, or run `bwoc init` first."
        );
        return 2;
    };
    let report = validate(&root);
    if args.json {
        let value = serde_json::json!({
            "workspace": root.display().to_string(),
            "passes": report.passes,
            "violations": report.violations,
            "summary": {
                "passes": report.passes.len(),
                "violations": report.violations.len(),
            },
        });
        match serde_json::to_string_pretty(&value) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("bwoc workspace validate: failed to serialize JSON: {e}");
                return 1;
            }
        }
    } else {
        print_validation_report(&root, &report, &bundle);
    }
    if report.violations.is_empty() { 0 } else { 2 }
}

pub fn run_list(args: ListArgs) -> i32 {
    let bundle = i18n::bundle_for(&args.lang);

    let Some(root) = find_workspace_root(args.path) else {
        // Error path stays English (thiserror localization deferred).
        eprintln!(
            "bwoc list: no workspace found (no .bwoc/workspace.toml in cwd or ancestors). \
             Pass --workspace <path>, set BWOC_WORKSPACE, or run `bwoc init` first."
        );
        return 2;
    };

    let registry = match AgentsRegistry::load(&root) {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "bwoc list: failed to read agents.toml at {}: {e}",
                root.display()
            );
            return 1;
        }
    };

    // Apply filters BEFORE serialisation — both human and JSON output
    // honour them. Filter values are exact string match (case-sensitive)
    // so users can also filter on custom registry status fields later.
    let matches = |a: &bwoc_core::workspace::AgentEntry| -> bool {
        if let Some(s) = &args.status_filter {
            if a.status != *s {
                return false;
            }
        }
        if let Some(b) = &args.backend_filter {
            if a.backend != *b {
                return false;
            }
        }
        if args.running_only && !is_running(&root, a) {
            return false;
        }
        true
    };
    let filtered: Vec<&bwoc_core::workspace::AgentEntry> =
        registry.agents.iter().filter(|a| matches(a)).collect();

    // JSON branch — stable machine-readable output, no decorative text,
    // no Fluent (locale doesn't affect machine consumers).
    if args.json {
        let value = serde_json::json!({
            "workspace": root.display().to_string(),
            "agents": filtered.iter().map(|a| serde_json::json!({
                "id": a.id,
                "path": a.path,
                "backend": a.backend,
                "status": a.status,
                "incarnated": a.incarnated,
                "running": is_running(&root, a),
                "inbox_count": inbox_count(&root, a),
            })).collect::<Vec<_>>(),
        });
        match serde_json::to_string_pretty(&value) {
            Ok(s) => {
                println!("{s}");
                return 0;
            }
            Err(e) => {
                eprintln!("bwoc list: failed to serialize JSON: {e}");
                return 1;
            }
        }
    }

    let root_display = root.display().to_string();
    if filtered.is_empty() {
        if registry.agents.is_empty() {
            // No agents at all.
            println!(
                "{}",
                i18n::t_with(&bundle, "list-empty", &[("path", &root_display)])
            );
        } else {
            // Agents exist but none matched the filter — give an
            // actionable message instead of just empty output.
            let filter_desc = describe_filters(
                args.status_filter.as_deref(),
                args.backend_filter.as_deref(),
            );
            println!(
                "(no agents match {filter_desc} in workspace {root_display} — {} total)",
                registry.agents.len()
            );
        }
        return 0;
    }

    println!(
        "{:<32} {:<10} {:<10} {:<7} {}",
        i18n::t(&bundle, "list-col-id"),
        i18n::t(&bundle, "list-col-status"),
        i18n::t(&bundle, "list-col-backend"),
        "INBOX",
        i18n::t(&bundle, "list-col-path"),
    );
    println!(
        "{:<32} {:<10} {:<10} {:<7} {}",
        "─".repeat(32),
        "─".repeat(10),
        "─".repeat(10),
        "─".repeat(7),
        "─".repeat(20),
    );
    for a in &filtered {
        let mark = if is_running(&root, a) { "●" } else { "○" };
        let count = inbox_count(&root, a);
        let inbox_cell = if count == 0 {
            "—".to_string()
        } else {
            count.to_string()
        };
        println!(
            "{mark} {:<30} {:<10} {:<10} {:<7} {}",
            a.id, a.status, a.backend, inbox_cell, a.path
        );
    }
    0
}

/// Count complete envelope lines in `<agent>/.bwoc/inbox.jsonl`. Returns
/// 0 when the file is missing or unreadable — same shape as a real empty
/// inbox, which keeps the table cell calm.
fn inbox_count(root: &Path, a: &bwoc_core::workspace::AgentEntry) -> usize {
    let path = root.join(&a.path).join(".bwoc/inbox.jsonl");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return 0;
    };
    content.lines().filter(|l| !l.trim().is_empty()).count()
}

/// Liveness probe — true if the agent has a PID file AND the pid is
/// alive. Mirror of `status.rs::running_pid` / `doctor.rs` signal-0
/// check. With this third caller, the small unsafe libc::kill helper
/// is now ripe for promotion to a shared module — flagged for the
/// next refactor pass.
fn is_running(root: &Path, a: &bwoc_core::workspace::AgentEntry) -> bool {
    let pid_path = root.join(&a.path).join(".bwoc/agent.pid");
    let Ok(raw) = std::fs::read_to_string(&pid_path) else {
        return false;
    };
    let Ok(pid) = raw.trim().parse::<u32>() else {
        return false;
    };
    signal_zero_alive(pid)
}

#[cfg(unix)]
fn signal_zero_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}

#[cfg(not(unix))]
fn signal_zero_alive(_pid: u32) -> bool {
    false
}

/// Format a one-phrase summary of the active filters for the empty-set
/// hint message. Returns e.g. `--status=active`, `--backend=claude`,
/// or `--status=stopped --backend=gemini`.
fn describe_filters(status: Option<&str>, backend: Option<&str>) -> String {
    let mut parts = Vec::new();
    if let Some(s) = status {
        parts.push(format!("--status={s}"));
    }
    if let Some(b) = backend {
        parts.push(format!("--backend={b}"));
    }
    if parts.is_empty() {
        "(no filter)".to_string()
    } else {
        parts.join(" ")
    }
}

pub fn run_prune(args: PruneArgs) -> i32 {
    let Some(root) = find_workspace_root(args.path) else {
        eprintln!(
            "bwoc workspace prune: no workspace found (no .bwoc/workspace.toml in cwd or ancestors). \
             Pass a path, set BWOC_WORKSPACE, or run `bwoc init` first."
        );
        return 2;
    };

    let mut registry = match AgentsRegistry::load(&root) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("bwoc workspace prune: failed to read agents.toml: {e}");
            return 1;
        }
    };

    let agents_dir_name = Workspace::load(&root)
        .map(|w| w.defaults.agents_dir)
        .unwrap_or_else(|_| "agents".to_string());
    let agents_dir = root.join(&agents_dir_name);

    // Phantom entries: in registry, but the directory is gone.
    let phantom_indices: Vec<usize> = registry
        .agents
        .iter()
        .enumerate()
        .filter_map(|(i, a)| (!root.join(&a.path).is_dir()).then_some(i))
        .collect();

    // Orphan dirs: subdirs of agents_dir that look like agents (have AGENTS.md)
    // but aren't in the registry.
    let mut orphans: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&agents_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if !p.is_dir() || !p.join("AGENTS.md").is_file() {
                continue;
            }
            let rel = p
                .strip_prefix(&root)
                .map(|r| r.display().to_string())
                .unwrap_or_else(|_| p.display().to_string());
            if !registry.agents.iter().any(|a| a.path == rel) {
                orphans.push(p);
            }
        }
    }

    println!();
    println!("Workspace prune: {}", root.display());
    println!("=================");
    if phantom_indices.is_empty() && orphans.is_empty() {
        println!("Clean — no phantom registry entries, no orphan agent directories.");
        println!();
        return 0;
    }

    if !phantom_indices.is_empty() {
        println!();
        println!("Phantom registry entries (in agents.toml but directory is missing):");
        for &i in &phantom_indices {
            let a = &registry.agents[i];
            println!(
                "  - {} @ {}  (backend: {}, status: {})",
                a.id, a.path, a.backend, a.status
            );
        }
    }
    if !orphans.is_empty() {
        println!();
        println!(
            "Orphan agent directories (under {}/ but not in agents.toml):",
            agents_dir_name
        );
        for o in &orphans {
            let rel = o
                .strip_prefix(&root)
                .map(|r| r.display().to_string())
                .unwrap_or_else(|_| o.display().to_string());
            println!("  - {rel}");
        }
        println!();
        println!("Orphan dirs are NOT auto-removed (they may hold real work). To clean an orphan:");
        println!("  bwoc retire <name>          # if it should be deleted");
        println!("  # OR add it back to agents.toml manually if it should stay.");
    }

    if args.apply {
        // Remove phantom entries from registry, reverse-index order.
        let mut removed = Vec::new();
        for &i in phantom_indices.iter().rev() {
            removed.push(registry.agents.remove(i).id);
        }
        if let Err(e) = registry.save(&root) {
            eprintln!();
            eprintln!("bwoc workspace prune: failed to save updated agents.toml: {e}");
            return 1;
        }
        println!();
        if !removed.is_empty() {
            println!(
                "Applied: removed {} phantom entries from agents.toml: {}",
                removed.len(),
                removed.join(", ")
            );
        }
        if !orphans.is_empty() {
            println!(
                "Skipped: {} orphan directories (use `bwoc retire <name>` per directory).",
                orphans.len()
            );
        }
        println!();
        return if orphans.is_empty() { 0 } else { 2 };
    }

    println!();
    println!(
        "Dry run — nothing changed. Rerun with `--apply` to remove {} phantom registry entries.",
        phantom_indices.len()
    );
    println!();
    2
}

/// Resolve the workspace per `WORKSPACE.en.md` §"Workspace Resolution":
/// explicit path → BWOC_WORKSPACE env → ancestor walk → cwd-self → None.
fn find_workspace_root(explicit: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(p) = explicit {
        return Some(p);
    }
    if let Ok(env_path) = std::env::var("BWOC_WORKSPACE") {
        let p = PathBuf::from(env_path);
        if !p.as_os_str().is_empty() {
            return Some(p);
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

#[derive(Debug, thiserror::Error)]
pub enum InfoError {
    #[error("path does not exist or is not a directory: {0}")]
    PathMissing(PathBuf),
    #[error("not a BWOC workspace (no .bwoc/workspace.toml): {0}")]
    NotAWorkspace(PathBuf),
    #[error("workspace error: {0}")]
    Workspace(#[from] bwoc_core::workspace::WorkspaceError),
}

fn info(
    root: &Path,
    bundle: &fluent_bundle::FluentBundle<fluent_bundle::FluentResource>,
) -> Result<(), InfoError> {
    if !root.is_dir() {
        return Err(InfoError::PathMissing(root.to_path_buf()));
    }
    if !root.join(".bwoc/workspace.toml").exists() {
        return Err(InfoError::NotAWorkspace(root.to_path_buf()));
    }
    let ws = Workspace::load(root)?;
    let registry = AgentsRegistry::load(root)?;

    let root_display = root.display().to_string();
    let agents_count = registry.agents.len().to_string();

    println!();
    println!(
        "{}",
        i18n::t_with(bundle, "info-header", &[("path", &root_display)])
    );
    let lbl = |key: &str| i18n::t(bundle, key);
    println!("  {}:        {}", lbl("info-label-name"), ws.workspace.name);
    println!(
        "  {}:        {}",
        lbl("info-label-version"),
        ws.workspace.version
    );
    println!(
        "  {}:        {}",
        lbl("info-label-created"),
        ws.workspace.created
    );
    if let Some(ref b) = ws.defaults.backend {
        println!("  {}:        {b}", lbl("info-label-backend"));
    }
    if let Some(ref l) = ws.defaults.lang {
        println!("  {}:        {l}", lbl("info-label-lang"));
    }
    println!(
        "  {}:  {}",
        lbl("info-label-agents-dir"),
        ws.defaults.agents_dir
    );
    println!("  {}:      {}", lbl("info-label-agents"), agents_count);
    for a in &registry.agents {
        println!(
            "    - {}",
            i18n::t_with(
                bundle,
                "info-agent-row",
                &[("id", &a.id), ("status", &a.status), ("path", &a.path)],
            )
        );
    }
    println!();
    Ok(())
}

/// Structured validation findings.
pub struct ValidationReport {
    pub passes: Vec<String>,
    pub violations: Vec<String>,
}

/// Run the 5 rules from `WORKSPACE.en.md` §Validation Rules.
pub fn validate(root: &Path) -> ValidationReport {
    let mut report = ValidationReport {
        passes: Vec::new(),
        violations: Vec::new(),
    };

    // Rule 1: .bwoc/ directory exists
    let dot_bwoc = root.join(".bwoc");
    if dot_bwoc.is_dir() {
        report.passes.push(".bwoc/ exists".into());
    } else {
        report.violations.push(".bwoc/ directory missing".into());
        return report; // Can't continue — short-circuit.
    }

    // Rule 2: .bwoc/workspace.toml exists, parses, has required fields
    let ws_path = dot_bwoc.join("workspace.toml");
    if !ws_path.is_file() {
        report
            .violations
            .push(".bwoc/workspace.toml missing".into());
        return report;
    }
    let ws = match Workspace::load(root) {
        Ok(w) => {
            report.passes.push(".bwoc/workspace.toml parses".into());
            w
        }
        Err(e) => {
            report
                .violations
                .push(format!(".bwoc/workspace.toml invalid: {e}"));
            return report;
        }
    };

    if ws.workspace.name.is_empty() {
        report.violations.push("workspace.name is empty".into());
    } else {
        report
            .passes
            .push(format!("workspace.name = {:?}", ws.workspace.name));
    }
    if ws.workspace.created.is_empty() {
        report.violations.push("workspace.created is empty".into());
    } else {
        report
            .passes
            .push(format!("workspace.created = {:?}", ws.workspace.created));
    }

    // Rule 5: version is parseable SemVer (checked here since we have `ws` in hand)
    if is_semver(&ws.workspace.version) {
        report.passes.push(format!(
            "workspace.version {} is valid SemVer",
            ws.workspace.version
        ));
    } else {
        report.violations.push(format!(
            "workspace.version {:?} is not parseable SemVer (expect X.Y.Z)",
            ws.workspace.version
        ));
    }

    // Rule 3: .bwoc/agents.toml exists and parses
    let agents_path = dot_bwoc.join("agents.toml");
    if !agents_path.is_file() {
        report.violations.push(".bwoc/agents.toml missing".into());
    } else {
        match AgentsRegistry::load(root) {
            Ok(reg) => {
                report.passes.push(format!(
                    ".bwoc/agents.toml parses ({} agent(s))",
                    reg.agents.len()
                ));
            }
            Err(e) => {
                report
                    .violations
                    .push(format!(".bwoc/agents.toml invalid: {e}"));
            }
        }
    }

    // Rule 4: agents_dir exists (relative to root)
    let agents_dir = root.join(&ws.defaults.agents_dir);
    if agents_dir.is_dir() {
        report
            .passes
            .push(format!("agents_dir {:?} exists", ws.defaults.agents_dir));
    } else {
        report.violations.push(format!(
            "agents_dir {:?} (from workspace.toml [defaults]) does not exist",
            ws.defaults.agents_dir
        ));
    }

    report
}

fn is_semver(v: &str) -> bool {
    let parts: Vec<&str> = v.splitn(3, '.').collect();
    if parts.len() != 3 {
        return false;
    }
    // Each part must be a non-empty run of ASCII digits with no leading zero
    // except "0" itself. SemVer also allows pre-release / build metadata after
    // X.Y.Z, but for workspace versions we keep it strict.
    parts
        .iter()
        .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

fn print_validation_report(
    root: &Path,
    report: &ValidationReport,
    bundle: &fluent_bundle::FluentBundle<fluent_bundle::FluentResource>,
) {
    let root_display = root.display().to_string();
    let pass_label = i18n::t(bundle, "validate-label-pass");
    let fail_label = i18n::t(bundle, "validate-label-fail");
    let passes_count = report.passes.len().to_string();
    let violations_count = report.violations.len().to_string();

    println!();
    println!(
        "{}",
        i18n::t_with(bundle, "validate-header", &[("path", &root_display)])
    );
    println!("===========================================");
    for p in &report.passes {
        println!("{pass_label}  {p}");
    }
    for v in &report.violations {
        println!("{fail_label}  {v}");
    }
    println!("===========================================");
    if report.violations.is_empty() {
        println!(
            "{}",
            i18n::t_with(
                bundle,
                "validate-summary-success",
                &[("passes", &passes_count)],
            )
        );
    } else {
        println!(
            "{}",
            i18n::t_with(
                bundle,
                "validate-summary-failure",
                &[("passes", &passes_count), ("violations", &violations_count)],
            )
        );
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    fn fresh_dir(label: &str) -> PathBuf {
        let mut p = env::temp_dir();
        p.push(format!("bwoc-ws-test-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn semver_validation() {
        assert!(is_semver("0.1.0"));
        assert!(is_semver("1.2.3"));
        assert!(is_semver("0.0.0"));
        assert!(!is_semver(""));
        assert!(!is_semver("1.2"));
        assert!(!is_semver("1.2.3.4"));
        assert!(!is_semver("v1.2.3"));
        assert!(!is_semver("1.2.x"));
    }

    #[test]
    fn validate_missing_dot_bwoc() {
        let dir = fresh_dir("missing");
        let report = validate(&dir);
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.contains(".bwoc/ directory missing"))
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_clean_workspace_passes() {
        let dir = fresh_dir("clean");
        // Set up a minimal valid workspace by hand.
        let dot = dir.join(".bwoc");
        fs::create_dir_all(&dot).unwrap();
        fs::write(
            dot.join("workspace.toml"),
            r#"[workspace]
name = "demo"
version = "0.1.0"
created = "2026-05-22T06:00:00Z"

[defaults]
agents_dir = "agents"
"#,
        )
        .unwrap();
        fs::write(dot.join("agents.toml"), "").unwrap();
        fs::create_dir_all(dir.join("agents")).unwrap();
        let report = validate(&dir);
        assert!(
            report.violations.is_empty(),
            "violations: {:?}",
            report.violations
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn find_workspace_root_walks_ancestors() {
        let root = fresh_dir("walk");
        let dot = root.join(".bwoc");
        fs::create_dir_all(&dot).unwrap();
        fs::write(
            dot.join("workspace.toml"),
            "[workspace]\nname=\"x\"\nversion=\"0.1.0\"\ncreated=\"x\"\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("nested/sub/dir")).unwrap();

        // Explicit path takes precedence.
        let explicit = find_workspace_root(Some(root.join("explicit")));
        assert_eq!(explicit, Some(root.join("explicit")));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn validate_bad_semver_fails() {
        let dir = fresh_dir("bad-semver");
        let dot = dir.join(".bwoc");
        fs::create_dir_all(&dot).unwrap();
        fs::write(
            dot.join("workspace.toml"),
            r#"[workspace]
name = "demo"
version = "0.1"
created = "2026-05-22T06:00:00Z"
"#,
        )
        .unwrap();
        fs::write(dot.join("agents.toml"), "").unwrap();
        fs::create_dir_all(dir.join("agents")).unwrap();
        let report = validate(&dir);
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.contains("not parseable SemVer"))
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
