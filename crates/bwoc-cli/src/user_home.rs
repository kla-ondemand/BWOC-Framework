//! `~/.bwoc/` — the per-user, machine-level state directory.
//!
//! Phase 1 v2.0 minimum: ensure the directory + an empty `config.toml`
//! exist. The rest of the spec'd contents (memory/, workspaces.toml, logs/)
//! are created on-demand by the commands that need them (Mattaññutā —
//! don't create speculatively). See `docs/en/WORKSPACE.en.md` §"Central
//! Memory".

use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum UserHomeError {
    #[error(
        "could not determine user home directory ($HOME unset on Unix or %USERPROFILE% unset on Windows)"
    )]
    NoHome,
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

/// Returns the absolute path to `~/.bwoc/` without creating it.
pub fn bwoc_home() -> Result<PathBuf, UserHomeError> {
    let home = home_dir().ok_or(UserHomeError::NoHome)?;
    Ok(home.join(".bwoc"))
}

/// Ensure `~/.bwoc/` and `~/.bwoc/config.toml` exist. Idempotent and cheap
/// when they already do. Returns the resolved `~/.bwoc/` path.
pub fn ensure_initialized() -> Result<PathBuf, UserHomeError> {
    let root = bwoc_home()?;
    fs::create_dir_all(&root)?;

    let config = root.join("config.toml");
    if !config.exists() {
        fs::write(
            &config,
            "# bwoc user-level config (managed by you).\n\
             # See docs/en/WORKSPACE.en.md §\"Central Memory\" for the schema.\n",
        )?;
    }
    Ok(root)
}

/// Cross-platform home-directory lookup without pulling in `dirs`. `HOME` on
/// Unix, `USERPROFILE` on Windows. Returns `None` if the env var is unset.
fn home_dir() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(PathBuf::from)
    }
    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // `std::env::set_var` is not thread-safe; serialize the env-mutating tests.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    // These tests stub HOME to a tempdir to exercise the resolver. On
    // Windows the resolver reads %USERPROFILE% instead — the tempdir
    // override path doesn't apply, so cfg(unix)-only.
    #[cfg(unix)]
    #[test]
    fn ensure_initialized_creates_directory_and_config() {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = std::env::temp_dir().join(format!("bwoc-uh-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        // SAFETY: We hold ENV_LOCK; no other test in this crate touches HOME.
        unsafe {
            std::env::set_var("HOME", &tmp);
        }

        let root = ensure_initialized().unwrap();
        assert_eq!(root, tmp.join(".bwoc"));
        assert!(root.is_dir());
        assert!(root.join("config.toml").is_file());

        // Idempotent: a second call shouldn't error or overwrite.
        let second = ensure_initialized().unwrap();
        assert_eq!(second, root);

        let _ = fs::remove_dir_all(&tmp);
    }

    #[cfg(unix)]
    #[test]
    fn ensure_initialized_does_not_overwrite_existing_config() {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = std::env::temp_dir().join(format!("bwoc-uh-keep-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join(".bwoc")).unwrap();
        fs::write(
            tmp.join(".bwoc/config.toml"),
            "[defaults]\nbackend=\"agy\"\n",
        )
        .unwrap();

        unsafe {
            std::env::set_var("HOME", &tmp);
        }
        ensure_initialized().unwrap();
        let content = fs::read_to_string(tmp.join(".bwoc/config.toml")).unwrap();
        assert!(content.contains("agy"), "existing content preserved");

        let _ = fs::remove_dir_all(&tmp);
    }
}
