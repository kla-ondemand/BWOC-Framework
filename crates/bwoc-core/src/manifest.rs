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
    /// Inter-agent trust declaration per the Kalyāṇamitta 7 spec
    /// (`modules/agent-template/interconnect/trust.md`). Absent block
    /// means "no qualities declared, no qualities required" — the
    /// framework ships permissive by default; recipients opt in via
    /// `requiredTrust`. See `TrustBlock` for shape + the `Default`
    /// impl for how missing-field semantics resolve.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trust: Option<TrustBlock>,
    pub version: String,
}

/// Kalyāṇamitta 7 trust block. Two halves: `declared` (what this agent
/// claims about itself) and `required_trust` (what this agent demands
/// from peers that want to message it). They're independent — see
/// `interconnect/trust.md` §"Manifest Schema".
///
/// `schema_version` is currently 1. Future spec revisions may add
/// fields to `TrustDeclared`; per the spec, missing fields in declared
/// are treated as `false` (Anicca seam). `serde(default)` on each
/// boolean implements this — a v2 agent's manifest with extra fields
/// deserializes cleanly on v1; a v1 manifest deserializing on v2
/// silently gets `false` for unknown new fields.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrustBlock {
    /// Always `1` for this spec revision. Future spec bumps increment.
    /// Required field (no `default`) — a malformed manifest without
    /// it fails to load, which is the right escalation when trust
    /// semantics ride on the block being well-formed.
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    /// 7 booleans per the AN 7.36 (Mitta Sutta) canonical list. Missing
    /// fields → `false` per the spec.
    #[serde(default)]
    pub declared: TrustDeclared,
    /// Qualities required from peer senders. Empty vec ≡ no gating
    /// for this recipient. Names match the camelCase manifest keys
    /// in `TrustDeclared`.
    #[serde(
        rename = "requiredTrust",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub required_trust: Vec<String>,
}

impl Default for TrustBlock {
    fn default() -> Self {
        Self {
            schema_version: 1,
            declared: TrustDeclared::default(),
            required_trust: Vec::new(),
        }
    }
}

/// The 7 Kalyāṇamitta qualities as declared booleans. Each is
/// `#[serde(default)]` so missing fields deserialize as `false` —
/// implements the spec's Anicca seam: a v2 spec adding `mudu` doesn't
/// silently refuse v1 peers who never declared it. Names match the
/// camelCase manifest keys: piyo / garu / bhavaniyo / vatta /
/// vacanakkhamo / gambhira / noCatthana.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TrustDeclared {
    /// Piyo — pleasant to delegate to.
    #[serde(default)]
    pub piyo: bool,
    /// Garu — respectable in capability.
    #[serde(default)]
    pub garu: bool,
    /// Bhāvanīyo — helps us improve.
    #[serde(default)]
    pub bhavaniyo: bool,
    /// Vattā — speaks beneficial truth.
    #[serde(default)]
    pub vatta: bool,
    /// Vacanakkhamo — can take feedback.
    #[serde(default)]
    pub vacanakkhamo: bool,
    /// Gambhīrañca kathaṃ kattā — can explain depth.
    #[serde(default)]
    pub gambhira: bool,
    /// No caṭṭhāne niyojaye — does not lead astray.
    #[serde(rename = "noCatthana", default)]
    pub no_catthana: bool,
}

impl TrustDeclared {
    /// Read a quality by its camelCase manifest key. Unknown keys
    /// resolve to `false` — implements the spec's "missing fields in
    /// declared → false" rule (interconnect/trust.md §schemaVersion).
    pub fn has(&self, key: &str) -> bool {
        match key {
            "piyo" => self.piyo,
            "garu" => self.garu,
            "bhavaniyo" => self.bhavaniyo,
            "vatta" => self.vatta,
            "vacanakkhamo" => self.vacanakkhamo,
            "gambhira" => self.gambhira,
            "noCatthana" => self.no_catthana,
            _ => false,
        }
    }
}

