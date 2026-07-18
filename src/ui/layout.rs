use ratatui::prelude::*;

use crate::config::Config;

pub struct Rects {
    pub sidebar: Rect,
    pub main: Rect,
    pub player_bar: Rect,
}

pub struct AppLayout {
    pub sidebar_visible: bool,
}

impl AppLayout {
    pub fn render_area(&self, area: Rect, config: &Config) -> Rects {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(area);

        let player_bar = vertical[1];
        let main_area = vertical[0];

        if self.sidebar_visible {
            let sidebar_width = config.ui.sidebar_width;
            let horizontal = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(sidebar_width), Constraint::Min(1)])
                .split(main_area);
            Rects {
                sidebar: horizontal[0],
                main: horizontal[1],
                player_bar,
            }
        } else {
            Rects {
                sidebar: Rect::default(),
                main: main_area,
                player_bar,
            }
        }
    }
}
