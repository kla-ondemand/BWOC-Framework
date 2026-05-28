//! `bwoc gcloud <verb>` — operator-facing CLI surface for the `workflow/gcloud-*`
//! plugins (BWOC-52). Foundation of `BWOC-EPIC-8` (Google Cloud).
//!
//! ## What this is
//!
//! The CLI half of the contract framed in
//! `notes/2026-05-28_gcloud-workflow-plugin-architecture.md` (BWOC-51). It owns
//! **argument parsing, workspace + auth-shape resolution, the write-confirmation
//! gate for `project set-default`, and the JSON shapes** — it does NOT speak to
//! Google directly. The live `gcloud` calls (`gcloud auth status`,
//! `gcloud projects list`, …) belong to the `workflow/gcloud-auth` and
//! `workflow/gcloud-project` reference plugins (BWOC-53, in flight). This CLI
//! discovers each enabled `workflow/gcloud-*` plugin and invokes its
//! `[plugin].entry`; when a plugin is absent the live verbs **stub-error
//! gracefully** (exit `4`) rather than panicking.
//!
//! ## Verb table (foundation slice — read-mostly)
//!
//! | Verb                          | Needs plugin       | Notes                                                              |
//! |---|---|---|
//! | `auth status`                 | `gcloud-auth`      | JSON: `{ active_source, account_email, has_credential }`. Never the token. |
//! | `auth login`                  | `gcloud-auth`      | Operator-driven (shells to `gcloud auth login`). Gated behind confirmation.|
//! | `project list`                | `gcloud-project`   | Read-only listing of accessible projects.                          |
//! | `project show [--project <id>]` | `gcloud-project` | Read-only descriptor for one project (default = active).          |
//! | `project set-default --project <id>` | `gcloud-project` | **Write** to local `gcloud` config. Gated.                  |
//! | `status`                      | both, but degrades | Combined auth + project view, never fails when plugins are absent. |
//!
//! ## Verb table (EPIC-9 compute slice — first write-capable verbs)
//!
//! | Verb                                | Needs plugin    | Gate | Notes                                          |
//! |---|---|---|---|
//! | `compute list [--project] [--zone]` | `gcloud-compute`| none | Read-only instance listing.                    |
//! | `compute start --instance --zone`   | `gcloud-compute`| **operator-confirm** | Boots a VM (cost). `--yes` for headless. |
//! | `compute stop --instance --zone`    | `gcloud-compute`| **operator-confirm** | Halts a VM (interrupts workloads). `--yes`. |
//!
//! The write gate lives **here in the CLI**, not the plugin (BWOC-66 §3 /
//! BWOC-67): it shows the exact effect + the literal `gcloud` command, defaults
//! to No, accepts `--yes` for non-interactive use, and reports a refused write
//! as "no change" with a reason — never a bare failure. `delete` is **out of
//! scope** for EPIC-9 (irreversible; deferred to a stronger gate). The write
//! request carries `confirmed: true` so the plugin can refuse a gate-bypassing
//! direct invoke.
//!
//! ## Auth model — operator credentials, never echoed
//!
//! Credential resolution shape (precedence order, decision 3 in the design note):
//!
//! 1. ADC at `~/.config/gcloud/application_default_credentials.json` (the local
//!    `gcloud` CLI's own state — we never read the file ourselves);
//! 2. Service-account JSON at `<workspace>/.bwoc/secrets/gcloud-sa.json`
//!    (gitignored, never committed — surfaced only by **presence**, never by value);
//! 3. Environment variables — `BWOC_GCLOUD_ACCOUNT`, `BWOC_GCLOUD_PROJECT`,
//!    `BWOC_GCLOUD_IMPERSONATE_SERVICE_ACCOUNT`.
//!
//! The CLI carries only the **shape** (which source resolved, account email
//! reported by `gcloud`, presence/absence of files and env vars). Token values
//! never enter this module's state — the `gcloud` CLI inside the plugin owns
//! refresh and the credential cache. Mirrors the Adinnādāna invariant the
//! `jira` lane established in BWOC-42.
//!
//! ## Exit codes — normative
//!
//! - `0` — success.
//! - `1` — local I/O error.
//! - `2` — operator/usage error (no workspace, malformed project id, gated write
//!   requested with `--json` but without `--yes`).
//! - `4` — required `workflow/gcloud-*` plugin not enabled in this workspace
//!   (the live path is unavailable; remediation message names the missing one).
//! - `255` — plugin runtime error (spawn failure or non-JSON output).
//!
//! Passing `--json` makes the exit code redundant: the structured envelope
//! carries `ok`/`error` fields with the same signal.

use clap::{Args, Subcommand};
use serde::Serialize;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

// ---------------------------------------------------------------------------
// Exit codes + plugin names + env vars (single source of truth).
// ---------------------------------------------------------------------------

const EXIT_OK: i32 = 0;
const EXIT_LOCAL_ERROR: i32 = 1;
const EXIT_USAGE: i32 = 2;
const EXIT_NO_PLUGIN: i32 = 4;
const EXIT_PLUGIN_ERROR: i32 = 255;

const PLUGIN_AUTH: &str = "gcloud-auth";
const PLUGIN_PROJECT: &str = "gcloud-project";
const PLUGIN_COMPUTE: &str = "gcloud-compute";
const PLUGIN_KIND: &str = "workflow";

const ENV_ACCOUNT: &str = "BWOC_GCLOUD_ACCOUNT";
const ENV_PROJECT: &str = "BWOC_GCLOUD_PROJECT";
const ENV_IMPERSONATE: &str = "BWOC_GCLOUD_IMPERSONATE_SERVICE_ACCOUNT";

const ADC_REL: &str = ".config/gcloud/application_default_credentials.json";
const SA_REL: &str = ".bwoc/secrets/gcloud-sa.json";

// ---------------------------------------------------------------------------
// CLI surface — defined here so arg parsing is unit-testable against
// `GcloudCommand` directly (see `tests` module).
// ---------------------------------------------------------------------------

#[derive(Subcommand, Debug)]
pub enum GcloudCommand {
    /// Credential state operations (gcloud-auth plugin).
    #[command(subcommand)]
    Auth(AuthCommand),
    /// Project context operations (gcloud-project plugin).
    #[command(subcommand)]
    Project(ProjectCommand),
    /// Compute instance lifecycle operations (gcloud-compute plugin, EPIC-9).
    #[command(subcommand)]
    Compute(ComputeCommand),
    /// Combined auth + project view. Degrades cleanly when plugins are missing.
    Status(StatusArgs),
}

#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    /// Report the active credential source + account email (never the token).
    Status(AuthStatusArgs),
    /// Operator-driven `gcloud auth login`. Gated behind confirmation; never
    /// invoked from agents directly.
    Login(AuthLoginArgs),
}

#[derive(Subcommand, Debug)]
pub enum ProjectCommand {
    /// List projects the active credential can see.
    List(ProjectListArgs),
    /// Show one project descriptor (default = active project).
    Show(ProjectShowArgs),
    /// Set the local `gcloud` default project. WRITE — gated behind confirmation.
    SetDefault(ProjectSetDefaultArgs),
}

