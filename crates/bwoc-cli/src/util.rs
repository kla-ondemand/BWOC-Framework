//! Cross-module helpers for `bwoc-cli`. Time helpers live in `bwoc-core::time`
//! since both `bwoc-cli` and `bwoc-agent` consume them.

use std::path::{Component, Path};

pub use bwoc_core::time::utc_now_iso8601;

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
