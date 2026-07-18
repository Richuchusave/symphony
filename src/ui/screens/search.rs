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
        if state.search_focused {
            let cursor_x = chunks[0]
                .x
                .saturating_add(1)
                .saturating_add(state.search_query.chars().count() as u16)
                .min(chunks[0].right().saturating_sub(2));
            f.set_cursor_position((cursor_x, chunks[0].y.saturating_add(1)));
        }

        if state.search_query.is_empty() {
            let hint = Paragraph::new(
                " Type a search and press Enter. Use j/k to select a track, then Enter to play.",
            )
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(hint, chunks[1]);
            return;
        }

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
                .enumerate()
                .map(|(index, t)| {
                    let style = if !state.search_focused && index == state.selected_index {
                        Style::default().fg(Color::Black).bg(Color::Cyan).bold()
                    } else {
                        Style::default().fg(Color::White)
                    };
                    Line::from(format!(" \u{266b} {} \u{2014} {}", t.artist, t.title)).style(style)
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
