use crate::analyzer::{AnalyzerEvent, RouteTree};
use crate::runner::{ExtractedMetadata, LogKind, ParsedLog, RunnerEvent, RunnerStatus};
use crate::system::{calculate_cache_size, check_env_status, purge_cache, CacheStatus, EnvStatus};
use ratatui::widgets::TableState;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePanel {
    Routes,
    Logs,
}

impl ActivePanel {
    pub fn next(&self) -> Self {
        match self {
            ActivePanel::Routes => ActivePanel::Logs,
            ActivePanel::Logs => ActivePanel::Routes,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            ActivePanel::Routes => ActivePanel::Logs,
            ActivePanel::Logs => ActivePanel::Routes,
        }
    }
}

pub struct AppState {
    pub base_dir: PathBuf,
    pub running: bool,
    pub active_panel: ActivePanel,
    pub status: RunnerStatus,
    pub next_version: String,
    pub turbopack_active: bool,
    pub port: u16,
    pub pid: u32,
    pub local_url: String,
    pub network_url: String,

    // Routes
    pub routes: RouteTree,
    pub selected_route_index: usize,
    pub collapsed_paths: std::collections::HashSet<PathBuf>,

    // Logs (Table structure)
    pub logs: Vec<ParsedLog>,
    pub logs_table_state: TableState,
    pub auto_scroll: bool,
    pub logs_horizontal_scroll: usize,

    // System
    pub env_status: EnvStatus,
    pub cache_status: CacheStatus,

    // UI state
    pub show_help: bool,
    pub notification: Option<(String, Instant)>,
}

impl AppState {
    pub fn new(base_dir: PathBuf) -> Self {
        let env_status = check_env_status(&base_dir);
        let cache_status = calculate_cache_size(&base_dir);
        let routes = RouteTree::scan(&base_dir);

        let mut logs_table_state = TableState::default();
        logs_table_state.select(Some(0));

        Self {
            base_dir,
            running: true,
            active_panel: ActivePanel::Logs,
            status: RunnerStatus::Starting,
            next_version: "—".to_string(),
            turbopack_active: false,
            port: 0,
            pid: 0,
            local_url: "—".to_string(),
            network_url: "—".to_string(),

            routes,
            selected_route_index: 0,
            collapsed_paths: std::collections::HashSet::new(),

            logs: Vec::new(),
            logs_table_state,
            auto_scroll: true,
            logs_horizontal_scroll: 0,

            env_status,
            cache_status,

            show_help: false,
            notification: None,
        }
    }

    pub fn handle_runner_event(&mut self, event: RunnerEvent) {
        match event {
            RunnerEvent::Log(log) => {
                self.logs.push(log);
                if self.logs.len() > 1500 {
                    self.logs.remove(0);
                }
                if self.auto_scroll {
                    let len = self.logs.len();
                    if len > 0 {
                        self.logs_table_state.select(Some(len - 1));
                    }
                }
            }
            RunnerEvent::StatusChanged(st) => {
                self.status = st;
            }
            RunnerEvent::Metadata(meta) => {
                self.update_metadata(meta);
            }
            RunnerEvent::Pid(p) => {
                self.pid = p;
            }
            RunnerEvent::Exited(code) => {
                match code {
                    Some(0) => {
                        self.status = RunnerStatus::Stopped;
                        self.set_notification("Process stopped".to_string());
                    }
                    Some(c) => {
                        self.status = RunnerStatus::Failed(Some(c));
                        self.set_notification(format!("Process failed (exit code {})", c));
                    }
                    None => {
                        self.status = RunnerStatus::Failed(None);
                        self.set_notification("Process terminated".to_string());
                    }
                }
            }
        }
    }

    pub fn handle_analyzer_event(&mut self, event: AnalyzerEvent) {
        match event {
            AnalyzerEvent::RoutesUpdated => {
                self.routes = RouteTree::scan(&self.base_dir);
                self.set_notification("Routes updated".to_string());
            }
            AnalyzerEvent::EnvUpdated => {
                self.env_status = check_env_status(&self.base_dir);
                self.set_notification("Environment reloaded".to_string());
            }
        }
    }

