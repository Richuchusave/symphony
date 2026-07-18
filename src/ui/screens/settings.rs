use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::state::AppState;

pub struct SettingsScreen;

impl SettingsScreen {
    pub fn render(&self, f: &mut Frame, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(6),
                Constraint::Length(6),
                Constraint::Min(1),
            ])
            .split(area);

        let title = Paragraph::new(" Settings ")
            .style(Style::default().fg(Color::Cyan).bold())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        f.render_widget(title, chunks[0]);

        let cfg = &state.config;
        let lines1 = vec![
            Line::from(" General ").style(Style::default().fg(Color::Cyan).bold()),
            Line::from(format!(
                " Default Provider: {}    Log Level: {}",
                cfg.general.default_provider, cfg.general.log_level
            ))
            .style(Style::default().fg(Color::White)),
            Line::from(format!(
                " Check Updates: {}    Data Directory: {:?}",
                cfg.general.check_updates, cfg.general.data_dir
            ))
            .style(Style::default().fg(Color::DarkGray)),
        ];
        let general = Paragraph::new(lines1).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(general, chunks[1]);

        let lines2 = vec![
            Line::from(" Playback ").style(Style::default().fg(Color::Cyan).bold()),
            Line::from(format!(
                " Volume: {}%    Seek Step: {}s    Crossfade: {}    Gapless: {}",
                (cfg.playback.default_volume * 100.0) as u8,
                cfg.playback.seek_step_seconds,
                cfg.playback.crossfade,
                cfg.playback.gapless,
            ))
            .style(Style::default().fg(Color::White)),
        ];
        let playback = Paragraph::new(lines2).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(playback, chunks[2]);

        let lines3 =
            vec![
                Line::from(" User Interface ").style(Style::default().fg(Color::Cyan).bold()),
                Line::from(format!(
                " Sidebar Width: {}    Show Clock: {}    Show Cover Art: {}    Mouse Support: {}",
                cfg.ui.sidebar_width, cfg.ui.show_clock, cfg.ui.show_cover_art, cfg.ui.mouse_support
            ))
                .style(Style::default().fg(Color::White)),
            ];
        let ui_section = Paragraph::new(lines3).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(ui_section, chunks[3]);
    }
}
