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

    // 11. Trust evidence — Kalyāṇamitta 7. Each `true` declaration in
    //     config.manifest.json's `trust.declared` block must have the
    //     evidence documented in `interconnect/trust.md`. A claim without
    //     evidence is a violation. Skipped silently if no trust block.
    check_trust_evidence(target, &mut report);

    report
}

/// Verify the Kalyāṇamitta 7 evidence rules from
/// `modules/agent-template/interconnect/trust.md`. For each quality
/// the agent's manifest declares `true`, this checks the corresponding
/// structural evidence. `false` declarations are always valid (no
/// evidence needed). A missing `trust` block skips the check entirely.
fn check_trust_evidence(target: &Path, report: &mut AuditReport) {
    use bwoc_core::manifest::Manifest;
    let manifest_path = target.join("config.manifest.json");
    let Ok(m) = Manifest::load_from_path(&manifest_path) else {
        return; // no manifest or unparseable — handled by earlier check
    };
    let Some(trust) = m.trust.as_ref() else {
        return; // no trust block — nothing to verify
    };
    let d = &trust.declared;

    if d.piyo {
        check_piyo(target, report);
    }
    if d.garu {
        check_garu(target, report);
    }
    if d.bhavaniyo {
        check_bhavaniyo(target, report);
    }
    if d.vatta {
        check_vatta(target, report);
    }
    if d.vacanakkhamo {
        check_vacanakkhamo(target, report);
    }
    if d.gambhira {
        check_gambhira(target, report);
    }
    if d.no_catthana {
        check_no_catthana(target, report);
    }
}

/// Extract a section body from a Markdown doc. Looks for `## <heading>`
/// (case-insensitive), returns the lines between it and the next
/// same-level heading. Returns None if heading isn't found.
fn extract_section(content: &str, heading: &str) -> Option<String> {
    let lower_heading = heading.to_lowercase();
    let mut collecting = false;
    let mut body = String::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("## ") {
            if collecting {
                // Reached the next ## — stop.
                break;
            }
            if rest.trim().to_lowercase() == lower_heading {
                collecting = true;
                continue;
            }
        } else if collecting {
            body.push_str(line);
            body.push('\n');
        }
    }
    if collecting { Some(body) } else { None }
}

/// Section body is "filled in" if it contains at least one non-empty,
/// non-placeholder line. A line with only `{{placeholder}}` doesn't
/// count — that's an un-resolved scaffold, not content.
fn section_is_filled(body: &str) -> bool {
    body.lines().any(|l| {
        let t = l.trim();
        // Skip empty, blockquote, raw placeholder, and template labels —
        // anything else counts as filled content.
        !(t.is_empty()
            || t.starts_with('>')
            || (t.starts_with("{{") && t.ends_with("}}"))
            || t.starts_with("**Does:**")
            || t.starts_with("**Does not:**"))
    })
}

/// Piyo — persona/README.md "Scope" section filled with concrete content.
fn check_piyo(target: &Path, report: &mut AuditReport) {
    let p = target.join("persona/README.md");
    let Ok(content) = fs::read_to_string(&p) else {
        report.violations.push(
            "trust.piyo=true but persona/README.md is missing — scope cannot be declared".into(),
        );
        return;
    };
    match extract_section(&content, "Scope") {
        Some(body) if section_is_filled(&body) => report
            .passes
            .push("trust.piyo: Scope section filled".into()),
        Some(_) => report.violations.push(
            "trust.piyo=true but persona/README.md Scope section is empty / unresolved placeholder"
                .into(),
        ),
        None => report
            .violations
            .push("trust.piyo=true but persona/README.md has no Scope section".into()),
    }
}

