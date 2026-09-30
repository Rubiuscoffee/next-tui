use crate::app::{ActivePanel, AppState};
use crate::runner::parser::LogKind;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

pub fn render_logs(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let is_focused = app.active_panel == ActivePanel::Logs;

    // Split area into full-width banner header and table view
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // 1 line banner + 1 blank line
            Constraint::Min(1),    // Table
        ])
        .split(area);

    // Full-width panel banner header (occupies the entire horizontal space, no shade gradients)
    let header_style = Theme::panel_header(is_focused);
    let left_title = " 2: LOGS ";
    let right_status = if app.auto_scroll {
        format!("STREAMING ({} logs) ", app.logs.len())
    } else {
        format!("PAUSED [G to resume] ({} logs) ", app.logs.len())
    };

    let total_width = chunks[0].width as usize;
    let used_width = left_title.len() + right_status.len();
    let spacer = if total_width > used_width {
        " ".repeat(total_width - used_width)
    } else {
        " ".to_string()
    };

    let header_line = Line::from(vec![
        Span::styled(left_title, header_style),
        Span::styled(spacer, header_style),
        Span::styled(right_status, header_style),
    ]);

    let banner_widget = Paragraph::new(header_line);
    frame.render_widget(banner_widget, chunks[0]);

    // Construct Table rows
    let h_scroll = app.logs_horizontal_scroll;

    let rows: Vec<Row> = if app.logs.is_empty() {
        vec![Row::new(vec![
            Cell::from(Span::styled("—", Style::default().fg(Theme::COLOR_TEXT_DIM))),
            Cell::from(Span::styled("—", Style::default().fg(Theme::COLOR_TEXT_DIM))),
            Cell::from(Span::styled("—", Style::default().fg(Theme::COLOR_TEXT_DIM))),
            Cell::from(Span::styled(
                "Waiting for development server output...",
                Style::default().fg(Theme::COLOR_TEXT_MUTED),
            )),
        ])]
    } else {
        app.logs
            .iter()
            .map(|log| {
                let (
                    method_str,
                    method_style,
                    status_str,
                    status_style,
                    message_str,
                    message_style,
                ) = match &log.kind {
                    LogKind::Turbopack => {
                        let clean_msg = log.message.replace("[Turbopack]", "").trim().to_string();
                        (
                            "TURBO ".to_string(),
                            Theme::http_method("TURBO"),
                            "—     ".to_string(),
                            Style::default().fg(Theme::COLOR_TEXT_DIM),
                            clean_msg,
                            Style::default().fg(Theme::COLOR_TEXT_PRIMARY),
                        )
                    }
                    LogKind::Http { method, status } => {
                        let rest = log
                            .message
                            .strip_prefix(method.as_str())
                            .unwrap_or(&log.message)
                            .trim()
                            .to_string();
                        (
                            format!("{:<6}", method),
                            Theme::http_method(method),
                            format!("{:<6}", status),
                            Theme::http_status(*status),
                            rest,
                            Style::default().fg(Theme::COLOR_TEXT_PRIMARY),
                        )
                    }
                    LogKind::Action(name) => {
                        let clean_msg = if log.message.contains(name) {
                            log.message.replace("[Action]", "").trim().to_string()
                        } else {
                            format!("{} {}", name, log.message)
                        };
                        (
                            "ACT   ".to_string(),
                            Theme::http_method("ACT"),
                            "—     ".to_string(),
                            Style::default().fg(Theme::COLOR_TEXT_DIM),
                            clean_msg,
                            Style::default().fg(Theme::COLOR_WARNING),
                        )
                    }
                    LogKind::Server => {
                        let clean_msg = log
                            .message
                            .strip_prefix("[Server]")
                            .unwrap_or(&log.message)
                            .trim()
                            .to_string();
                        (
                            "SRV   ".to_string(),
                            Theme::http_method("SRV"),
                            "—     ".to_string(),
                            Style::default().fg(Theme::COLOR_TEXT_DIM),
                            clean_msg,
                            Style::default().fg(Theme::COLOR_TEXT_MUTED),
                        )
                    }
                    LogKind::Info => (
                        "INFO  ".to_string(),
                        Theme::http_method("INFO"),
                        "—     ".to_string(),
                        Style::default().fg(Theme::COLOR_TEXT_DIM),
                        log.message.clone(),
                        Style::default().fg(Theme::COLOR_CYAN),
                    ),
                    LogKind::Warn => (
                        "WARN  ".to_string(),
                        Theme::http_method("WARN"),
                        "—     ".to_string(),
                        Style::default().fg(Theme::COLOR_TEXT_DIM),
                        log.message.clone(),
                        Style::default().fg(Theme::COLOR_WARNING),
                    ),
                    LogKind::Error => (
                        "ERR   ".to_string(),
                        Theme::http_method("ERR"),
                        "—     ".to_string(),
                        Style::default().fg(Theme::COLOR_TEXT_DIM),
                        log.message.clone(),
                        Style::default().fg(Theme::COLOR_ERROR),
                    ),
                    LogKind::Raw => (
                        "RAW   ".to_string(),
                        Style::default().fg(Theme::COLOR_TEXT_DIM),
                        "—     ".to_string(),
                        Style::default().fg(Theme::COLOR_TEXT_DIM),
                        log.message.clone(),
                        Style::default().fg(Theme::COLOR_TEXT_PRIMARY),
                    ),
                };

                let display_message = if h_scroll > 0 {
                    safe_slice_chars(&message_str, h_scroll)
                } else {
                    message_str
                };

                Row::new(vec![
                    Cell::from(Span::styled(
                        log.timestamp.clone(),
                        Style::default().fg(Theme::COLOR_TEXT_DIM),
                    )),
                    Cell::from(Span::styled(method_str, method_style)),
                    Cell::from(Span::styled(status_str, status_style)),
                    Cell::from(Span::styled(display_message, message_style)),
                ])
            })
            .collect()
    };

    let widths = [
        Constraint::Length(8), // TIME: HH:MM:SS
        Constraint::Length(6), // METHOD: GET, POST, TURBO...
        Constraint::Length(6), // STATUS: 200, 404, —...
        Constraint::Min(20),   // PATH / MESSAGE
    ];

    let path_header_title = if h_scroll > 0 {
        format!("PATH / MESSAGE [◄ +{}]", h_scroll)
    } else {
        "PATH / MESSAGE".to_string()
    };

    let header_row = Row::new(vec![
        Cell::from(Span::styled(
            " TIME",
            Style::default()
                .fg(Theme::COLOR_TEXT_MUTED)
                .add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "METHOD",
            Style::default()
                .fg(Theme::COLOR_TEXT_MUTED)
                .add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "STATUS",
            Style::default()
                .fg(Theme::COLOR_TEXT_MUTED)
                .add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            path_header_title,
            Style::default()
                .fg(if h_scroll > 0 {
                    Theme::COLOR_CYAN
                } else {
                    Theme::COLOR_TEXT_MUTED
                })
                .add_modifier(Modifier::BOLD),
        )),
    ]);

    let mut table = Table::new(rows, widths)
        .header(header_row)
        .block(Block::default().borders(Borders::NONE))
        .highlight_symbol("▌ ");

    if is_focused {
        table = table.row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
    } else {
        table = table.row_highlight_style(Style::default());
    }

    frame.render_stateful_widget(table, chunks[1], &mut app.logs_table_state);
}

fn safe_slice_chars(s: &str, skip: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if skip >= chars.len() {
        String::new()
    } else {
        chars[skip..].iter().collect()
    }
}
