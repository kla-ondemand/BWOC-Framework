//! `bwoc team …` + `bwoc task …` — Saṅgha v1 CLI (Phase A foundation).
//!
//! Thin IO + locking layer over `bwoc_core::team`. The data model and
//! state-transition rules live in core; this module resolves the
//! workspace, reads/writes the on-disk files, and serializes task
//! mutations behind an advisory lock so two teammates never claim the
//! same task.
//!
//! Layout under the workspace:
//!
//! ```text
//! .bwoc/teams/<team-id>.toml          # Team membership
//! .bwoc/teams/<team-id>/tasks.jsonl   # shared task list
//! .bwoc/teams/<team-id>/tasks.lock    # advisory lock (this module)
//! ```
//!
//! The human operator is the implicit lead — there is no `lead` field.
//! Teammates **self-claim** tasks; the lock makes the claim a
//! Saṅghakamma (a communal act settled by exactly one member).

use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

use bwoc_core::team::{self, Task, Team, TeamError};

// --- path helpers ----------------------------------------------------------

fn teams_dir(workspace: &Path) -> PathBuf {
    workspace.join(".bwoc/teams")
}

fn team_toml_path(workspace: &Path, team_id: &str) -> PathBuf {
    teams_dir(workspace).join(format!("{team_id}.toml"))
}

fn team_task_dir(workspace: &Path, team_id: &str) -> PathBuf {
    teams_dir(workspace).join(team_id)
}

fn tasks_jsonl_path(workspace: &Path, team_id: &str) -> PathBuf {
    team_task_dir(workspace, team_id).join("tasks.jsonl")
}

/// Resolve the workspace per `WORKSPACE.en.md`: explicit → BWOC_WORKSPACE
/// → ancestor walk → None. (Local copy per the per-module convention used
/// by `chat`/`send`/`trust`.)
fn resolve_workspace(explicit: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(p) = explicit {
        return Some(p);
    }
    if let Ok(env_path) = std::env::var("BWOC_WORKSPACE") {
        if !env_path.is_empty() {
            return Some(PathBuf::from(env_path));
        }
    }
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

// --- advisory lock ---------------------------------------------------------

/// A best-effort advisory lock over a team's task file, acquired by
/// atomically creating `tasks.lock` (`O_CREAT | O_EXCL` via `create_new`).
/// Dependency-free; staleness is detected with a signal-0 probe on the
/// recorded PID (a crash mid-claim leaves a lock whose owner is dead, so
/// we steal it). Released on drop.
struct TaskLock {
    path: PathBuf,
}

impl TaskLock {
    fn acquire(task_dir: &Path) -> io::Result<Self> {
        fs::create_dir_all(task_dir)?;
        let path = task_dir.join("tasks.lock");
        // ~5s budget: 50 attempts × 100ms. Inbox-rare contention, so a
        // generous budget costs nothing in the common case.
        for _ in 0..50 {
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut f) => {
                    let _ = write!(f, "{}", std::process::id());
                    return Ok(Self { path });
                }
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                    if lock_is_stale(&path) {
                        let _ = fs::remove_file(&path);
                        continue; // retry immediately after stealing
                    }
                    sleep(Duration::from_millis(100));
                }
                Err(e) => return Err(e),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "could not acquire task lock (another `bwoc task` holding it? \
             remove .bwoc/teams/<id>/tasks.lock if stale)",
        ))
    }
}

impl Drop for TaskLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// A lock is stale if its recorded PID is no longer alive. An unreadable
/// or non-numeric lock body is treated as NOT stale (conservative — don't
/// steal a lock we can't reason about).
fn lock_is_stale(path: &Path) -> bool {
    let Ok(raw) = fs::read_to_string(path) else {
        return false;
    };
    match raw.trim().parse::<u32>() {
        Ok(pid) => !crate::livecheck::signal_zero_alive(pid),
        Err(_) => false,
    }
}

// --- shared loaders --------------------------------------------------------

fn load_team(workspace: &Path, team_id: &str) -> Result<Team, String> {
    let path = team_toml_path(workspace, team_id);
    let body = fs::read_to_string(&path)
        .map_err(|_| format!("no team '{team_id}' in workspace (expected {})", path.display()))?;
    Team::from_toml(&body).map_err(|e| format!("team '{team_id}' is malformed: {e}"))
}

fn load_tasks(workspace: &Path, team_id: &str) -> Result<Vec<Task>, String> {
    let path = tasks_jsonl_path(workspace, team_id);
    match fs::read_to_string(&path) {
        Ok(body) => team::parse_tasks(&body).map_err(|e| format!("tasks.jsonl malformed: {e}")),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(format!("failed to read {}: {e}", path.display())),
    }
}

/// Atomic write: render to a sibling tmp file then rename over the target.
fn save_tasks(workspace: &Path, team_id: &str, tasks: &[Task]) -> Result<(), String> {
    let path = tasks_jsonl_path(workspace, team_id);
    let body = team::render_tasks(tasks).map_err(|e| format!("serialize failed: {e}"))?;
    let tmp = path.with_extension("jsonl.tmp");
    fs::write(&tmp, body).map_err(|e| format!("failed to write {}: {e}", tmp.display()))?;
    fs::rename(&tmp, &path).map_err(|e| format!("failed to rename into {}: {e}", path.display()))?;
    Ok(())
}

