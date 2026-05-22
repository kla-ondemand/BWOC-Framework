//! `bwoc dashboard` — Phase 1 TUI.
//!
//! Phase 0 (shipped): bare ratatui shell that draws a title + empty
//! body + footer; quits cleanly on `q`/`Esc`/`Ctrl-C`.
//!
//! Phase 1 (this file): populates the agents pane from
//! `.bwoc/agents.toml` with `↑`/`↓` (or `j`/`k`) navigation, a
//! highlighted selection row, and `r` to refresh.
//!
//! Phase 2 (next): detail pane reusing `doctor`-style probes.
//! Phase 3: Fluent i18n. Phase 4: log tail + editor handoff.

use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::i18n;
use bwoc_core::manifest::Manifest;
use bwoc_core::workspace::{AgentEntry, AgentsRegistry};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use fluent_bundle::{FluentBundle, FluentResource};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};

pub struct DashboardArgs {
    pub workspace: Option<PathBuf>,
    pub lang: String,
}

pub fn run(args: DashboardArgs) -> i32 {
    use std::io::IsTerminal;
    if !io::stdout().is_terminal() {
        eprintln!(
            "bwoc dashboard: stdout is not a TTY. Use `bwoc list` / `bwoc status` for non-interactive output."
        );
        return 2;
    }

    let mut app = App::new(args.workspace, args.lang);

    let mut term = match setup_terminal() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("bwoc dashboard: failed to enter alt screen: {e}");
            return 1;
        }
    };

    let result = event_loop(&mut term, &mut app);

    if let Err(e) = restore_terminal() {
        eprintln!("bwoc dashboard: warning — failed to restore terminal: {e}");
    }

    match result {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("bwoc dashboard: {e}");
            1
        }
    }
}

// --- app state ------------------------------------------------------------

/// How often the dashboard auto-refreshes the agents registry. Live-
/// changing data (runtime indicators, uptime, inbox counts) re-reads
/// on every draw regardless — this only controls how often we re-load
/// agents.toml itself. 2s is fast enough to feel live without thrashing
/// the disk for huge registries.
const AUTO_REFRESH_INTERVAL: Duration = Duration::from_secs(2);

struct App {
    workspace: Option<PathBuf>,
    agents: Vec<AgentEntry>,
    table_state: TableState,
    last_refresh_error: Option<String>,
    bundle: FluentBundle<FluentResource>,
    last_refresh_at: Instant,
    /// One-line transient feedback from the last user action (e.g. tmux
    /// open result). Shown in the footer; cleared on next action.
    last_action: Option<String>,
}

impl App {
    fn new(workspace_arg: Option<PathBuf>, lang: String) -> Self {
        let workspace = resolve_workspace(workspace_arg);
        let mut app = Self {
            workspace,
            agents: Vec::new(),
            table_state: TableState::default(),
            last_refresh_error: None,
            bundle: i18n::bundle_for(&lang),
            last_refresh_at: Instant::now(),
            last_action: None,
        };
        app.refresh();
        app
    }

    fn refresh(&mut self) {
        self.last_refresh_at = Instant::now();
        if let Some(root) = &self.workspace {
            match AgentsRegistry::load(root) {
                Ok(r) => {
                    self.agents = r.agents;
                    self.last_refresh_error = None;
                    // Keep selection valid as the registry shrinks/grows.
                    if self.agents.is_empty() {
                        self.table_state.select(None);
                    } else {
                        let cur = self.table_state.selected().unwrap_or(0);
                        self.table_state
                            .select(Some(cur.min(self.agents.len() - 1)));
                    }
                }
                Err(e) => {
                    self.agents.clear();
                    self.last_refresh_error = Some(format!("agents.toml: {e}"));
                    self.table_state.select(None);
                }
            }
        } else {
            self.agents.clear();
            self.last_refresh_error = None;
            self.table_state.select(None);
        }
    }

    fn next(&mut self) {
        if self.agents.is_empty() {
            return;
        }
        let i = self.table_state.selected().unwrap_or(0);
        self.table_state.select(Some((i + 1) % self.agents.len()));
    }

    fn prev(&mut self) {
        if self.agents.is_empty() {
            return;
        }
        let i = self.table_state.selected().unwrap_or(0);
        let new = if i == 0 { self.agents.len() - 1 } else { i - 1 };
        self.table_state.select(Some(new));
    }
}

