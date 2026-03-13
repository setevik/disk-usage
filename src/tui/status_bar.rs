use crate::tui::state::{AppState, Tab};
use crate::util::{epoch_to_string, format_size};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let tab_spans: Vec<Span> = Tab::all()
        .iter()
        .map(|tab| {
            let style = if *tab == state.active_tab {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            Span::styled(format!(" {} ", tab.label()), style)
        })
        .collect();

    let keybinds = match state.active_tab {
        Tab::Tree => "j/k:nav Enter:expand Bksp:parent",
        Tab::Types => "",
        Tab::Diff => "j/k:scroll",
        Tab::Stale => "j/k:scroll",
    };

    let info = format!(
        " {} files | {} | {} | A:AI ?:help q:quit {}",
        state.scan.total_files,
        format_size(state.scan.total_size),
        epoch_to_string(state.scan.timestamp),
        keybinds,
    );

    let mut spans = tab_spans;
    spans.push(Span::styled(info, Style::default().fg(Color::Gray)));

    let line = Line::from(spans);
    let bar = Paragraph::new(line);
    f.render_widget(bar, area);
}
