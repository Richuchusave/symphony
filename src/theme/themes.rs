use ratatui::style::{Color, Modifier, Style};
use std::collections::HashMap;

use crate::errors::*;

pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
    pub styles: ThemeStyles,
}

pub struct ThemeColors {
    pub background: Color,
    pub surface: Color,
    pub surface_light: Color,
    pub primary: Color,
    pub primary_dim: Color,
    pub secondary: Color,
    pub accent: Color,
    pub text: Color,
    pub text_dim: Color,
    pub text_bright: Color,
    pub highlight: Color,
    pub error: Color,
    pub success: Color,
    pub warning: Color,
    pub border: Color,
    pub progress_bar: Color,
    pub progress_bar_track: Color,
    pub sidebar_bg: Color,
    pub sidebar_selected: Color,
    pub search_bg: Color,
    pub player_bar_bg: Color,
}

pub struct ThemeStyles {
    pub title: Style,
    pub subtitle: Style,
    pub selected: Style,
    pub highlighted: Style,
    pub dim: Style,
    pub error: Style,
    pub success: Style,
    pub link: Style,
    pub active_tab: Style,
    pub inactive_tab: Style,
}

impl Theme {
    pub fn new(name: &str, colors: ThemeColors, styles: ThemeStyles) -> Self {
        Self {
            name: name.to_string(),
            colors,
            styles,
        }
    }

    pub fn default() -> Self {
        default_dark()
    }

    pub fn from_name(name: &str) -> Result<Self> {
        Self::by_name(name)
            .ok_or_else(|| SymphonyError::theme(format!("Unknown theme: {name}")))
    }

    pub fn by_name(name: &str) -> Option<Theme> {
        match name {
            "default" => Some(default_dark()),
            "catppuccin-mocha" => Some(catppuccin_mocha()),
            "dracula" => Some(dracula()),
            "nord" => Some(nord()),
            _ => None,
        }
    }

    pub fn customize(mut self, overrides: HashMap<String, String>) -> Result<Self> {
        for (key, value) in overrides {
            let color = parse_color(&value)?;
            match key.as_str() {
                "background" => self.colors.background = color,
                "surface" => self.colors.surface = color,
                "surface_light" => self.colors.surface_light = color,
                "primary" => self.colors.primary = color,
                "primary_dim" => self.colors.primary_dim = color,
                "secondary" => self.colors.secondary = color,
                "accent" => self.colors.accent = color,
                "text" => self.colors.text = color,
                "text_dim" => self.colors.text_dim = color,
                "text_bright" => self.colors.text_bright = color,
                "highlight" => self.colors.highlight = color,
                "error" => self.colors.error = color,
                "success" => self.colors.success = color,
                "warning" => self.colors.warning = color,
                "border" => self.colors.border = color,
                "progress_bar" => self.colors.progress_bar = color,
                "progress_bar_track" => self.colors.progress_bar_track = color,
                "sidebar_bg" => self.colors.sidebar_bg = color,
                "sidebar_selected" => self.colors.sidebar_selected = color,
                "search_bg" => self.colors.search_bg = color,
                "player_bar_bg" => self.colors.player_bar_bg = color,
                _ => {
                    return Err(SymphonyError::theme(format!(
                        "Unknown theme color key: {key}"
                    )))
                }
            }
        }
        Ok(self)
    }
}

fn default_dark() -> Theme {
    let colors = ThemeColors {
        background: Color::Rgb(15, 15, 20),
        surface: Color::Rgb(22, 22, 30),
        surface_light: Color::Rgb(30, 30, 42),
        primary: Color::Rgb(0, 150, 255),
        primary_dim: Color::Rgb(0, 110, 200),
        secondary: Color::Rgb(100, 180, 255),
        accent: Color::Rgb(0, 200, 200),
        text: Color::Rgb(220, 220, 230),
        text_dim: Color::Rgb(130, 130, 150),
        text_bright: Color::Rgb(255, 255, 255),
        highlight: Color::Rgb(0, 120, 220),
        error: Color::Rgb(230, 60, 60),
        success: Color::Rgb(60, 200, 100),
        warning: Color::Rgb(255, 180, 50),
        border: Color::Rgb(60, 60, 80),
        progress_bar: Color::Rgb(0, 150, 255),
        progress_bar_track: Color::Rgb(30, 30, 42),
        sidebar_bg: Color::Rgb(18, 18, 26),
        sidebar_selected: Color::Rgb(0, 100, 200),
        search_bg: Color::Rgb(25, 25, 36),
        player_bar_bg: Color::Rgb(18, 18, 26),
    };
    let styles = ThemeStyles {
        title: Style::default().fg(colors.text_bright).add_modifier(Modifier::BOLD),
        subtitle: Style::default().fg(colors.text_dim),
        selected: Style::default()
            .fg(colors.text_bright)
            .bg(colors.highlight)
            .add_modifier(Modifier::BOLD),
        highlighted: Style::default().fg(colors.primary).add_modifier(Modifier::BOLD),
        dim: Style::default().fg(colors.text_dim),
        error: Style::default().fg(colors.error).add_modifier(Modifier::BOLD),
        success: Style::default().fg(colors.success),
        link: Style::default().fg(colors.secondary).add_modifier(Modifier::UNDERLINED),
        active_tab: Style::default()
            .fg(colors.text_bright)
            .bg(colors.surface)
            .add_modifier(Modifier::BOLD),
        inactive_tab: Style::default().fg(colors.text_dim).bg(colors.background),
    };
    Theme {
        name: "default".to_string(),
        colors,
        styles,
    }
}