impl TrustBlock {
    /// Given a peer's declaration, return the qualities this block's
    /// `required_trust` demands that the peer does NOT satisfy. Result
    /// preserves the order of `required_trust`. Empty result ≡ peer
    /// satisfies every required quality (no refusal).
    pub fn missing_in(&self, declared: &TrustDeclared) -> Vec<String> {
        self.required_trust
            .iter()
            .filter(|q| !declared.has(q))
            .cloned()
            .collect()
    }
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
            trust: None,
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
        // Trust block absent → no "trust" key in serialized output.
        assert!(!json.contains("\"trust\""));
    }

    // ---- TrustBlock tests ---------------------------------------------------

    /// Backward-compat: a manifest without a `trust` block deserializes
    /// fine with `trust = None`. This is the most important test — every
    /// existing agent's manifest predates the trust spec.
    #[test]
    fn trust_block_absent_is_none() {
        let json = r#"{
            "name": "demo", "agentId": "agent-demo", "agentRole": "x",
            "primaryModel": "m", "memoryPath": "memories/",
            "lintCmd": "true", "formatCmd": "true", "testCmd": "true",
            "buildCmd": "true", "version": "2.0"
        }"#;
        let m: Manifest = serde_json::from_str(json).unwrap();
        assert!(m.trust.is_none());
    }

    /// Full-block roundtrip — every quality declared, requiredTrust array.
    #[test]
    fn trust_block_full_roundtrip() {
        let mut m = sample();
        m.trust = Some(TrustBlock {
            schema_version: 1,
            declared: TrustDeclared {
                piyo: true,
                garu: false,
                bhavaniyo: true,
                vatta: true,
                vacanakkhamo: true,
                gambhira: false,
                no_catthana: true,
            },
            required_trust: vec!["vatta".into(), "noCatthana".into()],
        });
        let json = serde_json::to_string(&m).unwrap();
        // Wire format uses camelCase + the rename for noCatthana.
        assert!(json.contains("\"trust\""));
        assert!(json.contains("\"schemaVersion\":1"));
        assert!(json.contains("\"noCatthana\":true"));
        assert!(json.contains("\"requiredTrust\":[\"vatta\",\"noCatthana\"]"));
        // Roundtrip preserves every boolean.
        let back: Manifest = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    /// Missing-field rule (Anicca seam): a partial `declared` block
    /// with only 2 fields → other 5 are `false`. Critical because a
    /// v2 spec that adds quality `mudu` must not silently refuse all
    /// v1 manifests that never declared it.
    #[test]
    fn trust_declared_partial_missing_fields_are_false() {
        let json = r#"{ "piyo": true, "vatta": true }"#;
        let d: TrustDeclared = serde_json::from_str(json).unwrap();
        assert!(d.piyo);
        assert!(d.vatta);
        // The other 5 default to false.
        assert!(!d.garu);
        assert!(!d.bhavaniyo);
        assert!(!d.vacanakkhamo);
        assert!(!d.gambhira);
        assert!(!d.no_catthana);
    }

    /// `requiredTrust` empty array is the same as missing — both serialize
    /// out of the picture (`skip_serializing_if = Vec::is_empty`) and both
    /// deserialize back to the empty vec.
    #[test]
    fn required_trust_empty_skipped_on_serialize() {
        let block = TrustBlock::default();
        let json = serde_json::to_string(&block).unwrap();
        assert!(!json.contains("\"requiredTrust\""));
        let back: TrustBlock = serde_json::from_str(&json).unwrap();
        assert!(back.required_trust.is_empty());
    }

    /// Forward-compat: a v2 manifest that adds an unknown field to
    /// `TrustDeclared` deserializes cleanly on v1 (serde ignores unknown
    /// JSON keys by default). This is the other half of the Anicca seam.
    #[test]
    fn trust_declared_unknown_field_ignored() {
        let json = r#"{ "piyo": true, "mudu": true, "futureField": "anything" }"#;
        let d: TrustDeclared = serde_json::from_str(json).unwrap();
        assert!(d.piyo);
        // Unknown fields don't error or attach.
    }

    // ---- Refusal-helper tests (step 4) --------------------------------------

    #[test]
    fn has_returns_declared_value_for_known_keys() {
        let d = TrustDeclared {
            vatta: true,
            no_catthana: true,
            ..Default::default()
        };
        assert!(d.has("vatta"));
        assert!(d.has("noCatthana"));
        assert!(!d.has("piyo"));
    }

    #[test]
    fn has_returns_false_for_unknown_key() {
        let d = TrustDeclared {
            piyo: true,
            ..Default::default()
        };
        // Future spec quality not yet known — must be false, not panic.
        assert!(!d.has("mudu"));
        assert!(!d.has("")); // empty key
        assert!(!d.has("PIYO")); // case-sensitive — wrong case is unknown
    }

    #[test]
    fn missing_in_empty_required_returns_empty() {
        let block = TrustBlock::default();
        let declared = TrustDeclared::default();
        assert!(block.missing_in(&declared).is_empty());
    }

    #[test]
    fn missing_in_all_satisfied_returns_empty() {
        let block = TrustBlock {
            schema_version: 1,
            declared: TrustDeclared::default(),
            required_trust: vec!["vatta".into(), "noCatthana".into()],
        };
        let peer = TrustDeclared {
            vatta: true,
            no_catthana: true,
            ..Default::default()
        };
        assert!(block.missing_in(&peer).is_empty());
    }

    #[test]
    fn missing_in_partial_returns_only_missing() {
        let block = TrustBlock {
            schema_version: 1,
            declared: TrustDeclared::default(),
            required_trust: vec!["vatta".into(), "noCatthana".into(), "gambhira".into()],
        };
        let peer = TrustDeclared {
            vatta: true,
            // no_catthana: false → missing
            // gambhira: false → missing
            ..Default::default()
        };
        let missing = block.missing_in(&peer);
        assert_eq!(missing, vec!["noCatthana", "gambhira"]);
    }

    #[test]
    fn missing_in_preserves_required_order() {
        // Order in the required_trust array is the order reported back —
        // recipient's preferences drive the surfaced diagnostic.
        let block = TrustBlock {
            schema_version: 1,
            declared: TrustDeclared::default(),
            required_trust: vec!["gambhira".into(), "vatta".into(), "piyo".into()],
        };
        let peer = TrustDeclared::default(); // nothing declared
        assert_eq!(block.missing_in(&peer), vec!["gambhira", "vatta", "piyo"]);
    }

    #[test]
    fn missing_in_unknown_quality_is_always_missing() {
        // A recipient that requires a future-spec quality the sender's
        // v1 manifest doesn't know about → quality is missing (since
        // unknown → false). Forward-compat works as expected.
        let block = TrustBlock {
            schema_version: 1,
            declared: TrustDeclared::default(),
            required_trust: vec!["mudu".into()], // future-spec quality
        };
        let peer = TrustDeclared {
            piyo: true,
            vatta: true,
            ..Default::default()
        };
        assert_eq!(block.missing_in(&peer), vec!["mudu"]);
    }
}
