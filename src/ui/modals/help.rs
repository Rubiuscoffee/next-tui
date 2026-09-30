use crate::ui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_help(frame: &mut Frame) {
    let area = centered_rect(68, 65, frame.area());

    let help_text = vec![
        Line::from(vec![
            Span::styled("█", Style::default().fg(Color::White)),
            Span::styled(
                " Next.js 16 Dev Companion ",
                Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("░▒▓", Style::default().fg(Theme::COLOR_TEXT_DIM)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "█ NAVIGATION & PANELS ▓▒░",
                Style::default()
                    .fg(Theme::COLOR_CYAN)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  1 / 2 / 3         ", Style::default().fg(Theme::COLOR_TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled("Direct switch: 1:Routes, 2:Logs, 3:Environment", Style::default().fg(Theme::COLOR_TEXT_MUTED)),
        ]),
        Line::from(vec![
            Span::styled("  Tab / Shift+Tab   ", Style::default().fg(Theme::COLOR_TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled("Cycle active panel focus", Style::default().fg(Theme::COLOR_TEXT_MUTED)),
        ]),
        Line::from(vec![
            Span::styled("  h / l or ← / →    ", Style::default().fg(Theme::COLOR_TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled("Switch panel between Routes and Logs", Style::default().fg(Theme::COLOR_TEXT_MUTED)),
        ]),
        Line::from(vec![
            Span::styled("  Mouse Click       ", Style::default().fg(Theme::COLOR_TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled("Click any panel to focus directly", Style::default().fg(Theme::COLOR_TEXT_MUTED)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "█ SCROLLING & ACTIONS ▓▒░",
                Style::default()
                    .fg(Theme::COLOR_WARNING)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  j / k or ↑ / ↓    ", Style::default().fg(Theme::COLOR_TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled("Navigate routes / Scroll logs history (1 line)", Style::default().fg(Theme::COLOR_TEXT_MUTED)),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+d / Ctrl+u   ", Style::default().fg(Theme::COLOR_TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled("Page down / Page up fast scroll (10 lines)", Style::default().fg(Theme::COLOR_TEXT_MUTED)),
        ]),
        Line::from(vec![
            Span::styled("  g / G             ", Style::default().fg(Theme::COLOR_TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled("Jump to top (g) / Tail logs & resume auto-scroll (G)", Style::default().fg(Theme::COLOR_TEXT_MUTED)),
        ]),
        Line::from(vec![
            Span::styled("  Mouse Scroll      ", Style::default().fg(Theme::COLOR_TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled("Contextual wheel scroll over routes or logs", Style::default().fg(Theme::COLOR_TEXT_MUTED)),
        ]),
        Line::from(vec![
            Span::styled("  c / C             ", Style::default().fg(Theme::COLOR_TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled("Purge .next/cache directory immediately", Style::default().fg(Theme::COLOR_TEXT_MUTED)),
        ]),
        Line::from(vec![
            Span::styled("  r / R             ", Style::default().fg(Theme::COLOR_TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled("Manual refresh (re-scan routes, env and cache)", Style::default().fg(Theme::COLOR_TEXT_MUTED)),
        ]),
        Line::from(vec![
            Span::styled("  q / F10 / Ctrl+C  ", Style::default().fg(Theme::COLOR_TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled("Graceful shutdown (kills subprocess cleanly)", Style::default().fg(Theme::COLOR_TEXT_MUTED)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "█ SYMBOLS & CONVENTIONS ▓▒░",
                Style::default()
                    .fg(Theme::COLOR_SUCCESS)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  λ                 ", Style::default().fg(Theme::COLOR_SUCCESS).add_modifier(Modifier::BOLD)),
            Span::styled("React Server Component (RSC - default)", Style::default().fg(Theme::COLOR_TEXT_MUTED)),
        ]),
        Line::from(vec![
            Span::styled("  client            ", Style::default().fg(Theme::COLOR_WARNING).add_modifier(Modifier::BOLD)),
            Span::styled("Client Component ('use client')", Style::default().fg(Theme::COLOR_TEXT_MUTED)),
        ]),
        Line::from(vec![
            Span::styled("  api               ", Style::default().fg(Theme::COLOR_CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("Route Handler (route.ts)", Style::default().fg(Theme::COLOR_TEXT_MUTED)),
        ]),
        Line::from(vec![
            Span::styled("  * action()        ", Style::default().fg(Theme::COLOR_WARNING).add_modifier(Modifier::BOLD)),
            Span::styled("Server Action ('use server')", Style::default().fg(Theme::COLOR_TEXT_MUTED)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press Esc, F1, or q to dismiss", Style::default().fg(Theme::COLOR_TEXT_DIM)),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::COLOR_DIVIDER))
        .title(
            Line::from(vec![
                Span::styled("█", Style::default().fg(Theme::COLOR_CYAN)),
                Span::styled(
                    " HELP & REFERENCE ",
                    Style::default()
                        .bg(Theme::COLOR_CYAN)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("▓▒░", Style::default().fg(Theme::COLOR_CYAN)),
            ])
            .alignment(Alignment::Center),
        );

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    frame.render_widget(Clear, area);
    frame.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
