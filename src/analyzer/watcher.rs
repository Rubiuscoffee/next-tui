use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc as std_mpsc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub enum AnalyzerEvent {
    RoutesUpdated,
    EnvUpdated,
}

pub struct ProjectWatcher {
    _watcher: Option<RecommendedWatcher>,
}

impl ProjectWatcher {
    pub fn start(
        base_dir: PathBuf,
        tx: UnboundedSender<AnalyzerEvent>,
    ) -> Result<Self, notify::Error> {
        let (std_tx, std_rx) = std_mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = std_tx.send(event);
                }
            },
            Config::default().with_poll_interval(Duration::from_millis(500)),
        )?;

        // Try watching app/ or src/app/
        let app_dir = if base_dir.join("src").join("app").exists() {
            Some(base_dir.join("src").join("app"))
        } else if base_dir.join("app").exists() {
            Some(base_dir.join("app"))
        } else {
            None
        };

        if let Some(ref dir) = app_dir {
            let _ = watcher.watch(dir.as_ref(), RecursiveMode::Recursive);
        }

        // Watch root directory (non-recursive) for .env and package.json changes
        let _ = watcher.watch(&base_dir, RecursiveMode::NonRecursive);

        // Spawn background task to process events with basic debouncing
        tokio::task::spawn_blocking(move || {
            let mut last_event_time = std::time::Instant::now();
            while let Ok(event) = std_rx.recv() {
                // Throttle notifications to at most 1 every 250ms
                if last_event_time.elapsed() < Duration::from_millis(250) {
                    continue;
                }
                last_event_time = std::time::Instant::now();

                let mut routes_changed = false;
                let mut env_changed = false;

                for path in &event.paths {
                    let p_str = path.to_string_lossy();
                    if p_str.contains(".env") {
                        env_changed = true;
                    } else if p_str.ends_with(".ts")
                        || p_str.ends_with(".tsx")
                        || p_str.ends_with(".js")
                        || p_str.ends_with(".jsx")
                    {
                        routes_changed = true;
                    }
                }

                if routes_changed {
                    let _ = tx.send(AnalyzerEvent::RoutesUpdated);
                }
                if env_changed {
                    let _ = tx.send(AnalyzerEvent::EnvUpdated);
                }
            }
        });

        Ok(Self {
            _watcher: Some(watcher),
        })
    }
}