// --- team commands ---------------------------------------------------------

pub fn run_team_create(
    workspace: Option<PathBuf>,
    id: String,
    members: Vec<String>,
    json: bool,
) -> i32 {
    let Some(ws) = resolve_workspace(workspace) else {
        eprintln!("bwoc team create: no workspace found. Pass --workspace or run `bwoc init`.");
        return 2;
    };
    let path = team_toml_path(&ws, &id);
    if path.exists() {
        eprintln!("bwoc team create: team '{id}' already exists ({})", path.display());
        return 2;
    }
    let team = Team::new(&id, members);
    let toml = match team.to_toml() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("bwoc team create: {e}");
            return 1;
        }
    };
    if let Err(e) = fs::create_dir_all(teams_dir(&ws)).and_then(|_| fs::write(&path, toml)) {
        eprintln!("bwoc team create: failed to write {}: {e}", path.display());
        return 1;
    }
    if json {
        println!(
            "{}",
            serde_json::json!({
                "team": team.id,
                "members": team.members,
                "created_at": team.created_at,
                "path": path.display().to_string(),
            })
        );
    } else {
        println!("Created team '{}' ({} member(s))", team.id, team.members.len());
        for m in &team.members {
            println!("  - {m}");
        }
        println!("  Tasks: {}", tasks_jsonl_path(&ws, &id).display());
    }
    0
}

pub fn run_team_list(workspace: Option<PathBuf>, json: bool) -> i32 {
    let Some(ws) = resolve_workspace(workspace) else {
        eprintln!("bwoc team list: no workspace found. Pass --workspace or run `bwoc init`.");
        return 2;
    };
    let dir = teams_dir(&ws);
    let mut teams: Vec<Team> = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("toml")
                && let Ok(body) = fs::read_to_string(&p)
                && let Ok(t) = Team::from_toml(&body)
            {
                teams.push(t);
            }
        }
    }
    teams.sort_by(|a, b| a.id.cmp(&b.id));

    if json {
        let arr: Vec<_> = teams
            .iter()
            .map(|t| {
                serde_json::json!({
                    "team": t.id,
                    "members": t.members,
                    "created_at": t.created_at,
                })
            })
            .collect();
        println!("{}", serde_json::Value::Array(arr));
        return 0;
    }
    if teams.is_empty() {
        println!("(no teams in workspace {} — `bwoc team create <id> --members …`)", ws.display());
        return 0;
    }
    for t in &teams {
        let tasks = load_tasks(&ws, &t.id).unwrap_or_default();
        let pending = tasks
            .iter()
            .filter(|x| x.state == bwoc_core::team::TaskState::Pending)
            .count();
        println!(
            "{}  ({} member(s), {} task(s), {} pending)",
            t.id,
            t.members.len(),
            tasks.len(),
            pending
        );
    }
    0
}

pub fn run_team_retire(workspace: Option<PathBuf>, id: String, yes: bool, json: bool) -> i32 {
    let Some(ws) = resolve_workspace(workspace) else {
        eprintln!("bwoc team retire: no workspace found. Pass --workspace or run `bwoc init`.");
        return 2;
    };
    let toml_path = team_toml_path(&ws, &id);
    if !toml_path.exists() {
        eprintln!("bwoc team retire: no team '{id}' in workspace");
        return 2;
    }
    if !yes {
        if !io::stdin().is_terminal() {
            eprintln!(
                "bwoc team retire: not a TTY and --yes not given. \
                 Pass --yes to confirm or run from an interactive shell."
            );
            return 2;
        }
        print!("Retire team '{id}' and delete its task list? [y/N]: ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            return 1;
        }
        let a = line.trim().to_ascii_lowercase();
        if a != "y" && a != "yes" {
            eprintln!("aborted");
            return 2;
        }
    }
    let task_dir = team_task_dir(&ws, &id);
    let _ = fs::remove_dir_all(&task_dir); // best-effort (may not exist)
    if let Err(e) = fs::remove_file(&toml_path) {
        eprintln!("bwoc team retire: failed to remove {}: {e}", toml_path.display());
        return 1;
    }
    if json {
        println!("{}", serde_json::json!({ "team": id, "retired": true }));
    } else {
        println!("Retired team '{id}' (membership + task list removed)");
    }
    0
}

// --- task commands ---------------------------------------------------------

