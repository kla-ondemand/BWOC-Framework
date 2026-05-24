//! Generic workspace document-kind engine.
//!
//! All three CLI aliases (`bwoc notes`, `bwoc retro`, `bwoc research`) dispatch
//! into `run()` here — one code path for all kinds.  The `DocKind` descriptor
//! from `bwoc-core::doc_kind` drives every decision.

use std::fs;
use std::path::{Path, PathBuf};

use bwoc_core::doc_kind::DocKind;
use bwoc_core::time::utc_now_iso8601;

/// Actions available on every document kind.
pub enum DocAction {
    /// Create a new document with the given title.
    New { title: String },
    /// List documents in the kind's directory (newest first).
    List,
    /// Print a document that matches the given date prefix or exact filename.
    View { name: String },
}

/// Run a document-kind action relative to `workspace_root`.
///
/// Returns a UNIX-style exit code: 0 = success, 1 = user error, 2 = I/O error.
pub fn run(kind: DocKind, action: DocAction, workspace_root: &Path) -> i32 {
    match action {
        DocAction::New { title } => cmd_new(kind, title, workspace_root),
        DocAction::List => cmd_list(kind, workspace_root),
        DocAction::View { name } => cmd_view(kind, name, workspace_root),
    }
}

// ---------------------------------------------------------------------------
// New
// ---------------------------------------------------------------------------

fn cmd_new(kind: DocKind, title: String, root: &Path) -> i32 {
    let slug = slugify(&title);
    let date = date_prefix();
    let filename = format!("{date}_{slug}.md");

    let dir = root.join(kind.dir.trim_end_matches('/'));
    if let Err(e) = fs::create_dir_all(&dir) {
        eprintln!(
            "bwoc {}: could not create directory {}: {e}",
            kind.name,
            dir.display()
        );
        return 2;
    }

    let path = dir.join(&filename);
    if path.exists() {
        eprintln!(
            "bwoc {}: file already exists — refusing to clobber: {}",
            kind.name,
            path.display()
        );
        return 1;
    }

    let body = kind.template();
    if let Err(e) = fs::write(&path, body) {
        eprintln!(
            "bwoc {}: failed to write {}: {e}",
            kind.name,
            path.display()
        );
        return 2;
    }

    println!("{}", path.display());
    0
}

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

fn cmd_list(kind: DocKind, root: &Path) -> i32 {
    let dir = root.join(kind.dir.trim_end_matches('/'));
    if !dir.exists() {
        println!("No {} documents yet.", kind.name);
        return 0;
    }

    let mut entries = match collect_md_files(&dir) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("bwoc {}: could not read {}: {e}", kind.name, dir.display());
            return 2;
        }
    };

    if entries.is_empty() {
        println!("No {} documents yet.", kind.name);
        return 0;
    }

    // Newest first: lexicographic descending (YYYY-MM-DD prefix guarantees this).
    entries.sort_by(|a, b| b.cmp(a));

    for name in &entries {
        println!("{name}");
    }
    0
}

// ---------------------------------------------------------------------------
// View
// ---------------------------------------------------------------------------

