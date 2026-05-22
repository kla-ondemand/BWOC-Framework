//! `bwoc new` — incarnate a new agent from the template.
//!
//! Rust port of `modules/agent-template/scripts/incarnate.sh` with the
//! manifest-input behavior spec'd in `docs/en/INCARNATION.en.md` §"Setting
//! the Manifest". This iteration adds interactive TTY prompts for missing
//! required fields (non-TTY = fail-fast). Unix symlinks only (Windows
//! symlink handling deferred).

use std::collections::HashMap;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

use bwoc_core::manifest::Manifest;
use bwoc_core::workspace::{AgentEntry, AgentsRegistry};
use include_dir::{Dir, include_dir};

use crate::i18n;
use crate::spawn::Backend;
use crate::util::utc_now_iso8601;

/// Agent template embedded into the binary at compile time. Used as the
/// final fallback when no on-disk template is found, so `bwoc new` works
/// from any directory after a `cargo install`.
static EMBEDDED_TEMPLATE: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/../../modules/agent-template");

/// Arguments collected from the CLI for `bwoc new`. Required fields are
/// `Option<String>` because they can be filled by interactive prompt when
/// stdin is a TTY.
pub struct NewArgs {
    pub name: String,
    pub target: Option<PathBuf>,
    pub template: Option<PathBuf>,
    pub backend: Backend,
    pub lang: String,
    // Required at incarnation time; promptable.
    pub role: Option<String>,
    pub primary_model: Option<String>,
    pub lint_cmd: Option<String>,
    pub format_cmd: Option<String>,
    pub test_cmd: Option<String>,
    pub build_cmd: Option<String>,
    // Truly optional.
    pub fallback_model: Option<String>,
    pub memory_path: String,
    pub sessions_path: Option<String>,
    pub deep_memory_cmd: Option<String>,
    pub worktree_base: Option<String>,
}

/// All required fields resolved to concrete strings; ready for incarnate.
struct Resolved {
    name: String,
    target: PathBuf,
    template: PathBuf,
    backend: Backend,
    role: String,
    primary_model: String,
    fallback_model: Option<String>,
    memory_path: String,
    sessions_path: Option<String>,
    deep_memory_cmd: Option<String>,
    lint_cmd: String,
    format_cmd: String,
    test_cmd: String,
    build_cmd: String,
    worktree_base: Option<String>,
}

/// Entry point — returns the process exit code.
pub fn run(args: NewArgs) -> i32 {
    let bundle = i18n::bundle_for(&args.lang);
    match incarnate(args, &bundle) {
        Ok(report) => {
            print_report(&report, &bundle);
            0
        }
        Err(e) => {
            // Error path stays English (thiserror localization deferred).
            eprintln!("bwoc new: {e}");
            match e {
                NewError::InvalidName(_)
                | NewError::TargetExists(_)
                | NewError::TemplateNotFound
                | NewError::MissingFields(_) => 2,
                _ => 1,
            }
        }
    }
}

