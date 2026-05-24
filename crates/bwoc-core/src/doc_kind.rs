//! Workspace document-kind registry.
//!
//! A `DocKind` describes a first-class document category that lives under a
//! well-known directory and follows the `YYYY-MM-DD_<slug>.md` naming
//! convention.  All kinds are committed to the repository (`committed: true`).
//!
//! The built-in lookup covers three kinds: `notes`, `retrospectives`, and
//! `research`.  One generic engine (`bwoc-cli::doc_cmd`) consumes the
//! descriptor — there are no per-kind code paths.
//!
//! # Extension point
//! // TODO(#12): workspace-declared custom kinds (via `.bwoc/doc-kinds.toml`)

/// Descriptor for a workspace document kind.
///
/// Two `DocKind` values are equal when they have the same `name`.
#[derive(Debug, Clone)]
pub struct DocKind {
    /// Short name used in CLI commands (e.g. `"notes"`).
    pub name: &'static str,
    /// Workspace-relative directory where documents of this kind are stored.
    pub dir: &'static str,
    /// Whether documents of this kind are tracked in version control.
    pub committed: bool,
    template_fn: fn() -> String,
}

impl PartialEq for DocKind {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for DocKind {}

impl DocKind {
    /// Return the starter template for a new document of this kind.
    pub fn template(&self) -> String {
        (self.template_fn)()
    }
}

// ---------------------------------------------------------------------------
// Templates
// ---------------------------------------------------------------------------

fn notes_template() -> String {
    // Mirrors the note skeleton in CLAUDE.md §"Implementation Logs".
    "# YYYY-MM-DD — <Title>\n\
     \n\
     <one-paragraph summary>\n\
     \n\
     ## What changed\n\
     \n\
     ## Decisions\n\
     \n\
     ## Alternatives considered\n\
     \n\
     ## Bugs surfaced and fixed\n\
     \n\
     ## Status / deferred\n\
     \n\
     ## Related (links)\n"
        .to_string()
}

fn retrospectives_template() -> String {
    // Paññā-3 structure from AGENTS.md §11 Self-Improvement.
    "# YYYY-MM-DD — <Sprint / Session Title>\n\
     \n\
     ## Sutamayā (what the data / docs say)\n\
     \n\
     ## Cintāmayā (synthesis — what pattern emerged)\n\
     \n\
     ## Bhāvanāmayā (action — feedback / memory saved)\n\
     \n\
     ## Metrics\n\
     \n\
     | Metric | Value |\n\
     |---|---|\n\
     | tasks_attempted | |\n\
     | tasks_completed | |\n\
     | gate_pass_rate  | |\n"
        .to_string()
}

fn research_template() -> String {
    "# YYYY-MM-DD — <Research Title>\n\
     \n\
     ## Question\n\
     \n\
     ## Scope\n\
     \n\
     ## Sources (Sutamayā)\n\
     \n\
     ## Findings\n\
     \n\
     ## Recommendation\n"
        .to_string()
}

// ---------------------------------------------------------------------------
// Built-in kinds
// ---------------------------------------------------------------------------

const NOTES: DocKind = DocKind {
    name: "notes",
    dir: "notes/",
    committed: true,
    template_fn: notes_template,
};

const RETROSPECTIVES: DocKind = DocKind {
    name: "retrospectives",
    dir: "retrospectives/",
    committed: true,
    template_fn: retrospectives_template,
};

const RESEARCH: DocKind = DocKind {
    name: "research",
    dir: "research/",
    committed: true,
    template_fn: research_template,
};

/// Look up a built-in `DocKind` by name.  Returns `None` for unknown names.
///
/// # Examples
/// ```
/// use bwoc_core::doc_kind::kind;
/// assert!(kind("notes").is_some());
/// assert!(kind("unknown").is_none());
/// ```
pub fn kind(name: &str) -> Option<DocKind> {
    match name {
        "notes" => Some(NOTES),
        "retrospectives" => Some(RETROSPECTIVES),
        "research" => Some(RESEARCH),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_kinds_resolve() {
        for name in ["notes", "retrospectives", "research"] {
            let k = kind(name).unwrap_or_else(|| panic!("kind '{name}' not found"));
            assert_eq!(k.name, name);
            assert!(k.committed);
            // Template must be non-empty and contain at least one `##` heading.
            let tmpl = k.template();
            assert!(!tmpl.is_empty(), "template for '{name}' is empty");
            assert!(tmpl.contains("##"), "template for '{name}' has no sections");
        }
    }

    #[test]
    fn unknown_kind_returns_none() {
        assert!(kind("unknown").is_none());
        assert!(kind("").is_none());
        assert!(kind("NOTES").is_none()); // case-sensitive
    }

    #[test]
    fn notes_dir() {
        assert_eq!(kind("notes").unwrap().dir, "notes/");
    }

    #[test]
    fn retrospectives_dir() {
        assert_eq!(kind("retrospectives").unwrap().dir, "retrospectives/");
    }

    #[test]
    fn research_dir() {
        assert_eq!(kind("research").unwrap().dir, "research/");
    }
}
