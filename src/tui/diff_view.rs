use crate::snapshot::DiffStatus;
use crate::tui::state::AppState;
use crate::util::format_size;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let diff = match &state.diff {
        Some(d) => d,
        None => {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Diff View ");
            let msg = Row::new(vec![Cell::from(
                "No snapshot loaded. Use --diff <file> to compare.",
            )]);
            let table = Table::new(vec![msg], [ratatui::layout::Constraint::Min(0)])
                .block(block);
            f.render_widget(table, area);
            return;
        }
    };

    let header = Row::new(vec![
        Cell::from("Path"),
        Cell::from("Old Size"),
        Cell::from("New Size"),
        Cell::from("Delta"),
        Cell::from("Status"),
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    let height = area.height.saturating_sub(4) as usize;
    let rows: Vec<Row> = diff
        .entries
        .iter()
        .skip(state.diff_cursor)
        .take(height)
        .map(|entry| {
            let status_color = match entry.status {
                DiffStatus::New => Color::Green,
                DiffStatus::Deleted => Color::Red,
                DiffStatus::Changed => Color::Yellow,
                DiffStatus::Unchanged => Color::Gray,
            };

            let delta_str = if entry.delta >= 0 {
                format!("+{}", format_size(entry.delta as u64))
            } else {
                format!("-{}", format_size((-entry.delta) as u64))
            };

            Row::new(vec![
                Cell::from(entry.path.clone()),
                Cell::from(
                    entry
                        .old_size
                        .map(format_size)
                        .unwrap_or_else(|| "-".to_string()),
                ),
                Cell::from(
                    entry
                        .new_size
                        .map(format_size)
                        .unwrap_or_else(|| "-".to_string()),
                ),
                Cell::from(delta_str),
                Cell::from(entry.status.to_string()).style(Style::default().fg(status_color)),
            ])
        })
        .collect();

    let widths = [
        ratatui::layout::Constraint::Percentage(40),
        ratatui::layout::Constraint::Percentage(15),
        ratatui::layout::Constraint::Percentage(15),
        ratatui::layout::Constraint::Percentage(15),
        ratatui::layout::Constraint::Percentage(15),
    ];

    let title = format!(
        " Diff View — Total: {} ",
        diff.format_total_delta()
    );

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(table, area);
}