pub fn run_task_add(
    workspace: Option<PathBuf>,
    team_id: String,
    title: String,
    deps: Vec<String>,
    id_override: Option<String>,
    json: bool,
) -> i32 {
    let Some(ws) = resolve_workspace(workspace) else {
        eprintln!("bwoc task add: no workspace found. Pass --workspace or run `bwoc init`.");
        return 2;
    };
    if let Err(e) = load_team(&ws, &team_id) {
        eprintln!("bwoc task add: {e}");
        return 2;
    }
    let task_dir = team_task_dir(&ws, &team_id);
    let _lock = match TaskLock::acquire(&task_dir) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("bwoc task add: {e}");
            return 1;
        }
    };
    let mut tasks = match load_tasks(&ws, &team_id) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("bwoc task add: {e}");
            return 1;
        }
    };
    // Monotonic id `t<N>` (tasks are never removed, so len only grows),
    // unless the caller passed an explicit --id.
    let id = id_override.unwrap_or_else(|| format!("t{}", tasks.len() + 1));
    let task = Task::new(&id, &title, deps);
    if let Err(e) = team::add_task(&mut tasks, task) {
        eprintln!("bwoc task add: {e}");
        return 2;
    }
    if let Err(e) = save_tasks(&ws, &team_id, &tasks) {
        eprintln!("bwoc task add: {e}");
        return 1;
    }
    if json {
        println!(
            "{}",
            serde_json::json!({ "team": team_id, "task": id, "state": "pending" })
        );
    } else {
        println!("Added task '{id}' to team '{team_id}': {title}");
    }
    0
}

pub fn run_task_list(workspace: Option<PathBuf>, team_id: String, json: bool) -> i32 {
    let Some(ws) = resolve_workspace(workspace) else {
        eprintln!("bwoc task list: no workspace found. Pass --workspace or run `bwoc init`.");
        return 2;
    };
    if let Err(e) = load_team(&ws, &team_id) {
        eprintln!("bwoc task list: {e}");
        return 2;
    }
    let tasks = match load_tasks(&ws, &team_id) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("bwoc task list: {e}");
            return 1;
        }
    };
    if json {
        let body = team::render_tasks(&tasks).unwrap_or_default();
        // Emit a JSON array (parse each line back) for consumer-friendliness.
        let arr: Vec<serde_json::Value> = body
            .lines()
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();
        println!("{}", serde_json::Value::Array(arr));
        return 0;
    }
    if tasks.is_empty() {
        println!("(team '{team_id}' has no tasks — `bwoc task add {team_id} \"…\"`)");
        return 0;
    }
    for t in &tasks {
        let owner = t.claimed_by.as_deref().unwrap_or("—");
        let deps = if t.deps.is_empty() {
            String::new()
        } else {
            format!("  deps=[{}]", t.deps.join(","))
        };
        println!(
            "{:<6} {:<12} {:<14} {}{}",
            t.id,
            t.state.as_str(),
            owner,
            t.title,
            deps
        );
    }
    0
}

pub fn run_task_claim(
    workspace: Option<PathBuf>,
    team_id: String,
    task_id: String,
    agent: String,
    json: bool,
) -> i32 {
    mutate_task(workspace, &team_id, &agent, json, "claim", |tasks| {
        team::claim_task(tasks, &task_id, &agent)
    })
}

pub fn run_task_complete(
    workspace: Option<PathBuf>,
    team_id: String,
    task_id: String,
    agent: String,
    json: bool,
) -> i32 {
    mutate_task(workspace, &team_id, &agent, json, "complete", |tasks| {
        team::complete_task(tasks, &task_id, &agent)
    })
}

/// Shared claim/complete path: resolve workspace, verify the actor is a
/// team member, acquire the lock, load → mutate → save. The `op` closure
/// runs the core transition; `verb` is just for messages.
fn mutate_task(
    workspace: Option<PathBuf>,
    team_id: &str,
    agent: &str,
    json: bool,
    verb: &str,
    op: impl FnOnce(&mut [Task]) -> Result<(), TeamError>,
) -> i32 {
    let Some(ws) = resolve_workspace(workspace) else {
        eprintln!("bwoc task {verb}: no workspace found. Pass --workspace or run `bwoc init`.");
        return 2;
    };
    let team = match load_team(&ws, team_id) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("bwoc task {verb}: {e}");
            return 2;
        }
    };
    if let Err(e) = team::ensure_member(&team, agent) {
        eprintln!("bwoc task {verb}: {e} (members: {})", team.members.join(", "));
        return 2;
    }
    let task_dir = team_task_dir(&ws, team_id);
    let _lock = match TaskLock::acquire(&task_dir) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("bwoc task {verb}: {e}");
            return 1;
        }
    };
    let mut tasks = match load_tasks(&ws, team_id) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("bwoc task {verb}: {e}");
            return 1;
        }
    };
    if let Err(e) = op(&mut tasks) {
        eprintln!("bwoc task {verb}: {e}");
        // Transition errors (wrong state, blocked, not claimant) are user
        // errors → exit 2; the file is untouched.
        return 2;
    }
    if let Err(e) = save_tasks(&ws, team_id, &tasks) {
        eprintln!("bwoc task {verb}: {e}");
        return 1;
    }
    if json {
        println!(
            "{}",
            serde_json::json!({ "team": team_id, "agent": agent, "op": verb, "ok": true })
        );
    } else {
        println!("{verb}: ok ({agent} on team '{team_id}')");
    }
    0
}