#[derive(Args, Debug)]
pub struct AuthStatusArgs {
    /// Workspace root. Resolution: --workspace > BWOC_WORKSPACE env > ancestor walk > cwd.
    #[arg(long = "workspace")]
    workspace: Option<PathBuf>,
    /// Emit the structured envelope instead of the human-readable summary.
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
pub struct AuthLoginArgs {
    /// Hint the email to log in with (passed through to `gcloud auth login`).
    #[arg(long)]
    account: Option<String>,
    /// Acknowledge the operator-driven write up front (required in --json mode).
    #[arg(long)]
    yes: bool,
    /// Workspace root.
    #[arg(long = "workspace")]
    workspace: Option<PathBuf>,
    /// Emit the structured envelope instead of the human-readable summary.
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
pub struct ProjectListArgs {
    /// Workspace root.
    #[arg(long = "workspace")]
    workspace: Option<PathBuf>,
    /// Emit the structured envelope instead of the human-readable table.
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
pub struct ProjectShowArgs {
    /// Project id (default: whatever `gcloud config get project` reports).
    #[arg(long = "project")]
    project: Option<String>,
    /// Workspace root.
    #[arg(long = "workspace")]
    workspace: Option<PathBuf>,
    /// Emit the structured envelope instead of the human-readable summary.
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
pub struct ProjectSetDefaultArgs {
    /// Project id to set as the local `gcloud` default. Required.
    #[arg(long = "project")]
    project: String,
    /// Acknowledge the write up front (required in --json mode).
    #[arg(long)]
    yes: bool,
    /// Workspace root.
    #[arg(long = "workspace")]
    workspace: Option<PathBuf>,
    /// Emit the structured envelope instead of the human-readable summary.
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand, Debug)]
pub enum ComputeCommand {
    /// List compute instances the active credential can see. Read — no gate.
    List(ComputeListArgs),
    /// Start a stopped instance. **Write** — operator-confirm gated.
    Start(ComputeStartArgs),
    /// Stop a running instance. **Write** — operator-confirm gated.
    Stop(ComputeStopArgs),
}

#[derive(Args, Debug)]
pub struct ComputeListArgs {
    /// Project id to list instances from (default: active gcloud project).
    #[arg(long = "project")]
    project: Option<String>,
    /// Restrict the listing to a single zone.
    #[arg(long = "zone")]
    zone: Option<String>,
    /// Workspace root.
    #[arg(long = "workspace")]
    workspace: Option<PathBuf>,
    /// Emit the structured envelope instead of the human-readable table.
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
pub struct ComputeStartArgs {
    /// Instance name to start. Required.
    #[arg(long = "instance")]
    instance: String,
    /// Zone the instance lives in. Required.
    #[arg(long = "zone")]
    zone: String,
    /// Project id (default: active gcloud project). Shown in the gate.
    #[arg(long = "project")]
    project: Option<String>,
    /// Acknowledge the write up front (required in --json mode). An agent sets
    /// this ONLY when the operator authorized this specific action.
    #[arg(long)]
    yes: bool,
    /// Workspace root.
    #[arg(long = "workspace")]
    workspace: Option<PathBuf>,
    /// Emit the structured envelope instead of the human-readable summary.
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
pub struct ComputeStopArgs {
    /// Instance name to stop. Required.
    #[arg(long = "instance")]
    instance: String,
    /// Zone the instance lives in. Required.
    #[arg(long = "zone")]
    zone: String,
    /// Project id (default: active gcloud project). Shown in the gate.
    #[arg(long = "project")]
    project: Option<String>,
    /// Acknowledge the write up front (required in --json mode). An agent sets
    /// this ONLY when the operator authorized this specific action.
    #[arg(long)]
    yes: bool,
    /// Workspace root.
    #[arg(long = "workspace")]
    workspace: Option<PathBuf>,
    /// Emit the structured envelope instead of the human-readable summary.
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Workspace root.
    #[arg(long = "workspace")]
    workspace: Option<PathBuf>,
    /// Emit the structured envelope instead of the human-readable summary.
    #[arg(long)]
    json: bool,
}

/// Dispatch a parsed `GcloudCommand`. Returns the process exit code.
pub fn run(cmd: GcloudCommand) -> i32 {
    match cmd {
        GcloudCommand::Auth(AuthCommand::Status(a)) => run_auth_status(a),
        GcloudCommand::Auth(AuthCommand::Login(a)) => run_auth_login(a),
        GcloudCommand::Project(ProjectCommand::List(a)) => run_project_list(a),
        GcloudCommand::Project(ProjectCommand::Show(a)) => run_project_show(a),
        GcloudCommand::Project(ProjectCommand::SetDefault(a)) => run_project_set_default(a),
        GcloudCommand::Compute(ComputeCommand::List(a)) => run_compute_list(a),
        GcloudCommand::Compute(ComputeCommand::Start(a)) => run_compute_start(a),
        GcloudCommand::Compute(ComputeCommand::Stop(a)) => run_compute_stop(a),
        GcloudCommand::Status(a) => run_combined_status(a),
    }
}

// ---------------------------------------------------------------------------
// Workspace resolution — same shape as jira.rs / audit.rs.
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

fn resolve_workspace(explicit: Option<PathBuf>) -> Result<PathBuf, String> {
    find_workspace_root(explicit).ok_or_else(|| {
        "no workspace found (no .bwoc/workspace.toml in cwd or ancestors). \
         Pass --workspace, set BWOC_WORKSPACE, or run `bwoc init` first."
            .to_string()
    })
}

// ---------------------------------------------------------------------------
// Auth shape — credentials NEVER captured. We surface presence + which source
// would win, derived from filesystem + env probes. The `gcloud` CLI inside the
// plugin remains the source of truth for the actual active credential and the
// account email — this shape is the offline pre-check.
// ---------------------------------------------------------------------------

/// Where a credential would resolve from. The plugin's `status` verb returns
/// the live answer; this is the offline pre-check used by `bwoc gcloud status`
/// (degraded mode) and as the input to the remediation message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum AuthSource {
    Adc,
    ServiceAccount,
    Env,
    None,
}

impl AuthSource {
    fn as_str(self) -> &'static str {
        match self {
            AuthSource::Adc => "adc",
            AuthSource::ServiceAccount => "service-account",
            AuthSource::Env => "env",
            AuthSource::None => "none",
        }
    }
}

/// Offline credential probe. Files + env vars only — never reads tokens.
#[derive(Debug, Clone, Serialize, PartialEq)]
struct AuthShape {
    /// First source that would resolve, per the precedence in the design note.
    active_source: AuthSource,
    adc_present: bool,
    /// Service-account JSON at `<workspace>/.bwoc/secrets/gcloud-sa.json`.
    /// Reports **presence only** — the file is never read or hashed here.
    service_account_present: bool,
    /// Whether `BWOC_GCLOUD_ACCOUNT` is set (non-empty).
    env_account: bool,
    /// Whether `BWOC_GCLOUD_PROJECT` is set (non-empty).
    env_project: bool,
    /// Whether `BWOC_GCLOUD_IMPERSONATE_SERVICE_ACCOUNT` is set (non-empty).
    env_impersonate: bool,
}

fn probe_auth_shape(
    workspace: &Path,
    home: Option<&Path>,
    getenv: &dyn Fn(&str) -> Option<String>,
) -> AuthShape {
    let nonempty = |k: &str| getenv(k).filter(|s| !s.is_empty());
    let adc_present = home.map(|h| h.join(ADC_REL).is_file()).unwrap_or(false);
    let sa_present = workspace.join(SA_REL).is_file();
    let env_account = nonempty(ENV_ACCOUNT).is_some();
    let env_project = nonempty(ENV_PROJECT).is_some();
    let env_impersonate = nonempty(ENV_IMPERSONATE).is_some();

    // Precedence: ADC > SA JSON > env. "Has any env signal" counts for the
    // env tier — the design note treats the three vars as one source.
    let active = if adc_present {
        AuthSource::Adc
    } else if sa_present {
        AuthSource::ServiceAccount
    } else if env_account || env_project || env_impersonate {
        AuthSource::Env
    } else {
        AuthSource::None
    };

    AuthShape {
        active_source: active,
        adc_present,
        service_account_present: sa_present,
        env_account,
        env_project,
        env_impersonate,
    }
}

fn real_getenv(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

fn home_dir() -> Option<PathBuf> {
    // Cheap, no extra dep — mirror what `dirs::home_dir` does on the two
    // platforms we actually run on (macOS + Linux). Windows users go through
    // USERPROFILE; keep the path read for completeness without pulling a crate.
    if let Ok(h) = std::env::var("HOME") {
        if !h.is_empty() {
            return Some(PathBuf::from(h));
        }
    }
    if let Ok(h) = std::env::var("USERPROFILE") {
        if !h.is_empty() {
            return Some(PathBuf::from(h));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Plugin discovery — finds the enabled `workflow/gcloud-{auth,project}` plugin
// by name + kind=workflow. Checks both the flat layout (`modules/plugins/<name>/`)
// and the kind-namespaced layout (`modules/plugins/workflow/<name>/`) so the
// CLI works regardless of which layout BWOC-53 ships with.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Deserialize)]
struct ManifestRaw {
    plugin: PluginSection,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct PluginSection {
    name: String,
    kind: String,
    entry: String,
}

#[derive(Debug, Clone, PartialEq)]
struct GcloudPlugin {
    name: String,
    dir: PathBuf,
    entry: String,
}

/// Read `.bwoc/workspace.toml [plugins.<name>] enabled` flags.
fn workspace_enabled_set(root: &Path) -> Result<BTreeMap<String, bool>, String> {
    let path = root.join(".bwoc/workspace.toml");
    let body =
        std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let value: toml::Value =
        toml::from_str(&body).map_err(|e| format!("{}: parse: {e}", path.display()))?;
    let mut out = BTreeMap::new();
    let Some(plugins) = value.get("plugins").and_then(|v| v.as_table()) else {
        return Ok(out);
    };
    for (name, entry) in plugins {
        let enabled = entry
            .as_table()
            .and_then(|t| t.get("enabled"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        out.insert(name.clone(), enabled);
    }
    Ok(out)
}

/// Try the two known plugin layouts in order — flat, then `workflow/`-namespaced.
fn candidate_plugin_dirs(root: &Path, name: &str) -> [PathBuf; 2] {
    [
        root.join("modules/plugins").join(name),
        root.join("modules/plugins/workflow").join(name),
    ]
}

/// Find a `workflow`-kind plugin by name across both layouts. Returns `None`
/// when no manifest matches; returns `Err` on parse failure (the plugin
/// *exists* but is malformed — surface, don't silently degrade).
fn discover_plugin(root: &Path, name: &str) -> Result<Option<GcloudPlugin>, String> {
    for plugin_dir in candidate_plugin_dirs(root, name) {
        let manifest = plugin_dir.join("manifest.toml");
        if !manifest.is_file() {
            continue;
        }
        let body = std::fs::read_to_string(&manifest)
            .map_err(|e| format!("read {}: {e}", manifest.display()))?;
        let parsed: ManifestRaw =
            toml::from_str(&body).map_err(|e| format!("parse {}: {e}", manifest.display()))?;
        if parsed.plugin.name != name {
            // Wrong manifest at this path — keep looking.
            continue;
        }
        if parsed.plugin.kind != PLUGIN_KIND {
            // Right name, wrong kind. Surface — this is a misconfiguration.
            return Err(format!(
                "{}: [plugin].kind = {:?}, expected {:?}",
                manifest.display(),
                parsed.plugin.kind,
                PLUGIN_KIND
            ));
        }
        return Ok(Some(GcloudPlugin {
            name: parsed.plugin.name,
            dir: plugin_dir,
            entry: parsed.plugin.entry,
        }));
    }
    Ok(None)
}

/// Discover + check the `enabled` flag in `workspace.toml`. A plugin installed
/// but disabled returns `None` — same stub-error path as "not installed".
fn find_enabled_plugin(root: &Path, name: &str) -> Result<Option<GcloudPlugin>, String> {
    let Some(plugin) = discover_plugin(root, name)? else {
        return Ok(None);
    };
    let enabled = workspace_enabled_set(root)?;
    if matches!(enabled.get(name), Some(true)) {
        Ok(Some(plugin))
    } else {
        Ok(None)
    }
}

fn resolve_entry_program(plugin_dir: &Path, entry: &str) -> OsString {
    let candidate = plugin_dir.join(entry);
    if candidate.is_file() {
        candidate.into_os_string()
    } else {
        OsString::from(entry)
    }
}

// ---------------------------------------------------------------------------
// Plugin invocation — same shape as jira.rs::invoke_jira_plugin.
// ---------------------------------------------------------------------------

fn invoke_plugin(
    plugin: &GcloudPlugin,
    workspace: &Path,
    request: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    // BWOC-36: guard against path-traversal RCE before spawning the entry.
    crate::util::validate_plugin_entry(&plugin.entry)?;
    let program = resolve_entry_program(&plugin.dir, &plugin.entry);
    let operation = request
        .get("operation")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let mut child = Command::new(&program)
        .current_dir(&plugin.dir)
        .env("BWOC_WORKSPACE", workspace)
        .env("BWOC_PLUGIN_DIR", &plugin.dir)
        .env("BWOC_GCLOUD_OPERATION", operation)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn '{}': {e}", program.to_string_lossy()))?;

    if let Some(stdin) = child.stdin.as_mut() {
        let _ = writeln!(stdin, "{request}");
    }
    drop(child.stdin.take());

    let output = child
        .wait_with_output()
        .map_err(|e| format!("wait '{}': {e}", program.to_string_lossy()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "plugin '{}' exited {} (stderr: {})",
            plugin.name,
            output.status.code().unwrap_or(-1),
            stderr.trim()
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout)
        .map_err(|e| format!("plugin '{}' did not emit valid JSON: {e}", plugin.name))
}

// ---------------------------------------------------------------------------
// Request payloads handed to the plugin over stdin (one per verb).
// ---------------------------------------------------------------------------

fn auth_status_request(workspace: &Path, plugin_dir: &Path) -> serde_json::Value {
    serde_json::json!({
        "operation": "status",
        "workspace": workspace.display().to_string(),
        "plugin_dir": plugin_dir.display().to_string(),
    })
}

fn auth_login_request(
    workspace: &Path,
    plugin_dir: &Path,
    account: Option<&str>,
) -> serde_json::Value {
    serde_json::json!({
        "operation": "login",
        "workspace": workspace.display().to_string(),
        "plugin_dir": plugin_dir.display().to_string(),
        "account": account,
    })
}

fn project_list_request(workspace: &Path, plugin_dir: &Path) -> serde_json::Value {
    serde_json::json!({
        "operation": "list",
        "workspace": workspace.display().to_string(),
        "plugin_dir": plugin_dir.display().to_string(),
    })
}

fn project_show_request(
    workspace: &Path,
    plugin_dir: &Path,
    project: Option<&str>,
) -> serde_json::Value {
    serde_json::json!({
        "operation": "show",
        "workspace": workspace.display().to_string(),
        "plugin_dir": plugin_dir.display().to_string(),
        "project": project,
    })
}

fn project_set_default_request(
    workspace: &Path,
    plugin_dir: &Path,
    project: &str,
) -> serde_json::Value {
    serde_json::json!({
        "operation": "set-default",
        "workspace": workspace.display().to_string(),
        "plugin_dir": plugin_dir.display().to_string(),
        "project": project,
    })
}

fn compute_list_request(
    workspace: &Path,
    plugin_dir: &Path,
    project: Option<&str>,
    zone: Option<&str>,
) -> serde_json::Value {
    serde_json::json!({
        "operation": "list",
        "workspace": workspace.display().to_string(),
        "plugin_dir": plugin_dir.display().to_string(),
        "project": project,
        "zone": zone,
    })
}

/// Write request for `start` / `stop`. Carries `confirmed: true` — the
/// CLI-set marker the plugin's write verbs check for, so a direct plugin
/// invoke that bypasses the CLI gate can be refused (BWOC-66 §3 / BWOC-67).
/// The CLI only builds this after the operator-confirm gate has passed.
fn compute_write_request(
    workspace: &Path,
    plugin_dir: &Path,
    verb: &str,
    instance: &str,
    zone: &str,
    project: Option<&str>,
) -> serde_json::Value {
    serde_json::json!({
        "operation": verb,
        "workspace": workspace.display().to_string(),
        "plugin_dir": plugin_dir.display().to_string(),
        "instance": instance,
        "zone": zone,
        "project": project,
        "confirmed": true,
    })
}

// ---------------------------------------------------------------------------
// Shared helpers.
// ---------------------------------------------------------------------------

fn print_json(value: &serde_json::Value) -> bool {
    match serde_json::to_string_pretty(value) {
        Ok(s) => {
            println!("{s}");
            true
        }
        Err(e) => {
            eprintln!("bwoc gcloud: serialize JSON: {e}");
            false
        }
    }
}

/// A gated write requested in `--json` mode cannot prompt — it requires `--yes`.
fn json_write_blocked(json: bool, yes: bool) -> bool {
    json && !yes
}

fn confirm(prompt: &str) -> bool {
    eprint!("{prompt} [y/N]: ");
    let _ = std::io::stderr().flush();
    let mut line = String::new();
    if std::io::stdin().read_line(&mut line).is_err() {
        return false;
    }
    matches!(line.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

fn emit_error_json(verb: &str, code: &str, message: &str) {
    let value = serde_json::json!({
        "ok": false,
        "verb": verb,
        "error": code,
        "message": message,
    });
    print_json(&value);
}

/// Project ids: lowercase letters, digits, hyphens; 6–30 chars; can't start
/// with a digit or hyphen. The plugin will re-validate against the live API;
/// this is the local pre-check so we never spawn the plugin for obvious junk.
fn is_valid_project_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    if !(6..=30).contains(&bytes.len()) {
        return false;
    }
    let first = bytes[0];
    if !first.is_ascii_lowercase() {
        return false;
    }
    bytes
        .iter()
        .all(|&b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// GCE instance name: 1–30 chars (RFC1035-ish; gcloud caps at 63 but the
/// shorter cap is harmless here), lowercase letter first, then lowercase
/// letters / digits / hyphens, not ending in a hyphen. Local pre-check — its
/// primary job is to reject `-`-leading option-injection before we ever spawn
/// the plugin; the live API re-validates.
fn is_valid_instance_name(name: &str) -> bool {
    let b = name.as_bytes();
    if !(1..=63).contains(&b.len()) {
        return false;
    }
    if !b[0].is_ascii_lowercase() {
        return false;
    }
    if *b.last().unwrap() == b'-' {
        return false;
    }
    b.iter()
        .all(|&c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

/// GCE zone: lowercase letter first, lowercase letters/digits/hyphens, not
/// ending in a hyphen, 3–63 chars (e.g. `us-central1-a`). Same `-`-leading
/// injection guard as the instance/project pre-checks.
fn is_valid_zone(zone: &str) -> bool {
    let b = zone.as_bytes();
    if !(3..=63).contains(&b.len()) {
        return false;
    }
    if !b[0].is_ascii_lowercase() {
        return false;
    }
    if *b.last().unwrap() == b'-' {
        return false;
    }
    b.iter()
        .all(|&c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

/// The literal `gcloud` command a compute write verb will run, shown verbatim
/// in the confirmation gate. The user-supplied positional (instance) goes after
/// the `--` separator so it can never be parsed as a flag; flag values use the
/// `--flag=value` form for the same reason (#92 / #91 option-injection guard).
fn gcloud_compute_command(verb: &str, instance: &str, zone: &str, project: Option<&str>) -> String {
    let mut s = format!("gcloud compute instances {verb} --zone={zone}");
    if let Some(p) = project {
        s.push_str(&format!(" --project={p}"));
    }
    s.push_str(&format!(" -- {instance}"));
    s
}

/// One-line description of the remote effect, surfaced in the gate so the
/// operator confirms against the consequence, not just the verb name.
fn compute_write_effect(verb: &str) -> &'static str {
    match verb {
        "start" => "boots the instance (incurs compute cost)",
        "stop" => "halts the instance (interrupts running workloads)",
        _ => "changes instance state",
    }
}

/// Outcome of the operator-confirm gate. Pure so it is unit-testable away from
/// the stdin/stderr of the actual prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GateOutcome {
    /// Confirmed (or `--yes`) — perform the write.
    Proceed,
    /// `--json` cannot prompt and `--yes` was not passed.
    BlockedJsonNeedsYes,
    /// Interactive operator answered No (or stdin was unreadable).
    Declined,
}

/// Decide the gate outcome. `interactive_confirm` is the y/N answer when a
/// prompt was shown (`Some`), or `None` when no prompt happened (`--yes` or
/// `--json` mode).
fn gate_decision(yes: bool, json: bool, interactive_confirm: Option<bool>) -> GateOutcome {
    if yes {
        return GateOutcome::Proceed;
    }
    if json {
        return GateOutcome::BlockedJsonNeedsYes;
    }
    match interactive_confirm {
        Some(true) => GateOutcome::Proceed,
        _ => GateOutcome::Declined,
    }
}

/// Report a write that did not happen — "no change" with a reason, never a bare
/// failure (BWOC-67 gate contract point 4 / BWOC-66 §3).
fn emit_compute_no_change(
    label: &str,
    instance: &str,
    zone: &str,
    code: &str,
    reason: &str,
    json: bool,
) {
    if json {
        let value = serde_json::json!({
            "ok": false,
            "verb": label,
            "changed": false,
            "instance": instance,
            "zone": zone,
            "error": code,
            "reason": reason,
        });
        print_json(&value);
    } else {
        eprintln!(
            "bwoc gcloud {label}: no change — {reason} \
             (instance '{instance}' in zone '{zone}' unchanged)"
        );
    }
}

/// Stub-error envelope for the missing-plugin path. Names the exact plugin and
/// the install hint the operator needs.
fn no_plugin_message(plugin_name: &str) -> String {
    format!(
        "no enabled '{plugin_name}' plugin (workflow kind) in this workspace. \
         The live GCP path is provided by `workflow/{plugin_name}` (see the EPIC-8 \
         design note). Install it (BWOC-53) with `bwoc plugin install <source>` \
         then `bwoc plugin enable {plugin_name}`. \
         `bwoc gcloud status` continues to work offline."
    )
}

fn require_plugin(
    root: &Path,
    plugin_name: &str,
    verb: &str,
    json: bool,
) -> Result<GcloudPlugin, i32> {
    match find_enabled_plugin(root, plugin_name) {
        Ok(Some(p)) => Ok(p),
        Ok(None) => {
            let msg = no_plugin_message(plugin_name);
            if json {
                emit_error_json(verb, "no_plugin", &msg);
            } else {
                eprintln!("bwoc gcloud {verb}: {msg}");
            }
            Err(EXIT_NO_PLUGIN)
        }
        Err(e) => {
            if json {
                emit_error_json(verb, "discovery_error", &e);
            } else {
                eprintln!("bwoc gcloud {verb}: {e}");
            }
            Err(EXIT_PLUGIN_ERROR)
        }
    }
}

// ---------------------------------------------------------------------------
// Verb implementations.
// ---------------------------------------------------------------------------

fn run_auth_status(args: AuthStatusArgs) -> i32 {
    let root = match resolve_workspace(args.workspace) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("bwoc gcloud auth status: {e}");
            return EXIT_USAGE;
        }
    };
    let shape = probe_auth_shape(&root, home_dir().as_deref(), &real_getenv);

    let plugin = match require_plugin(&root, PLUGIN_AUTH, "auth status", args.json) {
        Ok(p) => p,
        Err(code) => return code,
    };
    let request = auth_status_request(&root, &plugin.dir);
    match invoke_plugin(&plugin, &root, &request) {
        Ok(value) => {
            if args.json {
                let merged = serde_json::json!({
                    "ok": true,
                    "workspace": root.display().to_string(),
                    "auth": value,
                    "shape": shape,
                });
                if print_json(&merged) {
                    EXIT_OK
                } else {
                    EXIT_PLUGIN_ERROR
                }
            } else {
                let source = value
                    .get("active_source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("(unknown)");
                let email = value
                    .get("account_email")
                    .and_then(|v| v.as_str())
                    .unwrap_or("(unknown)");
                let has = value
                    .get("has_credential")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                println!(
                    "bwoc gcloud auth: source={source}, account={email}, has_credential={has}"
                );
                EXIT_OK
            }
        }
        Err(e) => {
            if args.json {
                emit_error_json("auth status", "plugin_error", &e);
            } else {
                eprintln!("bwoc gcloud auth status: {e}");
            }
            EXIT_PLUGIN_ERROR
        }
    }
}

fn run_auth_login(args: AuthLoginArgs) -> i32 {
    let root = match resolve_workspace(args.workspace) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("bwoc gcloud auth login: {e}");
            return EXIT_USAGE;
        }
    };

    // Write gate — login is operator-driven (a browser opens), never agent-auto.
    if !args.yes {
        if json_write_blocked(args.json, args.yes) {
            eprintln!("bwoc gcloud auth login: --json requires --yes (login is operator-driven)");
            return EXIT_USAGE;
        }
        let prompt = match &args.account {
            Some(a) => format!("Run `gcloud auth login --account {a}`?"),
            None => "Run `gcloud auth login`?".to_string(),
        };
        if !confirm(&prompt) {
            eprintln!("bwoc gcloud auth login: aborted (no login performed)");
            return EXIT_USAGE;
        }
    }

    let plugin = match require_plugin(&root, PLUGIN_AUTH, "auth login", args.json) {
        Ok(p) => p,
        Err(code) => return code,
    };
    let request = auth_login_request(&root, &plugin.dir, args.account.as_deref());
    match invoke_plugin(&plugin, &root, &request) {
        Ok(value) => {
            if args.json {
                if print_json(&value) {
                    EXIT_OK
                } else {
                    EXIT_PLUGIN_ERROR
                }
            } else {
                println!("bwoc gcloud auth login: completed");
                EXIT_OK
            }
        }
        Err(e) => {
            if args.json {
                emit_error_json("auth login", "plugin_error", &e);
            } else {
                eprintln!("bwoc gcloud auth login: {e}");
            }
            EXIT_PLUGIN_ERROR
        }
    }
}

fn run_project_list(args: ProjectListArgs) -> i32 {
    let root = match resolve_workspace(args.workspace) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("bwoc gcloud project list: {e}");
            return EXIT_USAGE;
        }
    };
    let plugin = match require_plugin(&root, PLUGIN_PROJECT, "project list", args.json) {
        Ok(p) => p,
        Err(code) => return code,
    };
    let request = project_list_request(&root, &plugin.dir);
    match invoke_plugin(&plugin, &root, &request) {
        Ok(value) => {
            if args.json {
                if print_json(&value) {
                    EXIT_OK
                } else {
                    EXIT_PLUGIN_ERROR
                }
            } else {
                let total = value.as_array().map(|a| a.len()).unwrap_or(0);
                println!("bwoc gcloud project list: {total} project(s)");
                if let Some(arr) = value.as_array() {
                    for p in arr {
                        let id = p.get("project_id").and_then(|v| v.as_str()).unwrap_or("?");
                        let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                        let state = p
                            .get("lifecycle_state")
                            .and_then(|v| v.as_str())
                            .unwrap_or("?");
                        println!("  {id} — {name} [{state}]");
                    }
                }
                EXIT_OK
            }
        }
        Err(e) => {
            if args.json {
                emit_error_json("project list", "plugin_error", &e);
            } else {
                eprintln!("bwoc gcloud project list: {e}");
            }
            EXIT_PLUGIN_ERROR
        }
    }
}

fn run_project_show(args: ProjectShowArgs) -> i32 {
    let root = match resolve_workspace(args.workspace) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("bwoc gcloud project show: {e}");
            return EXIT_USAGE;
        }
    };
    if let Some(id) = &args.project {
        if !is_valid_project_id(id) {
            let msg = format!(
                "invalid project id '{id}' — expected 6–30 chars, lowercase \
                 letters/digits/hyphens, starting with a letter"
            );
            if args.json {
                emit_error_json("project show", "bad_project_id", &msg);
            } else {
                eprintln!("bwoc gcloud project show: {msg}");
            }
            return EXIT_USAGE;
        }
    }
    let plugin = match require_plugin(&root, PLUGIN_PROJECT, "project show", args.json) {
        Ok(p) => p,
        Err(code) => return code,
    };
    let request = project_show_request(&root, &plugin.dir, args.project.as_deref());
    match invoke_plugin(&plugin, &root, &request) {
        Ok(value) => {
            if args.json {
                if print_json(&value) {
                    EXIT_OK
                } else {
                    EXIT_PLUGIN_ERROR
                }
            } else {
                let id = value
                    .get("project_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let name = value.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                let state = value
                    .get("lifecycle_state")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                println!("bwoc gcloud project show: {id} — {name} [{state}]");
                EXIT_OK
            }
        }
        Err(e) => {
            if args.json {
                emit_error_json("project show", "plugin_error", &e);
            } else {
                eprintln!("bwoc gcloud project show: {e}");
            }
            EXIT_PLUGIN_ERROR
        }
    }
}

fn run_project_set_default(args: ProjectSetDefaultArgs) -> i32 {
    let root = match resolve_workspace(args.workspace) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("bwoc gcloud project set-default: {e}");
            return EXIT_USAGE;
        }
    };
    if !is_valid_project_id(&args.project) {
        let msg = format!(
            "invalid project id '{}' — expected 6–30 chars, lowercase \
             letters/digits/hyphens, starting with a letter",
            args.project
        );
        if args.json {
            emit_error_json("project set-default", "bad_project_id", &msg);
        } else {
            eprintln!("bwoc gcloud project set-default: {msg}");
        }
        return EXIT_USAGE;
    }

    // Write gate.
    if !args.yes {
        if json_write_blocked(args.json, args.yes) {
            eprintln!(
                "bwoc gcloud project set-default: --json requires --yes \
                 (a write needs explicit ack)"
            );
            return EXIT_USAGE;
        }
        let prompt = format!("Set local gcloud default project to '{}'?", args.project);
        if !confirm(&prompt) {
            eprintln!("bwoc gcloud project set-default: aborted (no write performed)");
            return EXIT_USAGE;
        }
    }

    let plugin = match require_plugin(&root, PLUGIN_PROJECT, "project set-default", args.json) {
        Ok(p) => p,
        Err(code) => return code,
    };
    let request = project_set_default_request(&root, &plugin.dir, &args.project);
    match invoke_plugin(&plugin, &root, &request) {
        Ok(value) => {
            if args.json {
                if print_json(&value) {
                    EXIT_OK
                } else {
                    EXIT_PLUGIN_ERROR
                }
            } else {
                let previous = value
                    .get("previous")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let current = value.get("current").and_then(|v| v.as_str()).unwrap_or("?");
                println!(
                    "bwoc gcloud project set-default: {previous} → {current} (local gcloud config)"
                );
                EXIT_OK
            }
        }
        Err(e) => {
            if args.json {
                emit_error_json("project set-default", "plugin_error", &e);
            } else {
                eprintln!("bwoc gcloud project set-default: {e}");
            }
            EXIT_PLUGIN_ERROR
        }
    }
}

// ---------------------------------------------------------------------------
// Compute verbs (EPIC-9) — `list` reads (no gate); `start`/`stop` write
// (operator-confirm gated in the CLI, never the plugin).
// ---------------------------------------------------------------------------

fn run_compute_list(args: ComputeListArgs) -> i32 {
    let root = match resolve_workspace(args.workspace) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("bwoc gcloud compute list: {e}");
            return EXIT_USAGE;
        }
    };
    if let Some(p) = &args.project {
        if !is_valid_project_id(p) {
            let msg = format!(
                "invalid project id '{p}' — expected 6–30 chars, lowercase \
                 letters/digits/hyphens, starting with a letter"
            );
            if args.json {
                emit_error_json("compute list", "bad_project_id", &msg);
            } else {
                eprintln!("bwoc gcloud compute list: {msg}");
            }
            return EXIT_USAGE;
        }
    }
    if let Some(z) = &args.zone {
        if !is_valid_zone(z) {
            let msg = format!(
                "invalid zone '{z}' — expected lowercase letters/digits/hyphens, \
                 starting with a letter (e.g. us-central1-a)"
            );
            if args.json {
                emit_error_json("compute list", "bad_zone", &msg);
            } else {
                eprintln!("bwoc gcloud compute list: {msg}");
            }
            return EXIT_USAGE;
        }
    }
    let plugin = match require_plugin(&root, PLUGIN_COMPUTE, "compute list", args.json) {
        Ok(p) => p,
        Err(code) => return code,
    };
    let request = compute_list_request(
        &root,
        &plugin.dir,
        args.project.as_deref(),
        args.zone.as_deref(),
    );
    match invoke_plugin(&plugin, &root, &request) {
        Ok(value) => {
            if args.json {
                if print_json(&value) {
                    EXIT_OK
                } else {
                    EXIT_PLUGIN_ERROR
                }
            } else {
                // The plugin returns gcloud's JSON, surfaced through; accept
                // either a `{ instances: [...] }` envelope or a bare array.
                let instances = value
                    .get("instances")
                    .and_then(|v| v.as_array())
                    .or_else(|| value.as_array());
                let total = instances.map(|a| a.len()).unwrap_or(0);
                println!("bwoc gcloud compute list: {total} instance(s)");
                if let Some(arr) = instances {
                    for inst in arr {
                        let name = inst.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                        let zone = inst.get("zone").and_then(|v| v.as_str()).unwrap_or("?");
                        let status = inst.get("status").and_then(|v| v.as_str()).unwrap_or("?");
                        println!("  {name} [{zone}] — {status}");
                    }
                }
                EXIT_OK
            }
        }
        Err(e) => {
            if args.json {
                emit_error_json("compute list", "plugin_error", &e);
            } else {
                eprintln!("bwoc gcloud compute list: {e}");
            }
            EXIT_PLUGIN_ERROR
        }
    }
}

fn run_compute_start(args: ComputeStartArgs) -> i32 {
    run_compute_write(
        "start",
        &args.instance,
        &args.zone,
        args.project.as_deref(),
        args.yes,
        args.json,
        args.workspace,
    )
}

fn run_compute_stop(args: ComputeStopArgs) -> i32 {
    run_compute_write(
        "stop",
        &args.instance,
        &args.zone,
        args.project.as_deref(),
        args.yes,
        args.json,
        args.workspace,
    )
}

/// Shared body for the two gated write verbs. The gate lives here (the operator
/// boundary), not in the plugin: show the exact effect + the literal `gcloud`
/// command, default No, `--yes` to bypass, refused → "no change" with a reason.
fn run_compute_write(
    verb: &str,
    instance: &str,
    zone: &str,
    project: Option<&str>,
    yes: bool,
    json: bool,
    workspace: Option<PathBuf>,
) -> i32 {
    let label = format!("compute {verb}");
    let root = match resolve_workspace(workspace) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("bwoc gcloud {label}: {e}");
            return EXIT_USAGE;
        }
    };
    if !is_valid_instance_name(instance) {
        let msg = format!(
            "invalid instance name '{instance}' — expected 1–63 chars, lowercase \
             letter first, then lowercase letters/digits/hyphens, no trailing hyphen"
        );
        if json {
            emit_error_json(&label, "bad_instance", &msg);
        } else {
            eprintln!("bwoc gcloud {label}: {msg}");
        }
        return EXIT_USAGE;
    }
    if !is_valid_zone(zone) {
        let msg = format!(
            "invalid zone '{zone}' — expected lowercase letters/digits/hyphens, \
             starting with a letter (e.g. us-central1-a)"
        );
        if json {
            emit_error_json(&label, "bad_zone", &msg);
        } else {
            eprintln!("bwoc gcloud {label}: {msg}");
        }
        return EXIT_USAGE;
    }
    if let Some(p) = project {
        if !is_valid_project_id(p) {
            let msg = format!(
                "invalid project id '{p}' — expected 6–30 chars, lowercase \
                 letters/digits/hyphens, starting with a letter"
            );
            if json {
                emit_error_json(&label, "bad_project_id", &msg);
            } else {
                eprintln!("bwoc gcloud {label}: {msg}");
            }
            return EXIT_USAGE;
        }
    }

    let command = gcloud_compute_command(verb, instance, zone, project);

    // Operator-confirm gate. Show the exact effect + literal command, then
    // prompt (interactive only). `--json` cannot prompt, so it needs `--yes`.
    let interactive_confirm = if !yes && !json {
        eprintln!("bwoc gcloud {label}: operator confirmation required (write verb).");
        eprintln!("  instance: {instance}");
        eprintln!("  zone:     {zone}");
        if let Some(p) = project {
            eprintln!("  project:  {p}");
        }
        eprintln!("  effect:   {}", compute_write_effect(verb));
        eprintln!("  command:  {command}");
        Some(confirm(&format!(
            "Proceed to {verb} instance '{instance}' in zone '{zone}'?"
        )))
    } else {
        None
    };

    match gate_decision(yes, json, interactive_confirm) {
        GateOutcome::BlockedJsonNeedsYes => {
            emit_compute_no_change(
                &label,
                instance,
                zone,
                "confirmation_required",
                "--json mode cannot prompt; pass --yes only when the operator \
                 authorized this specific action",
                json,
            );
            return EXIT_USAGE;
        }
        GateOutcome::Declined => {
            emit_compute_no_change(
                &label,
                instance,
                zone,
                "declined",
                "operator declined confirmation",
                json,
            );
            return EXIT_USAGE;
        }
        GateOutcome::Proceed => {}
    }

    let plugin = match require_plugin(&root, PLUGIN_COMPUTE, &label, json) {
        Ok(p) => p,
        Err(code) => return code,
    };
    let request = compute_write_request(&root, &plugin.dir, verb, instance, zone, project);
    match invoke_plugin(&plugin, &root, &request) {
        Ok(value) => {
            if json {
                if print_json(&value) {
                    EXIT_OK
                } else {
                    EXIT_PLUGIN_ERROR
                }
            } else {
                let status = value
                    .get("status")
                    .and_then(|v| v.as_str())
                    .or_else(|| value.get("current_status").and_then(|v| v.as_str()))
                    .unwrap_or("done");
                println!("bwoc gcloud {label}: instance '{instance}' in zone '{zone}' — {status}");
                EXIT_OK
            }
        }
        Err(e) => {
            if json {
                emit_error_json(&label, "plugin_error", &e);
            } else {
                eprintln!("bwoc gcloud {label}: {e}");
            }
            EXIT_PLUGIN_ERROR
        }
    }
}

/// `bwoc gcloud status` — combined view. **Degrades when plugins are missing:**
/// reports the auth shape + project env hints from local state alone, and
/// notes which plugins are absent. Always exits `0` unless the workspace
/// itself can't be resolved or local I/O fails — the absence of plugins is a
/// reportable condition, not an error.
fn run_combined_status(args: StatusArgs) -> i32 {
    let root = match resolve_workspace(args.workspace) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("bwoc gcloud status: {e}");
            return EXIT_USAGE;
        }
    };
    let shape = probe_auth_shape(&root, home_dir().as_deref(), &real_getenv);

