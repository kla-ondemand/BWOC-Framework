//! `bwoc skill list / show / verify` — read-side framework-skill surface.
//!
//! Implements the read-only subcommands from `docs/en/SKILLS.en.md` §"CLI
//! Surface" (BWOC-4). Skills live under `<workspace>/modules/skills/<name>/`,
//! each with a `manifest.toml` (schema §"Manifest"). Discovery (§"Discovery")
//! is workspace-local — no network calls — and per-agent opt-in is gated on
//! the agent's `config.manifest.json` `skills.framework[]` array.
//!
//! Lifecycle writers (`init`, `install`, `enable`, `disable`, `remove`) land
//! in later stories; this module is read-side only.
//!
//! Every read-only command has a `--json` twin. Human output is intentionally
//! terse — JSON is the contract for scripts.

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Top-level arguments shared by every `bwoc skill …` subcommand.
#[derive(Debug, Clone)]
pub struct CommonArgs {
    /// Workspace root override. Resolution: `--workspace` > `BWOC_WORKSPACE`
    /// env > ancestor-walk for `.bwoc/workspace.toml` > error.
    pub workspace: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ListArgs {
    pub common: CommonArgs,
    /// Filter to skills `enabled = true` for the current agent.
    pub enabled: bool,
    /// Override current-agent resolution. Used with `--enabled`.
    pub agent: Option<String>,
    pub json: bool,
}

#[derive(Debug, Clone)]
pub struct ShowArgs {
    pub common: CommonArgs,
    pub name: String,
    pub json: bool,
}

#[derive(Debug, Clone)]
pub struct VerifyArgs {
    pub common: CommonArgs,
    pub name: Option<String>,
    pub all: bool,
    pub json: bool,
}

// ---------------------------------------------------------------------------
// Manifest schema (mirror of SKILLS.en.md §"Manifest", lines 45–74).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
struct ManifestRaw {
    skill: SkillSection,
    #[serde(default)]
    contract: ContractSection,
    #[serde(default)]
    gates: GatesSection,
}

#[derive(Debug, Clone, Deserialize)]
struct SkillSection {
    name: String,
    version: String,
    description: String,
    maturity: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ContractSection {
    #[serde(default)]
    requires: Vec<String>,
    #[serde(default)]
    exposes: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct GatesSection {
    #[serde(default)]
    verify: Option<String>,
}

/// One discovered skill — manifest contents + filesystem location.
#[derive(Debug, Clone)]
struct DiscoveredSkill {
    dir_name: String,
    path: PathBuf,
    manifest: ManifestRaw,
    spec_path: Option<PathBuf>,
}

// ---------------------------------------------------------------------------
// Workspace resolution (mirror of the pattern in workspace.rs:634).
// ---------------------------------------------------------------------------

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

fn resolve_workspace(common: &CommonArgs) -> Result<PathBuf, String> {
    find_workspace_root(common.workspace.clone()).ok_or_else(|| {
        "no workspace found (no .bwoc/workspace.toml in cwd or ancestors). \
         Pass --workspace, set BWOC_WORKSPACE, or run `bwoc init` first."
            .to_string()
    })
}

// ---------------------------------------------------------------------------
// Discovery.
// ---------------------------------------------------------------------------

fn skills_dir(root: &Path) -> PathBuf {
    root.join("modules/skills")
}

/// Walk `<root>/modules/skills/*/manifest.toml`, parse each, and return them
/// sorted by directory name. Missing or invalid manifests are surfaced as
/// errors keyed by directory — callers decide whether to fail or skip.
fn discover(root: &Path) -> Result<Vec<DiscoveredSkill>, String> {
    let dir = skills_dir(root);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .map_err(|e| format!("read {}: {e}", dir.display()))?
        .filter_map(|r| r.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut out = Vec::with_capacity(entries.len());
    for entry in entries {
        let skill_dir = entry.path();
        let dir_name = entry.file_name().to_string_lossy().into_owned();
        let manifest_path = skill_dir.join("manifest.toml");
        if !manifest_path.is_file() {
            // Skip dirs that lack a manifest — they may be the README.md
            // top-level file or future scaffolding. `bwoc check` is the
            // authoritative validator (SKILLS.en.md §"Verification").
            continue;
        }
        let manifest = parse_manifest(&manifest_path)
            .map_err(|e| format!("{}/manifest.toml: {e}", dir_name))?;
        if manifest.skill.name != dir_name {
            return Err(format!(
                "modules/skills/{dir_name}/manifest.toml: [skill].name = {:?} \
                 does not match directory name {dir_name:?}",
                manifest.skill.name
            ));
        }
        let spec = skill_dir.join("SPEC.md");
        let spec_path = if spec.is_file() { Some(spec) } else { None };
        out.push(DiscoveredSkill {
            dir_name,
            path: skill_dir,
            manifest,
            spec_path,
        });
    }
    Ok(out)
}

fn parse_manifest(path: &Path) -> Result<ManifestRaw, String> {
    let body =
        std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    toml::from_str::<ManifestRaw>(&body).map_err(|e| format!("parse: {e}"))
}

// ---------------------------------------------------------------------------
// Per-agent enabled-skills resolution (SKILLS.en.md §"Discovery").
// ---------------------------------------------------------------------------

/// Read `<agent>/config.manifest.json` and return the names of skills the
/// agent has enabled. An agent without a `skills.framework[]` block has an
/// empty enabled set — that is the same as having opted into none.
fn enabled_skill_names(agent_dir: &Path) -> Result<Vec<String>, String> {
    let manifest_path = agent_dir.join("config.manifest.json");
    let body = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("read {}: {e}", manifest_path.display()))?;
    let value: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("{}: parse: {e}", manifest_path.display()))?;
    let Some(entries) = value
        .get("skills")
        .and_then(|s| s.get("framework"))
        .and_then(|f| f.as_array())
    else {
        return Ok(Vec::new());
    };

    let mut out = Vec::new();
    for (idx, entry) in entries.iter().enumerate() {
        let name = entry.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            format!(
                "{}: skills.framework[{idx}] is missing 'name'",
                manifest_path.display()
            )
        })?;
        // Per SKILLS.en.md §"Discovery" lines 154–156, `enabled` is required
        // and has no implicit default. Mirror that strictness here so the
        // CLI surfaces the manifest error the same way `bwoc check` will.
        let enabled = entry
            .get("enabled")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| {
                format!(
                    "{}: skills.framework[{idx}] ({name}) is missing required 'enabled' field",
                    manifest_path.display()
                )
            })?;
        if enabled {
            out.push(name.to_string());
        }
    }
    Ok(out)
}

