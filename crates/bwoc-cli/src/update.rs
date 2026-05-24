//! `bwoc update [--check]` — release-drift detection (read-only).
//!
//! ## Design
//!
//! The binary's own release identity is embedded at compile time via
//! `option_env!("BWOC_RELEASE_CALVER")`. Released binaries set this to the
//! Git tag (e.g. `v2026.5.24-0`). Dev/source builds leave it unset.
//!
//! `--check` fetches the latest GitHub release tag and compares CalVer tuples.
//! Plain `bwoc update` detects the install method and delegates the upgrade to
//! the package manager (brew / cargo) or points a raw binary at the release
//! page — it never self-replaces the running binary (that path is deferred).
//!
//! ## Fetch strategy
//!
//! 1. Primary: `gh release view --json tagName -q .tagName`
//! 2. Fallback: `curl -s https://api.github.com/repos/bemindlabs/BWOC-Framework/releases/latest`
//!    parsed with `serde_json`.
//!
//! Both are shells-out — no HTTP client dep added. `ShellRunner` is a small
//! stdout-capture seam for offline unit-testability. It is intentionally
//! distinct from `run.rs`'s `CommandRunner` (which adds cwd / timeout / rich
//! `RunError` handling for launching agent processes) — these are different
//! needs, not one abstraction, so they are kept separate rather than merged.

use serde_json::Value;

// ── CalVer types ──────────────────────────────────────────────────────────────

/// Parsed form of `vYYYY.M.D-<patch>`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CalVer {
    pub year: u32,
    pub month: u32,
    pub day: u32,
    pub patch: u32,
}

impl CalVer {
    /// Parse `vYYYY.M.D-<patch>` (leading `v` optional).
    /// Returns `None` on any parse failure.
    pub fn parse(s: &str) -> Option<Self> {
        // Strip optional leading 'v'.
        let s = s.strip_prefix('v').unwrap_or(s);
        // Format: YYYY.M.D-patch  (M and D are not zero-padded in the spec)
        let (date_part, patch_str) = s.split_once('-')?;
        let mut parts = date_part.splitn(3, '.');
        let year: u32 = parts.next()?.parse().ok()?;
        let month: u32 = parts.next()?.parse().ok()?;
        let day: u32 = parts.next()?.parse().ok()?;
        let patch: u32 = patch_str.parse().ok()?;
        Some(Self {
            year,
            month,
            day,
            patch,
        })
    }

    /// Canonical string form: `vYYYY.M.D-patch`.
    pub fn to_tag(&self) -> String {
        format!("v{}.{}.{}-{}", self.year, self.month, self.day, self.patch)
    }
}

// ── ShellRunner seam (stdout-capture; distinct from run.rs's CommandRunner) ──

/// Result of one shelled-out command.
pub struct ShellOutcome {
    pub exit_code: i32,
    pub stdout: String,
}

/// Abstraction over shell-out so tests inject a mock.
pub trait ShellRunner {
    fn run(&self, program: &str, args: &[&str]) -> ShellOutcome;
}

/// Production runner: forks the real process, captures stdout.
pub struct ProcessShellRunner;

impl ShellRunner for ProcessShellRunner {
    fn run(&self, program: &str, args: &[&str]) -> ShellOutcome {
        use std::process::{Command, Stdio};
        let result = Command::new(program)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();
        match result {
            Ok(out) => ShellOutcome {
                exit_code: out.status.code().unwrap_or(1),
                stdout: String::from_utf8_lossy(&out.stdout).trim().to_string(),
            },
            Err(_) => ShellOutcome {
                exit_code: 127,
                stdout: String::new(),
            },
        }
    }
}

// ── Fetch latest tag ──────────────────────────────────────────────────────────

const GITHUB_REPO: &str = "bemindlabs/BWOC-Framework";
const GITHUB_API_URL: &str =
    "https://api.github.com/repos/bemindlabs/BWOC-Framework/releases/latest";

