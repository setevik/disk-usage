use crate::tree::{DirEntry, DirNode, FileEntry};
use crate::tui::state::AppState;
use crate::util::{epoch_to_string, format_size};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let stale_threshold = state
        .scan
        .timestamp
        .saturating_sub(state.config.stale_days * 86400);
    let mut stale_files: Vec<&FileEntry> = Vec::new();
    collect_stale(&state.scan.root, stale_threshold, &mut stale_files);
    stale_files.sort_by(|a, b| b.size.cmp(&a.size));

    let header = Row::new(vec![
        Cell::from("Path"),
        Cell::from("Size"),
        Cell::from("Last Accessed"),
        Cell::from("Last Modified"),
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    let height = area.height.saturating_sub(4) as usize;
    let rows: Vec<Row> = stale_files
        .iter()
        .skip(state.stale_cursor)
        .take(height)
        .map(|f| {
            Row::new(vec![
                Cell::from(f.path.to_string_lossy().to_string())
                    .style(Style::default().fg(Color::Gray)),
                Cell::from(format_size(f.size)),
                Cell::from(epoch_to_string(f.atime)),
                Cell::from(epoch_to_string(f.mtime)),
            ])
        })
        .collect();

    let widths = [
        ratatui::layout::Constraint::Percentage(45),
        ratatui::layout::Constraint::Percentage(15),
        ratatui::layout::Constraint::Percentage(20),
        ratatui::layout::Constraint::Percentage(20),
    ];

    let title = format!(
        " Stale Files (>{} days, ≥10 MiB) — {} found ",
        state.config.stale_days,
        stale_files.len()
    );

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(table, area);
}

fn collect_stale<'a>(
    node: &'a DirNode,
    threshold: u64,
    result: &mut Vec<&'a FileEntry>,
) {
    for child in &node.children {
        match child {
            DirEntry::File(f) => {
                if f.atime < threshold && f.mtime < threshold && f.size >= 10 * 1024 * 1024 {
                    result.push(f);
                }
            }
            DirEntry::Dir(d) => collect_stale(d, threshold, result),
        }
    }
}