fn catppuccin_mocha() -> Theme {
    let colors = ThemeColors {
        background: Color::Rgb(30, 30, 46),
        surface: Color::Rgb(49, 50, 68),
        surface_light: Color::Rgb(69, 71, 90),
        primary: Color::Rgb(137, 180, 250),
        primary_dim: Color::Rgb(116, 163, 240),
        secondary: Color::Rgb(148, 226, 213),
        accent: Color::Rgb(243, 139, 168),
        text: Color::Rgb(205, 214, 244),
        text_dim: Color::Rgb(147, 153, 178),
        text_bright: Color::Rgb(255, 255, 255),
        highlight: Color::Rgb(137, 180, 250),
        error: Color::Rgb(243, 139, 168),
        success: Color::Rgb(166, 227, 161),
        warning: Color::Rgb(249, 226, 175),
        border: Color::Rgb(69, 71, 90),
        progress_bar: Color::Rgb(137, 180, 250),
        progress_bar_track: Color::Rgb(49, 50, 68),
        sidebar_bg: Color::Rgb(24, 24, 37),
        sidebar_selected: Color::Rgb(137, 180, 250),
        search_bg: Color::Rgb(49, 50, 68),
        player_bar_bg: Color::Rgb(24, 24, 37),
    };
    let styles = ThemeStyles {
        title: Style::default().fg(colors.text_bright).add_modifier(Modifier::BOLD),
        subtitle: Style::default().fg(colors.text_dim),
        selected: Style::default()
            .fg(colors.background)
            .bg(colors.primary)
            .add_modifier(Modifier::BOLD),
        highlighted: Style::default().fg(colors.accent).add_modifier(Modifier::BOLD),
        dim: Style::default().fg(colors.text_dim),
        error: Style::default().fg(colors.error).add_modifier(Modifier::BOLD),
        success: Style::default().fg(colors.success),
        link: Style::default().fg(colors.primary).add_modifier(Modifier::UNDERLINED),
        active_tab: Style::default()
            .fg(colors.text_bright)
            .bg(colors.surface)
            .add_modifier(Modifier::BOLD),
        inactive_tab: Style::default().fg(colors.text_dim).bg(colors.background),
    };
    Theme {
        name: "catppuccin-mocha".to_string(),
        colors,
        styles,
    }
}

fn dracula() -> Theme {
    let colors = ThemeColors {
        background: Color::Rgb(40, 42, 54),
        surface: Color::Rgb(68, 71, 90),
        surface_light: Color::Rgb(98, 102, 120),
        primary: Color::Rgb(189, 147, 249),
        primary_dim: Color::Rgb(165, 120, 230),
        secondary: Color::Rgb(139, 233, 253),
        accent: Color::Rgb(255, 121, 198),
        text: Color::Rgb(248, 248, 242),
        text_dim: Color::Rgb(98, 102, 120),
        text_bright: Color::Rgb(255, 255, 255),
        highlight: Color::Rgb(189, 147, 249),
        error: Color::Rgb(255, 85, 85),
        success: Color::Rgb(80, 250, 123),
        warning: Color::Rgb(241, 250, 140),
        border: Color::Rgb(68, 71, 90),
        progress_bar: Color::Rgb(189, 147, 249),
        progress_bar_track: Color::Rgb(68, 71, 90),
        sidebar_bg: Color::Rgb(33, 34, 44),
        sidebar_selected: Color::Rgb(189, 147, 249),
        search_bg: Color::Rgb(68, 71, 90),
        player_bar_bg: Color::Rgb(33, 34, 44),
    };
    let styles = ThemeStyles {
        title: Style::default().fg(colors.text_bright).add_modifier(Modifier::BOLD),
        subtitle: Style::default().fg(colors.text_dim),
        selected: Style::default()
            .fg(colors.background)
            .bg(colors.primary)
            .add_modifier(Modifier::BOLD),
        highlighted: Style::default().fg(colors.accent).add_modifier(Modifier::BOLD),
        dim: Style::default().fg(colors.text_dim),
        error: Style::default().fg(colors.error).add_modifier(Modifier::BOLD),
        success: Style::default().fg(colors.success),
        link: Style::default().fg(colors.primary).add_modifier(Modifier::UNDERLINED),
        active_tab: Style::default()
            .fg(colors.text_bright)
            .bg(colors.surface)
            .add_modifier(Modifier::BOLD),
        inactive_tab: Style::default().fg(colors.text_dim).bg(colors.background),
    };
    Theme {
        name: "dracula".to_string(),
        colors,
        styles,
    }
}

