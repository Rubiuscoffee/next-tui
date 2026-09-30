pub mod footer;
pub mod header;
pub mod logs_view;
pub mod modals;
pub mod routes_view;
pub mod theme;

use crate::app::AppState;
use footer::render_footer;
use header::render_header;
use logs_view::render_logs;
use modals::help::render_help;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use routes_view::render_routes;
use theme::Theme;

#[derive(Debug, Clone, Copy)]
pub struct AppLayout {
    pub header: Rect,
    pub routes: Rect,
    pub divider: Rect,
    pub logs: Rect,
    pub footer: Rect,
}

pub fn get_layout(area: Rect) -> AppLayout {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header: 2 content lines + 1 shade divider line
            Constraint::Min(6),    // Main Body: Routes, Vertical Shade Divider, Logs
            Constraint::Length(2), // Footer: 1 shade divider line + 1 content line
        ])
        .split(area);

    let body_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(32), // Route tree width
            Constraint::Length(1),  // Vertical shade divider column
            Constraint::Min(30),    // Real-time logs viewport
        ])
        .split(main_layout[1]);

    AppLayout {
        header: main_layout[0],
        routes: body_layout[0],
        divider: body_layout[1],
        logs: body_layout[2],
        footer: main_layout[2],
    }
}

pub fn draw(frame: &mut Frame, app: &mut AppState) {
    let layout = get_layout(frame.area());

    // Render Header
    let header_widget = render_header(app);
    frame.render_widget(header_widget, layout.header);

    // Render Routes View
    let routes_widget = render_routes(app, layout.routes);
    frame.render_widget(routes_widget, layout.routes);

    // Render Vertical Textured Shade Divider
    let divider_lines: Vec<Line> = (0..layout.divider.height)
        .map(|_| Line::from(Span::styled("░", Style::default().fg(Theme::COLOR_DIVIDER))))
        .collect();
    frame.render_widget(Paragraph::new(divider_lines), layout.divider);

    // Render Logs View (Stateful Table)
    render_logs(frame, app, layout.logs);

    // Render Footer
    let footer_widget = render_footer(app);
    frame.render_widget(footer_widget, layout.footer);

    // Render Modal if open
    if app.show_help {
        render_help(frame);
    }
}