/// Resolve which agent's `config.manifest.json` to consult, per SKILLS.en.md
/// §"Current agent resolution" (lines 205–213). v1 supports `--agent`,
/// `BWOC_AGENT` env, and cwd-descent; falls through to a clear error.
fn resolve_current_agent(
    root: &Path,
    explicit: Option<&str>,
) -> Result<(String, PathBuf), String> {
    if let Some(id) = explicit {
        let dir = root.join("agents").join(id);
        if !dir.is_dir() {
            return Err(format!(
                "--agent {id}: no such agent in workspace ({})",
                dir.display()
            ));
        }
        return Ok((id.to_string(), dir));
    }
    if let Ok(env_id) = std::env::var("BWOC_AGENT") {
        if !env_id.is_empty() {
            let dir = root.join("agents").join(&env_id);
            if !dir.is_dir() {
                return Err(format!(
                    "BWOC_AGENT={env_id}: no such agent in workspace ({})",
                    dir.display()
                ));
            }
            return Ok((env_id, dir));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        if let Ok(rel) = cwd.strip_prefix(root.join("agents")) {
            if let Some(first) = rel.components().next() {
                let id = first.as_os_str().to_string_lossy().into_owned();
                let dir = root.join("agents").join(&id);
                if dir.is_dir() {
                    return Ok((id, dir));
                }
            }
        }
    }
    Err(
        "no agent context; pass --agent <name> or run from within an agent directory".to_string(),
    )
}

// ---------------------------------------------------------------------------
// JSON helpers.
// ---------------------------------------------------------------------------

fn skill_summary_json(s: &DiscoveredSkill, enabled: Option<bool>) -> serde_json::Value {
    let mut v = serde_json::json!({
        "name": s.manifest.skill.name,
        "version": s.manifest.skill.version,
        "description": s.manifest.skill.description,
        "maturity": s.manifest.skill.maturity,
        "requires": s.manifest.contract.requires,
        "exposes": s.manifest.contract.exposes,
        "verify": s.manifest.gates.verify,
        "path": s.path.display().to_string(),
        "spec_path": s.spec_path.as_ref().map(|p| p.display().to_string()),
    });
    if let Some(b) = enabled {
        v["enabled"] = serde_json::Value::Bool(b);
    }
    v
}

fn print_json(value: &serde_json::Value) -> i32 {
    match serde_json::to_string_pretty(value) {
        Ok(s) => {
            println!("{s}");
            0
        }
        Err(e) => {
            eprintln!("bwoc skill: serialize JSON failed: {e}");
            1
        }
    }
}

// ---------------------------------------------------------------------------
// `bwoc skill list` ----------------------------------------------------------
// ---------------------------------------------------------------------------

pub fn run_list(args: ListArgs) -> i32 {
    let root = match resolve_workspace(&args.common) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("bwoc skill list: {e}");
            return 2;
        }
    };
    let skills = match discover(&root) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("bwoc skill list: {e}");
            return 1;
        }
    };

    // If `--enabled` (or `--agent`) is in play we annotate each skill with
    // its enabled-on-this-agent status. Without it, `enabled` is absent —
    // a skill installed at workspace scope has no per-agent opinion yet.
    let agent_filter: Option<Vec<String>> = if args.enabled || args.agent.is_some() {
        let (_id, agent_dir) = match resolve_current_agent(&root, args.agent.as_deref()) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("bwoc skill list: {e}");
                return 2;
            }
        };
        match enabled_skill_names(&agent_dir) {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("bwoc skill list: {e}");
                return 1;
            }
        }
    } else {
        None
    };

    let annotated: Vec<(DiscoveredSkill, Option<bool>)> = skills
        .into_iter()
        .map(|s| {
            let enabled = agent_filter
                .as_ref()
                .map(|names| names.iter().any(|n| n == &s.manifest.skill.name));
            (s, enabled)
        })
        .collect();
    let filtered: Vec<_> = annotated
        .into_iter()
        .filter(|(_, en)| !args.enabled || matches!(en, Some(true)))
        .collect();

    if args.json {
        let value = serde_json::json!({
            "workspace": root.display().to_string(),
            "skills": filtered
                .iter()
                .map(|(s, en)| skill_summary_json(s, *en))
                .collect::<Vec<_>>(),
        });
        return print_json(&value);
    }

    if filtered.is_empty() {
        println!(
            "(no framework skills installed at {})",
            skills_dir(&root).display()
        );
        return 0;
    }

    // Compact human table — name · version · maturity · description.
    // `enabled` column appears only when an agent filter is active.
    let with_enabled = filtered.iter().any(|(_, en)| en.is_some());
    let name_w = filtered
        .iter()
        .map(|(s, _)| s.manifest.skill.name.len())
        .max()
        .unwrap_or(4)
        .max(4);
    let ver_w = filtered
        .iter()
        .map(|(s, _)| s.manifest.skill.version.len())
        .max()
        .unwrap_or(7)
        .max(7);

    if with_enabled {
        println!(
            "{:<name_w$}  {:<ver_w$}  {:<8}  {:<8}  DESCRIPTION",
            "NAME", "VERSION", "MATURITY", "ENABLED",
        );
    } else {
        println!(
            "{:<name_w$}  {:<ver_w$}  {:<8}  DESCRIPTION",
            "NAME", "VERSION", "MATURITY",
        );
    }
    for (s, en) in &filtered {
        if with_enabled {
            let en_str = match en {
                Some(true) => "yes",
                Some(false) => "no",
                None => "-",
            };
            println!(
                "{:<name_w$}  {:<ver_w$}  {:<8}  {:<8}  {}",
                s.manifest.skill.name,
                s.manifest.skill.version,
                s.manifest.skill.maturity,
                en_str,
                s.manifest.skill.description,
            );
        } else {
            println!(
                "{:<name_w$}  {:<ver_w$}  {:<8}  {}",
                s.manifest.skill.name,
                s.manifest.skill.version,
                s.manifest.skill.maturity,
                s.manifest.skill.description,
            );
        }
    }
    0
}