/// Garu — at least one user-authored .md (not README.md) under
/// skills/ OR mindsets/. Respectability needs a demonstrated surface.
fn check_garu(target: &Path, report: &mut AuditReport) {
    let mut count = 0;
    for sub in &["skills", "mindsets"] {
        let dir = target.join(sub);
        if let Ok(read) = fs::read_dir(&dir) {
            for entry in read.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".md") && name != "README.md" {
                    count += 1;
                }
            }
        }
    }
    if count > 0 {
        report
            .passes
            .push(format!("trust.garu: {count} skill/mindset stub(s) present"));
    } else {
        report
            .violations
            .push("trust.garu=true but no .md files under skills/ or mindsets/ (only README.md doesn't count)".into());
    }
}

/// Bhāvanīyo — mindsets/ has a file whose name or content references
/// improvement / verification / yoniso / mattaññutā.
fn check_bhavaniyo(target: &Path, report: &mut AuditReport) {
    let dir = target.join("mindsets");
    let Ok(read) = fs::read_dir(&dir) else {
        report
            .violations
            .push("trust.bhavaniyo=true but mindsets/ directory is missing".into());
        return;
    };
    const KEYWORDS: &[&str] = &[
        "improve",
        "improvement",
        "verify",
        "verification",
        "yoniso",
        "manasikara",
        "manasikāra",
        "mattaññutā",
        "mattanutata",
        "right amount",
    ];
    let mut hit = false;
    for entry in read.flatten() {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if !name.ends_with(".md") || name == "readme.md" {
            continue;
        }
        // name match
        if KEYWORDS.iter().any(|k| name.contains(k)) {
            hit = true;
            break;
        }
        // content match
        if let Ok(c) = fs::read_to_string(entry.path()) {
            let lc = c.to_lowercase();
            if KEYWORDS.iter().any(|k| lc.contains(k)) {
                hit = true;
                break;
            }
        }
    }
    if hit {
        report.passes.push(
            "trust.bhavaniyo: mindsets/ references improvement/verify/yoniso/mattaññutā".into(),
        );
    } else {
        report.violations.push(
            "trust.bhavaniyo=true but no mindset references improvement/verify/yoniso/mattaññutā keywords".into(),
        );
    }
}

/// Vattā — persona/README.md "Anti-scope" / "Out-of-scope" section filled.
/// "Speaks beneficial truth" needs an honest declaration of what the
/// agent DOES NOT do.
fn check_vatta(target: &Path, report: &mut AuditReport) {
    let p = target.join("persona/README.md");
    let Ok(content) = fs::read_to_string(&p) else {
        report
            .violations
            .push("trust.vatta=true but persona/README.md is missing".into());
        return;
    };
    // Accept either "Anti-scope" or "Out-of-scope" heading.
    let body = extract_section(&content, "Anti-scope")
        .or_else(|| extract_section(&content, "Out-of-scope"))
        .or_else(|| extract_section(&content, "Scope")); // fallback: Scope section may include "Does not:" line
    match body {
        Some(b) if section_is_filled(&b) || b.to_lowercase().contains("does not:") => report
            .passes
            .push("trust.vatta: anti-scope declared".into()),
        Some(_) => report
            .violations
            .push("trust.vatta=true but anti-scope section is empty".into()),
        None => report
            .violations
            .push("trust.vatta=true but no Anti-scope / Out-of-scope section found".into()),
    }
}

/// Vacanakkhamo — agent has exercised inbox listening, OR has a
/// `interconnect/feedback.md` documenting how it handles feedback.
fn check_vacanakkhamo(target: &Path, report: &mut AuditReport) {
    let inbox = target.join(".bwoc/inbox.jsonl");
    let inbox_used = fs::metadata(&inbox).map(|m| m.len() > 0).unwrap_or(false);
    let feedback_doc = target.join("interconnect/feedback.md").is_file();
    if inbox_used || feedback_doc {
        report
            .passes
            .push("trust.vacanakkhamo: inbox used OR interconnect/feedback.md present".into());
    } else {
        report.violations.push(
            "trust.vacanakkhamo=true but inbox.jsonl is empty AND interconnect/feedback.md is missing".into(),
        );
    }
}

