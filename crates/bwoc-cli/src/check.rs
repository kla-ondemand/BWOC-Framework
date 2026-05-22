//! `bwoc check` — backend-neutrality audit.
//!
//! Rust port of `modules/agent-template/scripts/check-agent-neutrality.sh`
//! with feature parity. Pure-data audit + separate printer for testability.

use std::fs;
use std::path::Path;

use crate::i18n;

/// Result of a single audit run. Each finding is a human-readable line.
pub struct AuditReport {
    pub target: String,
    pub passes: Vec<String>,
    pub warnings: Vec<String>,
    pub violations: Vec<String>,
}

const HARDCODED_MODELS: &[&str] = &[
    "claude-opus",
    "claude-sonnet",
    "claude-haiku",
    "claude-3",
    "claude-4",
    "gemini-2",
    "gemini-1",
    "gemini-pro",
    "gpt-4",
    "gpt-3",
    "o3-",
    "o4-",
    "codex-",
    "kimi-k2",
];

const HARDCODED_TOOLS: &[&str] = &["mempalace", "chromadb", "pinecone", "pgvector", "weaviate"];

const BACKEND_PHRASES: &[&str] = &[
    "Claude will",
    "Claude can",
    "Gemini will",
    "Gemini can",
    "Codex will",
    "Kimi will",
];

const REQUIRED_PLACEHOLDERS: &[&str] = &[
    "{{agentId}}",
    "{{memoryPath}}",
    "{{taskId}}",
    "{{deepMemoryCmd}}",
];

/// Run all neutrality checks against `target` and return the structured report.
pub fn audit(target: &Path) -> AuditReport {
    let mut report = AuditReport {
        target: target.display().to_string(),
        passes: Vec::new(),
        warnings: Vec::new(),
        violations: Vec::new(),
    };

    let agents_md = target.join("AGENTS.md");

    // 1. AGENTS.md exists and is a regular file
    if agents_md.is_file() {
        report.passes.push("AGENTS.md exists".to_string());
    } else {
        report
            .violations
            .push("AGENTS.md not found — this is the single source of truth".to_string());
    }

    // 2. Backend symlinks (GEMINI, CODEX, KIMI must symlink to AGENTS.md)
    for backend in &["GEMINI.md", "CODEX.md", "KIMI.md"] {
        let p = target.join(backend);
        check_symlink_to_agents(&p, backend, &mut report);
    }

    // 3. CLAUDE.md — can be a symlink or a standalone guidance file
    let claude_md = target.join("CLAUDE.md");
    if claude_md.is_symlink() {
        match fs::read_link(&claude_md) {
            Ok(t) if t == Path::new("AGENTS.md") => {
                report.passes.push("CLAUDE.md -> AGENTS.md".to_string());
            }
            Ok(t) => report.violations.push(format!(
                "CLAUDE.md points to '{}' instead of AGENTS.md",
                t.display()
            )),
            Err(_) => report
                .warnings
                .push("CLAUDE.md unreadable symlink".to_string()),
        }
    } else if claude_md.is_file() {
        report
            .passes
            .push("CLAUDE.md exists (standalone guidance file)".to_string());
    } else {
        report.warnings.push("CLAUDE.md missing".to_string());
    }

    // 4. config.manifest.json — exists and is valid JSON
    let manifest = target.join("config.manifest.json");
    if manifest.is_file() {
        match fs::read_to_string(&manifest) {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(_) => report
                    .passes
                    .push("config.manifest.json is valid JSON".to_string()),
                Err(_) => report
                    .violations
                    .push("config.manifest.json is not valid JSON".to_string()),
            },
            Err(_) => report
                .violations
                .push("config.manifest.json unreadable".to_string()),
        }
    } else {
        report
            .warnings
            .push("config.manifest.json missing (recommended for cloning readiness)".to_string());
    }

    // Content-based checks on AGENTS.md
    if let Ok(content) = fs::read_to_string(&agents_md) {
        // 5. Required placeholders present
        for ph in REQUIRED_PLACEHOLDERS {
            if content.contains(ph) {
                report.passes.push(format!("AGENTS.md contains {ph}"));
            } else {
                report
                    .warnings
                    .push(format!("AGENTS.md missing recommended placeholder {ph}"));
            }
        }

        // 6. No YAML frontmatter
        if content.starts_with("---\n") || content.starts_with("---\r\n") || content == "---" {
            report.violations.push(
                "AGENTS.md has YAML frontmatter — instruction files must use plain Markdown"
                    .to_string(),
            );
        } else {
            report
                .passes
                .push("AGENTS.md has no YAML frontmatter".to_string());
        }

        // 7. No wikilinks (Obsidian [[...]] syntax)
        if contains_wikilink(&content) {
            report.violations.push(
                "AGENTS.md contains wikilinks — instruction files must use plain Markdown"
                    .to_string(),
            );
        } else {
            report.passes.push("AGENTS.md has no wikilinks".to_string());
        }

        // 8. No hardcoded model IDs (case-insensitive substring match)
        let lower = content.to_lowercase();
        let mut model_ok = true;
        for model in HARDCODED_MODELS {
            if lower.contains(model) {
                report
                    .violations
                    .push(format!("AGENTS.md contains hardcoded model ID '{model}'"));
                model_ok = false;
            }
        }
        if model_ok {
            report
                .passes
                .push("No hardcoded model IDs in AGENTS.md".to_string());
        }

        // 9. No hardcoded tool names
        let mut tool_ok = true;
        for tool in HARDCODED_TOOLS {
            if lower.contains(tool) {
                report
                    .violations
                    .push(format!("AGENTS.md contains hardcoded tool name '{tool}'"));
                tool_ok = false;
            }
        }
        if tool_ok {
            report
                .passes
                .push("No hardcoded tool names in AGENTS.md".to_string());
        }

        // 10. No backend-specific phrasing
        let mut lang_ok = true;
        for phrase in BACKEND_PHRASES {
            if content.contains(phrase) {
                report.violations.push(format!(
                    "AGENTS.md contains backend-specific phrase '{phrase}'"
                ));
                lang_ok = false;
            }
        }
        if lang_ok {
            report
                .passes
                .push("No backend-specific language in AGENTS.md".to_string());
        }
    }

    report
}