fn cmd_view(kind: DocKind, name: String, root: &Path) -> i32 {
    let dir = root.join(kind.dir.trim_end_matches('/'));

    // Resolve: try exact filename first, then prefix match.
    let path = resolve_name(&dir, &name);
    match path {
        Some(p) => match fs::read_to_string(&p) {
            Ok(content) => {
                print!("{content}");
                0
            }
            Err(e) => {
                eprintln!("bwoc {}: failed to read {}: {e}", kind.name, p.display());
                2
            }
        },
        None => {
            eprintln!(
                "bwoc {}: no document matching {:?} in {}",
                kind.name,
                name,
                dir.display()
            );
            1
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return `YYYY-MM-DD` derived from the current UTC timestamp.
fn date_prefix() -> String {
    // utc_now_iso8601() returns "YYYY-MM-DDTHH:MM:SSZ"
    utc_now_iso8601()[..10].to_string()
}

/// Turn a human title into a lowercase-hyphen slug suitable for filenames.
/// Collapses runs of non-alphanumeric characters into single hyphens and
/// strips leading/trailing hyphens.
fn slugify(title: &str) -> String {
    let mut slug = String::with_capacity(title.len());
    let mut prev_hyphen = false;
    for ch in title.chars() {
        if ch.is_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            prev_hyphen = false;
        } else if !prev_hyphen && !slug.is_empty() {
            slug.push('-');
            prev_hyphen = true;
        }
    }
    // Strip trailing hyphen that may result from trailing non-alnum chars.
    slug.trim_end_matches('-').to_string()
}

/// Collect all `*.md` filenames (stem + extension, not full paths) from `dir`.
fn collect_md_files(dir: &Path) -> Result<Vec<String>, std::io::Error> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                out.push(name.to_string());
            }
        }
    }
    Ok(out)
}

