use crate::tui::state::AppState;
use crate::util::format_size;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, state: &mut AppState) {
    let height = area.height.saturating_sub(2) as usize; // borders
    state.tree_state.ensure_visible(height);

    let items: Vec<ListItem> = state
        .tree_state
        .visible_items
        .iter()
        .enumerate()
        .skip(state.tree_state.scroll_offset)
        .take(height)
        .map(|(idx, item)| {
            let indent = "  ".repeat(item.depth);
            let arrow = if item.is_dir {
                if item.has_children {
                    if item.is_expanded {
                        "▼ "
                    } else {
                        "▶ "
                    }
                } else {
                    "  "
                }
            } else {
                "  "
            };

            let name_color = if item.is_dir {
                Color::Blue
            } else if item.size > 100 * 1024 * 1024 {
                Color::Red
            } else if item.size > 10 * 1024 * 1024 {
                Color::Yellow
            } else {
                Color::White
            };

            let size_str = format_size(item.size);
            let pct_str = format!("{:5.1}%", item.pct_of_parent);

            let style = if idx == state.tree_state.cursor {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled(format!("{}{}", indent, arrow), style.fg(Color::Gray)),
                Span::styled(item.name.clone(), style.fg(name_color)),
                Span::styled(
                    format!("  {} {}", size_str, pct_str),
                    style.fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Tree View ");

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}
