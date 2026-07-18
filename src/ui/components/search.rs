use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::state::AppState;

pub struct SearchInput;

impl SearchInput {
    pub fn render(&self, f: &mut Frame, area: Rect, state: &AppState) {
        let border_style = if state.search_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .title(" Search ")
            .borders(Borders::ALL)
            .border_style(border_style);

        let input = Paragraph::new(state.search_query.as_str())
            .style(Style::default().fg(Color::White))
            .block(block);

        f.render_widget(input, area);

        if state.search_focused {
            let cursor_x = area.x + state.search_query.len() as u16 + 1;
            let cursor_y = area.y + 1;
            f.set_cursor_position(Position::new(
                cursor_x.min(area.x + area.width.saturating_sub(2)),
                cursor_y,
            ));
        }
    }

    pub fn handle_mouse_click(&self, x: u16, y: u16, area: Rect) -> bool {
        x >= area.x && x < area.x + area.width && y >= area.y && y < area.y + area.height
    }
}