/// Gambhīra — at least one doc under the agent root is ≥50 lines AND
/// contains a `[[PHILOSOPHY.en.md]]` or `[[PHILOSOPHY.th.md]]` wikilink.
/// Pi's review: backlink-to-canon is harder to fake than keyword sniff.
fn check_gambhira(target: &Path, report: &mut AuditReport) {
    let mut found = None;
    visit_md_files(target, 0, &mut |path, content| {
        let line_count = content.lines().count();
        if line_count >= 50
            && (content.contains("[[PHILOSOPHY.en.md]]")
                || content.contains("[[PHILOSOPHY.th.md]]"))
        {
            found = Some(path.display().to_string());
        }
    });
    match found {
        Some(p) => report
            .passes
            .push(format!("trust.gambhira: depth doc anchored to PHILOSOPHY at {p}")),
        None => report.violations.push(
            "trust.gambhira=true but no doc has ≥50 lines AND a [[PHILOSOPHY.en.md]] wikilink — backlink to canon is required (anti-padding rule from Pi review)".into(),
        ),
    }
}

/// No-caṭṭhāne — persona Anti-scope section exists AND contains at
/// least one explicit "will refuse" entry (or similar refusal verb).
fn check_no_catthana(target: &Path, report: &mut AuditReport) {
    let p = target.join("persona/README.md");
    let Ok(content) = fs::read_to_string(&p) else {
        report
            .violations
            .push("trust.noCatthana=true but persona/README.md is missing".into());
        return;
    };
    let body = extract_section(&content, "Anti-scope")
        .or_else(|| extract_section(&content, "Out-of-scope"))
        .or_else(|| extract_section(&content, "Scope"));
    let Some(body) = body else {
        report.violations.push(
            "trust.noCatthana=true but no Anti-scope / Out-of-scope section in persona/README.md"
                .into(),
        );
        return;
    };
    let lc = body.to_lowercase();
    const REFUSAL_VERBS: &[&str] = &[
        "will refuse",
        "refuses",
        "will not",
        "does not",
        "never ",
        "must not",
        "refuse to",
        "decline",
    ];
    if REFUSAL_VERBS.iter().any(|v| lc.contains(v)) {
        report
            .passes
            .push("trust.noCatthana: anti-scope contains explicit refusal entry".into());
    } else {
        report.violations.push(
            "trust.noCatthana=true but anti-scope has no explicit refusal verb (will refuse / does not / never / must not / ...)".into(),
        );
    }
}

/// Walk all .md files under `target` (skipping `node_modules`, `target/`,
/// `.git/`, `.bwoc/`) up to a small depth, calling `visit` per file. Used
/// by `check_gambhira` to find the backlinked-to-canon evidence doc.
fn visit_md_files<F: FnMut(&Path, &str)>(target: &Path, depth: usize, visit: &mut F) {
    if depth > 4 {
        return;
    }
    let Ok(read) = fs::read_dir(target) else {
        return;
    };
    for entry in read.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if matches!(name.as_str(), ".git" | ".bwoc" | "node_modules" | "target") {
                continue;
            }
            visit_md_files(&path, depth + 1, visit);
        } else if name.ends_with(".md") {
            if let Ok(c) = fs::read_to_string(&path) {
                visit(&path, &c);
            }
        }
    }
}

/// Verify the backend entry file (`GEMINI.md`, `CODEX.md`, `KIMI.md`) is
/// a symlink pointing at `AGENTS.md`. Missing files are warnings, not
/// violations — an agent may not declare every backend. Symlinks
/// pointing elsewhere are violations.
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
pub fn run(target: &Path, lang: &str, json: bool) -> i32 {
    let report = audit(target);
    if json {
        let value = serde_json::json!({
            "target": report.target,
            "passes": report.passes,
            "warnings": report.warnings,
            "violations": report.violations,
            "summary": {
                "passes": report.passes.len(),
                "warnings": report.warnings.len(),
                "violations": report.violations.len(),
            },
        });
        match serde_json::to_string_pretty(&value) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("bwoc check: failed to serialize JSON: {e}");
                return 1;
            }
        }
    } else {
        let bundle = i18n::bundle_for(lang);
        print_report(&report, &bundle);
    }
    if report.violations.is_empty() { 0 } else { 1 }
}