    // Try each plugin — capture either the live response or the "not enabled"
    // signal. We do NOT fail the status command on plugin error; we report.
    let (auth_live, auth_state) = match find_enabled_plugin(&root, PLUGIN_AUTH) {
        Ok(Some(p)) => {
            let req = auth_status_request(&root, &p.dir);
            match invoke_plugin(&p, &root, &req) {
                Ok(v) => (Some(v), "ok"),
                Err(_) => (None, "plugin_error"),
            }
        }
        Ok(None) => (None, "not_enabled"),
        Err(_) => (None, "discovery_error"),
    };
    let (project_live, project_state) = match find_enabled_plugin(&root, PLUGIN_PROJECT) {
        Ok(Some(p)) => {
            let req = project_show_request(&root, &p.dir, None);
            match invoke_plugin(&p, &root, &req) {
                Ok(v) => (Some(v), "ok"),
                Err(_) => (None, "plugin_error"),
            }
        }
        Ok(None) => (None, "not_enabled"),
        Err(_) => (None, "discovery_error"),
    };

    // Reachability is "we have a credential shape AND the auth plugin reported
    // success". We deliberately don't ping Google — the plugin's own status verb
    // is the reachability signal we trust.
    let reachable = matches!(auth_state, "ok")
        && auth_live
            .as_ref()
            .and_then(|v| v.get("has_credential"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

    if args.json {
        let value = serde_json::json!({
            "ok": true,
            "workspace": root.display().to_string(),
            "shape": shape,
            "auth": { "state": auth_state, "live": auth_live },
            "project": { "state": project_state, "live": project_live },
            "reachable": reachable,
        });
        return if print_json(&value) {
            EXIT_OK
        } else {
            EXIT_LOCAL_ERROR
        };
    }

    println!("bwoc gcloud status — workspace: {}", root.display());
    println!("  active_source: {}", shape.active_source.as_str());
    println!(
        "  adc_present={}, service_account_present={}, env_account={}, env_project={}, env_impersonate={}",
        shape.adc_present,
        shape.service_account_present,
        shape.env_account,
        shape.env_project,
        shape.env_impersonate,
    );
    match (auth_state, auth_live.as_ref()) {
        ("ok", Some(v)) => {
            let email = v
                .get("account_email")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let has = v
                .get("has_credential")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            println!("  auth plugin: ok (account={email}, has_credential={has})");
        }
        ("not_enabled", _) => println!("  auth plugin: not enabled ({PLUGIN_AUTH})"),
        (state, _) => println!("  auth plugin: {state}"),
    }
    match (project_state, project_live.as_ref()) {
        ("ok", Some(v)) => {
            let id = v.get("project_id").and_then(|v| v.as_str()).unwrap_or("?");
            let name = v.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            println!("  project plugin: ok (default project: {id} — {name})");
        }
        ("not_enabled", _) => println!("  project plugin: not enabled ({PLUGIN_PROJECT})"),
        (state, _) => println!("  project plugin: {state}"),
    }
    println!("  reachable: {reachable}");
    EXIT_OK
}

// ===========================================================================
// Tests — arg parsing, JSON shapes, project-id validation, no-plugin stub
// path, write gate, auth-shape probe.
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::collections::HashMap;

    #[derive(Parser, Debug)]
    struct TestCli {
        #[command(subcommand)]
        cmd: GcloudCommand,
    }

    fn parse(args: &[&str]) -> Result<GcloudCommand, clap::Error> {
        let mut full = vec!["bwoc-gcloud-test"];
        full.extend_from_slice(args);
        TestCli::try_parse_from(full).map(|c| c.cmd)
    }

    fn getenv_from(map: HashMap<&'static str, &'static str>) -> impl Fn(&str) -> Option<String> {
        move |k: &str| map.get(k).map(|v| v.to_string())
    }

    // --- arg parsing -------------------------------------------------------

    #[test]
    fn parses_auth_status() {
        match parse(&["auth", "status", "--json"]).unwrap() {
            GcloudCommand::Auth(AuthCommand::Status(a)) => assert!(a.json),
            other => panic!("expected Auth::Status, got {other:?}"),
        }
    }

    #[test]
    fn parses_auth_login_with_account() {
        match parse(&["auth", "login", "--account", "op@example.com", "--yes"]).unwrap() {
            GcloudCommand::Auth(AuthCommand::Login(a)) => {
                assert_eq!(a.account.as_deref(), Some("op@example.com"));
                assert!(a.yes);
            }
            other => panic!("expected Auth::Login, got {other:?}"),
        }
    }

    #[test]
    fn parses_project_list_and_show() {
        match parse(&["project", "list", "--json"]).unwrap() {
            GcloudCommand::Project(ProjectCommand::List(a)) => assert!(a.json),
            other => panic!("expected Project::List, got {other:?}"),
        }
        match parse(&["project", "show", "--project", "my-proj-123"]).unwrap() {
            GcloudCommand::Project(ProjectCommand::Show(a)) => {
                assert_eq!(a.project.as_deref(), Some("my-proj-123"));
            }
            other => panic!("expected Project::Show, got {other:?}"),
        }
    }

    #[test]
    fn parses_project_set_default() {
        match parse(&[
            "project",
            "set-default",
            "--project",
            "my-proj-123",
            "--yes",
            "--json",
        ])
        .unwrap()
        {
            GcloudCommand::Project(ProjectCommand::SetDefault(a)) => {
                assert_eq!(a.project, "my-proj-123");
                assert!(a.yes);
                assert!(a.json);
            }
            other => panic!("expected Project::SetDefault, got {other:?}"),
        }
    }

    #[test]
    fn parses_combined_status() {
        match parse(&["status", "--json"]).unwrap() {
            GcloudCommand::Status(a) => assert!(a.json),
            other => panic!("expected Status, got {other:?}"),
        }
    }

    #[test]
    fn project_set_default_requires_project_flag() {
        assert!(parse(&["project", "set-default"]).is_err());
        assert!(parse(&["project", "set-default", "--yes"]).is_err());
    }

    #[test]
    fn rejects_unknown_subcommand() {
        assert!(parse(&["frobnicate"]).is_err());
    }

    // --- project id validation --------------------------------------------

    #[test]
    fn accepts_valid_project_ids() {
        assert!(is_valid_project_id("my-proj-123"));
        assert!(is_valid_project_id("abcdef"));
        assert!(is_valid_project_id("a-very-long-id-but-fine"));
    }

    #[test]
    fn rejects_invalid_project_ids() {
        assert!(!is_valid_project_id("short")); // < 6
        assert!(!is_valid_project_id("1bad-first-char"));
        assert!(!is_valid_project_id("-bad-first-char"));
        assert!(!is_valid_project_id("UPPER-not-allowed"));
        assert!(!is_valid_project_id("has spaces"));
        assert!(!is_valid_project_id(""));
        // 31 chars — too long.
        assert!(!is_valid_project_id(&"a".repeat(31)));
    }

    // --- auth-shape probe (file + env, no network) -------------------------

    #[test]
    fn auth_shape_none_when_nothing_present() {
        let ws = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        let env = getenv_from(HashMap::new());
        let shape = probe_auth_shape(ws.path(), Some(home.path()), &env);
        assert_eq!(shape.active_source, AuthSource::None);
        assert!(!shape.adc_present);
        assert!(!shape.service_account_present);
        assert!(!shape.env_account && !shape.env_project && !shape.env_impersonate);
    }

    #[test]
    fn auth_shape_adc_wins_when_present() {
        let ws = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        // Plant a fake ADC file (presence-only — we never read the contents).
        let adc = home.path().join(ADC_REL);
        std::fs::create_dir_all(adc.parent().unwrap()).unwrap();
        std::fs::write(&adc, "fake-adc-not-real").unwrap();
        // Also plant SA + env signals — ADC must still win.
        std::fs::create_dir_all(ws.path().join(".bwoc/secrets")).unwrap();
        std::fs::write(ws.path().join(SA_REL), "fake-sa-not-real").unwrap();
        let env = getenv_from(HashMap::from([(ENV_ACCOUNT, "x@example.com")]));
        let shape = probe_auth_shape(ws.path(), Some(home.path()), &env);
        assert_eq!(shape.active_source, AuthSource::Adc);
        assert!(shape.adc_present);
        assert!(shape.service_account_present);
        assert!(shape.env_account);
    }

    #[test]
    fn auth_shape_service_account_when_no_adc() {
        let ws = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(ws.path().join(".bwoc/secrets")).unwrap();
        std::fs::write(ws.path().join(SA_REL), "fake-sa-not-real").unwrap();
        let env = getenv_from(HashMap::new());
        let shape = probe_auth_shape(ws.path(), Some(home.path()), &env);
        assert_eq!(shape.active_source, AuthSource::ServiceAccount);
        assert!(!shape.adc_present);
        assert!(shape.service_account_present);
    }

    #[test]
    fn auth_shape_env_when_only_env() {
        let ws = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        let env = getenv_from(HashMap::from([(ENV_PROJECT, "my-proj-123")]));
        let shape = probe_auth_shape(ws.path(), Some(home.path()), &env);
        assert_eq!(shape.active_source, AuthSource::Env);
        assert!(shape.env_project);
        assert!(!shape.env_account);
    }

    #[test]
    fn auth_shape_serializes_active_source_as_kebab_case() {
        let ws = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(ws.path().join(".bwoc/secrets")).unwrap();
        std::fs::write(ws.path().join(SA_REL), "x").unwrap();
        let env = getenv_from(HashMap::new());
        let shape = probe_auth_shape(ws.path(), Some(home.path()), &env);
        let json = serde_json::to_string(&shape).unwrap();
        // The whole point of the enum is the wire shape — assert it.
        assert!(
            json.contains("\"active_source\":\"service-account\""),
            "{json}"
        );
    }

    // --- request payload shapes (what the CLI hands the plugin) ------------

    #[test]
    fn auth_status_request_shape() {
        let v = auth_status_request(Path::new("/ws"), Path::new("/p"));
        assert_eq!(v["operation"], "status");
        assert_eq!(v["workspace"], "/ws");
        assert_eq!(v["plugin_dir"], "/p");
    }

    #[test]
    fn auth_login_request_carries_account_when_set() {
        let v = auth_login_request(Path::new("/ws"), Path::new("/p"), Some("op@example.com"));
        assert_eq!(v["operation"], "login");
        assert_eq!(v["account"], "op@example.com");
    }

    #[test]
    fn auth_login_request_null_account_when_absent() {
        let v = auth_login_request(Path::new("/ws"), Path::new("/p"), None);
        assert!(v["account"].is_null());
    }

    #[test]
    fn project_show_request_carries_explicit_project() {
        let v = project_show_request(Path::new("/ws"), Path::new("/p"), Some("my-proj-123"));
        assert_eq!(v["operation"], "show");
        assert_eq!(v["project"], "my-proj-123");
    }

    #[test]
    fn project_set_default_request_shape() {
        let v = project_set_default_request(Path::new("/ws"), Path::new("/p"), "my-proj-123");
        assert_eq!(v["operation"], "set-default");
        assert_eq!(v["project"], "my-proj-123");
    }

    // --- write gate --------------------------------------------------------

    #[test]
    fn json_write_blocked_without_yes() {
        assert!(json_write_blocked(true, false));
        assert!(!json_write_blocked(true, true));
        assert!(!json_write_blocked(false, false));
    }

    // --- compute: arg parsing ---------------------------------------------

    #[test]
    fn parses_compute_list() {
        match parse(&["compute", "list", "--zone", "us-central1-a", "--json"]).unwrap() {
            GcloudCommand::Compute(ComputeCommand::List(a)) => {
                assert_eq!(a.zone.as_deref(), Some("us-central1-a"));
                assert!(a.json);
            }
            other => panic!("expected Compute::List, got {other:?}"),
        }
    }

    #[test]
    fn parses_compute_start_and_stop() {
        match parse(&[
            "compute",
            "start",
            "--instance",
            "web-1",
            "--zone",
            "us-central1-a",
            "--yes",
        ])
        .unwrap()
        {
            GcloudCommand::Compute(ComputeCommand::Start(a)) => {
                assert_eq!(a.instance, "web-1");
                assert_eq!(a.zone, "us-central1-a");
                assert!(a.yes);
            }
            other => panic!("expected Compute::Start, got {other:?}"),
        }
        match parse(&[
            "compute",
            "stop",
            "--instance",
            "web-1",
            "--zone",
            "us-central1-a",
        ])
        .unwrap()
        {
            GcloudCommand::Compute(ComputeCommand::Stop(a)) => {
                assert_eq!(a.instance, "web-1");
                assert!(!a.yes);
            }
            other => panic!("expected Compute::Stop, got {other:?}"),
        }
    }

    #[test]
    fn compute_write_requires_instance_and_zone() {
        assert!(parse(&["compute", "start"]).is_err());
        assert!(parse(&["compute", "start", "--instance", "web-1"]).is_err());
        assert!(parse(&["compute", "start", "--zone", "us-central1-a"]).is_err());
        assert!(parse(&["compute", "stop", "--instance", "web-1"]).is_err());
    }

    // --- compute: gate decision (confirm required / --yes bypass / refused) -

    #[test]
    fn gate_yes_bypasses_prompt() {
        // --yes proceeds regardless of json or a (never-shown) prompt answer.
        assert_eq!(gate_decision(true, false, None), GateOutcome::Proceed);
        assert_eq!(gate_decision(true, true, None), GateOutcome::Proceed);
    }

    #[test]
    fn gate_json_without_yes_is_blocked() {
        assert_eq!(
            gate_decision(false, true, None),
            GateOutcome::BlockedJsonNeedsYes
        );
    }

    #[test]
    fn gate_interactive_confirm_proceeds() {
        assert_eq!(
            gate_decision(false, false, Some(true)),
            GateOutcome::Proceed
        );
    }

    #[test]
    fn gate_refused_is_no_change() {
        // Operator said No → declined (no change).
        assert_eq!(
            gate_decision(false, false, Some(false)),
            GateOutcome::Declined
        );
        // Unreadable stdin (None while interactive) is treated as a decline too.
        assert_eq!(gate_decision(false, false, None), GateOutcome::Declined);
    }

    // --- compute: instance / zone validation -------------------------------

    #[test]
    fn accepts_valid_instance_names_and_zones() {
        assert!(is_valid_instance_name("web-1"));
        assert!(is_valid_instance_name("a"));
        assert!(is_valid_instance_name("instance-with-many-parts-9"));
        assert!(is_valid_zone("us-central1-a"));
        assert!(is_valid_zone("europe-west4-b"));
    }

    #[test]
    fn rejects_invalid_instance_names_and_zones() {
        assert!(!is_valid_instance_name("")); // empty
        assert!(!is_valid_instance_name("-leading-hyphen")); // option-injection guard
        assert!(!is_valid_instance_name("1-digit-first"));
        assert!(!is_valid_instance_name("UPPER"));
        assert!(!is_valid_instance_name("trailing-")); // trailing hyphen
        assert!(!is_valid_instance_name(&"a".repeat(64))); // too long
        assert!(!is_valid_zone("--inject")); // option-injection guard
        assert!(!is_valid_zone("Us-Central")); // uppercase
        assert!(!is_valid_zone("ab")); // too short
        assert!(!is_valid_zone("zone-")); // trailing hyphen
    }

    // --- compute: literal command + request shapes -------------------------

    #[test]
    fn compute_command_puts_instance_after_separator() {
        let cmd = gcloud_compute_command("start", "web-1", "us-central1-a", None);
        assert_eq!(
            cmd,
            "gcloud compute instances start --zone=us-central1-a -- web-1"
        );
        // `--` precedes the positional so a `-`-leading value can't be a flag.
        assert!(cmd.contains(" -- web-1"));
    }

    #[test]
    fn compute_command_includes_project_when_set() {
        let cmd = gcloud_compute_command("stop", "web-1", "us-central1-a", Some("my-proj-123"));
        assert_eq!(
            cmd,
            "gcloud compute instances stop --zone=us-central1-a --project=my-proj-123 -- web-1"
        );
    }

    #[test]
    fn compute_list_request_carries_optional_filters() {
        let v = compute_list_request(
            Path::new("/ws"),
            Path::new("/p"),
            Some("my-proj-123"),
            Some("us-central1-a"),
        );
        assert_eq!(v["operation"], "list");
        assert_eq!(v["project"], "my-proj-123");
        assert_eq!(v["zone"], "us-central1-a");
        // Omitted filters serialize as null, not missing.
        let bare = compute_list_request(Path::new("/ws"), Path::new("/p"), None, None);
        assert!(bare["project"].is_null());
        assert!(bare["zone"].is_null());
    }

    #[test]
    fn compute_write_request_carries_confirmed_marker() {
        let v = compute_write_request(
            Path::new("/ws"),
            Path::new("/p"),
            "start",
            "web-1",
            "us-central1-a",
            None,
        );
        assert_eq!(v["operation"], "start");
        assert_eq!(v["instance"], "web-1");
        assert_eq!(v["zone"], "us-central1-a");
        // The CLI-set gate marker the plugin checks for (BWOC-66 §3).
        assert_eq!(v["confirmed"], true);
    }

    #[test]
    fn compute_write_effect_names_consequence() {
        assert!(compute_write_effect("start").contains("cost"));
        assert!(compute_write_effect("stop").contains("workloads"));
    }

    // --- plugin discovery / stub-error path --------------------------------

    fn write_workspace(root: &Path, workspace_toml: &str) {
        std::fs::create_dir_all(root.join(".bwoc")).unwrap();
        std::fs::write(root.join(".bwoc/workspace.toml"), workspace_toml).unwrap();
    }

    fn write_plugin_at(root: &Path, layout: &str, name: &str, kind: &str) {
        let dir = root.join("modules/plugins").join(layout).join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("manifest.toml"),
            format!(
                "[plugin]\nname = \"{name}\"\nkind = \"{kind}\"\nversion = \"0.1.0\"\n\
                 description = \"x\"\ncompat = \">=2.5.0\"\nentry = \"gcloud.sh\"\n"
            ),
        )
        .unwrap();
    }

