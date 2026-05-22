//! `bwoc-agent` — minimal runtime shipped with each incarnated BWOC agent.
//!
//! Phase 1 v2.0 DoD: read `config.manifest.json` from the current directory
//! and print structured liveness with the agent identity. The full task-loop
//! and control-socket responsibilities land in Phase 2.

use std::path::PathBuf;
use std::process::ExitCode;

use bwoc_core::manifest::Manifest;

mod i18n;

fn main() -> ExitCode {
    let lang = i18n::resolve_lang();
    let bundle = i18n::bundle_for(&lang);

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let manifest_path = cwd.join("config.manifest.json");

    if !manifest_path.exists() {
        let cwd_display = cwd.display().to_string();
        eprintln!(
            "{}",
            i18n::t_with(&bundle, "error-missing-manifest", &[("cwd", &cwd_display)])
        );
        return ExitCode::from(2);
    }

    let manifest = match Manifest::load_from_path(&manifest_path) {
        Ok(m) => m,
        Err(e) => {
            // Parse-error path stays English (thiserror localization deferred).
            eprintln!(
                "bwoc-agent: failed to load manifest at {}: {e}",
                manifest_path.display()
            );
            return ExitCode::from(1);
        }
    };

    println!("{}", liveness_banner(&manifest, &bundle));
    ExitCode::SUCCESS
}

/// Pure-data formatter for the liveness output. Kept separate from `main` so
/// it can be unit-tested without needing a real manifest on disk.
fn liveness_banner(
    m: &Manifest,
    bundle: &fluent_bundle::FluentBundle<fluent_bundle::FluentResource>,
) -> String {
    let mut lines = Vec::with_capacity(8);
    lines.push(i18n::t_with(
        bundle,
        "liveness-alive",
        &[("agent_id", m.agent_id.as_str())],
    ));
    lines.push(i18n::t_with(
        bundle,
        "liveness-role",
        &[("role", m.agent_role.as_str())],
    ));
    lines.push(i18n::t_with(
        bundle,
        "liveness-model",
        &[("model", m.primary_model.as_str())],
    ));
    if let Some(ref fb) = m.fallback_model {
        lines.push(i18n::t_with(
            bundle,
            "liveness-fallback",
            &[("fallback", fb.as_str())],
        ));
    }
    lines.push(i18n::t_with(
        bundle,
        "liveness-memory",
        &[("memory_path", m.memory_path.as_str())],
    ));
    lines.push(i18n::t_with(
        bundle,
        "liveness-version",
        &[("version", m.version.as_str())],
    ));
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Manifest {
        Manifest {
            name: "demo".into(),
            agent_id: "agent-demo".into(),
            agent_role: "demo role".into(),
            primary_model: "model-x".into(),
            fallback_model: Some("model-y".into()),
            memory_path: "memories/".into(),
            sessions_path: None,
            deep_memory_cmd: None,
            lint_cmd: "true".into(),
            format_cmd: "true".into(),
            test_cmd: "true".into(),
            build_cmd: "true".into(),
            worktree_base: None,
            version: "2.0".into(),
        }
    }

    #[test]
    fn banner_shows_required_fields_en() {
        let bundle = i18n::bundle_for("en");
        let b = liveness_banner(&sample(), &bundle);
        assert!(b.contains("I am alive: agent-demo"));
        assert!(b.contains("demo role"));
        assert!(b.contains("model-x"));
        assert!(b.contains("model-y"));
        assert!(b.contains("memories/"));
        assert!(b.contains("2.0"));
    }

    #[test]
    fn banner_shows_required_fields_th() {
        let bundle = i18n::bundle_for("th");
        let b = liveness_banner(&sample(), &bundle);
        assert!(b.contains("ฉันยังมีชีวิตอยู่: agent-demo"));
        assert!(b.contains("demo role"));
        assert!(b.contains("model-x"));
    }

    #[test]
    fn banner_omits_optional_fallback_when_none() {
        let bundle = i18n::bundle_for("en");
        let mut m = sample();
        m.fallback_model = None;
        let b = liveness_banner(&m, &bundle);
        assert!(b.contains("I am alive:"));
        assert!(!b.contains("fallback:"));
    }
}
