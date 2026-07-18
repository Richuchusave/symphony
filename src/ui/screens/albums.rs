use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::state::AppState;

pub struct AlbumsScreen;

impl AlbumsScreen {
    pub fn render(&self, f: &mut Frame, area: Rect, state: &AppState) {
        let mut album_data: Vec<&crate::types::Album> = state.library.albums.values().collect();
        album_data.sort_by(|a, b| a.artist.cmp(&b.artist).then(a.title.cmp(&b.title)));
        let albums: Vec<ListItem> = album_data
            .iter()
            .map(|a| {
                ListItem::new(format!(
                    " {} \u{2014} {}  ({} tracks)",
                    a.artist, a.title, a.track_count
                ))
            })
            .collect();

        let list = List::new(albums)
            .block(
                Block::default()
                    .title(" Albums ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(Style::default().fg(Color::Cyan));

        f.render_widget(list, area);
    }
}
