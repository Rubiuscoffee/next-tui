use crate::app::AppState;
use crate::ui::theme::Theme;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn render_footer(app: &AppState) -> Paragraph<'static> {
    let line0 = Line::from(Span::styled(
        "░".repeat(300),
        Style::default().fg(Theme::COLOR_DIVIDER),
    ));

    let mut line1_spans = Vec::new();

    // 1. ENV Pill: Clean solid pill with no shade artifacts
    line1_spans.push(Span::styled(
        " ENV ",
        Style::default()
            .bg(Theme::COLOR_TEXT_DIM)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));
    line1_spans.push(Span::raw(" "));

    if app.env_status.active_files.is_empty() && app.env_status.loaded_count == 0 {
        line1_spans.push(Span::styled(
            "0 loaded (no .env)   ",
            Style::default().fg(Theme::COLOR_TEXT_DIM),
        ));
    } else {
        line1_spans.push(Span::styled(
            format!("{} loaded ", app.env_status.loaded_count),
            Style::default().fg(Theme::COLOR_TEXT_PRIMARY),
        ));

        if !app.env_status.missing_keys.is_empty() {
            line1_spans.push(Span::styled(
                format!("({} missing)   ", app.env_status.missing_keys.len()),
                Style::default()
                    .fg(Theme::COLOR_WARNING)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            line1_spans.push(Span::raw("  "));
        }
    }

    // 2. CACHE Pill: Clean solid pill
    line1_spans.push(Span::styled(
        " CACHE ",
        Style::default()
            .bg(Theme::COLOR_TEXT_DIM)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));
    line1_spans.push(Span::raw(" "));

    if app.cache_status.exists {
        line1_spans.push(Span::styled(
            format!("{} ", app.cache_status.formatted_size),
            Style::default()
                .fg(Theme::COLOR_TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        ));
    } else {
        line1_spans.push(Span::styled(
            "not found ",
            Style::default().fg(Theme::COLOR_TEXT_DIM),
        ));
    }

    line1_spans.push(Span::styled(
        "[c: purge]",
        Style::default().fg(Theme::COLOR_TEXT_DIM),
    ));

    // 3. Notification Pill (if any)
    if let Some(notif) = app.active_notification() {
        line1_spans.push(Span::raw("   "));
        line1_spans.push(Span::styled(
            " INFO ",
            Style::default()
                .bg(Theme::COLOR_SUCCESS)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ));
        line1_spans.push(Span::raw(" "));
        line1_spans.push(Span::styled(
            format!("* {}", notif),
            Style::default()
                .fg(Theme::COLOR_SUCCESS)
                .add_modifier(Modifier::BOLD),
        ));
    }

    Paragraph::new(vec![line0, Line::from(line1_spans)])
        .block(Block::default().borders(Borders::NONE))
}