    #[test]
    fn no_plugins_dir_discovers_nothing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(discover_plugin(dir.path(), PLUGIN_AUTH).unwrap().is_none());
    }

    #[test]
    fn discovers_flat_layout() {
        let dir = tempfile::tempdir().unwrap();
        write_plugin_at(dir.path(), "", PLUGIN_AUTH, "workflow");
        let p = discover_plugin(dir.path(), PLUGIN_AUTH).unwrap().unwrap();
        assert_eq!(p.name, PLUGIN_AUTH);
        assert_eq!(p.entry, "gcloud.sh");
    }

    #[test]
    fn discovers_workflow_namespaced_layout() {
        let dir = tempfile::tempdir().unwrap();
        write_plugin_at(dir.path(), "workflow", PLUGIN_PROJECT, "workflow");
        let p = discover_plugin(dir.path(), PLUGIN_PROJECT)
            .unwrap()
            .unwrap();
        assert_eq!(p.name, PLUGIN_PROJECT);
    }

    #[test]
    fn discovery_rejects_wrong_kind() {
        let dir = tempfile::tempdir().unwrap();
        write_plugin_at(dir.path(), "", PLUGIN_AUTH, "audit");
        let err = discover_plugin(dir.path(), PLUGIN_AUTH).unwrap_err();
        assert!(err.contains("expected"), "{err}");
        assert!(err.contains("workflow"), "{err}");
    }

