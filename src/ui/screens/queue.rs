use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::state::AppState;

pub struct QueueScreen;

impl QueueScreen {
    pub fn render(&self, f: &mut Frame, area: Rect, state: &AppState) {
        if state.queue.is_empty() {
            let empty = Paragraph::new(" Queue is empty\n\n Add tracks to start listening")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .title(" Queue ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan)),
                );
            f.render_widget(empty, area);
            return;
        }

        let items: Vec<ListItem> = state
            .queue
            .iter()
            .enumerate()
            .map(|(i, track_id)| {
                let is_current = state.queue_index == Some(i);
                let track_info = state
                    .track_cache
                    .get(track_id)
                    .or_else(|| state.library.tracks.get(track_id))
                    .map(|t| format!("{} \u{2014} {}", t.artist, t.title))
                    .unwrap_or_else(|| track_id.clone());

                let prefix = if is_current { "\u{25b6} " } else { "   " };
                let style = if i == state.selected_index {
                    Style::default().fg(Color::Black).bg(Color::Cyan).bold()
                } else if is_current {
                    Style::default().fg(Color::Cyan).bold()
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(format!("{prefix}{track_info}")).style(style)
            })
            .collect();

        let queue_len = state.queue.len();
        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!(" Queue ({queue_len} items) "))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(Style::default().fg(Color::Cyan));

        f.render_widget(list, area);
    }
}