/// Successful incarnation result; printed at the end of `bwoc new`.
pub struct IncarnationReport {
    pub agent_id: String,
    pub target: PathBuf,
    pub symlinks: Vec<String>,
    /// Workspace root that the new agent was registered to, if any.
    pub registered_in: Option<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub enum NewError {
    #[error("invalid agent name '{0}' — must be lowercase kebab-case (a-z, 0-9, -)")]
    InvalidName(String),
    #[error("target path already exists: {0}")]
    TargetExists(PathBuf),
    #[error(
        "template not found — pass --template <path> or run from a directory under the framework"
    )]
    TemplateNotFound,
    #[error(
        "missing required field(s) and stdin is not a TTY (cannot prompt): {}",
        .0.join(", ")
    )]
    MissingFields(Vec<String>),
    #[error("agent id '{agent_id}' already registered in workspace {workspace}")]
    DuplicateRegistration {
        agent_id: String,
        workspace: PathBuf,
    },
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("manifest error: {0}")]
    Manifest(#[from] bwoc_core::manifest::ManifestError),
    #[error("workspace error: {0}")]
    Workspace(#[from] bwoc_core::workspace::WorkspaceError),
    #[error("symlink failure for {file}: {source}")]
    Symlink {
        file: String,
        #[source]
        source: io::Error,
    },
}

/// Perform the full incarnation atomically (best-effort — if a step fails
/// after copy, the partial target remains for the user to inspect).
pub fn incarnate(
    args: NewArgs,
    bundle: &fluent_bundle::FluentBundle<fluent_bundle::FluentResource>,
) -> Result<IncarnationReport, NewError> {
    validate_name(&args.name)?;
    let resolved = resolve(args, bundle)?;

    if resolved.target.exists() {
        return Err(NewError::TargetExists(resolved.target));
    }

    // 1. Copy template tree to target (skip .git, *.example.*).
    copy_tree(&resolved.template, &resolved.target)?;

    // 2. Create backend symlinks (replace any copied files first).
    let symlinks = create_symlinks(&resolved.target)?;

    // 3. Build and write the resolved manifest.
    let manifest = build_manifest(&resolved);
    let manifest_path = resolved.target.join("config.manifest.json");
    manifest.save_to_path(&manifest_path)?;

    // 4. Register with the nearest workspace if one is found.
    let registered_in = match register_in_workspace(&resolved.target, &manifest, resolved.backend) {
        Ok(ws) => ws,
        Err(e) => {
            // Registration is best-effort: log but don't fail the incarnation,
            // since the agent files are already on disk and valid.
            eprintln!("bwoc new: warning — agent created but workspace registration failed: {e}");
            None
        }
    };

    Ok(IncarnationReport {
        agent_id: manifest.agent_id,
        target: resolved.target,
        symlinks,
        registered_in,
    })
}

/// Walk up from `target.parent()` looking for `.bwoc/workspace.toml`. If
/// found, load `agents.toml`, append an entry for the new agent, and save.
/// Returns the workspace root on success; `None` if no workspace exists.
fn register_in_workspace(
    target: &Path,
    manifest: &Manifest,
    backend: Backend,
) -> Result<Option<PathBuf>, NewError> {
    let Some(workspace_root) = find_workspace(target) else {
        return Ok(None);
    };

    let mut registry = AgentsRegistry::load(&workspace_root)?;

    // Refuse to register a duplicate id; the user can `bwoc retire` first.
    if registry.agents.iter().any(|a| a.id == manifest.agent_id) {
        return Err(NewError::DuplicateRegistration {
            agent_id: manifest.agent_id.clone(),
            workspace: workspace_root,
        });
    }

    let rel_path = target
        .strip_prefix(&workspace_root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| target.display().to_string());

    registry.agents.push(AgentEntry {
        id: manifest.agent_id.clone(),
        path: rel_path,
        backend: backend.cli_name().to_string(),
        incarnated: utc_now_iso8601(),
        status: "active".to_string(),
    });
    registry.save(&workspace_root)?;
    Ok(Some(workspace_root))
}

/// Walk up from `target.parent()` looking for an ancestor directory that
/// contains `.bwoc/workspace.toml`. Returns the workspace root if found.
fn find_workspace(target: &Path) -> Option<PathBuf> {
    let mut cur = target.parent()?.to_path_buf();
    loop {
        if cur.join(".bwoc/workspace.toml").is_file() {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

fn validate_name(name: &str) -> Result<(), NewError> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        || name.starts_with('-')
        || name.ends_with('-')
    {
        return Err(NewError::InvalidName(name.to_string()));
    }
    Ok(())
}

fn resolve_template(explicit: Option<&Path>) -> Result<PathBuf, NewError> {
    // 1. Explicit --template flag wins.
    if let Some(p) = explicit {
        if !p.is_dir() {
            return Err(NewError::TemplateNotFound);
        }
        return Ok(p.to_path_buf());
    }
    // 2. `BWOC_TEMPLATE` env var (advanced user override).
    if let Some(env) = std::env::var_os("BWOC_TEMPLATE") {
        let p = PathBuf::from(env);
        if p.is_dir() && p.join("AGENTS.md").is_file() {
            return Ok(p);
        }
    }
    // 3. Ancestor walk for `modules/agent-template/` (framework-developer path).
    if let Ok(cwd) = std::env::current_dir() {
        let mut cur = cwd;
        loop {
            let candidate = cur.join("modules/agent-template");
            if candidate.is_dir() && candidate.join("AGENTS.md").is_file() {
                return Ok(candidate);
            }
            if !cur.pop() {
                break;
            }
        }
    }
    // 4. `~/.bwoc/template/` (pre-populated cache; survives across invocations).
    if let Some(home) = std::env::var_os("HOME") {
        let cached = PathBuf::from(home).join(".bwoc/template");
        if cached.is_dir() && cached.join("AGENTS.md").is_file() {
            return Ok(cached);
        }
    }
    // 5. Final fallback: extract the embedded template to a pid-tagged tmp dir.
    // This is what makes `bwoc new` work from any directory after `cargo install`.
    let tmp = std::env::temp_dir().join(format!("bwoc-template-{}", std::process::id()));
    if tmp.exists() {
        fs::remove_dir_all(&tmp)?;
    }
    fs::create_dir_all(&tmp)?;
    EMBEDDED_TEMPLATE.extract(&tmp).map_err(|e| {
        NewError::Io(io::Error::other(format!(
            "failed to extract embedded template: {e}"
        )))
    })?;
    Ok(tmp)
}

fn default_target(template: &Path, name: &str) -> PathBuf {
    // If the template lives under `modules/agent-template/` (framework-developer
    // workflow), drop the new agent next to it (so it lands in the framework
    // tree). Otherwise default to cwd + `agent-<name>` — the natural place
    // when running `bwoc new` from an arbitrary workspace.
    if template.ends_with("modules/agent-template") {
        if let Some(p) = template.parent().and_then(|p| p.parent()) {
            return p.join(format!("agent-{name}"));
        }
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    cwd.join(format!("agent-{name}"))
}

/// Fill in any missing required fields by interactive prompt (TTY) or fail
/// fast with the list of missing fields (non-TTY).
fn resolve(
    args: NewArgs,
    bundle: &fluent_bundle::FluentBundle<fluent_bundle::FluentResource>,
) -> Result<Resolved, NewError> {
    let template = resolve_template(args.template.as_deref())?;
    let target = args
        .target
        .clone()
        .unwrap_or_else(|| default_target(&template, &args.name));

    let descriptions = read_descriptions(&template)?;
    let tty = io::stdin().is_terminal();

    // First pass — collect missing fields without prompting, so we can fail
    // fast with the complete list when stdin is not a TTY.
    if !tty {
        let mut missing = Vec::new();
        if args.role.is_none() {
            missing.push("agentRole".to_string());
        }
        if args.primary_model.is_none() {
            missing.push("primaryModel".to_string());
        }
        if args.lint_cmd.is_none() {
            missing.push("lintCmd".to_string());
        }
        if args.format_cmd.is_none() {
            missing.push("formatCmd".to_string());
        }
        if args.test_cmd.is_none() {
            missing.push("testCmd".to_string());
        }
        if args.build_cmd.is_none() {
            missing.push("buildCmd".to_string());
        }
        if !missing.is_empty() {
            return Err(NewError::MissingFields(missing));
        }
    }

    let role = resolve_one(args.role, "agentRole", &descriptions, tty, bundle)?;
    let primary_model = resolve_one(
        args.primary_model,
        "primaryModel",
        &descriptions,
        tty,
        bundle,
    )?;
    let lint_cmd = resolve_one(args.lint_cmd, "lintCmd", &descriptions, tty, bundle)?;
    let format_cmd = resolve_one(args.format_cmd, "formatCmd", &descriptions, tty, bundle)?;
    let test_cmd = resolve_one(args.test_cmd, "testCmd", &descriptions, tty, bundle)?;
    let build_cmd = resolve_one(args.build_cmd, "buildCmd", &descriptions, tty, bundle)?;

    Ok(Resolved {
        name: args.name,
        target,
        template,
        backend: args.backend,
        role,
        primary_model,
        fallback_model: args.fallback_model,
        memory_path: args.memory_path,
        sessions_path: args.sessions_path,
        deep_memory_cmd: args.deep_memory_cmd,
        lint_cmd,
        format_cmd,
        test_cmd,
        build_cmd,
        worktree_base: args.worktree_base,
    })
}

/// Either return the already-supplied value, or interactively prompt
/// (assumed to only be called when `tty` is true after fail-fast guard).
fn resolve_one(
    cur: Option<String>,
    key: &str,
    descriptions: &HashMap<String, String>,
    tty: bool,
    bundle: &fluent_bundle::FluentBundle<fluent_bundle::FluentResource>,
) -> Result<String, NewError> {
    if let Some(v) = cur {
        return Ok(v);
    }
    if !tty {
        // Defensive: fail-fast guard should have caught this earlier.
        return Err(NewError::MissingFields(vec![key.to_string()]));
    }
    let desc = descriptions
        .get(key)
        .map(|s| s.as_str())
        .unwrap_or("required field");
    let prompt = i18n::t_with(bundle, "new-prompt-format", &[("key", key), ("desc", desc)]);
    let mut stdout = io::stdout();
    write!(stdout, "{prompt}")?;
    stdout.flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let trimmed = line.trim().to_string();
    if trimmed.is_empty() {
        return Err(NewError::MissingFields(vec![key.to_string()]));
    }
    Ok(trimmed)
}

/// Read the template's `config.manifest.json` and return a map of
/// `requiredConfig.<field>.description` strings. Errors propagate as
/// `NewError::Io` / `NewError::Manifest`.
fn read_descriptions(template: &Path) -> Result<HashMap<String, String>, NewError> {
    let manifest_path = template.join("config.manifest.json");
    let content = fs::read_to_string(&manifest_path)?;
    let v: serde_json::Value =
        serde_json::from_str(&content).map_err(bwoc_core::manifest::ManifestError::from)?;
    let mut out = HashMap::new();
    if let Some(rc) = v.get("requiredConfig").and_then(|x| x.as_object()) {
        for (key, schema) in rc {
            if let Some(desc) = schema.get("description").and_then(|d| d.as_str()) {
                out.insert(key.clone(), desc.to_string());
            }
        }
    }
    Ok(out)
}

/// Recursive copy. Skips `.git/` and `*.example.*` to match the shell script.
fn copy_tree(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == ".git" {
            continue;
        }
        if name_str.contains(".example.") {
            continue;
        }
        let from = entry.path();
        let to = dst.join(&name);
        let ft = entry.file_type()?;
        if ft.is_dir() {
            copy_tree(&from, &to)?;
        } else if ft.is_symlink() {
            // Preserve symlinks rather than dereferencing them.
            #[cfg(unix)]
            {
                let target = fs::read_link(&from)?;
                let _ = fs::remove_file(&to);
                std::os::unix::fs::symlink(&target, &to)?;
            }
            #[cfg(not(unix))]
            {
                // On non-Unix, copy the link target's content rather than the link.
                fs::copy(&from, &to)?;
            }
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// Force-create `CLAUDE.md`, `GEMINI.md`, `CODEX.md`, `KIMI.md` → `AGENTS.md`
/// in the target directory. Removes any pre-existing file/symlink first.
fn create_symlinks(target: &Path) -> Result<Vec<String>, NewError> {
    let backends = ["CLAUDE.md", "GEMINI.md", "CODEX.md", "KIMI.md"];
    let mut created = Vec::new();
    for backend in backends {
        let p = target.join(backend);
        let _ = fs::remove_file(&p);
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("AGENTS.md", &p).map_err(|e| NewError::Symlink {
                file: backend.to_string(),
                source: e,
            })?;
        }
        #[cfg(not(unix))]
        {
            // Windows symlinks require special privileges; defer per spec.
            return Err(NewError::Symlink {
                file: backend.to_string(),
                source: io::Error::new(
                    io::ErrorKind::Other,
                    "Windows symlink support deferred to Phase 2",
                ),
            });
        }
        created.push(format!("{backend} -> AGENTS.md"));
    }
    Ok(created)
}

fn build_manifest(r: &Resolved) -> Manifest {
    Manifest {
        name: r.name.clone(),
        agent_id: format!("agent-{}", r.name),
        agent_role: r.role.clone(),
        primary_model: r.primary_model.clone(),
        fallback_model: r.fallback_model.clone(),
        memory_path: r.memory_path.clone(),
        sessions_path: r.sessions_path.clone(),
        deep_memory_cmd: r.deep_memory_cmd.clone(),
        lint_cmd: r.lint_cmd.clone(),
        format_cmd: r.format_cmd.clone(),
        test_cmd: r.test_cmd.clone(),
        build_cmd: r.build_cmd.clone(),
        worktree_base: r.worktree_base.clone(),
        version: "2.0".to_string(),
    }
}

fn print_report(
    report: &IncarnationReport,
    bundle: &fluent_bundle::FluentBundle<fluent_bundle::FluentResource>,
) {
    let target_display = report.target.display().to_string();

    println!();
    println!(
        "{}",
        i18n::t_with(
            bundle,
            "new-report-incarnated",
            &[("agent_id", &report.agent_id)]
        )
    );
    println!(
        "{}",
        i18n::t_with(bundle, "new-report-target", &[("path", &target_display)])
    );
    println!();
    for s in &report.symlinks {
        // Symlink lines are data ("CLAUDE.md -> AGENTS.md"); no localization.
        println!("+ {s}");
    }
    println!();
    match &report.registered_in {
        Some(ws) => {
            let ws_display = ws.display().to_string();
            println!(
                "{}",
                i18n::t_with(bundle, "new-report-registered", &[("path", &ws_display)])
            );
        }
        None => {
            println!("{}", i18n::t(bundle, "new-report-not-registered"));
        }
    }
    println!();
    println!("{}", i18n::t(bundle, "new-report-next-steps-header"));
    println!(
        "  {}",
        i18n::t_with(
            bundle,
            "new-report-step-check",
            &[("path", &target_display)]
        )
    );
    println!("  {}", i18n::t(bundle, "new-report-step-edit-agents"));
    println!("  {}", i18n::t(bundle, "new-report-step-edit-persona"));
    println!("  {}", i18n::t(bundle, "new-report-step-git"));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args_with_role_only() -> NewArgs {
        NewArgs {
            name: "demo".to_string(),
            target: None,
            template: None,
            backend: Backend::Claude,
            lang: "en".to_string(),
            role: Some("tester".to_string()),
            primary_model: None,
            lint_cmd: None,
            format_cmd: None,
            test_cmd: None,
            build_cmd: None,
            fallback_model: None,
            memory_path: "memories/".to_string(),
            sessions_path: None,
            deep_memory_cmd: None,
            worktree_base: None,
        }
    }

    #[test]
    fn name_validation() {
        assert!(validate_name("agent-foo").is_ok());
        assert!(validate_name("foo").is_ok());
        assert!(validate_name("agent-foo-2").is_ok());
        assert!(validate_name("").is_err());
        assert!(validate_name("Agent").is_err());
        assert!(validate_name("agent_foo").is_err());
        assert!(validate_name("-foo").is_err());
        assert!(validate_name("foo-").is_err());
    }

    #[test]
    fn build_manifest_carries_required_fields() {
        let r = Resolved {
            name: "demo".to_string(),
            target: PathBuf::from("/tmp/demo"),
            template: PathBuf::from("/template"),
            backend: Backend::Claude,
            role: "tester".to_string(),
            primary_model: "model-x".to_string(),
            fallback_model: Some("model-y".to_string()),
            memory_path: "memories/".to_string(),
            sessions_path: None,
            deep_memory_cmd: None,
            lint_cmd: "true".to_string(),
            format_cmd: "true".to_string(),
            test_cmd: "true".to_string(),
            build_cmd: "true".to_string(),
            worktree_base: None,
        };
        let m = build_manifest(&r);
        assert_eq!(m.agent_id, "agent-demo");
        assert_eq!(m.primary_model, "model-x");
        assert_eq!(m.fallback_model.as_deref(), Some("model-y"));
    }

    #[test]
    fn find_workspace_walks_ancestors() {
        // Build a temp tree:
        //   /tmp/bwoc-new-test-<pid>/ws/.bwoc/workspace.toml
        //   /tmp/bwoc-new-test-<pid>/ws/agents/agent-foo  (the target)
        let mut root = std::env::temp_dir();
        root.push(format!("bwoc-new-find-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("ws/.bwoc")).unwrap();
        std::fs::write(
            root.join("ws/.bwoc/workspace.toml"),
            "[workspace]\nname=\"x\"\nversion=\"0.1.0\"\ncreated=\"x\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(root.join("ws/agents/agent-foo")).unwrap();

        let target = root.join("ws/agents/agent-foo");
        let ws = find_workspace(&target);
        assert_eq!(ws.as_deref(), Some(root.join("ws").as_path()));

        // Non-workspace path returns None.
        let lone = root.join("ws/agents/agent-foo/sub/dir");
        std::fs::create_dir_all(&lone).unwrap();
        // The .bwoc/ at /ws is still an ancestor → still finds it.
        assert!(find_workspace(&lone).is_some());

        let _ = std::fs::remove_dir_all(&root);
    }

    /// Construct a fresh non-TTY check: `resolve` must collect all missing
    /// required fields into `MissingFields` rather than blocking on stdin.
    /// We can't change is_terminal() in-process; instead exercise the
    /// fail-fast logic directly by calling `resolve` here — `cargo test`
    /// runs with stdin redirected from /dev/null, so `is_terminal()` is
    /// false during tests.
    #[test]
    fn non_tty_missing_required_fields_fails_fast() {
        let args = args_with_role_only();
        // Provide a template that has the config.manifest.json so
        // `resolve_template` succeeds when --template is None and we're
        // running from the framework root. cargo test sets CWD = the
        // crate directory by default, so use an explicit template path.
        let mut args = args;
        args.template =
            Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../modules/agent-template"));
        let bundle = i18n::bundle_for("en");
        match resolve(args, &bundle) {
            Err(NewError::MissingFields(fields)) => {
                // role is supplied; the other 5 are missing.
                assert!(!fields.contains(&"agentRole".to_string()));
                assert!(fields.contains(&"primaryModel".to_string()));
                assert!(fields.contains(&"lintCmd".to_string()));
                assert!(fields.contains(&"formatCmd".to_string()));
                assert!(fields.contains(&"testCmd".to_string()));
                assert!(fields.contains(&"buildCmd".to_string()));
            }
            Ok(_) => panic!("expected MissingFields error, got Ok"),
            Err(e) => panic!("expected MissingFields, got {e:?}"),
        }
    }

    #[test]
    fn descriptions_loaded_from_template() {
        let template =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../modules/agent-template");
        let descs = read_descriptions(&template).expect("template should be readable");
        assert!(descs.contains_key("agentRole"));
        assert!(descs.contains_key("primaryModel"));
        assert!(descs.contains_key("lintCmd"));
    }
}
