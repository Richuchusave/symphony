use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::state::AppState;

pub struct PlaylistsScreen;

impl PlaylistsScreen {
    pub fn render(&self, f: &mut Frame, area: Rect, state: &AppState) {
        let mut playlist_data: Vec<&crate::types::Playlist> =
            state.library.playlists.values().collect();
        playlist_data.sort_by(|a, b| a.name.cmp(&b.name));
        let playlists: Vec<ListItem> = playlist_data
            .iter()
            .map(|p| {
                let track_count = p.tracks.len();
                ListItem::new(format!(" {}  ({} tracks)", p.name, track_count))
            })
            .collect();

        let list = List::new(playlists)
            .block(
                Block::default()
                    .title(" Playlists ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(Style::default().fg(Color::Cyan));

        f.render_widget(list, area);
    }
}
