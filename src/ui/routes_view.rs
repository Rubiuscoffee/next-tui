use crate::app::{ActivePanel, AppState};
use crate::ui::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn render_routes(app: &AppState, area: Rect) -> Paragraph<'static> {
    let is_focused = app.active_panel == ActivePanel::Routes;
    let visible_items = app.routes.visible_items(&app.collapsed_paths);

    let mut lines = Vec::new();

    // Full-width panel banner header (occupies the entire horizontal space, no shade gradients)
    let header_style = Theme::panel_header(is_focused);
    let left_title = " 1: ROUTES ";
    let right_info = if let Some(ref p) = app.routes.app_dir {
        if p.ends_with("src/app") {
            format!("src/app/ ({}) ", visible_items.len())
        } else {
            format!("app/ ({}) ", visible_items.len())
        }
    } else {
        format!("({}) ", visible_items.len())
    };

    let total_width = area.width as usize;
    let used_width = left_title.len() + right_info.len();
    let spacer = if total_width > used_width {
        " ".repeat(total_width - used_width)
    } else {
        " ".to_string()
    };

    lines.push(Line::from(vec![
        Span::styled(left_title, header_style),
        Span::styled(spacer, header_style),
        Span::styled(right_info, header_style),
    ]));
    lines.push(Line::from("")); // Clean breathing space

    if visible_items.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(
                "   No Next.js app directory found",
                Style::default().fg(Theme::COLOR_TEXT_MUTED),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled(
                "   Expected app/ or src/app/",
                Style::default().fg(Theme::COLOR_TEXT_DIM),
            ),
        ]));
    } else {
        for (idx, v_item) in visible_items.iter().enumerate() {
            let item = v_item.item;
            let is_selected = is_focused && idx == app.selected_route_index;

            let row_style = if is_selected {
                Theme::selection_highlight()
            } else {
                Style::default()
            };

            let mut spans = Vec::new();

            // Cursor prefix
            if is_selected {
                spans.push(Span::styled(
                    "▌ ",
                    Style::default()
                        .fg(Theme::COLOR_CYAN)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::raw("  "));
            }

            // Display text formatting
            let disp = &v_item.display;
            if disp.contains("[λ]") {
                let parts: Vec<&str> = disp.split("[λ]").collect();
                spans.push(Span::styled(parts[0].to_string(), row_style));
                spans.push(Span::styled(
                    "λ",
                    if is_selected {
                        row_style
                    } else {
                        Style::default()
                            .fg(Theme::COLOR_SUCCESS)
                            .add_modifier(Modifier::BOLD)
                    },
                ));
            } else if disp.contains("[⚡]") {
                let parts: Vec<&str> = disp.split("[⚡]").collect();
                spans.push(Span::styled(parts[0].to_string(), row_style));
                spans.push(Span::styled(
                    "client",
                    if is_selected {
                        row_style
                    } else {
                        Style::default()
                            .fg(Theme::COLOR_WARNING)
                            .add_modifier(Modifier::BOLD)
                    },
                ));
            } else if disp.contains("[API]") {
                let parts: Vec<&str> = disp.split("[API]").collect();
                spans.push(Span::styled(parts[0].to_string(), row_style));
                spans.push(Span::styled(
                    "api",
                    if is_selected {
                        row_style
                    } else {
                        Style::default()
                            .fg(Theme::COLOR_CYAN)
                            .add_modifier(Modifier::BOLD)
                    },
                ));
            } else if item.is_action {
                spans.push(Span::styled(
                    disp.clone(),
                    if is_selected {
                        row_style
                    } else {
                        Style::default().fg(Theme::COLOR_WARNING)
                    },
                ));
            } else if item.is_group {
                spans.push(Span::styled(
                    disp.clone(),
                    if is_selected {
                        row_style
                    } else {
                        Style::default().fg(Theme::COLOR_TEXT_DIM)
                    },
                ));
            } else if disp.trim_start().starts_with('▾') || disp.trim_start().starts_with('▸') {
                spans.push(Span::styled(
                    disp.clone(),
                    row_style.add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::styled(disp.clone(), row_style));
            }

            // Subtle padding on selection without blinding white
            if is_selected {
                let current_len: usize = spans.iter().map(|s| s.width()).sum();
                let target_width = area.width.saturating_sub(1) as usize;
                if target_width > current_len {
                    let padding = " ".repeat(target_width - current_len);
                    spans.push(Span::styled(padding, row_style));
                }
            }

            lines.push(Line::from(spans));
        }
    }

    // Scroll calculation
    let content_height = area.height.saturating_sub(2) as usize;
    let scroll_y = if content_height == 0 || app.selected_route_index < content_height {
        0
    } else {
        (app.selected_route_index + 1).saturating_sub(content_height)
    };

    let block = Block::default().borders(Borders::NONE);

    Paragraph::new(lines)
        .block(block)
        .scroll((scroll_y as u16, 0))
}
