use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::state::AppState;
use crate::types::Screen;

pub struct Sidebar {
    pub items: Vec<(Screen, &'static str, &'static str)>,
    pub selected_index: usize,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            items: vec![
                (Screen::Home, "\u{f021c}", "Home"),
                (Screen::Search, "\u{f0948}", "Search"),
                (Screen::Library, "\u{f02ed}", "Library"),
                (Screen::Albums, "\u{f0025}", "Albums"),
                (Screen::Artists, "\u{f0803}", "Artists"),
                (Screen::Playlists, "\u{f02cb}", "Playlists"),
                (Screen::Queue, "\u{f0661}", "Queue"),
                (Screen::Downloads, "\u{f1d85}", "Downloads"),
                (Screen::Settings, "\u{f0493}", "Settings"),
            ],
            selected_index: 0,
        }
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Sidebar {
    pub fn render(&self, f: &mut Frame, area: Rect, state: &AppState) {
        let block = Block::default()
            .title(" Symphony ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let list_items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, (screen, icon, label))| {
                let is_current = *screen == state.current_screen;
                let is_selected = i == self.selected_index && state.sidebar_focused;

                let style = if is_current {
                    Style::default().fg(Color::Cyan).bold()
                } else if is_selected {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if is_current { "\u{25b8} " } else { "  " };
                ListItem::new(format!("{}{} {}", prefix, icon, label)).style(style)
            })
            .collect();

        let list = List::new(list_items).block(block);
        f.render_widget(list, area);
    }

    pub fn handle_click(&self, x: u16, y: u16, area: Rect) -> Option<Screen> {
        if x < area.x
            || x >= area.x + area.width
            || y < area.y + 1
            || y >= area.y + area.height - 1
        {
            return None;
        }

        let y_offset = (y - area.y - 1) as usize;
        if y_offset < self.items.len() {
            Some(self.items[y_offset].0.clone())
        } else {
            None
        }
    }
}
