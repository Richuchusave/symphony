use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::state::AppState;
use crate::types::PlaybackStatus;
use crate::ui::input::Action;

pub struct PlayerBar;

impl PlayerBar {
    pub fn render(&self, f: &mut Frame, area: Rect, state: &AppState) {
        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray));
        let inner = block.inner(area);
        f.render_widget(block, area);

        let playback = &state.playback;
        let progress = playback.progress_pct();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(15),
                Constraint::Length(22),
                Constraint::Length(10),
            ])
            .split(inner);

        let icon_str = format!("{}", playback.status);
        let icon = Paragraph::new(icon_str).style(Style::default().fg(Color::Cyan));
        f.render_widget(icon, chunks[0]);

        let info = match state.current_track() {
            Some(t) => format!(" {} \u{2014} {} ", t.artist, t.title),
            None => " No track ".to_string(),
        };
        let track = Paragraph::new(info)
            .style(Style::default().fg(Color::White))
            .scroll((0, 0));
        f.render_widget(track, chunks[1]);

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
            .percent((progress * 100.0) as u16)
            .label(format!(
                " {} / {} ",
                playback.position_formatted(),
                playback.duration_formatted()
            ));
        f.render_widget(gauge, chunks[2]);

        let vol_pct = (playback.volume * 100.0) as u8;
        let vol_icon = if playback.volume > 0.0 { "\u{266a}" } else { "\u{1f507}" };
        let volume = Paragraph::new(format!(" {} {}% ", vol_icon, vol_pct))
            .style(Style::default().fg(Color::Green));
        f.render_widget(volume, chunks[3]);
    }

    pub fn handle_click(&self, x: u16, y: u16, area: Rect, state: &AppState) -> Option<Action> {
        if x < area.x || x >= area.x + area.width || y < area.y || y >= area.y + area.height {
            return None;
        }
        let rel_x = x.saturating_sub(area.x);
        if rel_x < 2 {
            if state.playback.status != PlaybackStatus::Stopped {
                return Some(Action::PlayPause);
            }
        }
        let width = area.width;
        if rel_x >= width.saturating_sub(10) && rel_x < width {
            let vol_fraction = (rel_x - (width - 10)) as f64 / 10.0;
            if vol_fraction > 0.5 {
                return Some(Action::VolumeUp);
            } else {
                return Some(Action::VolumeDown);
            }
        }
        None
    }
}
