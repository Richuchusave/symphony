use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::state::AppState;

pub struct HomeScreen;

impl HomeScreen {
    pub fn render(&self, f: &mut Frame, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(8), Constraint::Length(6), Constraint::Min(1)])
            .split(area);

        let welcome = Paragraph::new(vec![
            Line::from(" Welcome to Symphony").style(Style::default().fg(Color::Cyan).bold()),
            Line::from(""),
            Line::from(" Your terminal music player").style(Style::default().fg(Color::White)),
            Line::from(""),
            Line::from(" Press ? for help, / to search").style(Style::default().fg(Color::DarkGray)),
        ])
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));
        f.render_widget(welcome, chunks[0]);

        let stats = vec![
            Line::from(" Library Stats ").style(Style::default().fg(Color::Cyan).bold()),
            Line::from(""),
            Line::from(format!(
                " Tracks: {}   Albums: {}   Artists: {}   Playlists: {}",
                state.library.track_count(),
                state.library.album_count(),
                state.library.artist_count(),
                state.library.playlist_count(),
            ))
            .style(Style::default().fg(Color::White)),
        ];
        let stats_block = Paragraph::new(stats)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
        f.render_widget(stats_block, chunks[1]);

        let now_playing = match state.current_track() {
            Some(t) => format!(
                " Now Playing: {} \u{2014} {}  ({})",
                t.artist,
                t.title,
                t.duration_formatted()
            ),
            None => " No track currently playing".to_string(),
        };
        let playing = Paragraph::new(now_playing)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title(" Now Playing "));
        f.render_widget(playing, chunks[2]);
    }
}