// --- terminal setup / event loop -----------------------------------------

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn event_loop(term: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        term.draw(|f| draw_frame(f, app))?;

        if event::poll(Duration::from_millis(200))?
            && let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
        {
            match (code, modifiers) {
                (KeyCode::Char('q'), KeyModifiers::NONE) => return Ok(()),
                (KeyCode::Esc, _) => return Ok(()),
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(()),
                (KeyCode::Char('r'), _) => app.refresh(),
                (KeyCode::Char('t'), _) => open_in_tmux(app),
                (KeyCode::Down | KeyCode::Char('j'), _) => app.next(),
                (KeyCode::Up | KeyCode::Char('k'), _) => app.prev(),
                _ => {}
            }
        }

        // Auto-tick: re-read agents.toml every AUTO_REFRESH_INTERVAL.
        // Live data (runtime/uptime/inbox) re-reads on every draw —
        // this only refreshes the agents Vec itself, so additions/
        // removals appear without the user pressing `r`.
        if app.last_refresh_at.elapsed() >= AUTO_REFRESH_INTERVAL {
            app.refresh();
        }
    }
}

/// `t` hotkey — spawn `bwoc spawn` for the selected agent in a new tmux
/// window. Auto-selects the LLM/model via the agent's manifest (`bwoc
/// spawn` already does this — backend lookup from registry, model from
/// manifest.primaryModel).
///
/// Requires `$TMUX` (the dashboard must itself be running inside a
/// tmux session). Failure modes:
///   - no `$TMUX`: updates `app.last_action` with a hint; no-op
///   - no agent selected: no-op (silent)
///   - tmux exec error: updates `app.last_action` with the error
fn open_in_tmux(app: &mut App) {
    let Some(idx) = app.table_state.selected() else {
        app.last_action = Some("(no agent selected — ↑↓ to pick first)".to_string());
        return;
    };
    let Some(entry) = app.agents.get(idx).cloned() else {
        return;
    };
    let Some(root) = &app.workspace else {
        app.last_action = Some("(no workspace — nothing to talk to)".to_string());
        return;
    };

    if std::env::var_os("TMUX").is_none() {
        app.last_action = Some(
            "(not inside tmux — run `tmux new-session` first, then re-launch dashboard)"
                .to_string(),
        );
        return;
    }

    let agent_path = root.join(&entry.path);
    let agent_path_str = agent_path.to_string_lossy().to_string();
    // `tmux new-window -n <name> -- bwoc spawn --path <p> --backend <b>`
    // The trailing args after `--` are run as the new window's command.
    let result = std::process::Command::new("tmux")
        .args([
            "new-window",
            "-n",
            entry.id.as_str(),
            "--",
            "bwoc",
            "spawn",
            "--path",
            agent_path_str.as_str(),
            "--backend",
            entry.backend.as_str(),
        ])
        .status();
    match result {
        Ok(s) if s.success() => {
            app.last_action = Some(format!(
                "→ tmux window '{}' opened (backend: {})",
                entry.id, entry.backend
            ));
        }
        Ok(s) => {
            app.last_action = Some(format!("tmux new-window exited {s}"));
        }
        Err(e) => {
            app.last_action = Some(format!("tmux exec failed: {e}"));
        }
    }
}

// --- workspace resolution -------------------------------------------------

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

// --- drawing --------------------------------------------------------------

fn draw_frame(f: &mut ratatui::Frame, app: &mut App) {
    let area = f.area();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // banner
            Constraint::Min(0),    // body (agents + detail)
            Constraint::Length(1), // footer
        ])
        .split(area);

    draw_banner(f, layout[0], app);
    draw_body(f, layout[1], app);
    draw_footer(f, layout[2], app);
}

fn draw_body(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    let h = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    draw_agents(f, h[0], app);
    draw_detail(f, h[1], app);
}

