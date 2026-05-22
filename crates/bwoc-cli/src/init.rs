//! `bwoc init` — create a BWOC workspace at the given path.
//!
//! Phase 1 v2.0. Writes `.bwoc/workspace.toml` + `.bwoc/agents.toml` and the
//! configured `agents_dir`. Refuses if the workspace already exists
//! (idempotency); the caller can pass `--force` to overwrite.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use bwoc_core::workspace::{AgentsRegistry, Workspace, WorkspaceDefaults, WorkspaceMeta};

use crate::i18n;
use crate::util::utc_now_iso8601;

pub struct InitArgs {
    pub path: Option<PathBuf>,
    pub force: bool,
    pub lang: String,
}

#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("path does not exist: {0}")]
    PathMissing(PathBuf),
    #[error("workspace already exists at {0} (pass --force to overwrite workspace.toml)")]
    AlreadyExists(PathBuf),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("workspace error: {0}")]
    Workspace(#[from] bwoc_core::workspace::WorkspaceError),
}

pub fn run(args: InitArgs) -> i32 {
    let bundle = i18n::bundle_for(&args.lang);
    match init(args) {
        Ok(ws_root) => {
            let path = ws_root.display().to_string();
            println!();
            println!(
                "{}",
                i18n::t_with(&bundle, "init-success-title", &[("path", &path)])
            );
            println!("{}", i18n::t(&bundle, "init-created-workspace-toml"));
            println!("{}", i18n::t(&bundle, "init-created-agents-toml"));
            println!("{}", i18n::t(&bundle, "init-created-agents-dir"));
            println!("{}", i18n::t(&bundle, "init-created-projects-dir"));
            println!("{}", i18n::t(&bundle, "init-created-notes-dir"));
            println!();
            println!("{}", i18n::t(&bundle, "init-next-steps-header"));
            println!(
                "{}",
                i18n::t_with(&bundle, "init-next-step-validate", &[("path", &path)])
            );
            println!("{}", i18n::t(&bundle, "init-next-step-new"));
            0
        }
        Err(e) => {
            // Error messages stay in English for now — thiserror localization
            // is its own challenge (deferred per Mattaññutā).
            eprintln!("bwoc init: {e}");
            match e {
                InitError::PathMissing(_) | InitError::AlreadyExists(_) => 2,
                _ => 1,
            }
        }
    }
}

fn init(args: InitArgs) -> Result<PathBuf, InitError> {
    let root = args
        .path
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    if !root.is_dir() {
        return Err(InitError::PathMissing(root));
    }

    let workspace_toml = root.join(".bwoc/workspace.toml");
    if workspace_toml.exists() && !args.force {
        return Err(InitError::AlreadyExists(root));
    }

    let name = workspace_name(&root);
    let created = utc_now_iso8601();

    let ws = Workspace {
        workspace: WorkspaceMeta {
            name,
            version: "0.1.0".to_string(),
            created,
        },
        defaults: WorkspaceDefaults::default(),
    };
    ws.save(&root)?;

    let registry = AgentsRegistry::default();
    registry.save(&root)?;

    // Create the agents/ directory so the workspace passes a `validate` later.
    let agents_dir = root.join(&ws.defaults.agents_dir);
    fs::create_dir_all(&agents_dir)?;

    // Scaffold the standard workspace layout: empty dirs the user is
    // expected to populate. `projects/` for the work agents help build,
    // `notes/` for implementation logs per `NAMING.en.md` §category 10
    // (`YYYY-MM-DD_<title>.md`). Pre-existing dirs are kept (create_dir_all
    // is idempotent). Add more here if/when conventions emerge.
    for extra in WORKSPACE_EXTRAS {
        fs::create_dir_all(root.join(extra))?;
    }

    Ok(root)
}

/// Standard sub-directories scaffolded by `bwoc init` alongside `.bwoc/`
/// and the configured `agents_dir`. Add new entries here when a new
/// convention lands in `WORKSPACE.en.md`.
const WORKSPACE_EXTRAS: &[&str] = &["projects", "notes"];

fn workspace_name(root: &Path) -> String {
    root.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("workspace")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn fresh_dir(label: &str) -> PathBuf {
        let mut p = env::temp_dir();
        p.push(format!("bwoc-init-test-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn args(dir: &Path, force: bool) -> InitArgs {
        InitArgs {
            path: Some(dir.to_path_buf()),
            force,
            lang: "en".to_string(),
        }
    }

    #[test]
    fn init_creates_workspace_files() {
        let dir = fresh_dir("creates");
        let code = run(args(&dir, false));
        assert_eq!(code, 0);
        assert!(dir.join(".bwoc/workspace.toml").exists());
        assert!(dir.join(".bwoc/agents.toml").exists());
        assert!(dir.join("agents").is_dir());
        assert!(dir.join("projects").is_dir());
        assert!(dir.join("notes").is_dir());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn init_refuses_existing_workspace_without_force() {
        let dir = fresh_dir("refuses");
        assert_eq!(run(args(&dir, false)), 0);
        assert_eq!(run(args(&dir, false)), 2);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn init_force_overwrites() {
        let dir = fresh_dir("force");
        assert_eq!(run(args(&dir, false)), 0);
        assert_eq!(run(args(&dir, true)), 0);
        let _ = fs::remove_dir_all(&dir);
    }
}
