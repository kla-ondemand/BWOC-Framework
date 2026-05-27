//! Cross-module helpers for `bwoc-cli`. Time helpers live in `bwoc-core::time`
//! since both `bwoc-cli` and `bwoc-agent` consume them.

use std::path::{Component, Path};

pub use bwoc_core::time::utc_now_iso8601;

/// Reject a tar listing that contains members which would escape the
/// extraction directory. `listing` is the stdout of `tar -tzf`, one member
/// path per line. Returns `Err` naming the first offending member.
///
/// SECURITY (BWOC-38): the install paths extract untrusted archives. A crafted
/// tarball can carry members with `..` traversal components or absolute paths
/// that, on extraction, write outside the staged directory (tar-slip). Both
/// `skill install` and `plugin install` MUST call this on the archive listing
/// BEFORE running `tar -xzf`.
pub fn assert_safe_tar_listing(listing: &str) -> Result<(), String> {
    for raw in listing.lines() {
        let member = raw.trim_end_matches('\r');
        if member.is_empty() {
            continue;
        }
        assert_safe_tar_member(member)?;
    }
    Ok(())
}

fn assert_safe_tar_member(member: &str) -> Result<(), String> {
    // Absolute paths ignore the `-C <dir>` extraction root entirely.
    if member.starts_with('/') || member.starts_with('\\') {
        return Err(format!("unsafe tar member '{member}': absolute path"));
    }
    // `Path::components` normalizes `.` and collapses separators, so this also
    // catches forms like `a/../../etc/passwd`.
    for comp in Path::new(member).components() {
        match comp {
            Component::ParentDir => {
                return Err(format!(
                    "unsafe tar member '{member}': '..' path-traversal component"
                ));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!("unsafe tar member '{member}': absolute path"));
            }
            _ => {}
        }
    }
    Ok(())
}

/// Reject a plugin `[plugin].entry` value that could escape the plugin
/// directory and execute an arbitrary host binary (path-traversal RCE).
///
/// `bwoc audit run` spawns the entry via `Command::new(plugin_dir.join(entry))`.
/// `Path::join` makes an absolute `entry` (`/tmp/evil`) discard `plugin_dir`
/// entirely, and a `..` component (`../../../../tmp/evil`) climbs out of it —
/// either way an attacker-authored manifest runs an arbitrary program. A safe
/// entry is EITHER a bare program name resolved on `PATH`, OR a relative path
/// that stays contained within the plugin directory. This is the single source
/// of truth shared by the runtime guard (`audit.rs`) and the static manifest
/// check (`check.rs`) so the two cannot drift.
pub fn validate_plugin_entry(entry: &str) -> Result<(), String> {
    for component in Path::new(entry).components() {
        match component {
            Component::ParentDir => {
                return Err(format!(
                    "[plugin].entry '{entry}' contains a '..' component — \
                     entry must stay within the plugin directory"
                ));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!(
                    "[plugin].entry '{entry}' is an absolute path — entry must be a \
                     bare program name or a relative path contained in the plugin directory"
                ));
            }
            Component::CurDir | Component::Normal(_) => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_normal_members() {
        let listing = "pkg-1.0/\npkg-1.0/manifest.toml\npkg-1.0/SPEC.md\npkg-1.0/sub/dir/file";
        assert!(assert_safe_tar_listing(listing).is_ok());
    }

    #[test]
    fn accepts_leading_dot_slash() {
        // `./pkg/file` normalizes to `pkg/file` — no traversal.
        assert!(assert_safe_tar_listing("./pkg/file\n").is_ok());
    }

    #[test]
    fn rejects_parent_dir_member() {
        let err = assert_safe_tar_listing("pkg/../../etc/passwd\n").unwrap_err();
        assert!(err.contains("traversal"), "{err}");
    }

    #[test]
    fn rejects_bare_parent_dir() {
        assert!(assert_safe_tar_listing("../evil\n").is_err());
    }

    #[test]
    fn rejects_absolute_member() {
        let err = assert_safe_tar_listing("/etc/passwd\n").unwrap_err();
        assert!(err.contains("absolute"), "{err}");
    }

    #[test]
    fn rejects_backslash_absolute_member() {
        assert!(assert_safe_tar_listing("\\windows\\system32\n").is_err());
    }

    #[test]
    fn one_bad_member_among_good_fails() {
        let listing = "pkg/ok\npkg/also-ok\npkg/../../escape\npkg/more";
        assert!(assert_safe_tar_listing(listing).is_err());
    }

    #[test]
    fn entry_accepts_bare_name() {
        assert!(validate_plugin_entry("audit.sh").is_ok());
    }

    #[test]
    fn entry_accepts_contained_relative() {
        assert!(validate_plugin_entry("bin/audit.sh").is_ok());
    }

    #[test]
    fn entry_rejects_parent_traversal() {
        let err = validate_plugin_entry("../../../../tmp/evil").unwrap_err();
        assert!(err.contains(".."), "{err}");
    }

    #[test]
    fn entry_rejects_absolute() {
        let err = validate_plugin_entry("/tmp/evil").unwrap_err();
        assert!(err.contains("absolute"), "{err}");
    }
}
