pub mod analyzer;
pub mod app;
pub mod event;
pub mod runner;
pub mod system;
pub mod ui;

use app::{ActivePanel, AppState};
use crossterm::{
    cursor::{Hide, Show},
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers, MouseButton, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use event::{AppEvent, EventHandler};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;
use runner::ProcessRunner;
use std::io::{stdout, Stdout};
use std::path::PathBuf;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Current working directory (or provided argument)
    let base_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // Install panic hook to safely restore terminal on unexpected panic
    let default_panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal_direct();
        default_panic_hook(panic_info);
    }));

    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App state
    let mut app = AppState::new(base_dir.clone());

    // Event loop handler
    let mut event_handler = EventHandler::new(Duration::from_millis(50));

    // File watcher for hot App Router & Env detection
    let _watcher = analyzer::ProjectWatcher::start(
        base_dir.clone(),
        event_handler.analyzer_tx.clone(),
    );

    // Development subprocess runner
    let mut runner = ProcessRunner::new();
    if let Err(e) = runner.spawn(&base_dir, event_handler.runner_tx.clone()).await {
        app.status = runner::RunnerStatus::Error(e.clone());
        app.set_notification(format!("Error starting runner: {}", e));
    }

    // Main event loop
    while app.running {
        terminal.draw(|frame| {
            ui::draw(frame, &mut app);
        })?;

        match event_handler.next().await {
            Ok(AppEvent::Key(key)) => {
                if app.show_help {
                    match key.code {
                        KeyCode::Esc | KeyCode::F(1) | KeyCode::Char('q') | KeyCode::Char('?') => {
                            app.show_help = false;
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        // Quit application
                        KeyCode::Char('q') | KeyCode::F(10) => {
                            app.running = false;
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.running = false;
                        }

                        // Direct panel switching
                        KeyCode::Char('1') => {
                            app.active_panel = ActivePanel::Routes;
                        }
                        KeyCode::Char('2') => {
                            app.active_panel = ActivePanel::Logs;
                        }

                        // Directory collapse / expand in Routes
                        KeyCode::Char(' ') | KeyCode::Enter => {
                            if app.active_panel == ActivePanel::Routes {
                                app.toggle_selected_route_collapse();
                            }
                        }

                        // Left / Right: Route collapse/expand or Log horizontal scroll
                        KeyCode::Left | KeyCode::Char('h') => {
                            if app.active_panel == ActivePanel::Logs {
                                app.scroll_logs_left(6);
                            } else if app.active_panel == ActivePanel::Routes {
                                app.collapse_selected_route();
                            }
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            if app.active_panel == ActivePanel::Logs {
                                app.scroll_logs_right(6);
                            } else if app.active_panel == ActivePanel::Routes {
                                app.expand_selected_route();
                            }
                        }
                        KeyCode::Char('0') | KeyCode::Home => {
                            if app.active_panel == ActivePanel::Logs {
                                app.reset_logs_horizontal_scroll();
                            }
                        }

                        // Tab cycling
                        KeyCode::Tab => {
                            if key.modifiers.contains(KeyModifiers::SHIFT) {
                                app.switch_panel_backward();
                            } else {
                                app.switch_panel_forward();
                            }
                        }
                        KeyCode::BackTab => {
                            app.switch_panel_backward();
                        }

                        // Scrolling
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.scroll_up();
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.scroll_down();
                        }
                        KeyCode::PageUp => {
                            app.scroll_page_up();
                        }
                        KeyCode::PageDown => {
                            app.scroll_page_down();
                        }
                        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.scroll_page_up();
                        }
                        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.scroll_page_down();
                        }
                        KeyCode::Char('g') => {
                            app.scroll_to_top();
                        }
                        KeyCode::Char('G') => {
                            app.tail_logs();
                        }

                        // Cache purge
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            app.purge_cache_action();
                        }

                        // Refresh
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            app.refresh_all();
                        }

                        // Help modal
                        KeyCode::F(1) | KeyCode::Char('?') => {
                            app.toggle_help();
                        }

                        _ => {}
                    }
                }
            }
            Ok(AppEvent::Mouse(mouse)) => {
                let term_rect = terminal.size().map(|s| Rect::new(0, 0, s.width, s.height)).unwrap_or_default();
                let layout = ui::get_layout(term_rect);
                let col = mouse.column;
                let row = mouse.row;

                match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        if rect_contains(layout.routes, col, row) {
                            app.active_panel = ActivePanel::Routes;
                            if row >= layout.routes.y + 2 {
                                let clicked = (row - (layout.routes.y + 2)) as usize;
                                if clicked < app.visible_routes_count() {
                                    if app.selected_route_index == clicked {
                                        app.toggle_selected_route_collapse();
                                    } else {
                                        app.selected_route_index = clicked;
                                    }
                                }
                            }
                        } else {
                            app.active_panel = ActivePanel::Logs;
                        }
                    }
                    MouseEventKind::ScrollUp => {
                        if rect_contains(layout.logs, col, row) {
                            app.scroll_logs_up(1);
                        } else if rect_contains(layout.routes, col, row) {
                            app.scroll_routes_up();
                        } else {
                            app.scroll_up();
                        }
                    }
                    MouseEventKind::ScrollDown => {
                        if rect_contains(layout.logs, col, row) {
                            app.scroll_logs_down(1);
                        } else if rect_contains(layout.routes, col, row) {
                            app.scroll_routes_down();
                        } else {
                            app.scroll_down();
                        }
                    }
                    MouseEventKind::ScrollLeft => {
                        if rect_contains(layout.logs, col, row) {
                            app.scroll_logs_left(4);
                        }
                    }
                    MouseEventKind::ScrollRight => {
                        if rect_contains(layout.logs, col, row) {
                            app.scroll_logs_right(4);
                        }
                    }
                    _ => {}
                }
            }
            Ok(AppEvent::Runner(runner_event)) => {
                app.handle_runner_event(runner_event);
            }
            Ok(AppEvent::Analyzer(analyzer_event)) => {
                app.handle_analyzer_event(analyzer_event);
            }
            Ok(AppEvent::Tick) => {
                // Tick triggers redraw
            }
            Err(_) => {
                break;
            }
        }
    }

    // Safe termination of child processes
    runner.terminate().await;

    // Clean restoration of terminal
    restore_terminal(&mut terminal)?;

    Ok(())
}

fn rect_contains(r: Rect, col: u16, row: u16) -> bool {
    col >= r.x && col < r.x.saturating_add(r.width) && row >= r.y && row < r.y.saturating_add(r.height)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> std::io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        Show
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn restore_terminal_direct() -> std::io::Result<()> {
    let _ = disable_raw_mode();
    let mut stdout = stdout();
    let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture, Show);
    Ok(())
}
