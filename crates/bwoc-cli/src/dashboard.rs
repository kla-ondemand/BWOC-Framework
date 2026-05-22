//! `bwoc dashboard` — Phase 0 TUI shell.
//!
//! Right now: full-screen ratatui app that draws a centered title block
//! and a footer hotkey legend. Quits on `q`, `Esc`, or `Ctrl-C`. Restores
//! the terminal on any exit path (including panic) via a guard pattern.
//!
//! Future phases (per `notes/2026-05-22_tui-runtime-plan.md`):
//!   Phase 1 — populate main pane from `agents.toml` (id · status · backend · model)
//!   Phase 2 — detail pane reusing `doctor` health probes
//!   Phase 3 — Fluent i18n
//!   Phase 4 — log tail + editor handoff
//!
//! This file deliberately contains no agent/workspace lookups yet — the
//! Phase 0 goal is "prove the stack works without breaking the terminal."

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub struct DashboardArgs {
    // Phase 1+ will add: workspace path, lang, refresh interval, etc.
}

pub fn run(_args: DashboardArgs) -> i32 {
    // Non-TTY refusal — TUIs need a real terminal.
    use std::io::IsTerminal;
    if !io::stdout().is_terminal() {
        eprintln!(
            "bwoc dashboard: stdout is not a TTY. Use `bwoc list` / `bwoc status` for non-interactive output."
        );
        return 2;
    }

    let mut term = match setup_terminal() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("bwoc dashboard: failed to enter alt screen: {e}");
            return 1;
        }
    };

    let result = event_loop(&mut term);

    // Always tear down, even on panic — the Drop guard in `Terminal`
    // owns the alt-screen restore via crossterm. Belt-and-suspenders.
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

fn event_loop(term: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    loop {
        term.draw(draw_frame)?;

        // Poll with a short timeout so we COULD refresh on tick if we want.
        // Phase 0 doesn't tick anything, but the poll structure is here.
        if event::poll(Duration::from_millis(200))?
            && let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
        {
            match (code, modifiers) {
                (KeyCode::Char('q'), KeyModifiers::NONE) => return Ok(()),
                (KeyCode::Esc, _) => return Ok(()),
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(()),
                _ => {}
            }
        }
    }
}

fn draw_frame(f: &mut ratatui::Frame) {
    let area = f.area();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // banner
            Constraint::Min(0),    // body (Phase 1+ fills this)
            Constraint::Length(1), // footer
        ])
        .split(area);

    draw_banner(f, layout[0]);
    draw_body_placeholder(f, layout[1]);
    draw_footer(f, layout[2]);
}

fn draw_banner(f: &mut ratatui::Frame, area: Rect) {
    let title_style = Style::default()
        .fg(ratatui::style::Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let lines = vec![
        Line::from(Span::styled("BWOC Framework", title_style)),
        Line::from(Span::styled(
            format!("v{}", env!("CARGO_PKG_VERSION")),
            Style::default().add_modifier(Modifier::DIM),
        )),
        Line::from(""),
        Line::from("Buddhist Way of Coding — agent framework"),
        Line::from(Span::styled(
            "Phase 0 TUI shell — populated panes coming in Phase 1+",
            Style::default().add_modifier(Modifier::ITALIC),
        )),
    ];
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" dashboard ")
        .border_style(Style::default().fg(ratatui::style::Color::Cyan));
    let p = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(block);
    f.render_widget(p, area);
}

fn draw_body_placeholder(f: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" agents ")
        .border_style(Style::default().add_modifier(Modifier::DIM));
    let p = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "(empty — Phase 1 will populate from .bwoc/agents.toml)",
            Style::default().add_modifier(Modifier::DIM),
        )),
    ])
    .alignment(Alignment::Center)
    .block(block);
    f.render_widget(p, area);
}

fn draw_footer(f: &mut ratatui::Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" / "),
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" / "),
        Span::styled("Ctrl-C", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" quit"),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(footer, area);
}
