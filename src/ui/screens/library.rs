use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::state::AppState;

pub struct LibraryScreen;

impl LibraryScreen {
    pub fn render(&self, f: &mut Frame, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(6), Constraint::Min(1)])
            .split(area);

        let total_dur = state.library.duration_total();
        let hours = total_dur.as_secs() / 3600;
        let mins = (total_dur.as_secs() % 3600) / 60;

        let stats = vec![
            Line::from(" Library ").style(Style::default().fg(Color::Cyan).bold()),
            Line::from(""),
            Line::from(format!(
                " Tracks: {}  |  Albums: {}  |  Artists: {}  |  Playlists: {}  |  Total: {}h {}m",
                state.library.track_count(),
                state.library.album_count(),
                state.library.artist_count(),
                state.library.playlist_count(),
                hours,
                mins,
            ))
            .style(Style::default().fg(Color::White)),
        ];
        let stats_block = Paragraph::new(stats)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));
        f.render_widget(stats_block, chunks[0]);

        let mut track_data: Vec<&crate::types::Track> = state.library.tracks.values().collect();
        track_data.sort_by(|a, b| {
            a.artist
                .cmp(&b.artist)
                .then(a.title.cmp(&b.title))
        });
        let lines: Vec<ListItem> = track_data
            .iter()
            .map(|t| {
                ListItem::new(format!(
                    " {} \u{2014} {}  [{}]",
                    t.artist,
                    t.title,
                    t.duration_formatted()
                ))
            })
            .collect();

        let list = List::new(lines)
            .block(
                Block::default()
                    .title(" All Tracks ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .highlight_style(Style::default().fg(Color::Cyan));
        f.render_widget(list, chunks[1]);
    }
}
