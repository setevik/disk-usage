use crate::categories::{aggregate_by_category, Category};
use crate::tui::state::AppState;
use crate::util::format_size;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Bar, BarChart, BarGroup, Block, Borders};
use ratatui::Frame;

fn category_color(cat: &Category) -> Color {
    match cat {
        Category::BuildArtifacts => Color::Red,
        Category::Logs => Color::Yellow,
        Category::Media => Color::Magenta,
        Category::Archives => Color::Cyan,
        Category::Docker => Color::Blue,
        Category::Vcs => Color::Green,
        Category::NodeModules => Color::LightRed,
        Category::Other => Color::Gray,
    }
}

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let stats = aggregate_by_category(&state.scan.root);

    let bars: Vec<Bar> = stats
        .iter()
        .filter(|s| s.total_size > 0)
        .map(|s| {
            let label = format!("{} ({})", s.category, format_size(s.total_size));
            // Value in MiB for readable bar lengths
            let value_mib = s.total_size / (1024 * 1024);
            Bar::default()
                .value(value_mib)
                .label(label.into())
                .style(Style::default().fg(category_color(&s.category)))
                .value_style(Style::default().fg(Color::White))
        })
        .collect();

    let group = BarGroup::default().bars(&bars);

    let chart = BarChart::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" File Types (by size in MiB) "),
        )
        .data(group)
        .bar_width(3)
        .bar_gap(1)
        .direction(ratatui::layout::Direction::Vertical);

    f.render_widget(chart, area);
}