fn check_symlink_to_agents(path: &Path, backend: &str, report: &mut AuditReport) {
    if path.is_symlink() {
        match fs::read_link(path) {
            Ok(t) if t == Path::new("AGENTS.md") => {
                report.passes.push(format!("{backend} -> AGENTS.md"));
            }
            Ok(t) => report.violations.push(format!(
                "{backend} points to '{}' instead of AGENTS.md",
                t.display()
            )),
            Err(_) => report
                .warnings
                .push(format!("{backend} unreadable symlink")),
        }
    } else {
        report.warnings.push(format!(
            "{backend} missing (create with: ln -s AGENTS.md {backend})"
        ));
    }
}

/// Conservative wikilink detector: look for `[[` followed by `]]` on the same
/// or nearby lines. Avoids a regex dep; deliberately matches the shell script's
/// `grep -qE '\[\[.*\]\]'` behavior (anything between `[[` and `]]` on one line).
fn contains_wikilink(content: &str) -> bool {
    for line in content.lines() {
        if let Some(open) = line.find("[[") {
            if line[open + 2..].contains("]]") {
                return true;
            }
        }
    }
    false
}

/// Print the report. Header / labels / summary are localized via Fluent;
/// finding descriptions stay English (rule-specific; deferred per Mattaññutā).
pub fn print_report(
    report: &AuditReport,
    bundle: &fluent_bundle::FluentBundle<fluent_bundle::FluentResource>,
) {
    let pass_label = i18n::t(bundle, "check-label-pass");
    let warn_label = i18n::t(bundle, "check-label-warn");
    let fail_label = i18n::t(bundle, "check-label-fail");
    let warnings_count = report.warnings.len().to_string();
    let violations_count = report.violations.len().to_string();

    println!();
    println!("{}", i18n::t(bundle, "check-header"));
    println!("============================");
    println!(
        "{}",
        i18n::t_with(bundle, "check-target", &[("target", &report.target)])
    );
    println!();
    for p in &report.passes {
        println!("{pass_label}  {p}");
    }
    for w in &report.warnings {
        println!("{warn_label}  {w}");
    }
    for v in &report.violations {
        println!("{fail_label}  {v}");
    }
    println!();
    println!("==============================");
    if !report.violations.is_empty() {
        println!(
            "{}",
            i18n::t_with(
                bundle,
                "check-summary-failure",
                &[
                    ("violations", &violations_count),
                    ("warnings", &warnings_count),
                ],
            )
        );
        println!("{}", i18n::t(bundle, "check-summary-failure-tail"));
    } else {
        println!(
            "{}",
            i18n::t_with(
                bundle,
                "check-summary-success",
                &[("warnings", &warnings_count)],
            )
        );
        println!("{}", i18n::t(bundle, "check-summary-success-tail"));
    }
}

/// Entry point. Returns the process exit code (0 = ok, 1 = violations).
pub fn run(target: &Path, lang: &str) -> i32 {
    let bundle = i18n::bundle_for(lang);
    let report = audit(target);
    print_report(&report, &bundle);
    if report.violations.is_empty() { 0 } else { 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wikilink_detection() {
        assert!(contains_wikilink("see [[neutrality|Neutrality]]"));
        assert!(contains_wikilink("[[plain]]"));
        assert!(!contains_wikilink("plain markdown link [text](url)"));
        assert!(!contains_wikilink("[[ on one line\n]] on another"));
        assert!(!contains_wikilink(""));
    }

    #[test]
    fn audit_missing_directory_reports_violation() {
        let report = audit(Path::new("/nonexistent/path/that/does/not/exist"));
        assert!(
            report
                .violations
                .iter()
                .any(|v| v.contains("AGENTS.md not found"))
        );
    }
}
