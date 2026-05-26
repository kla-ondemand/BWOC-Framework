//! `bwoc plugin list / show` — read-side framework-plugin surface.
//!
//! Implements the read-only subcommands from `docs/en/PLUGINS.en.md` §"CLI
//! Surface" (BWOC-5). Plugins live under `<workspace>/modules/plugins/<name>/`,
//! each with a `manifest.toml` (schema §"Manifest"). Discovery is
//! workspace-local — no network calls — and per-workspace opt-in is gated on
//! `workspace.toml` `[plugins.<name>]` tables (PLUGINS.en.md §"Loading").
//!
//! Lifecycle writers (`init`, `install`, `enable`, `disable`, `remove`) land
//! in later stories; this module is read-side only. There is no `verify` —
//! PLUGINS.en.md §"CLI Surface" line 237 calls it out as a v1 omission.
//!
//! Every read-only command has a `--json` twin. Human output is intentionally
//! terse — JSON is the contract for scripts.

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Top-level arguments shared by every `bwoc plugin …` subcommand.
#[derive(Debug, Clone)]
pub struct CommonArgs {
    /// Workspace root override. Resolution: `--workspace` > `BWOC_WORKSPACE`
    /// env > ancestor-walk for `.bwoc/workspace.toml` > error.
    pub workspace: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ListArgs {
    pub common: CommonArgs,
    /// Filter to plugins enabled in `workspace.toml [plugins.<name>]`.
    pub enabled: bool,
    /// Filter to one plugin kind (`memory-backend`, `llm-backend`, `workflow`).
    pub kind: Option<String>,
    pub json: bool,
}

#[derive(Debug, Clone)]
pub struct ShowArgs {
    pub common: CommonArgs,
    pub name: String,
    pub json: bool,
}

// ---------------------------------------------------------------------------
// Manifest schema (mirror of PLUGINS.en.md §"Manifest", lines 70–97).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
struct ManifestRaw {
    plugin: PluginSection,
}

#[derive(Debug, Clone, Deserialize)]
struct PluginSection {
    name: String,
    kind: String,
    version: String,
    description: String,
    compat: String,
    entry: String,
}

/// One discovered plugin — manifest contents + filesystem location.
#[derive(Debug, Clone)]
struct DiscoveredPlugin {
    dir_name: String,
    path: PathBuf,
    manifest: ManifestRaw,
    spec_path: Option<PathBuf>,
}

// ---------------------------------------------------------------------------
// Workspace resolution (mirror of skill.rs:100).
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

fn plugins_dir(root: &Path) -> PathBuf {
    root.join("modules/plugins")
}

/// Walk `<root>/modules/plugins/*/manifest.toml`, parse each, and return
/// them sorted by directory name. Dirs without a `manifest.toml` are skipped
/// — `bwoc check` is the authoritative validator (PLUGINS.en.md §"Verification").
fn discover(root: &Path) -> Result<Vec<DiscoveredPlugin>, String> {
    let dir = plugins_dir(root);
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
        let plugin_dir = entry.path();
        let dir_name = entry.file_name().to_string_lossy().into_owned();
        let manifest_path = plugin_dir.join("manifest.toml");
        if !manifest_path.is_file() {
            continue;
        }
        let manifest = parse_manifest(&manifest_path)
            .map_err(|e| format!("{}/manifest.toml: {e}", dir_name))?;
        if manifest.plugin.name != dir_name {
            return Err(format!(
                "modules/plugins/{dir_name}/manifest.toml: [plugin].name = {:?} \
                 does not match directory name {dir_name:?}",
                manifest.plugin.name
            ));
        }
        let spec = plugin_dir.join("SPEC.md");
        let spec_path = if spec.is_file() { Some(spec) } else { None };
        out.push(DiscoveredPlugin {
            dir_name,
            path: plugin_dir,
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
// `workspace.toml [plugins.<name>]` resolution (PLUGINS.en.md §"Loading").
// ---------------------------------------------------------------------------

/// Parsed entry from `workspace.toml [plugins.<name>]`. `extra` carries the
/// plugin-defined config keys (everything other than `enabled`).
#[derive(Debug, Clone)]
struct WorkspaceEntry {
    enabled: bool,
    extra: serde_json::Map<String, serde_json::Value>,
}

/// Read `<root>/.bwoc/workspace.toml` and return the `[plugins]` block as a
/// map of `name -> WorkspaceEntry`. An absent `[plugins]` table is not an
/// error — it just means no plugins have been registered yet.
///
/// Per PLUGINS.en.md line 193, a missing `enabled` field is a manifest error;
/// we surface it the same way `bwoc check` will.
fn workspace_plugins(
    root: &Path,
) -> Result<std::collections::BTreeMap<String, WorkspaceEntry>, String> {
    let path = root.join(".bwoc/workspace.toml");
    let body =
        std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let value: toml::Value =
        toml::from_str(&body).map_err(|e| format!("{}: parse: {e}", path.display()))?;
    let mut out = std::collections::BTreeMap::new();
    let Some(plugins) = value.get("plugins").and_then(|v| v.as_table()) else {
        return Ok(out);
    };
    for (name, entry) in plugins {
        let table = entry
            .as_table()
            .ok_or_else(|| format!("{}: [plugins.{name}] is not a table", path.display()))?;
        let enabled = table
            .get("enabled")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| {
                format!(
                    "{}: [plugins.{name}] is missing required 'enabled' field",
                    path.display()
                )
            })?;
        let mut extra = serde_json::Map::new();
        for (k, v) in table {
            if k == "enabled" {
                continue;
            }
            extra.insert(k.clone(), toml_to_json(v));
        }
        out.insert(name.clone(), WorkspaceEntry { enabled, extra });
    }
    Ok(out)
}

fn toml_to_json(v: &toml::Value) -> serde_json::Value {
    match v {
        toml::Value::String(s) => serde_json::Value::String(s.clone()),
        toml::Value::Integer(i) => serde_json::Value::Number((*i).into()),
        toml::Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        toml::Value::Boolean(b) => serde_json::Value::Bool(*b),
        toml::Value::Datetime(d) => serde_json::Value::String(d.to_string()),
        toml::Value::Array(a) => serde_json::Value::Array(a.iter().map(toml_to_json).collect()),
        toml::Value::Table(t) => {
            let mut m = serde_json::Map::new();
            for (k, v) in t {
                m.insert(k.clone(), toml_to_json(v));
            }
            serde_json::Value::Object(m)
        }
    }
}

// ---------------------------------------------------------------------------
// JSON helpers.
// ---------------------------------------------------------------------------

fn plugin_summary_json(p: &DiscoveredPlugin, entry: Option<&WorkspaceEntry>) -> serde_json::Value {
    let mut v = serde_json::json!({
        "name": p.manifest.plugin.name,
        "kind": p.manifest.plugin.kind,
        "version": p.manifest.plugin.version,
        "description": p.manifest.plugin.description,
        "compat": p.manifest.plugin.compat,
        "entry": p.manifest.plugin.entry,
        "path": p.path.display().to_string(),
        "spec_path": p.spec_path.as_ref().map(|p| p.display().to_string()),
        "registered": entry.is_some(),
        "enabled": entry.map(|e| e.enabled),
    });
    if let Some(e) = entry {
        if !e.extra.is_empty() {
            v["config"] = serde_json::Value::Object(e.extra.clone());
        }
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
            eprintln!("bwoc plugin: serialize JSON failed: {e}");
            1
        }
    }
}

// ---------------------------------------------------------------------------
// `bwoc plugin list` ---------------------------------------------------------
// ---------------------------------------------------------------------------

pub fn run_list(args: ListArgs) -> i32 {
    let root = match resolve_workspace(&args.common) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("bwoc plugin list: {e}");
            return 2;
        }
    };
    let plugins = match discover(&root) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("bwoc plugin list: {e}");
            return 1;
        }
    };
    let ws_entries = match workspace_plugins(&root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("bwoc plugin list: {e}");
            return 1;
        }
    };

    let annotated: Vec<(DiscoveredPlugin, Option<WorkspaceEntry>)> = plugins
        .into_iter()
        .map(|p| {
            let entry = ws_entries.get(&p.manifest.plugin.name).cloned();
            (p, entry)
        })
        .collect();

    let filtered: Vec<_> = annotated
        .into_iter()
        .filter(|(p, en)| {
            if args.enabled && !matches!(en, Some(e) if e.enabled) {
                return false;
            }
            if let Some(k) = args.kind.as_deref() {
                if p.manifest.plugin.kind != k {
                    return false;
                }
            }
            true
        })
        .collect();

    if args.json {
        let value = serde_json::json!({
            "workspace": root.display().to_string(),
            "plugins": filtered
                .iter()
                .map(|(p, en)| plugin_summary_json(p, en.as_ref()))
                .collect::<Vec<_>>(),
        });
        return print_json(&value);
    }

    if filtered.is_empty() {
        println!(
            "(no framework plugins installed at {})",
            plugins_dir(&root).display()
        );
        return 0;
    }

    // Compact human table — name · kind · version · enabled · description.
    let name_w = filtered
        .iter()
        .map(|(p, _)| p.manifest.plugin.name.len())
        .max()
        .unwrap_or(4)
        .max(4);
    let kind_w = filtered
        .iter()
        .map(|(p, _)| p.manifest.plugin.kind.len())
        .max()
        .unwrap_or(4)
        .max(4);
    let ver_w = filtered
        .iter()
        .map(|(p, _)| p.manifest.plugin.version.len())
        .max()
        .unwrap_or(7)
        .max(7);

    println!(
        "{:<name_w$}  {:<kind_w$}  {:<ver_w$}  {:<8}  DESCRIPTION",
        "NAME", "KIND", "VERSION", "ENABLED",
    );
    for (p, en) in &filtered {
        let en_str = match en {
            Some(e) if e.enabled => "yes",
            Some(_) => "no",
            None => "-",
        };
        println!(
            "{:<name_w$}  {:<kind_w$}  {:<ver_w$}  {:<8}  {}",
            p.manifest.plugin.name,
            p.manifest.plugin.kind,
            p.manifest.plugin.version,
            en_str,
            p.manifest.plugin.description,
        );
    }
    0
}