/// Try to find a file in `dir` matching `name`:
/// 1. Exact filename match (with or without `.md` extension).
/// 2. Any file whose name starts with `name` (date-prefix lookup).
fn resolve_name(dir: &Path, name: &str) -> Option<PathBuf> {
    // Normalise: strip trailing ".md" so both "2026-05-24_foo" and
    // "2026-05-24_foo.md" work as lookup keys.
    let stem = name.strip_suffix(".md").unwrap_or(name);

    let exact_md = dir.join(format!("{stem}.md"));
    if exact_md.exists() {
        return Some(exact_md);
    }

    // Prefix scan — pick the first lexicographic match.
    if let Ok(mut candidates) = collect_md_files(dir) {
        candidates.sort();
        for candidate in candidates {
            let cand_stem = candidate.strip_suffix(".md").unwrap_or(&candidate);
            if cand_stem.starts_with(stem) {
                return Some(dir.join(candidate));
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use bwoc_core::doc_kind::kind;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    // ---- kind lookup -------------------------------------------------------

    #[test]
    fn kind_lookup_notes() {
        assert!(kind("notes").is_some());
    }

    #[test]
    fn kind_lookup_retrospectives() {
        assert!(kind("retrospectives").is_some());
    }

    #[test]
    fn kind_lookup_research() {
        assert!(kind("research").is_some());
    }

    #[test]
    fn kind_lookup_unknown() {
        assert!(kind("unknown").is_none());
    }

    // ---- new ---------------------------------------------------------------

    #[test]
    fn new_creates_file_with_template() {
        let dir = tmp();
        let k = kind("notes").unwrap();
        let code = run(
            k,
            DocAction::New {
                title: "my first note".into(),
            },
            dir.path(),
        );
        assert_eq!(code, 0);

        let notes_dir = dir.path().join("notes");
        let files: Vec<_> = fs::read_dir(&notes_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(files.len(), 1, "expected exactly one file");

        let content = fs::read_to_string(files[0].path()).unwrap();
        // Template has section headings
        assert!(content.contains("##"));
    }

    #[test]
    fn new_path_contains_slug() {
        let dir = tmp();
        let k = kind("notes").unwrap();
        run(
            k,
            DocAction::New {
                title: "Workspace Design".into(),
            },
            dir.path(),
        );

        let notes_dir = dir.path().join("notes");
        let names: Vec<_> = fs::read_dir(&notes_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(names.len(), 1);
        assert!(
            names[0].contains("workspace-design"),
            "slug not found in {:?}",
            names[0]
        );
    }

    #[test]
    fn new_refuses_clobber() {
        let dir = tmp();
        let k = kind("notes").unwrap();

        // Create file once.
        let code1 = run(
            k.clone(),
            DocAction::New {
                title: "same title".into(),
            },
            dir.path(),
        );
        assert_eq!(code1, 0);

        // Attempt to create again with the same date → same filename → should fail.
        // We rely on the date being the same within a single test run.
        let code2 = run(
            k,
            DocAction::New {
                title: "same title".into(),
            },
            dir.path(),
        );
        assert_eq!(code2, 1, "expected clobber protection to return exit 1");
    }

    #[test]
    fn new_retro_creates_correct_dir() {
        let dir = tmp();
        let k = kind("retrospectives").unwrap();
        let code = run(
            k,
            DocAction::New {
                title: "sprint 1".into(),
            },
            dir.path(),
        );
        assert_eq!(code, 0);
        assert!(dir.path().join("retrospectives").exists());
    }

    #[test]
    fn new_research_creates_correct_dir() {
        let dir = tmp();
        let k = kind("research").unwrap();
        let code = run(
            k,
            DocAction::New {
                title: "llm caching".into(),
            },
            dir.path(),
        );
        assert_eq!(code, 0);
        assert!(dir.path().join("research").exists());
    }

    // ---- list --------------------------------------------------------------

    #[test]
    fn list_empty_dir_is_friendly() {
        let dir = tmp();
        let k = kind("notes").unwrap();
        let code = run(k, DocAction::List, dir.path());
        assert_eq!(code, 0);
    }

    #[test]
    fn list_returns_newest_first() {
        let dir = tmp();
        let notes_dir = dir.path().join("notes");
        fs::create_dir_all(&notes_dir).unwrap();

        // Write three files with explicit date prefixes to test ordering.
        for name in [
            "2026-01-01_alpha.md",
            "2026-03-15_gamma.md",
            "2026-02-10_beta.md",
        ] {
            fs::write(notes_dir.join(name), "# content").unwrap();
        }

        // Capture stdout by redirecting — instead, test the collect+sort helper directly.
        let mut files = collect_md_files(&notes_dir).unwrap();
        files.sort_by(|a, b| b.cmp(a));
        assert_eq!(files[0], "2026-03-15_gamma.md");
        assert_eq!(files[1], "2026-02-10_beta.md");
        assert_eq!(files[2], "2026-01-01_alpha.md");
    }

    // ---- view --------------------------------------------------------------

    #[test]
    fn view_not_found_returns_1() {
        let dir = tmp();
        let k = kind("notes").unwrap();
        let code = run(
            k,
            DocAction::View {
                name: "nonexistent".into(),
            },
            dir.path(),
        );
        assert_eq!(code, 1);
    }

    #[test]
    fn view_finds_by_prefix() {
        let dir = tmp();
        let notes_dir = dir.path().join("notes");
        fs::create_dir_all(&notes_dir).unwrap();
        let filename = "2026-05-24_doc-kinds.md";
        fs::write(notes_dir.join(filename), "# hello").unwrap();

        let k = kind("notes").unwrap();
        let code = run(
            k,
            DocAction::View {
                name: "2026-05-24".into(),
            },
            dir.path(),
        );
        assert_eq!(code, 0);
    }

    #[test]
    fn view_finds_by_exact_stem() {
        let dir = tmp();
        let notes_dir = dir.path().join("notes");
        fs::create_dir_all(&notes_dir).unwrap();
        fs::write(notes_dir.join("2026-05-24_exact.md"), "# exact").unwrap();

        let k = kind("notes").unwrap();
        // With .md extension
        let code = run(
            k.clone(),
            DocAction::View {
                name: "2026-05-24_exact.md".into(),
            },
            dir.path(),
        );
        assert_eq!(code, 0);
        // Without extension
        let code2 = run(
            k,
            DocAction::View {
                name: "2026-05-24_exact".into(),
            },
            dir.path(),
        );
        assert_eq!(code2, 0);
    }

    // ---- slugify -----------------------------------------------------------

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("My First Note!"), "my-first-note");
        assert_eq!(slugify("  spaces  "), "spaces");
        assert_eq!(slugify("A--B"), "a-b");
    }
}
