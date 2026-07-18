pub mod albums;
pub mod artists;
pub mod help;
pub mod home;
pub mod library;
pub mod playlists;
pub mod queue;
pub mod search;
pub mod settings;

use ratatui::prelude::*;

use crate::state::AppState;
use crate::types::Screen;

pub fn render_screen(f: &mut Frame, area: Rect, state: &AppState) {
    match state.current_screen {
        Screen::Home => home::HomeScreen.render(f, area, state),
        Screen::Search => search::SearchScreen.render(f, area, state),
        Screen::Library => library::LibraryScreen.render(f, area, state),
        Screen::Albums | Screen::AlbumDetail(_) => albums::AlbumsScreen.render(f, area, state),
        Screen::Artists | Screen::ArtistDetail(_) => artists::ArtistsScreen.render(f, area, state),
        Screen::Playlists | Screen::PlaylistDetail(_) => {
            playlists::PlaylistsScreen.render(f, area, state)
        }
        Screen::Queue => queue::QueueScreen.render(f, area, state),
        Screen::Downloads => home::HomeScreen.render(f, area, state),
        Screen::Settings => settings::SettingsScreen.render(f, area, state),
        Screen::Help => help::HelpScreen.render(f, area),
    }
}