/// Fleet-wide audit. Iterates the workspace's `agents.toml`, runs
/// `audit()` per agent, aggregates findings. Exit 0 only if every
/// agent passes; 1 if any has violations; 2 if the workspace itself
/// can't be located.
pub fn run_all(workspace_path: Option<&Path>, lang: &str, json: bool) -> i32 {
    use bwoc_core::workspace::AgentsRegistry;

    // Resolve workspace root: explicit path > BWOC_WORKSPACE env > ancestor walk.
    let root = match workspace_path {
        Some(p) => p.to_path_buf(),
        None => {
            if let Ok(env_path) = std::env::var("BWOC_WORKSPACE") {
                if !env_path.is_empty() {
                    std::path::PathBuf::from(env_path)
                } else {
                    let Some(p) = find_workspace_root_local() else {
                        eprintln!(
                            "bwoc check --all: no workspace found. Pass --workspace, set \
                             BWOC_WORKSPACE, or run from a workspace directory."
                        );
                        return 2;
                    };
                    p
                }
            } else {
                let Some(p) = find_workspace_root_local() else {
                    eprintln!(
                        "bwoc check --all: no workspace found. Pass --workspace, set \
                         BWOC_WORKSPACE, or run from a workspace directory."
                    );
                    return 2;
                };
                p
            }
        }
    };
    let registry = match AgentsRegistry::load(&root) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("bwoc check --all: failed to read agents.toml: {e}");
            return 1;
        }
    };
    if registry.agents.is_empty() {
        eprintln!(
            "bwoc check --all: no agents registered in {}. \
             Run `bwoc new <name>` to incarnate one.",
            root.display()
        );
        return 0;
    }

    let mut total_violations = 0u32;
    let mut total_warnings = 0u32;
    let mut total_passes = 0u32;
    let mut per_agent_reports: Vec<(String, AuditReport)> = Vec::new();
    for entry in &registry.agents {
        let path = root.join(&entry.path);
        let report = audit(&path);
        total_violations += report.violations.len() as u32;
        total_warnings += report.warnings.len() as u32;
        total_passes += report.passes.len() as u32;
        per_agent_reports.push((entry.id.clone(), report));
    }

    if json {
        let agents: Vec<serde_json::Value> = per_agent_reports
            .iter()
            .map(|(id, r)| {
                serde_json::json!({
                    "agent": id,
                    "target": r.target,
                    "passes": r.passes,
                    "warnings": r.warnings,
                    "violations": r.violations,
                    "summary": {
                        "passes": r.passes.len(),
                        "warnings": r.warnings.len(),
                        "violations": r.violations.len(),
                    },
                })
            })
            .collect();
        let value = serde_json::json!({
            "workspace": root.display().to_string(),
            "agents": agents,
            "summary": {
                "agents_checked": per_agent_reports.len(),
                "total_passes": total_passes,
                "total_warnings": total_warnings,
                "total_violations": total_violations,
            },
        });
        match serde_json::to_string_pretty(&value) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("bwoc check --all: failed to serialize JSON: {e}");
                return 1;
            }
        }
    } else {
        let bundle = i18n::bundle_for(lang);
        for (id, report) in &per_agent_reports {
            println!();
            println!("=== {id} ===");
            print_report(report, &bundle);
        }
        println!();
        println!(
            "=== Fleet summary ===\n  {} agent(s): {} pass, {} warn, {} violation(s)",
            per_agent_reports.len(),
            total_passes,
            total_warnings,
            total_violations,
        );
        println!();
    }

    if total_violations > 0 { 1 } else { 0 }
}

/// Local ancestor-walk helper (kept here to avoid pulling in
/// workspace.rs::find_workspace_root which threads Fluent bundles).
fn find_workspace_root_local() -> Option<std::path::PathBuf> {
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
