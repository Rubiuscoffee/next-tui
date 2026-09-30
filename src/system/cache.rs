use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct CacheStatus {
    pub path: PathBuf,
    pub exists: bool,
    pub size_bytes: u64,
    pub formatted_size: String,
}

impl Default for CacheStatus {
    fn default() -> Self {
        Self {
            path: PathBuf::from(".next/cache"),
            exists: false,
            size_bytes: 0,
            formatted_size: "0 B".to_string(),
        }
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.0} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn calculate_cache_size(base_dir: &Path) -> CacheStatus {
    let cache_path = base_dir.join(".next").join("cache");
    if !cache_path.exists() {
        return CacheStatus {
            path: cache_path,
            exists: false,
            size_bytes: 0,
            formatted_size: "0 B".to_string(),
        };
    }

    let mut total_size: u64 = 0;
    for entry in WalkDir::new(&cache_path).into_iter().filter_map(|e| e.ok()) {
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_file() {
                total_size = total_size.saturating_add(metadata.len());
            }
        }
    }

    CacheStatus {
        path: cache_path,
        exists: true,
        size_bytes: total_size,
        formatted_size: format_bytes(total_size),
    }
}

pub fn purge_cache(base_dir: &Path) -> std::io::Result<u64> {
    let status = calculate_cache_size(base_dir);
    if status.exists && status.path.exists() {
        fs::remove_dir_all(&status.path)?;
        // Re-create the empty directory if needed or let next recreate it
        Ok(status.size_bytes)
    } else {
        Ok(0)
    }
}