fn draw_detail(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let title = format!(" {} ", i18n::t(&app.bundle, "dash-pane-detail"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().add_modifier(Modifier::DIM));

    let Some(idx) = app.table_state.selected() else {
        let p = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                i18n::t(&app.bundle, "dash-empty-select"),
                Style::default().add_modifier(Modifier::DIM),
            )),
        ])
        .alignment(Alignment::Center)
        .block(block);
        f.render_widget(p, area);
        return;
    };
    let Some(entry) = app.agents.get(idx) else {
        f.render_widget(Paragraph::new("").block(block), area);
        return;
    };
    let Some(root) = &app.workspace else {
        f.render_widget(Paragraph::new("").block(block), area);
        return;
    };

    let agent_path = root.join(&entry.path);
    let mut lines: Vec<Line> = Vec::new();

    let key_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    lines.push(Line::from(vec![
        Span::styled("id          ", key_style),
        Span::raw(entry.id.clone()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("path        ", key_style),
        Span::raw(entry.path.clone()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("backend     ", key_style),
        Span::raw(entry.backend.clone()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("incarnated  ", key_style),
        Span::raw(entry.incarnated.clone()),
    ]));

    // Runtime (live process state) + inbox count — surfaces Phase 2/3
    // data inside the dashboard. Best-effort: missing files / no daemon
    // render as "○ not running" / "0 messages".
    let (runtime_mark, runtime_color, runtime_text) =
        match crate::livecheck::running_pid(root, entry) {
            Some(pid) => match crate::livecheck::query_uptime(root, entry) {
                Some(secs) => (
                    "●",
                    Color::Green,
                    format!(
                        "running (pid {pid}, uptime {})",
                        crate::livecheck::format_uptime(secs)
                    ),
                ),
                None => ("●", Color::Green, format!("running (pid {pid})")),
            },
            None => ("○", Color::DarkGray, "not running".to_string()),
        };
    lines.push(Line::from(vec![
        Span::styled("runtime     ", key_style),
        Span::styled(
            runtime_mark,
            Style::default()
                .fg(runtime_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(runtime_text, Style::default().fg(runtime_color)),
    ]));
    let count = crate::livecheck::inbox_count(root, entry);
    let inbox_color = if count == 0 {
        Color::DarkGray
    } else {
        Color::Yellow
    };
    lines.push(Line::from(vec![
        Span::styled("inbox       ", key_style),
        Span::styled(
            format!("{count} message(s)"),
            Style::default().fg(inbox_color),
        ),
    ]));
    lines.push(Line::from(""));

    // Manifest fields (load on demand; failures shown gracefully).
    match Manifest::load_from_path(&agent_path.join("config.manifest.json")) {
        Ok(m) => {
            lines.push(Line::from(Span::styled(
                "manifest:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(vec![
                Span::styled("  role        ", key_style),
                Span::raw(m.agent_role),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  model       ", key_style),
                Span::raw(m.primary_model),
            ]));
            if let Some(fb) = m.fallback_model {
                lines.push(Line::from(vec![
                    Span::styled("  fallback    ", key_style),
                    Span::raw(fb),
                ]));
            }
            lines.push(Line::from(vec![
                Span::styled("  memory      ", key_style),
                Span::raw(m.memory_path),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  version     ", key_style),
                Span::raw(m.version),
            ]));
            // Persona scope (from the new manifest fields populated by
            // `bwoc new --scope` / `--out-of-scope`). Shown only when
            // set — otherwise the {{placeholder}} fallback is just
            // noise here.
            if let Some(scope) = &m.scope_description {
                lines.push(Line::from(vec![
                    Span::styled("  scope       ", key_style),
                    Span::raw(scope.clone()),
                ]));
            }
            if let Some(out) = &m.out_of_scope {
                lines.push(Line::from(vec![
                    Span::styled("  out-of-scope", key_style),
                    Span::raw(format!(" {out}")),
                ]));
            }
        }
        Err(e) => {
            lines.push(Line::from(Span::styled(
                format!("manifest: failed ({e})"),
                Style::default().fg(Color::Red),
            )));
        }
    }

    // Resource counts — mindsets / skills / memories. Each is the
    // count of `.md` files in the directory excluding template
    // scaffolding (SPEC.md / README.md). 0 renders as "—".
    let counts_block = [
        (
            "mindsets",
            crate::livecheck::count_user_md_files(&agent_path.join("mindsets")),
        ),
        (
            "skills",
            crate::livecheck::count_user_md_files(&agent_path.join("skills")),
        ),
        (
            "memories",
            crate::livecheck::count_user_md_files(&agent_path.join("memories")),
        ),
    ];
    if counts_block.iter().any(|(_, n)| *n > 0) {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "resources:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        for (label, n) in &counts_block {
            let value = if *n == 0 {
                "—".to_string()
            } else {
                n.to_string()
            };
            let value_color = if *n == 0 {
                Color::DarkGray
            } else {
                Color::Reset
            };
            lines.push(Line::from(vec![
                Span::styled(format!("  {label:<12}"), key_style),
                Span::styled(value, Style::default().fg(value_color)),
            ]));
        }
    }

    lines.push(Line::from(""));

    // Health probe — same shape as `bwoc doctor` / `bwoc status` per-agent
    // checks, returning one summarised verdict.
    let (mark, color, msg) = match probe(root, entry) {
        Health::Ok => ("✓", Color::Green, "all probes passed".to_string()),
        Health::Warn(m) => ("⚠", Color::Yellow, m),
        Health::Fail(m) => ("✗", Color::Red, m),
    };
    lines.push(Line::from(vec![
        Span::styled("health      ", key_style),
        Span::styled(
            mark,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(msg, Style::default().fg(color)),
    ]));

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}

// Liveness + inbox helpers (signal_zero_alive, running_pid,
// query_uptime, format_uptime, inbox_count) moved to crate::livecheck.

const BACKEND_SYMLINKS: &[&str] = &["CLAUDE.md", "GEMINI.md", "CODEX.md", "KIMI.md"];

#[derive(Debug)]
enum Health {
    Ok,
    Warn(String),
    Fail(String),
}

fn probe(root: &std::path::Path, a: &AgentEntry) -> Health {
    let p = root.join(&a.path);
    if !p.is_dir() {
        return Health::Fail(format!("directory missing: {}", p.display()));
    }
    if !p.join("AGENTS.md").is_file() {
        return Health::Fail("missing AGENTS.md".to_string());
    }
    let missing: Vec<&str> = BACKEND_SYMLINKS
        .iter()
        .copied()
        .filter(|link| !p.join(link).exists())
        .collect();
    if !missing.is_empty() {
        return Health::Warn(format!(
            "missing symlinks: {} (run `bwoc doctor --auto`)",
            missing.join(", ")
        ));
    }
    if !p.join("config.manifest.json").is_file() {
        return Health::Warn("config.manifest.json missing".to_string());
    }
    Health::Ok
}

fn draw_banner(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let title_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let workspace_line = match &app.workspace {
        Some(p) => format!("Workspace: {}", p.display()),
        None => "Workspace: (none — pass --workspace, set BWOC_WORKSPACE, or run `bwoc init`)"
            .to_string(),
    };
    let lines = vec![
        Line::from(Span::styled("BWOC Framework", title_style)),
        Line::from(Span::styled(
            format!("v{}", env!("CARGO_PKG_VERSION")),
            Style::default().add_modifier(Modifier::DIM),
        )),
        Line::from(workspace_line),
    ];
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" dashboard ")
        .border_style(Style::default().fg(Color::Cyan));
    let p = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(block);
    f.render_widget(p, area);
}

fn draw_agents(f: &mut ratatui::Frame, area: Rect, app: &mut App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" agents ")
        .border_style(Style::default().add_modifier(Modifier::DIM));

    if let Some(err) = &app.last_refresh_error {
        let p = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("failed to read agents: {err}"),
                Style::default().fg(Color::Red),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "press `r` to retry",
                Style::default().add_modifier(Modifier::DIM),
            )),
        ])
        .alignment(Alignment::Center)
        .block(block);
        f.render_widget(p, area);
        return;
    }

    if app.agents.is_empty() {
        let msg = if app.workspace.is_some() {
            "(no agents registered — `bwoc new <name>` to incarnate the first)"
        } else {
            "(no workspace found — exit and run `bwoc init` first)"
        };
        let p = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                msg,
                Style::default().add_modifier(Modifier::DIM),
            )),
        ])
        .alignment(Alignment::Center)
        .block(block);
        f.render_widget(p, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("ID"),
        Cell::from("STATUS"),
        Cell::from("BACKEND"),
        Cell::from("PATH"),
    ])
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = app
        .agents
        .iter()
        .map(|a| {
            Row::new(vec![
                Cell::from(a.id.clone()),
                Cell::from(a.status.clone()),
                Cell::from(a.backend.clone()),
                Cell::from(a.path.clone()),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(30),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Percentage(40),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn draw_footer(f: &mut ratatui::Frame, area: Rect, app: &App) {
    // When we have transient feedback from the last action (tmux open
    // result, "not in tmux" hint, etc.), show that instead of the
    // hotkey legend — the user just performed an action and wants to
    // know what happened. The hotkey legend re-appears on next refresh.
    if let Some(msg) = &app.last_action {
        let p = Paragraph::new(Line::from(Span::styled(
            msg.clone(),
            Style::default().fg(Color::Cyan),
        )))
        .alignment(Alignment::Center);
        f.render_widget(p, area);
        return;
    }

    let bold = Style::default().add_modifier(Modifier::BOLD);
    let count = if app.agents.is_empty() {
        "0 agents".to_string()
    } else {
        let cur = app.table_state.selected().unwrap_or(0) + 1;
        format!("{}/{}", cur, app.agents.len())
    };
    let nav = i18n::t(&app.bundle, "dash-footer-navigate");
    let refresh = i18n::t(&app.bundle, "dash-footer-refresh");
    let quit = i18n::t(&app.bundle, "dash-footer-quit");
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(count, Style::default().fg(Color::Cyan)),
        Span::raw("    "),
        Span::styled("↑↓/jk", bold),
        Span::raw(format!(" {nav}    ")),
        Span::styled("t", bold),
        Span::raw(" talk (tmux)    "),
        Span::styled("r", bold),
        Span::raw(format!(" {refresh}    ")),
        Span::styled("q/Esc", bold),
        Span::raw(format!(" {quit}")),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(footer, area);
}
