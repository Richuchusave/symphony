use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::state::AppState;

pub struct ArtistsScreen;

impl ArtistsScreen {
    pub fn render(&self, f: &mut Frame, area: Rect, state: &AppState) {
        let mut artist_data: Vec<&crate::types::Artist> = state.library.artists.values().collect();
        artist_data.sort_by(|a, b| a.name.cmp(&b.name));
        let artists: Vec<ListItem> = artist_data
            .iter()
            .map(|a| ListItem::new(format!(" {}  ({} albums)", a.name, a.album_count)))
            .collect();

        let list = List::new(artists)
            .block(
                Block::default()
                    .title(" Artists ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(Style::default().fg(Color::Cyan));

        f.render_widget(list, area);
    }
}