// ---------------------------------------------------------------------------
// `bwoc skill show <name>` ---------------------------------------------------
// ---------------------------------------------------------------------------

pub fn run_show(args: ShowArgs) -> i32 {
    let root = match resolve_workspace(&args.common) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("bwoc skill show: {e}");
            return 2;
        }
    };
    let skills = match discover(&root) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("bwoc skill show: {e}");
            return 1;
        }
    };
    let Some(s) = skills.into_iter().find(|s| s.dir_name == args.name) else {
        eprintln!(
            "bwoc skill show: '{}' not installed in {}",
            args.name,
            skills_dir(&root).display()
        );
        return 2;
    };

    if args.json {
        let value = serde_json::json!({
            "workspace": root.display().to_string(),
            "skill": skill_summary_json(&s, None),
        });
        return print_json(&value);
    }

    println!("Skill         {}", s.manifest.skill.name);
    println!("Version       {}", s.manifest.skill.version);
    println!("Maturity      {}", s.manifest.skill.maturity);
    println!("Description   {}", s.manifest.skill.description);
    println!("Path          {}", s.path.display());
    if let Some(sp) = &s.spec_path {
        println!("Spec          {}", sp.display());
    } else {
        println!("Spec          (missing — SPEC.md not present)");
    }
    println!(
        "Requires      {}",
        if s.manifest.contract.requires.is_empty() {
            "(none)".to_string()
        } else {
            s.manifest.contract.requires.join(", ")
        }
    );
    println!("Exposes       {}", s.manifest.contract.exposes.join(", "));
    println!(
        "Verify gate   {}",
        s.manifest.gates.verify.as_deref().unwrap_or("(none)"),
    );
    0
}

