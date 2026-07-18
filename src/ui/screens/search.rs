use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::state::AppState;

pub struct SearchScreen;

impl SearchScreen {
    pub fn render(&self, f: &mut Frame, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);

        let border_style = if state.search_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let input = Paragraph::new(state.search_query.as_str())
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .title(" Search ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            );
        f.render_widget(input, chunks[0]);

        if state.search_results.total_results == 0 && !state.search_query.is_empty() {
            let empty = Paragraph::new(" No results found")
                .style(Style::default().fg(Color::DarkGray))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(empty, chunks[1]);
            return;
        }

        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3 + state.search_results.tracks.len().min(10) as u16),
                Constraint::Length(3 + state.search_results.albums.len().min(5) as u16),
                Constraint::Length(3 + state.search_results.artists.len().min(5) as u16),
            ])
            .split(chunks[1]);

        if !state.search_results.tracks.is_empty() {
            let track_lines: Vec<Line> = state
                .search_results
                .tracks
                .iter()
                .map(|t| {
                    Line::from(format!(" \u{266b} {} \u{2014} {}", t.artist, t.title))
                        .style(Style::default().fg(Color::White))
                })
                .collect();
            let tracks = Paragraph::new(track_lines)
                .block(Block::default().title(" Tracks ").borders(Borders::ALL));
            f.render_widget(tracks, sections[0]);
        }

        if !state.search_results.albums.is_empty() {
            let album_lines: Vec<Line> = state
                .search_results
                .albums
                .iter()
                .map(|a| {
                    Line::from(format!(" \u{f0025} {} \u{2014} {}", a.artist, a.title))
                        .style(Style::default().fg(Color::White))
                })
                .collect();
            let albums = Paragraph::new(album_lines)
                .block(Block::default().title(" Albums ").borders(Borders::ALL));
            f.render_widget(albums, sections[1]);
        }

        if !state.search_results.artists.is_empty() {
            let artist_lines: Vec<Line> = state
                .search_results
                .artists
                .iter()
                .map(|a| {
                    Line::from(format!(" \u{f0803} {}", a.name))
                        .style(Style::default().fg(Color::White))
                })
                .collect();
            let artists = Paragraph::new(artist_lines)
                .block(Block::default().title(" Artists ").borders(Borders::ALL));
            f.render_widget(artists, sections[2]);
        }
    }
}
