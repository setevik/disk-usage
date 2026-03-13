use crate::tui::state::AppState;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let content = if let Some(ref path) = state.ai_prompt_path {
        vec![
            Line::from(vec![
                Span::styled("Prompt exported: ", Style::default().fg(Color::Green)),
                Span::styled(
                    path.to_string_lossy().to_string(),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Run: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("claude < {}", path.display()),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![Span::styled(
                "Press Enter to export AI prompt...",
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(vec![Span::styled(
                "This will generate a structured analysis prompt for Claude.",
                Style::default().fg(Color::DarkGray),
            )]),
        ]
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" AI Prompt Export ")
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
}
