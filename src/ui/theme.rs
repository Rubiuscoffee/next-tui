use ratatui::style::{Color, Modifier, Style};

pub struct Theme;

impl Theme {
    // Adaptive ANSI 16-color Palette (inherits user terminal theme in Ghostty, Zed, etc.)
    pub const COLOR_BG: Color = Color::Reset;
    pub const COLOR_TEXT_PRIMARY: Color = Color::White;
    pub const COLOR_TEXT_MUTED: Color = Color::Gray;
    pub const COLOR_TEXT_DIM: Color = Color::DarkGray;
    pub const COLOR_DIVIDER: Color = Color::DarkGray;

    // Accents using standard ANSI palette
    pub const COLOR_SUCCESS: Color = Color::Green;
    pub const COLOR_ERROR: Color = Color::Red;
    pub const COLOR_WARNING: Color = Color::Yellow;
    pub const COLOR_CYAN: Color = Color::Cyan;
    pub const COLOR_PURPLE: Color = Color::Magenta;
    pub const COLOR_BLUE: Color = Color::Blue;

    pub fn divider() -> Style {
        Style::default().fg(Self::COLOR_DIVIDER)
    }

    pub fn section_header(is_active: bool) -> Style {
        if is_active {
            Style::default()
                .fg(Self::COLOR_TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Self::COLOR_TEXT_DIM)
        }
    }

    // Statusline / Block Badges with ANSI shade characters
    pub fn badge(is_active: bool) -> Style {
        if is_active {
            Style::default()
                .bg(Self::COLOR_CYAN)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .bg(Self::COLOR_TEXT_DIM)
                .fg(Color::White)
        }
    }

    pub fn badge_shade(is_active: bool) -> Style {
        if is_active {
            Style::default().fg(Self::COLOR_CYAN)
        } else {
            Style::default().fg(Self::COLOR_TEXT_DIM)
        }
    }

    pub fn badge_solid(bg: Color, fg: Color) -> Style {
        Style::default().bg(bg).fg(fg).add_modifier(Modifier::BOLD)
    }

    pub fn status_online() -> Style {
        Style::default()
            .bg(Self::COLOR_SUCCESS)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    }

    pub fn status_starting() -> Style {
        Style::default()
            .bg(Self::COLOR_WARNING)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    }

    pub fn status_offline() -> Style {
        Style::default()
            .bg(Self::COLOR_TEXT_DIM)
            .fg(Color::White)
    }

    pub fn status_failed() -> Style {
        Style::default()
            .bg(Self::COLOR_ERROR)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    }

    pub fn http_method(method: &str) -> Style {
        match method {
            "GET" => Style::default().fg(Self::COLOR_SUCCESS).add_modifier(Modifier::BOLD),
            "POST" => Style::default().fg(Self::COLOR_CYAN).add_modifier(Modifier::BOLD),
            "PUT" | "PATCH" => Style::default().fg(Self::COLOR_WARNING).add_modifier(Modifier::BOLD),
            "DELETE" => Style::default().fg(Self::COLOR_ERROR).add_modifier(Modifier::BOLD),
            "ACT" => Style::default().fg(Self::COLOR_WARNING).add_modifier(Modifier::BOLD),
            "TURBO" => Style::default().fg(Self::COLOR_PURPLE).add_modifier(Modifier::BOLD),
            "SRV" => Style::default().fg(Self::COLOR_BLUE),
            "ERR" => Style::default().fg(Self::COLOR_ERROR).add_modifier(Modifier::BOLD),
            "WARN" => Style::default().fg(Self::COLOR_WARNING),
            _ => Style::default().fg(Self::COLOR_TEXT_MUTED),
        }
    }

    pub fn http_status(status: u16) -> Style {
        if status < 300 {
            Style::default().fg(Self::COLOR_SUCCESS).add_modifier(Modifier::BOLD)
        } else if status < 400 {
            Style::default().fg(Self::COLOR_CYAN).add_modifier(Modifier::BOLD)
        } else if status < 500 {
            Style::default().fg(Self::COLOR_WARNING).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Self::COLOR_ERROR).add_modifier(Modifier::BOLD)
        }
    }


    // Full-width solid panel banner header style
    pub fn panel_header(is_active: bool) -> Style {
        if is_active {
            Style::default()
                .bg(Self::COLOR_CYAN)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .bg(Self::COLOR_TEXT_DIM)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        }
    }

    // Subtle dark background highlight (never blinding pure white!)
    pub fn selection_highlight() -> Style {
        Style::default()
            .bg(Self::COLOR_TEXT_DIM)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    }
}

// Clean URL formatter (avoids unhandled ANSI codes that corrupt Ratatui line widths)
pub fn osc8_url(url: &str) -> String {
    url.to_string()
}
