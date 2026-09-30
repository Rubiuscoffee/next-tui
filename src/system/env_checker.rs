use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct EnvStatus {
    pub loaded_count: usize,
    pub missing_keys: Vec<String>,
    pub example_file: Option<String>,
    pub active_files: Vec<String>,
}

pub fn check_env_status(base_dir: &Path) -> EnvStatus {
    let example_candidates = [".env.example", ".env.sample", ".env.template"];
    let mut example_path = None;
    let mut example_name = None;

    for candidate in example_candidates {
        let p = base_dir.join(candidate);
        if p.exists() {
            example_name = Some(candidate.to_string());
            example_path = Some(p);
            break;
        }
    }

    // Active env files in Next.js priority order
    let active_candidates = [
        ".env.development.local",
        ".env.local",
        ".env.development",
        ".env",
    ];

    let mut active_files = Vec::new();
    let mut active_keys = HashSet::new();

    for candidate in active_candidates {
        let p = base_dir.join(candidate);
        if p.exists() {
            active_files.push(candidate.to_string());
            if let Ok(content) = fs::read_to_string(&p) {
                parse_env_keys(&content, &mut active_keys);
            }
        }
    }

    let mut missing_keys = Vec::new();
    if let Some(ref ep) = example_path {
        if let Ok(content) = fs::read_to_string(ep) {
            let mut example_keys = HashSet::new();
            parse_env_keys(&content, &mut example_keys);

            for key in example_keys {
                if !active_keys.contains(&key) {
                    missing_keys.push(key);
                }
            }
        }
    }

    missing_keys.sort();

    EnvStatus {
        loaded_count: active_keys.len(),
        missing_keys,
        example_file: example_name,
        active_files,
    }
}

fn parse_env_keys(content: &str, set: &mut HashSet<String>) {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, _)) = trimmed.split_once('=') {
            let key = key.trim();
            if !key.is_empty() {
                set.insert(key.to_string());
            }
        }
    }
}
