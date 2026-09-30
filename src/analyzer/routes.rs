use crate::analyzer::actions::detect_server_actions;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct RouteTree {
    pub app_dir: Option<PathBuf>,
    pub items: Vec<FlatRouteItem>,
}

#[derive(Debug, Clone)]
pub struct FlatRouteItem {
    pub display: String,
    pub path: PathBuf,
    pub is_page: bool,
    pub is_action: bool,
    pub is_group: bool,
    pub is_dir: bool,
    pub depth: usize,
}

#[derive(Debug, Clone)]
pub struct VisibleRouteItem<'a> {
    pub item: &'a FlatRouteItem,
    pub display: String,
    pub is_collapsed: bool,
}

impl RouteTree {
    pub fn scan(base_dir: &Path) -> Self {
        let app_candidates = [base_dir.join("app"), base_dir.join("src").join("app")];
        let app_dir = app_candidates.into_iter().find(|p| p.is_dir());

        let dir = match app_dir {
            Some(d) => d,
            None => {
                return Self {
                    app_dir: None,
                    items: Vec::new(),
                };
            }
        };

        let mut items = Vec::new();
        items.push(FlatRouteItem {
            display: "▾ /".to_string(),
            path: dir.clone(),
            is_page: false,
            is_action: false,
            is_group: false,
            is_dir: true,
            depth: 0,
        });

        scan_directory(&dir, 1, &mut items, &mut Vec::new());

        Self {
            app_dir: Some(dir),
            items,
        }
    }

    pub fn visible_items<'a>(&'a self, collapsed: &HashSet<PathBuf>) -> Vec<VisibleRouteItem<'a>> {
        let mut visible = Vec::new();
        let mut skip_prefix: Option<PathBuf> = None;

        for item in &self.items {
            if let Some(ref prefix) = skip_prefix {
                if item.path.starts_with(prefix) && item.path != *prefix {
                    continue;
                } else {
                    skip_prefix = None;
                }
            }

            let is_collapsed = item.is_dir && collapsed.contains(&item.path);
            let display = if is_collapsed {
                item.display.replacen('▾', "▸", 1)
            } else {
                item.display.clone()
            };

            visible.push(VisibleRouteItem {
                item,
                display,
                is_collapsed,
            });

            if is_collapsed {
                skip_prefix = Some(item.path.clone());
            }
        }

        visible
    }
}

fn scan_directory(
    current: &Path,
    depth: usize,
    items: &mut Vec<FlatRouteItem>,
    ancestor_is_last: &mut Vec<bool>,
) {
    let mut entries = match fs::read_dir(current) {
        Ok(read_dir) => read_dir.filter_map(|e| e.ok()).collect::<Vec<_>>(),
        Err(_) => return,
    };

    // Sort entries: directories first, then files
    entries.sort_by(|a, b| {
        let a_is_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let b_is_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });

    let total = entries.len();
    for (i, entry) in entries.into_iter().enumerate() {
        let is_last = i == total - 1;
        let file_type = match entry.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };

        let file_name = entry.file_name().to_string_lossy().to_string();
        if file_name.starts_with('.') || file_name == "node_modules" {
            continue;
        }

        let path = entry.path();
        let indent = make_indent(ancestor_is_last, is_last);

        if file_type.is_dir() {
            let is_group = file_name.starts_with('(') && file_name.ends_with(')');
            let prefix = if is_group {
                format!("{}├─ {}", indent, file_name)
            } else {
                format!("{}▾ {}", indent, file_name)
            };

            items.push(FlatRouteItem {
                display: prefix,
                path: path.clone(),
                is_page: false,
                is_action: false,
                is_group,
                is_dir: true,
                depth,
            });

            ancestor_is_last.push(is_last);
            scan_directory(&path, depth + 1, items, ancestor_is_last);
            ancestor_is_last.pop();
        } else {
            let is_page = file_name.starts_with("page.");
            let is_action_file = file_name.contains("action");
            let is_route = file_name.starts_with("route.");
            let is_layout = file_name.starts_with("layout.");
            let is_loading = file_name.starts_with("loading.");
            let is_error = file_name.starts_with("error.");

            if is_page || is_action_file || is_route || is_layout || is_loading || is_error {
                let badge = if is_page {
                    // Check if "use client" exists in file
                    let is_client = fs::read_to_string(&path)
                        .map(|c| c.contains("\"use client\"") || c.contains("'use client'"))
                        .unwrap_or(false);
                    if is_client {
                        " [⚡]"
                    } else {
                        " [λ]"
                    }
                } else if is_route {
                    " [API]"
                } else {
                    ""
                };

                let branch = if is_last { "└─ " } else { "├─ " };
                let display = format!("{}{}{}{}", indent, branch, file_name, badge);

                items.push(FlatRouteItem {
                    display,
                    path: path.clone(),
                    is_page,
                    is_action: false,
                    is_group: false,
                    is_dir: false,
                    depth,
                });

                // Detect Server Actions inside this file
                let actions = detect_server_actions(&path);
                let actions_count = actions.len();
                for (a_idx, action) in actions.into_iter().enumerate() {
                    let a_is_last = a_idx == actions_count - 1;
                    let action_indent = make_indent_for_child(ancestor_is_last, is_last);
                    let a_branch = if a_is_last { "└─ " } else { "├─ " };
                    let a_display = format!("{}{}* {}()", action_indent, a_branch, action.name);

                    items.push(FlatRouteItem {
                        display: a_display,
                        path: path.clone(),
                        is_page: false,
                        is_action: true,
                        is_group: false,
                        is_dir: false,
                        depth: depth + 1,
                    });
                }
            }
        }
    }
}

fn make_indent(ancestor_is_last: &[bool], _is_last: bool) -> String {
    let mut s = String::new();
    for &last in ancestor_is_last {
        if last {
            s.push_str("   ");
        } else {
            s.push_str("│  ");
        }
    }
    s
}

fn make_indent_for_child(ancestor_is_last: &[bool], parent_is_last: bool) -> String {
    let mut s = make_indent(ancestor_is_last, parent_is_last);
    if parent_is_last {
        s.push_str("   ");
    } else {
        s.push_str("│  ");
    }
    s
}
