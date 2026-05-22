//! `config.manifest.json` types and I/O for an incarnated agent.
//!
//! The TEMPLATE's `config.manifest.json` is a schema document (lists
//! `requiredConfig` fields with type/description/default). An INCARNATED
//! agent's `config.manifest.json` is a flat resolved document with the
//! agent's concrete values. This module models the resolved form.

use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Resolved agent manifest. Every required field carries the value the
/// agent author supplied at incarnation time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    pub name: String,
    #[serde(rename = "agentId")]
    pub agent_id: String,
    #[serde(rename = "agentRole")]
    pub agent_role: String,
    #[serde(rename = "primaryModel")]
    pub primary_model: String,
    #[serde(rename = "fallbackModel", skip_serializing_if = "Option::is_none")]
    pub fallback_model: Option<String>,
    #[serde(rename = "memoryPath")]
    pub memory_path: String,
    #[serde(rename = "sessionsPath", skip_serializing_if = "Option::is_none")]
    pub sessions_path: Option<String>,
    #[serde(rename = "deepMemoryCmd", skip_serializing_if = "Option::is_none")]
    pub deep_memory_cmd: Option<String>,
    #[serde(rename = "lintCmd")]
    pub lint_cmd: String,
    #[serde(rename = "formatCmd")]
    pub format_cmd: String,
    #[serde(rename = "testCmd")]
    pub test_cmd: String,
    #[serde(rename = "buildCmd")]
    pub build_cmd: String,
    #[serde(rename = "worktreeBase", skip_serializing_if = "Option::is_none")]
    pub worktree_base: Option<String>,
    /// One-line description of what the agent DOES. Fills `{{scopeDescription}}`
    /// in AGENTS.md and persona/README.md at incarnation time.
    #[serde(rename = "scopeDescription", skip_serializing_if = "Option::is_none")]
    pub scope_description: Option<String>,
    /// One-line description of what the agent DOES NOT do. Fills
    /// `{{outOfScope}}` in AGENTS.md and persona/README.md.
    #[serde(rename = "outOfScope", skip_serializing_if = "Option::is_none")]
    pub out_of_scope: Option<String>,
    pub version: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("io error reading manifest: {0}")]
    Io(#[from] io::Error),
    #[error("invalid JSON in manifest: {0}")]
    Json(#[from] serde_json::Error),
}

impl Manifest {
    /// Parse a manifest from a JSON file on disk.
    pub fn load_from_path(path: &Path) -> Result<Self, ManifestError> {
        let content = fs::read_to_string(path)?;
        let m = serde_json::from_str(&content)?;
        Ok(m)
    }

    /// Serialize the manifest as pretty JSON and write to `path`.
    pub fn save_to_path(&self, path: &Path) -> Result<(), ManifestError> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, format!("{json}\n"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Manifest {
        Manifest {
            name: "demo".into(),
            agent_id: "agent-demo".into(),
            agent_role: "demo agent".into(),
            primary_model: "model-x".into(),
            fallback_model: None,
            memory_path: "memories/".into(),
            sessions_path: None,
            deep_memory_cmd: None,
            lint_cmd: "true".into(),
            format_cmd: "true".into(),
            test_cmd: "true".into(),
            build_cmd: "true".into(),
            worktree_base: None,
            scope_description: None,
            out_of_scope: None,
            version: "2.0".into(),
        }
    }

    #[test]
    fn roundtrip_json() {
        let m = sample();
        let json = serde_json::to_string(&m).unwrap();
        let back: Manifest = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn camel_case_keys() {
        let m = sample();
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.contains("\"agentId\""));
        assert!(json.contains("\"primaryModel\""));
        assert!(json.contains("\"lintCmd\""));
        // Optional none fields skipped
        assert!(!json.contains("\"fallbackModel\""));
    }
}
