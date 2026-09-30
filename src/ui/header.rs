use crate::app::AppState;
use crate::runner::RunnerStatus;
use crate::ui::theme::Theme;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn render_header(app: &AppState) -> Paragraph<'static> {
    let next_version_label = if app.next_version == "—" {
        "Next.js".to_string()
    } else {
        format!("Next.js {}", app.next_version)
    };

    let mut line1_spans = Vec::new();

    // 1. Runtime Pill: Next.js <version> (clean solid pill, no gradient/shade artifacts)
    line1_spans.push(Span::styled(
        format!(" {} ", next_version_label),
        Style::default()
            .bg(Color::White)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    ));

    // 2. Engine Pill: TURBO
    if app.turbopack_active {
        line1_spans.push(Span::raw(" "));
        line1_spans.push(Span::styled(
            " TURBO ",
            Style::default()
                .bg(Theme::COLOR_PURPLE)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ));
    }

    line1_spans.push(Span::raw(" "));

    // 3. Status Pill: ONLINE / STARTING / OFFLINE
    match &app.status {
        RunnerStatus::Online => {
            line1_spans.push(Span::styled(" ONLINE ", Theme::status_online()));
        }
        RunnerStatus::Starting => {
            line1_spans.push(Span::styled(" STARTING ", Theme::status_starting()));
        }
        RunnerStatus::Failed(Some(code)) => {
            line1_spans.push(Span::styled(
                format!(" OFFLINE (code {}) ", code),
                Theme::status_failed(),
            ));
        }
        RunnerStatus::Failed(None) | RunnerStatus::Error(_) => {
            line1_spans.push(Span::styled(" OFFLINE ", Theme::status_failed()));
        }
        RunnerStatus::Stopped => {
            line1_spans.push(Span::styled(" OFFLINE ", Theme::status_offline()));
        }
    }

    line1_spans.push(Span::raw(" "));

    // 4. Port & PID Pills (separated cleanly by a space, no checkered shade characters)
    let port_str = if app.port > 0 {
        app.port.to_string()
    } else {
        "—".to_string()
    };
    let pid_str = if app.pid > 0 {
        app.pid.to_string()
    } else {
        "—".to_string()
    };

    line1_spans.push(Span::styled(
        format!(" PORT {} ", port_str),
        Style::default()
            .bg(Theme::COLOR_TEXT_DIM)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));
    line1_spans.push(Span::raw(" "));
    line1_spans.push(Span::styled(
        format!(" PID {} ", pid_str),
        Style::default()
            .bg(Theme::COLOR_TEXT_DIM)
            .fg(Color::White),
    ));

    // 5. Help hint
    line1_spans.push(Span::raw("  "));
    line1_spans.push(Span::styled(
        "[?] help   [q] quit",
        Style::default().fg(Theme::COLOR_TEXT_DIM),
    ));

    // Line 2: URLs with clean solid block labels (no raw escape sequences so IP doesn't truncate)
    let local_str = if app.local_url.is_empty() || app.local_url == "—" {
        "—".to_string()
    } else {
        app.local_url.clone()
    };

    let network_str = if app.network_url.is_empty() || app.network_url == "—" {
        "—".to_string()
    } else {
        app.network_url.clone()
    };

    let line2_spans = vec![
        Span::styled(
            " LOCAL ",
            Style::default()
                .bg(Theme::COLOR_CYAN)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}   ", local_str),
            Style::default().fg(Theme::COLOR_CYAN),
        ),
        Span::styled(
            " NETWORK ",
            Style::default()
                .bg(Theme::COLOR_TEXT_DIM)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}   ", network_str),
            Style::default().fg(Theme::COLOR_TEXT_PRIMARY),
        ),
    ];

    // Line 3: Textured shade divider (the only place where shade blocks belong)
    let line3 = Line::from(Span::styled(
        "░".repeat(300),
        Style::default().fg(Theme::COLOR_DIVIDER),
    ));

    Paragraph::new(vec![Line::from(line1_spans), Line::from(line2_spans), line3])
        .block(Block::default().borders(Borders::NONE))
}
