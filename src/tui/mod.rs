mod ai_panel;
mod diff_view;
pub mod state;
mod stale_view;
mod status_bar;
mod tree_view;
mod types_view;

use crate::config::Config;
use crate::prompt;
use crate::snapshot::DiffResult;
use crate::tree::ScanResult;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Terminal;
use state::{AppState, Tab};
use std::io;
use std::time::Duration;

pub fn run(scan: ScanResult, diff: Option<DiffResult>, config: Config) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new(scan, diff, config);

    loop {
        terminal.draw(|f| ui(f, &mut state))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if state.show_help {
                    state.show_help = false;
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') => state.quit = true,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        state.quit = true
                    }
                    KeyCode::Char('1') => state.active_tab = Tab::Tree,
                    KeyCode::Char('2') => state.active_tab = Tab::Types,
                    KeyCode::Char('3') => state.active_tab = Tab::Diff,
                    KeyCode::Char('4') => state.active_tab = Tab::Stale,
                    KeyCode::Char('?') => state.show_help = !state.show_help,
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        if state.ai_drawer_open && state.ai_prompt_path.is_none() {
                            // Export prompt on Enter or second A press
                            if let Ok(path) =
                                prompt::export(&state.scan, state.diff.as_ref(), &state.config)
                            {
                                state.ai_prompt_path = Some(path);
                            }
                        }
                        state.ai_drawer_open = !state.ai_drawer_open;
                    }
                    // Tab-specific keys
                    _ => match state.active_tab {
                        Tab::Tree => handle_tree_keys(key.code, &mut state),
                        Tab::Diff => handle_scroll_keys(key.code, &mut state.diff_cursor),
                        Tab::Stale => handle_scroll_keys(key.code, &mut state.stale_cursor),
                        Tab::Types => {}
                    },
                }
            }

            if let Event::Resize(_, _) = event::read().unwrap_or(Event::FocusGained) {
                // Terminal will redraw on next loop
            }
        }

        if state.quit {
            break;
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Print AI command if prompt was exported
    if let Some(ref path) = state.ai_prompt_path {
        println!("Prompt exported to: {}", path.display());
        println!("Run:  claude < {}", path.display());
    }

    Ok(())
}

fn ui(f: &mut ratatui::Frame, state: &mut AppState) {
    let size = f.area();

    let mut constraints = vec![
        Constraint::Min(0),    // Main content
        Constraint::Length(1), // Status bar
    ];

    if state.ai_drawer_open {
        constraints = vec![
            Constraint::Min(0),    // Main content
            Constraint::Length(6), // AI panel
            Constraint::Length(1), // Status bar
        ];
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(size);

    // Main content area
    let content_area = chunks[0];
    match state.active_tab {
        Tab::Tree => tree_view::render(f, content_area, state),
        Tab::Types => types_view::render(f, content_area, state),
        Tab::Diff => diff_view::render(f, content_area, state),
        Tab::Stale => stale_view::render(f, content_area, state),
    }

    // AI panel if open
    if state.ai_drawer_open {
        ai_panel::render(f, chunks[1], state);
        status_bar::render(f, chunks[2], state);
    } else {
        status_bar::render(f, chunks[1], state);
    }

    // Help overlay
    if state.show_help {
        render_help(f, size);
    }
}

fn handle_tree_keys(code: KeyCode, state: &mut AppState) {
    let root = state.scan.root.clone();
    match code {
        KeyCode::Char('j') | KeyCode::Down => state.tree_state.move_down(),
        KeyCode::Char('k') | KeyCode::Up => state.tree_state.move_up(),
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
            state.tree_state.toggle_expand(&root)
        }
        KeyCode::Backspace | KeyCode::Char('h') | KeyCode::Left => {
            state.tree_state.go_to_parent(&root)
        }
        _ => {}
    }
}

fn handle_scroll_keys(code: KeyCode, cursor: &mut usize) {
    match code {
        KeyCode::Char('j') | KeyCode::Down => *cursor = cursor.saturating_add(1),
        KeyCode::Char('k') | KeyCode::Up => *cursor = cursor.saturating_sub(1),
        _ => {}
    }
}

fn render_help(f: &mut ratatui::Frame, area: Rect) {
    let help_width = 50u16.min(area.width.saturating_sub(4));
    let help_height = 16u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(help_width)) / 2;
    let y = (area.height.saturating_sub(help_height)) / 2;
    let help_area = Rect::new(x, y, help_width, help_height);

    f.render_widget(Clear, help_area);

    let lines = vec![
        Line::from(Span::styled(
            "Keybindings",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from("  1-4      Switch tabs"),
        Line::from("  j/k/↑/↓  Navigate"),
        Line::from("  Enter    Expand/collapse directory"),
        Line::from("  h/←      Collapse / go to parent"),
        Line::from("  l/→      Expand directory"),
        Line::from("  Bksp     Go to parent"),
        Line::from("  A        Toggle AI prompt panel"),
        Line::from("  ?        Toggle this help"),
        Line::from("  q        Quit"),
        Line::from("  Ctrl+C   Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help ")
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, help_area);
}
