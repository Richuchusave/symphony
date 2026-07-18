use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct HelpScreen;

impl HelpScreen {
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let rows = [
            ("/", "Focus search"),
            ("Enter", "Search or play selected track"),
            ("Esc", "Leave search or go back"),
            ("j / k", "Select next / previous item"),
            ("Space", "Play / pause"),
            ("h / l", "Previous / next track"),
            ("Left / Right", "Seek backward / forward"),
            ("Up / Down", "Volume up / down"),
            ("z / x", "Toggle shuffle / repeat"),
            ("Ctrl+Q", "Open queue"),
            ("Ctrl+B", "Toggle sidebar"),
            ("q", "Quit"),
        ];

        let table = Table::new(
            rows.into_iter().map(|(key, action)| {
                Row::new([key, action]).style(Style::default().fg(Color::White))
            }),
            [Constraint::Length(16), Constraint::Min(24)],
        )
        .header(
            Row::new(["Key", "Action"])
                .style(Style::default().fg(Color::Cyan).bold())
                .bottom_margin(1),
        )
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .column_spacing(2);

        f.render_widget(table, area);
    }
}