// ---------------------------------------------------------------------------
// `bwoc skill verify <name>` / `--all` ---------------------------------------
// ---------------------------------------------------------------------------

/// Sentinel env var: set on every spawned `[gates].verify` shell so a
/// manifest whose verify command recursively re-enters `bwoc skill verify`
/// fails fast instead of forking unbounded.
const VERIFY_INFLIGHT_ENV: &str = "BWOC_SKILL_VERIFY_INFLIGHT";

pub fn run_verify(args: VerifyArgs) -> i32 {
    let root = match resolve_workspace(&args.common) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("bwoc skill verify: {e}");
            return 2;
        }
    };

    // Recursion guard — see VERIFY_INFLIGHT_ENV. A manifest whose verify
    // command shells out to `bwoc skill verify` would otherwise recurse
    // forever; surface the manifest bug to the operator instead.
    if std::env::var_os(VERIFY_INFLIGHT_ENV).is_some() {
        let names = args
            .name
            .clone()
            .map(|n| format!(" for '{n}'"))
            .unwrap_or_default();
        eprintln!(
            "bwoc skill verify{names}: refusing to recurse — \
             a parent `bwoc skill verify` is already running. \
             Check the skill's [gates].verify command; it must not re-invoke `bwoc skill verify`."
        );
        return 3;
    }

    let skills = match discover(&root) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("bwoc skill verify: {e}");
            return 1;
        }
    };

    let targets: Vec<DiscoveredSkill> = if args.all {
        skills
    } else {
        let Some(name) = args.name.as_deref() else {
            // clap's ArgGroup guarantees one of (name, --all); defensive only.
            eprintln!("bwoc skill verify: pass <name> or --all");
            return 2;
        };
        let Some(s) = skills.into_iter().find(|s| s.dir_name == name) else {
            eprintln!(
                "bwoc skill verify: '{name}' not installed in {}",
                skills_dir(&root).display()
            );
            return 2;
        };
        vec![s]
    };

    let mut results: Vec<serde_json::Value> = Vec::with_capacity(targets.len());
    let mut overall_ok = true;

    for s in &targets {
        let started = std::time::Instant::now();
        let (exit_code, ok, command_present) = match &s.manifest.gates.verify {
            None => {
                // Per SKILLS.en.md line 74, [gates].verify is optional.
                // No gate → no claim of passing, no claim of failing. Report
                // as "skipped" via a null exit_code in JSON, and "ok = true"
                // so --all does not fail solely because gates were declared
                // absent.
                (None, true, false)
            }
            Some(cmd) => {
                let status = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .current_dir(&root)
                    .env(VERIFY_INFLIGHT_ENV, "1")
                    .status();
                match status {
                    Ok(st) => {
                        let code = st.code().unwrap_or(-1);
                        (Some(code), code == 0, true)
                    }
                    Err(e) => {
                        eprintln!(
                            "bwoc skill verify: '{}': spawn failed: {e}",
                            s.manifest.skill.name
                        );
                        (Some(-1), false, true)
                    }
                }
            }
        };
        if !ok {
            overall_ok = false;
        }
        let elapsed_ms = started.elapsed().as_millis() as u64;
        if args.json {
            results.push(serde_json::json!({
                "skill": s.manifest.skill.name,
                "verify_command": s.manifest.gates.verify,
                "exit_code": exit_code,
                "ok": ok,
                "skipped": !command_present,
                "duration_ms": elapsed_ms,
            }));
        } else if !command_present {
            println!("- {}  (skipped — no [gates].verify)", s.manifest.skill.name);
        } else {
            let tag = if ok { "OK" } else { "FAIL" };
            println!(
                "{tag:<5} {}  ({elapsed_ms} ms)",
                s.manifest.skill.name
            );
        }
    }

    if args.json {
        let value = serde_json::json!({
            "workspace": root.display().to_string(),
            "ok": overall_ok,
            "results": results,
        });
        return print_json(&value).max(if overall_ok { 0 } else { 1 });
    }

    if overall_ok { 0 } else { 1 }
}