    pub fn update_metadata(&mut self, meta: ExtractedMetadata) {
        if let Some(port) = meta.port {
            self.port = port;
            if self.local_url == "—" || self.local_url.is_empty() || meta.local_url.is_none() {
                self.local_url = format!("http://localhost:{}", port);
            }
        }
        if let Some(url) = meta.local_url {
            self.local_url = url;
        }
        if let Some(net) = meta.network_url {
            self.network_url = net;
        }
        if let Some(v) = meta.next_version {
            self.next_version = v;
        }
        if let Some(tb) = meta.turbopack {
            self.turbopack_active = tb;
        }
        if meta.is_ready {
            self.status = RunnerStatus::Online;
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn switch_panel_forward(&mut self) {
        self.active_panel = match self.active_panel {
            ActivePanel::Routes => ActivePanel::Logs,
            ActivePanel::Logs => ActivePanel::Routes,
        };
    }

    pub fn switch_panel_backward(&mut self) {
        self.switch_panel_forward();
    }

    pub fn visible_routes_count(&self) -> usize {
        self.routes.visible_items(&self.collapsed_paths).len()
    }

    pub fn scroll_routes_up(&mut self) {
        if self.selected_route_index > 0 {
            self.selected_route_index -= 1;
        }
    }

    pub fn scroll_routes_down(&mut self) {
        let count = self.visible_routes_count();
        if count > 0 && self.selected_route_index < count - 1 {
            self.selected_route_index += 1;
        }
    }

    pub fn toggle_selected_route_collapse(&mut self) {
        let visible = self.routes.visible_items(&self.collapsed_paths);
        if let Some(target) = visible.get(self.selected_route_index) {
            if target.item.is_dir {
                let path = target.item.path.clone();
                if self.collapsed_paths.contains(&path) {
                    self.collapsed_paths.remove(&path);
                } else {
                    self.collapsed_paths.insert(path);
                }
            }
        }
    }

    pub fn collapse_selected_route(&mut self) {
        let visible = self.routes.visible_items(&self.collapsed_paths);
        if let Some(target) = visible.get(self.selected_route_index) {
            if target.item.is_dir && !self.collapsed_paths.contains(&target.item.path) {
                self.collapsed_paths.insert(target.item.path.clone());
            } else if let Some(parent) = target.item.path.parent() {
                if let Some(parent_idx) = visible.iter().position(|v| v.item.path == parent) {
                    self.selected_route_index = parent_idx;
                }
            }
        }
    }

    pub fn expand_selected_route(&mut self) {
        let visible = self.routes.visible_items(&self.collapsed_paths);
        if let Some(target) = visible.get(self.selected_route_index) {
            if target.item.is_dir && self.collapsed_paths.contains(&target.item.path) {
                self.collapsed_paths.remove(&target.item.path);
            }
        }
    }

    pub fn scroll_logs_left(&mut self, amount: usize) {
        self.logs_horizontal_scroll = self.logs_horizontal_scroll.saturating_sub(amount);
    }

    pub fn scroll_logs_right(&mut self, amount: usize) {
        self.logs_horizontal_scroll = (self.logs_horizontal_scroll + amount).min(500);
    }

    pub fn reset_logs_horizontal_scroll(&mut self) {
        self.logs_horizontal_scroll = 0;
    }

    pub fn scroll_logs_up(&mut self, lines: usize) {
        self.auto_scroll = false;
        let current = self.logs_table_state.selected().unwrap_or(0);
        self.logs_table_state.select(Some(current.saturating_sub(lines)));
    }

    pub fn scroll_logs_down(&mut self, lines: usize) {
        let current = self.logs_table_state.selected().unwrap_or(0);
        if current + lines < self.logs.len() {
            self.logs_table_state.select(Some(current + lines));
        } else {
            self.tail_logs();
        }
    }

    pub fn scroll_up(&mut self) {
        match self.active_panel {
            ActivePanel::Routes => self.scroll_routes_up(),
            ActivePanel::Logs => self.scroll_logs_up(1),
        }
    }

    pub fn scroll_down(&mut self) {
        match self.active_panel {
            ActivePanel::Routes => self.scroll_routes_down(),
            ActivePanel::Logs => self.scroll_logs_down(1),
        }
    }

    pub fn scroll_page_up(&mut self) {
        match self.active_panel {
            ActivePanel::Routes => {
                self.selected_route_index = self.selected_route_index.saturating_sub(5);
            }
            ActivePanel::Logs => self.scroll_logs_up(10),
        }
    }

    pub fn scroll_page_down(&mut self) {
        match self.active_panel {
            ActivePanel::Routes => {
                let count = self.visible_routes_count();
                if count > 0 {
                    self.selected_route_index = (self.selected_route_index + 5).min(count - 1);
                }
            }
            ActivePanel::Logs => self.scroll_logs_down(10),
        }
    }

    pub fn tail_logs(&mut self) {
        self.auto_scroll = true;
        if !self.logs.is_empty() {
            self.logs_table_state.select(Some(self.logs.len() - 1));
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        self.tail_logs();
    }

    pub fn scroll_to_top(&mut self) {
        if self.active_panel == ActivePanel::Logs {
            self.auto_scroll = false;
            if !self.logs.is_empty() {
                self.logs_table_state.select(Some(0));
            }
        } else if self.active_panel == ActivePanel::Routes {
            self.selected_route_index = 0;
        }
    }

    pub fn purge_cache_action(&mut self) {
        match purge_cache(&self.base_dir) {
            Ok(bytes) => {
                self.cache_status = calculate_cache_size(&self.base_dir);
                let formatted = crate::system::cache::format_bytes(bytes);
                self.set_notification(format!("Cache purged: reclaimed {}", formatted));
                self.logs.push(ParsedLog {
                    timestamp: crate::runner::parser::LogParser::current_timestamp(),
                    kind: LogKind::Server,
                    message: format!("[Cache] Purged .next/cache ({} freed)", formatted),
                    raw: "".to_string(),
                });
            }
            Err(e) => {
                self.set_notification(format!("Cache purge failed: {}", e));
            }
        }
    }

    pub fn refresh_all(&mut self) {
        self.routes = RouteTree::scan(&self.base_dir);
        self.env_status = check_env_status(&self.base_dir);
        self.cache_status = calculate_cache_size(&self.base_dir);
        self.set_notification("Dashboard refreshed".to_string());
    }

    pub fn set_notification(&mut self, msg: String) {
        self.notification = Some((msg, Instant::now()));
    }

    pub fn active_notification(&self) -> Option<&str> {
        if let Some((msg, created_at)) = &self.notification {
            if created_at.elapsed().as_secs() < 4 {
                return Some(msg.as_str());
            }
        }
        None
    }
}