// ---------------------------------------------------------------------------
// `bwoc plugin show <name>` --------------------------------------------------
// ---------------------------------------------------------------------------

pub fn run_show(args: ShowArgs) -> i32 {
    let root = match resolve_workspace(&args.common) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("bwoc plugin show: {e}");
            return 2;
        }
    };
    let plugins = match discover(&root) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("bwoc plugin show: {e}");
            return 1;
        }
    };
    let Some(p) = plugins.into_iter().find(|p| p.dir_name == args.name) else {
        eprintln!(
            "bwoc plugin show: '{}' not installed in {}",
            args.name,
            plugins_dir(&root).display()
        );
        return 2;
    };
    let ws_entries = match workspace_plugins(&root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("bwoc plugin show: {e}");
            return 1;
        }
    };
    let entry = ws_entries.get(&p.manifest.plugin.name);

    if args.json {
        let value = serde_json::json!({
            "workspace": root.display().to_string(),
            "plugin": plugin_summary_json(&p, entry),
        });
        return print_json(&value);
    }

    println!("Plugin        {}", p.manifest.plugin.name);
    println!("Kind          {}", p.manifest.plugin.kind);
    println!("Version       {}", p.manifest.plugin.version);
    println!("Compat        {}", p.manifest.plugin.compat);
    println!("Entry         {}", p.manifest.plugin.entry);
    println!("Description   {}", p.manifest.plugin.description);
    println!("Path          {}", p.path.display());
    if let Some(sp) = &p.spec_path {
        println!("Spec          {}", sp.display());
    } else {
        println!("Spec          (missing — SPEC.md not present)");
    }
    match entry {
        Some(e) => {
            println!(
                "Registered    yes  (workspace.toml [plugins.{}])",
                p.manifest.plugin.name
            );
            println!("Enabled       {}", if e.enabled { "yes" } else { "no" });
            if e.extra.is_empty() {
                println!("Config        (none)");
            } else {
                println!("Config");
                for (k, v) in &e.extra {
                    println!("  {k} = {}", v);
                }
            }
        }
        None => {
            println!("Registered    no   (not present in workspace.toml [plugins])");
            println!("Enabled       -");
            println!("Config        (none)");
        }
    }
    0
}