/// Fetch the latest release tag. Tries `gh` first; falls back to `curl` + JSON.
/// Returns `None` if both fail or return empty.
pub fn fetch_latest_tag(runner: &dyn ShellRunner) -> Option<String> {
    // Primary: gh CLI
    let gh = runner.run(
        "gh",
        &[
            "release",
            "view",
            "--repo",
            GITHUB_REPO,
            "--json",
            "tagName",
            "-q",
            ".tagName",
        ],
    );
    if gh.exit_code == 0 && !gh.stdout.is_empty() {
        return Some(gh.stdout.trim().to_string());
    }

    // Fallback: curl + serde_json parse
    let curl = runner.run("curl", &["-s", GITHUB_API_URL]);
    if curl.exit_code == 0 && !curl.stdout.is_empty() {
        if let Ok(v) = serde_json::from_str::<Value>(&curl.stdout) {
            if let Some(tag) = v.get("tag_name").and_then(|t| t.as_str()) {
                let tag = tag.trim().to_string();
                if !tag.is_empty() {
                    return Some(tag);
                }
            }
        }
    }

    None
}

// ── Check result ──────────────────────────────────────────────────────────────

/// Outcome of the `--check` comparison.
#[derive(Debug, PartialEq, Eq)]
pub enum CheckResult {
    /// Binary CalVer == latest.
    UpToDate { tag: String },
    /// Binary CalVer < latest.
    UpdateAvailable { current: String, latest: String },
    /// Binary CalVer > latest (unusual; dev build tagged ahead).
    AheadOfLatest { current: String, latest: String },
    /// `BWOC_RELEASE_CALVER` not set — source/dev build.
    SourceBuild { latest: String },
    /// Latest tag could not be fetched.
    FetchFailed,
    /// Latest tag was fetched but is malformed.
    MalformedLatestTag { raw: String },
    /// The embedded CalVer is malformed (unusual; build misconfigured).
    MalformedCurrentTag { raw: String, latest: String },
}

/// Run the `--check` comparison. Returns a `CheckResult` and the integer exit
/// code: `0` = up-to-date, ahead-of-latest, or source build; `1` = update
/// available; `2` = error (fetch failed or a malformed tag).
pub fn check(runner: &dyn ShellRunner) -> (CheckResult, i32) {
    let latest_raw = match fetch_latest_tag(runner) {
        Some(t) => t,
        None => return (CheckResult::FetchFailed, 2),
    };

    let latest_ver = match CalVer::parse(&latest_raw) {
        Some(v) => v,
        None => {
            return (
                CheckResult::MalformedLatestTag {
                    raw: latest_raw.clone(),
                },
                2,
            );
        }
    };

    // The binary's embedded release tag (set at compile time by release.yml).
    match option_env!("BWOC_RELEASE_CALVER") {
        None => (
            CheckResult::SourceBuild {
                latest: latest_ver.to_tag(),
            },
            0,
        ),
        Some(current_raw) => match CalVer::parse(current_raw) {
            None => (
                CheckResult::MalformedCurrentTag {
                    raw: current_raw.to_string(),
                    latest: latest_ver.to_tag(),
                },
                2,
            ),
            Some(current_ver) => {
                use std::cmp::Ordering;
                match current_ver.cmp(&latest_ver) {
                    Ordering::Equal => (
                        CheckResult::UpToDate {
                            tag: latest_ver.to_tag(),
                        },
                        0,
                    ),
                    Ordering::Less => (
                        CheckResult::UpdateAvailable {
                            current: current_ver.to_tag(),
                            latest: latest_ver.to_tag(),
                        },
                        1,
                    ),
                    Ordering::Greater => (
                        CheckResult::AheadOfLatest {
                            current: current_ver.to_tag(),
                            latest: latest_ver.to_tag(),
                        },
                        0,
                    ),
                }
            }
        },
    }
}

// ── Public args struct ────────────────────────────────────────────────────────