    #[test]
    fn enabled_plugin_requires_enabled_flag() {
        let dir = tempfile::tempdir().unwrap();
        write_plugin_at(dir.path(), "", PLUGIN_AUTH, "workflow");
        // installed but disabled → stub path.
        write_workspace(dir.path(), "[plugins.gcloud-auth]\nenabled = false\n");
        assert!(
            find_enabled_plugin(dir.path(), PLUGIN_AUTH)
                .unwrap()
                .is_none()
        );
        // enabled → discovered.
        write_workspace(dir.path(), "[plugins.gcloud-auth]\nenabled = true\n");
        let p = find_enabled_plugin(dir.path(), PLUGIN_AUTH)
            .unwrap()
            .unwrap();
        assert_eq!(p.name, PLUGIN_AUTH);
    }

    #[test]
    fn no_plugin_message_names_install_command() {
        let m = no_plugin_message(PLUGIN_AUTH);
        assert!(m.contains(PLUGIN_AUTH));
        assert!(m.contains("bwoc plugin install"));
        assert!(m.contains("bwoc plugin enable"));
    }

    // --- never-leak guardrails --------------------------------------------

    #[test]
    fn auth_shape_never_carries_token_values() {
        let ws = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        // Plant suspicious-looking secret-shaped strings; the probe must NOT
        // surface their values — only presence flags.
        std::fs::create_dir_all(ws.path().join(".bwoc/secrets")).unwrap();
        std::fs::write(
            ws.path().join(SA_REL),
            "{\"private_key\":\"-----BEGIN PRIVATE KEY-----super-secret\"}",
        )
        .unwrap();
        let env = getenv_from(HashMap::from([
            (ENV_ACCOUNT, "leak-me-not@example.com"),
            (ENV_IMPERSONATE, "super-secret-sa@example.com"),
        ]));
        let shape = probe_auth_shape(ws.path(), Some(home.path()), &env);
        let json = serde_json::to_string(&shape).unwrap();
        assert!(
            !json.contains("super-secret"),
            "shape leaked secret: {json}"
        );
        assert!(!json.contains("leak-me-not"), "shape leaked email: {json}");
        assert!(
            !json.contains("BEGIN PRIVATE KEY"),
            "shape leaked key: {json}"
        );
    }
}
