use regex::Regex;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ServerAction {
    pub name: String,
    pub line_number: usize,
    pub is_exported: bool,
}

pub fn detect_server_actions(file_path: &Path) -> Vec<ServerAction> {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut actions = Vec::new();
    let file_has_use_server = content.lines().take(15).any(|line| {
        let trimmed = line.trim();
        trimmed == "\"use server\";"
            || trimmed == "'use server';"
            || trimmed == "\"use server\""
            || trimmed == "'use server'"
    });

    // Match exported async functions: export async function foo(
    let fn_regex = Regex::new(r"export\s+async\s+function\s+([a-zA-Z0-9_$]+)\s*\(").unwrap();
    // Match exported arrow async functions: export const foo = async (
    let const_regex = Regex::new(r"export\s+const\s+([a-zA-Z0-9_$]+)\s*=\s*async").unwrap();

    for (idx, line) in content.lines().enumerate() {
        if file_has_use_server {
            if let Some(caps) = fn_regex.captures(line) {
                if let Some(name) = caps.get(1) {
                    actions.push(ServerAction {
                        name: name.as_str().to_string(),
                        line_number: idx + 1,
                        is_exported: true,
                    });
                }
            } else if let Some(caps) = const_regex.captures(line) {
                if let Some(name) = caps.get(1) {
                    actions.push(ServerAction {
                        name: name.as_str().to_string(),
                        line_number: idx + 1,
                        is_exported: true,
                    });
                }
            }
        } else {
            // Check for inline "use server" inside function body
            if line.contains("\"use server\"") || line.contains("'use server'") {
                // Approximate action name from nearby context if possible
                actions.push(ServerAction {
                    name: format!("inlineAction_L{}", idx + 1),
                    line_number: idx + 1,
                    is_exported: false,
                });
            }
        }
    }

    actions
}