pub struct UpdateArgs {
    /// When true, perform the read-only release-drift check and exit.
    pub check: bool,
    /// When applying (no `--check`), actually execute the delegated upgrade
    /// command (e.g. `brew upgrade bwoc`) instead of only printing it.
    /// Self-replacing a raw binary is never done — that path is deferred.
    pub run: bool,
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Entry point called from `main.rs` — returns process exit code.
pub fn run(args: UpdateArgs) -> i32 {
    run_with(args, &ProcessShellRunner)
}

/// Testable entry accepting a `ShellRunner` impl.
pub fn run_with(args: UpdateArgs, runner: &dyn ShellRunner) -> i32 {
    if args.check {
        let (result, code) = check(runner);
        print_check_result(&result);
        return code;
    }
    apply(args.run, runner, &detect_install_method())
}

fn print_check_result(result: &CheckResult) {
    match result {
        CheckResult::UpToDate { tag } => {
            println!("bwoc update: up to date ({tag})");
        }
        CheckResult::UpdateAvailable { current, latest } => {
            println!("bwoc update: update available: {current} → {latest}");
            println!("  Download: https://github.com/{GITHUB_REPO}/releases/tag/{latest}");
        }
        CheckResult::AheadOfLatest { current, latest } => {
            println!(
                "bwoc update: ahead of latest release (dev build: {current}, latest: {latest})"
            );
        }
        CheckResult::SourceBuild { latest } => {
            println!("bwoc update: running a source build; latest release is {latest}");
        }
        CheckResult::FetchFailed => {
            eprintln!(
                "bwoc update: could not fetch latest release tag \
                 (requires 'gh' CLI or 'curl' on PATH with network access)"
            );
        }
        CheckResult::MalformedLatestTag { raw } => {
            eprintln!("bwoc update: latest release tag '{raw}' is not valid CalVer (vYYYY.M.D-N)");
        }
        CheckResult::MalformedCurrentTag { raw, latest } => {
            eprintln!(
                "bwoc update: embedded release tag '{raw}' is not valid CalVer; \
                 latest is {latest}"
            );
        }
    }
}

// ── Install method + delegate-only apply ───────────────────────────────────────

/// How the running `bwoc` binary was installed — picks the upgrade route.
#[derive(Debug, PartialEq, Eq)]
pub enum InstallMethod {
    /// Homebrew (Cellar prefix). Upgrade by delegating to `brew`.
    Homebrew,
    /// `cargo install` (under `~/.cargo/bin`). Upgrade by delegating to `cargo`.
    Cargo,
    /// A raw binary on `PATH`, managed by no package manager.
    Raw,
}

/// Classify an executable path into an [`InstallMethod`]. Pure (takes the path)
/// so it is unit-testable without touching the filesystem. `current_exe()`
/// resolves symlinks, so a Homebrew install surfaces its `…/Cellar/…` path.
pub fn classify_install_path(exe: &str) -> InstallMethod {
    if exe.contains("/Cellar/")
        || exe.starts_with("/opt/homebrew/")
        || exe.starts_with("/home/linuxbrew/")
    {
        InstallMethod::Homebrew
    } else if exe.contains("/.cargo/bin/") {
        InstallMethod::Cargo
    } else {
        InstallMethod::Raw
    }
}

/// Detect how the current process's binary was installed.
fn detect_install_method() -> InstallMethod {
    std::env::current_exe()
        .map(|p| classify_install_path(&p.to_string_lossy()))
        .unwrap_or(InstallMethod::Raw)
}

/// The decided apply action — pure, derived from the check result + install
/// method, so the routing is fully unit-testable.
#[derive(Debug, PartialEq, Eq)]
pub enum ApplyAction {
    /// Nothing to do — already at (or ahead of) the latest release.
    AlreadyCurrent { tag: String },
    /// Source/dev build — "upgrade" means pull + rebuild.
    SourceRebuild,
    /// Delegate to a package manager (program + args).
    Delegate { program: String, args: Vec<String> },
    /// Raw binary, update available — point at the release page (no self-swap).
    Manual,
    /// The drift check itself failed — don't upgrade blindly.
    CheckError,
}

/// Decide the apply action. Pure: no I/O, no process exec.
pub fn apply_action(result: &CheckResult, method: &InstallMethod) -> ApplyAction {
    match result {
        CheckResult::UpToDate { tag } => ApplyAction::AlreadyCurrent { tag: tag.clone() },
        CheckResult::AheadOfLatest { current, .. } => ApplyAction::AlreadyCurrent {
            tag: current.clone(),
        },
        CheckResult::SourceBuild { .. } => ApplyAction::SourceRebuild,
        CheckResult::FetchFailed
        | CheckResult::MalformedLatestTag { .. }
        | CheckResult::MalformedCurrentTag { .. } => ApplyAction::CheckError,
        CheckResult::UpdateAvailable { .. } => match method {
            InstallMethod::Homebrew => ApplyAction::Delegate {
                program: "brew".to_string(),
                args: vec!["upgrade".to_string(), "bwoc".to_string()],
            },
            InstallMethod::Cargo => ApplyAction::Delegate {
                program: "cargo".to_string(),
                args: vec![
                    "install".to_string(),
                    "--git".to_string(),
                    format!("https://github.com/{GITHUB_REPO}"),
                    "bwoc-cli".to_string(),
                ],
            },
            InstallMethod::Raw => ApplyAction::Manual,
        },
    }
}

/// Apply (delegate-only). Runs the drift check, decides the action, prints it,
/// and — only with `run` and only for a package-manager delegate — executes it.
/// A raw binary is never self-replaced here (that path is deferred for review).
fn apply(run: bool, runner: &dyn ShellRunner, method: &InstallMethod) -> i32 {
    let (result, _) = check(runner);
    match apply_action(&result, method) {
        ApplyAction::AlreadyCurrent { tag } => {
            println!("bwoc update: already up to date ({tag})");
            0
        }
        ApplyAction::SourceRebuild => {
            println!("bwoc update: running a source build — upgrade by rebuilding from latest:");
            println!("  git pull && cargo install --path crates/bwoc-cli");
            0
        }
        ApplyAction::CheckError => {
            // Surface the underlying check failure; never upgrade on uncertainty.
            print_check_result(&result);
            2
        }
        ApplyAction::Manual => {
            println!("bwoc update: a newer release is available.");
            println!("  This binary is managed by neither brew nor cargo — download the latest:");
            println!("  https://github.com/{GITHUB_REPO}/releases/latest");
            println!("  Verify the SHA-256 against the published checksum, then replace it.");
            0
        }
        ApplyAction::Delegate { program, args } => {
            let display = format!("{program} {}", args.join(" "));
            if run {
                println!("bwoc update: a newer release is available — running `{display}`");
                exec_delegated(&program, &args)
            } else {
                println!("bwoc update: a newer release is available. Upgrade with:");
                println!("  {display}");
                println!("  (or re-run `bwoc update --run` to execute it)");
                0
            }
        }
    }
}

/// Execute a delegated upgrade command with inherited stdio so the package
/// manager's progress streams straight to the terminal. Returns its exit code.
fn exec_delegated(program: &str, args: &[String]) -> i32 {
    use std::process::Command;
    match Command::new(program).args(args).status() {
        Ok(status) => status.code().unwrap_or(1),
        Err(_) => {
            eprintln!("bwoc update: could not run '{program}' — is it installed and on PATH?");
            127
        }
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── CalVer parse + compare ─────────────────────────────────────────────

    #[test]
    fn calver_parse_full() {
        let v = CalVer::parse("v2026.5.24-0").unwrap();
        assert_eq!(
            v,
            CalVer {
                year: 2026,
                month: 5,
                day: 24,
                patch: 0
            }
        );
    }

    #[test]
    fn calver_parse_without_v_prefix() {
        let v = CalVer::parse("2026.12.31-9").unwrap();
        assert_eq!(
            v,
            CalVer {
                year: 2026,
                month: 12,
                day: 31,
                patch: 9
            }
        );
    }

    #[test]
    fn calver_parse_malformed_returns_none() {
        assert!(CalVer::parse("v2026.5").is_none());
        assert!(CalVer::parse("not-a-version").is_none());
        assert!(CalVer::parse("").is_none());
        assert!(CalVer::parse("v2026.5.24").is_none()); // missing patch
    }

    #[test]
    fn calver_ordering_patch_wins_same_date() {
        let v0 = CalVer::parse("v2026.5.24-0").unwrap();
        let v1 = CalVer::parse("v2026.5.24-1").unwrap();
        assert!(v1 > v0);
    }

    #[test]
    fn calver_ordering_date_wins_over_patch() {
        let old = CalVer::parse("v2026.5.24-9").unwrap();
        let new = CalVer::parse("v2026.5.25-0").unwrap();
        assert!(new > old);
    }

    #[test]
    fn calver_to_tag_round_trips() {
        let tag = "v2026.5.24-0";
        assert_eq!(CalVer::parse(tag).unwrap().to_tag(), tag);
    }

    // ── Mock runner ────────────────────────────────────────────────────────

    struct MockRunner {
        /// Maps (program, first_arg) → (exit_code, stdout).
        responses: Vec<(String, String, i32, String)>,
        /// Records every invocation as (program, full_args) so tests can assert
        /// the complete command contract, not just the dispatch key.
        calls: std::cell::RefCell<Vec<(String, Vec<String>)>>,
    }

    impl MockRunner {
        /// Each entry: (program, first_arg, exit_code, stdout).
        fn new(responses: Vec<(&str, &str, i32, &str)>) -> Self {
            Self {
                responses: responses
                    .into_iter()
                    .map(|(p, a, c, s)| (p.to_string(), a.to_string(), c, s.to_string()))
                    .collect(),
                calls: std::cell::RefCell::new(Vec::new()),
            }
        }

        /// The full argument vector the runner received for `program`, if called.
        fn args_for(&self, program: &str) -> Option<Vec<String>> {
            self.calls
                .borrow()
                .iter()
                .find(|(p, _)| p == program)
                .map(|(_, a)| a.clone())
        }
    }

    impl ShellRunner for MockRunner {
        fn run(&self, program: &str, args: &[&str]) -> ShellOutcome {
            self.calls.borrow_mut().push((
                program.to_string(),
                args.iter().map(|s| s.to_string()).collect(),
            ));
            let first_arg = args.first().copied().unwrap_or("");
            for (p, a, code, out) in &self.responses {
                if p == program && a == first_arg {
                    return ShellOutcome {
                        exit_code: *code,
                        stdout: out.clone(),
                    };
                }
            }
            // No match → simulate not-found.
            ShellOutcome {
                exit_code: 127,
                stdout: String::new(),
            }
        }
    }

    // ── fetch_latest_tag ───────────────────────────────────────────────────

    #[test]
    fn fetch_uses_gh_primary() {
        let runner = MockRunner::new(vec![("gh", "release", 0, "v2026.5.24-0")]);
        let tag = fetch_latest_tag(&runner);
        assert_eq!(tag, Some("v2026.5.24-0".to_string()));
    }

    #[test]
    fn fetch_invokes_full_command_contract() {
        // gh fails so both branches run; assert the EXACT args of each — incl.
        // the `--repo` value and the curl URL, not just the first arg.
        let json = r#"{"tag_name":"v2026.5.24-0"}"#;
        let runner = MockRunner::new(vec![("gh", "release", 1, ""), ("curl", "-s", 0, json)]);
        let _ = fetch_latest_tag(&runner);

        let expected_gh: Vec<String> = [
            "release",
            "view",
            "--repo",
            "bemindlabs/BWOC-Framework",
            "--json",
            "tagName",
            "-q",
            ".tagName",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert_eq!(runner.args_for("gh").unwrap(), expected_gh);

        let expected_curl: Vec<String> = ["-s", GITHUB_API_URL]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(runner.args_for("curl").unwrap(), expected_curl);
    }

    #[test]
    fn fetch_falls_back_to_curl_when_gh_fails() {
        let json = r#"{"tag_name":"v2026.5.24-0","name":"v2026.5.24-0"}"#;
        let runner = MockRunner::new(vec![
            ("gh", "release", 1, ""), // gh fails
            ("curl", "-s", 0, json),  // curl succeeds
        ]);
        let tag = fetch_latest_tag(&runner);
        assert_eq!(tag, Some("v2026.5.24-0".to_string()));
    }

    #[test]
    fn fetch_returns_none_when_both_fail() {
        let runner = MockRunner::new(vec![("gh", "release", 1, ""), ("curl", "-s", 1, "")]);
        assert!(fetch_latest_tag(&runner).is_none());
    }

    // ── check() scenario matrix ────────────────────────────────────────────
    //
    // Note: option_env!("BWOC_RELEASE_CALVER") is resolved at compile time.
    // In a dev/test build this env is NOT set, so all check() tests land on
    // the SourceBuild branch. The scenarios below test the pure comparison
    // logic directly (compare_versions) rather than going through check().
    // The FetchFailed + MalformedLatestTag paths do NOT depend on the env.

    /// Directly test the comparison function logic used inside check().
    fn compare(current: &str, latest: &str) -> CheckResult {
        let latest_ver = CalVer::parse(latest).expect("test: bad latest");
        match CalVer::parse(current) {
            None => CheckResult::MalformedCurrentTag {
                raw: current.to_string(),
                latest: latest_ver.to_tag(),
            },
            Some(cv) => {
                use std::cmp::Ordering;
                match cv.cmp(&latest_ver) {
                    Ordering::Equal => CheckResult::UpToDate {
                        tag: latest_ver.to_tag(),
                    },
                    Ordering::Less => CheckResult::UpdateAvailable {
                        current: cv.to_tag(),
                        latest: latest_ver.to_tag(),
                    },
                    Ordering::Greater => CheckResult::AheadOfLatest {
                        current: cv.to_tag(),
                        latest: latest_ver.to_tag(),
                    },
                }
            }
        }
    }

    #[test]
    fn scenario_update_available() {
        let r = compare("v2026.5.20-0", "v2026.5.24-0");
        assert!(matches!(r, CheckResult::UpdateAvailable { .. }));
        if let CheckResult::UpdateAvailable { current, latest } = r {
            assert_eq!(current, "v2026.5.20-0");
            assert_eq!(latest, "v2026.5.24-0");
        }
    }

    #[test]
    fn scenario_up_to_date() {
        let r = compare("v2026.5.24-0", "v2026.5.24-0");
        assert!(matches!(r, CheckResult::UpToDate { .. }));
    }

    #[test]
    fn scenario_ahead_of_latest() {
        let r = compare("v2026.5.25-0", "v2026.5.24-0");
        assert!(matches!(r, CheckResult::AheadOfLatest { .. }));
    }

    #[test]
    fn scenario_malformed_latest_tag() {
        // check() itself handles this; replicate via fetch_latest_tag returning bad tag.
        let runner = MockRunner::new(vec![("gh", "release", 0, "not-a-calver")]);
        let tag = fetch_latest_tag(&runner);
        assert_eq!(tag, Some("not-a-calver".to_string()));
        // CalVer parse fails.
        assert!(CalVer::parse("not-a-calver").is_none());
    }

    #[test]
    fn scenario_source_build_flag() {
        // option_env!(BWOC_RELEASE_CALVER) == None in test builds.
        // Verify the arm is reachable: check() returns SourceBuild when env unset.
        let runner = MockRunner::new(vec![("gh", "release", 0, "v2026.5.24-0")]);
        let (result, code) = check(&runner);
        // In a source build (no env set), we always get SourceBuild and code 0.
        assert_eq!(code, 0);
        assert!(
            matches!(result, CheckResult::SourceBuild { .. }),
            "expected SourceBuild for dev build, got {result:?}"
        );
        if let CheckResult::SourceBuild { latest } = result {
            assert_eq!(latest, "v2026.5.24-0");
        }
    }

    #[test]
    fn scenario_fetch_failed() {
        let runner = MockRunner::new(vec![("gh", "release", 1, ""), ("curl", "-s", 1, "")]);
        let (result, code) = check(&runner);
        assert_eq!(code, 2);
        assert_eq!(result, CheckResult::FetchFailed);
    }

    // ── install-method classification ─────────────────────────────────────

    #[test]
    fn classify_homebrew_cellar() {
        // current_exe() resolves the brew symlink to a Cellar path.
        assert_eq!(
            classify_install_path("/opt/homebrew/Cellar/bwoc/2.2.0/bin/bwoc"),
            InstallMethod::Homebrew
        );
        assert_eq!(
            classify_install_path("/usr/local/Cellar/bwoc/2.2.0/bin/bwoc"),
            InstallMethod::Homebrew
        );
        assert_eq!(
            classify_install_path("/home/linuxbrew/.linuxbrew/bin/bwoc"),
            InstallMethod::Homebrew
        );
    }

    #[test]
    fn classify_cargo_bin() {
        assert_eq!(
            classify_install_path("/Users/dev/.cargo/bin/bwoc"),
            InstallMethod::Cargo
        );
    }

    #[test]
    fn classify_raw_binary() {
        assert_eq!(
            classify_install_path("/usr/local/bin/bwoc"),
            InstallMethod::Raw
        );
        assert_eq!(classify_install_path("/tmp/bwoc"), InstallMethod::Raw);
    }

    // ── apply-action routing (pure) ────────────────────────────────────────

    fn update_available() -> CheckResult {
        CheckResult::UpdateAvailable {
            current: "v2026.5.23-3".to_string(),
            latest: "v2026.5.24-0".to_string(),
        }
    }

    #[test]
    fn apply_homebrew_delegates_to_brew() {
        let action = apply_action(&update_available(), &InstallMethod::Homebrew);
        assert_eq!(
            action,
            ApplyAction::Delegate {
                program: "brew".to_string(),
                args: vec!["upgrade".to_string(), "bwoc".to_string()],
            }
        );
    }

    #[test]
    fn apply_cargo_delegates_to_cargo() {
        let action = apply_action(&update_available(), &InstallMethod::Cargo);
        match action {
            ApplyAction::Delegate { program, args } => {
                assert_eq!(program, "cargo");
                assert_eq!(args[0], "install");
                assert!(args.contains(&"bwoc-cli".to_string()));
            }
            other => panic!("expected cargo delegate, got {other:?}"),
        }
    }

    #[test]
    fn apply_raw_is_manual_no_self_swap() {
        let action = apply_action(&update_available(), &InstallMethod::Raw);
        assert_eq!(action, ApplyAction::Manual);
    }

    #[test]
    fn apply_up_to_date_is_noop() {
        let r = CheckResult::UpToDate {
            tag: "v2026.5.24-0".to_string(),
        };
        assert_eq!(
            apply_action(&r, &InstallMethod::Homebrew),
            ApplyAction::AlreadyCurrent {
                tag: "v2026.5.24-0".to_string()
            }
        );
    }

    #[test]
    fn apply_source_build_is_rebuild() {
        let r = CheckResult::SourceBuild {
            latest: "v2026.5.24-0".to_string(),
        };
        assert_eq!(
            apply_action(&r, &InstallMethod::Raw),
            ApplyAction::SourceRebuild
        );
    }

    #[test]
    fn apply_fetch_failure_is_check_error_not_blind_upgrade() {
        assert_eq!(
            apply_action(&CheckResult::FetchFailed, &InstallMethod::Homebrew),
            ApplyAction::CheckError
        );
    }

    // ── apply integration via run_with (no --check) ────────────────────────

    #[test]
    fn run_with_apply_source_build_returns_zero() {
        // In a test build `BWOC_RELEASE_CALVER` is unset → check() = SourceBuild
        // (given a successful fetch) → apply = source-rebuild guidance, code 0.
        let runner = MockRunner::new(vec![("gh", "release", 0, "v2026.5.24-0")]);
        let code = run_with(
            UpdateArgs {
                check: false,
                run: false,
            },
            &runner,
        );
        assert_eq!(code, 0);
    }

    #[test]
    fn run_with_apply_check_failure_returns_two() {
        // Fetch fails → never upgrade on uncertainty.
        let runner = MockRunner::new(vec![]);
        let code = run_with(
            UpdateArgs {
                check: false,
                run: false,
            },
            &runner,
        );
        assert_eq!(code, 2);
    }
}
