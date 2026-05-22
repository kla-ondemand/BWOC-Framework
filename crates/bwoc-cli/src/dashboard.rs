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
use std::time::Duration;

use bwoc_core::workspace::{AgentEntry, AgentsRegistry};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};

pub struct DashboardArgs {
    pub workspace: Option<PathBuf>,
}

pub fn run(args: DashboardArgs) -> i32 {
    use std::io::IsTerminal;
    if !io::stdout().is_terminal() {
        eprintln!(
            "bwoc dashboard: stdout is not a TTY. Use `bwoc list` / `bwoc status` for non-interactive output."
        );
        return 2;
    }

    let mut app = App::new(args.workspace);

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

struct App {
    workspace: Option<PathBuf>,
    agents: Vec<AgentEntry>,
    table_state: TableState,
    last_refresh_error: Option<String>,
}

impl App {
    fn new(workspace_arg: Option<PathBuf>) -> Self {
        let workspace = resolve_workspace(workspace_arg);
        let mut app = Self {
            workspace,
            agents: Vec::new(),
            table_state: TableState::default(),
            last_refresh_error: None,
        };
        app.refresh();
        app
    }

    fn refresh(&mut self) {
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
                (KeyCode::Down | KeyCode::Char('j'), _) => app.next(),
                (KeyCode::Up | KeyCode::Char('k'), _) => app.prev(),
                _ => {}
            }
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
            Constraint::Min(0),    // agents table
            Constraint::Length(1), // footer
        ])
        .split(area);

    draw_banner(f, layout[0], app);
    draw_agents(f, layout[1], app);
    draw_footer(f, layout[2], app);
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
    let bold = Style::default().add_modifier(Modifier::BOLD);
    let count = if app.agents.is_empty() {
        "0 agents".to_string()
    } else {
        let cur = app.table_state.selected().unwrap_or(0) + 1;
        format!("{}/{}", cur, app.agents.len())
    };
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(count, Style::default().fg(Color::Cyan)),
        Span::raw("    "),
        Span::styled("↑↓/jk", bold),
        Span::raw(" navigate    "),
        Span::styled("r", bold),
        Span::raw(" refresh    "),
        Span::styled("q/Esc", bold),
        Span::raw(" quit"),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(footer, area);
}