fn nord() -> Theme {
    let colors = ThemeColors {
        background: Color::Rgb(46, 52, 64),
        surface: Color::Rgb(59, 66, 82),
        surface_light: Color::Rgb(76, 86, 106),
        primary: Color::Rgb(136, 192, 208),
        primary_dim: Color::Rgb(105, 170, 190),
        secondary: Color::Rgb(163, 190, 140),
        accent: Color::Rgb(208, 135, 112),
        text: Color::Rgb(236, 239, 244),
        text_dim: Color::Rgb(147, 153, 178),
        text_bright: Color::Rgb(255, 255, 255),
        highlight: Color::Rgb(136, 192, 208),
        error: Color::Rgb(191, 97, 106),
        success: Color::Rgb(163, 190, 140),
        warning: Color::Rgb(235, 203, 139),
        border: Color::Rgb(76, 86, 106),
        progress_bar: Color::Rgb(136, 192, 208),
        progress_bar_track: Color::Rgb(59, 66, 82),
        sidebar_bg: Color::Rgb(40, 45, 56),
        sidebar_selected: Color::Rgb(136, 192, 208),
        search_bg: Color::Rgb(59, 66, 82),
        player_bar_bg: Color::Rgb(40, 45, 56),
    };
    let styles = ThemeStyles {
        title: Style::default().fg(colors.text_bright).add_modifier(Modifier::BOLD),
        subtitle: Style::default().fg(colors.text_dim),
        selected: Style::default()
            .fg(colors.background)
            .bg(colors.primary)
            .add_modifier(Modifier::BOLD),
        highlighted: Style::default().fg(colors.primary).add_modifier(Modifier::BOLD),
        dim: Style::default().fg(colors.text_dim),
        error: Style::default().fg(colors.error).add_modifier(Modifier::BOLD),
        success: Style::default().fg(colors.success),
        link: Style::default().fg(colors.primary).add_modifier(Modifier::UNDERLINED),
        active_tab: Style::default()
            .fg(colors.text_bright)
            .bg(colors.surface)
            .add_modifier(Modifier::BOLD),
        inactive_tab: Style::default().fg(colors.text_dim).bg(colors.background),
    };
    Theme {
        name: "nord".to_string(),
        colors,
        styles,
    }
}

fn parse_color(s: &str) -> Result<Color> {
    if let Some(hex) = s.strip_prefix('#') {
        let hex = if hex.len() == 3 {
            hex.chars()
                .flat_map(|c| std::iter::repeat(c).take(2))
                .collect::<String>()
        } else {
            hex.to_string()
        };
        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| SymphonyError::theme(format!("Invalid color hex: {s}")))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| SymphonyError::theme(format!("Invalid color hex: {s}")))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| SymphonyError::theme(format!("Invalid color hex: {s}")))?;
        Ok(Color::Rgb(r, g, b))
    } else {
        match s.to_lowercase().as_str() {
            "reset" => Ok(Color::Reset),
            "black" => Ok(Color::Black),
            "red" => Ok(Color::Red),
            "green" => Ok(Color::Green),
            "yellow" => Ok(Color::Yellow),
            "blue" => Ok(Color::Blue),
            "magenta" => Ok(Color::Magenta),
            "cyan" => Ok(Color::Cyan),
            "white" => Ok(Color::White),
            "gray" | "grey" => Ok(Color::Gray),
            "dark_gray" | "dark_grey" => Ok(Color::DarkGray),
            "light_red" => Ok(Color::LightRed),
            "light_green" => Ok(Color::LightGreen),
            "light_yellow" => Ok(Color::LightYellow),
            "light_blue" => Ok(Color::LightBlue),
            "light_magenta" => Ok(Color::LightMagenta),
            "light_cyan" => Ok(Color::LightCyan),
            "light_white" => Ok(Color::White),
            _ => Err(SymphonyError::theme(format!("Unknown color: {s}"))),
        }
    }
}
