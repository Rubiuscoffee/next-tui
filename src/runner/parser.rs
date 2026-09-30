use regex::Regex;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogKind {
    Turbopack,
    Http { method: String, status: u16 },
    Action(String),
    Server,
    Info,
    Warn,
    Error,
    Raw,
}

#[derive(Debug, Clone)]
pub struct ParsedLog {
    pub timestamp: String,
    pub kind: LogKind,
    pub message: String,
    pub raw: String,
}

#[derive(Debug, Default, Clone)]
pub struct ExtractedMetadata {
    pub local_url: Option<String>,
    pub network_url: Option<String>,
    pub port: Option<u16>,
    pub next_version: Option<String>,
    pub turbopack: Option<bool>,
    pub is_ready: bool,
}

pub struct LogParser {
    ansi_regex: Regex,
    local_url_regex: Regex,
    network_url_regex: Regex,
    version_regex: Regex,
    compiled_regex: Regex,
    http_regex: Regex,
    action_regex: Regex,
    port_reassigned_regex: Regex,
}

impl Default for LogParser {
    fn default() -> Self {
        Self {
            ansi_regex: Regex::new(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])").unwrap(),
            local_url_regex: Regex::new(r"(?:Local:|http://localhost:)(\s*)(http://localhost:(\d+)|localhost:(\d+))").unwrap(),
            network_url_regex: Regex::new(r"(?:Network:|http://)(\s*)(http://(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}:\d+))").unwrap(),
            version_regex: Regex::new(r"Next\.js\s+v?(\d+\.\d+\.\d+)").unwrap(),
            compiled_regex: Regex::new(r"(?:Compiled|compiled)\s+(?:in\s+|\/)?([^\s]+)?\s*in\s+(\d+(?:\.\d+)?(?:ms|s))").unwrap(),
            http_regex: Regex::new(r"(GET|POST|PUT|DELETE|PATCH)\s+([^\s]+)\s+(\d{3})").unwrap(),
            action_regex: Regex::new(r"(?:\[Action\]\s*|Action\s+call:\s*|action\s+)([a-zA-Z0-9_$]+)").unwrap(),
            port_reassigned_regex: Regex::new(r"[Pp]ort\s+\d+\s+is\s+in\s+use.*?(?:using|trying)\s+(?:available\s+)?(?:port\s+)?(\d+)").unwrap(),
        }
    }
}

impl LogParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn strip_ansi(&self, text: &str) -> String {
        self.ansi_regex.replace_all(text, "").to_string()
    }

    pub fn current_timestamp() -> String {
        if let Ok(duration) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            let total_secs = duration.as_secs();
            let secs = total_secs % 60;
            let mins = (total_secs / 60) % 60;
            let hours = (total_secs / 3600) % 24;
            format!("{:02}:{:02}:{:02}", hours, mins, secs)
        } else {
            "00:00:00".to_string()
        }
    }

    pub fn parse_line(&self, raw_line: &str) -> (ParsedLog, ExtractedMetadata) {
        let clean = self.strip_ansi(raw_line).trim().to_string();
        let timestamp = Self::current_timestamp();
        let mut meta = ExtractedMetadata::default();

        // Detect port reassignment e.g. "Port 3000 is in use, using available port 3001 instead"
        let is_port_reassigned = if let Some(caps) = self.port_reassigned_regex.captures(&clean) {
            if let Some(p) = caps.get(1) {
                if let Ok(new_port) = p.as_str().parse::<u16>() {
                    meta.port = Some(new_port);
                    meta.local_url = Some(format!("http://localhost:{}", new_port));
                }
            }
            true
        } else {
            false
        };

        // Extract Local URL & Port if not reassigned
        if meta.port.is_none() {
            if let Some(caps) = self.local_url_regex.captures(&clean) {
                if let Some(m) = caps.get(2) {
                    let mut url = m.as_str().to_string();
                    if !url.starts_with("http://") {
                        url = format!("http://{}", url);
                    }
                    meta.local_url = Some(url);
                }
                if let Some(p) = caps.get(3).or_else(|| caps.get(4)) {
                    if let Ok(port) = p.as_str().parse::<u16>() {
                        meta.port = Some(port);
                    }
                }
            } else if clean.contains("http://localhost:") {
                if let Some(idx) = clean.find("http://localhost:") {
                    let rest = &clean[idx..];
                    let end = rest.find(|c: char| c.is_whitespace() || c == ']').unwrap_or(rest.len());
                    let url = rest[..end].to_string();
                    if let Some(port_str) = url.strip_prefix("http://localhost:") {
                        if let Ok(port) = port_str.parse::<u16>() {
                            meta.port = Some(port);
                        }
                    }
                    meta.local_url = Some(url);
                }
            }
        }

        // Extract Network URL
        if let Some(caps) = self.network_url_regex.captures(&clean) {
            if let Some(m) = caps.get(2) {
                meta.network_url = Some(m.as_str().to_string());
            }
        }

        // Extract Next.js version
        if let Some(caps) = self.version_regex.captures(&clean) {
            if let Some(v) = caps.get(1) {
                meta.next_version = Some(format!("v{}", v.as_str()));
            }
        }

        // Detect Turbopack
        if clean.to_lowercase().contains("turbopack") {
            meta.turbopack = Some(true);
        }

        // Detect Ready
        if clean.contains("Ready in") || clean.contains("ready - started server") || clean.contains("started server on") {
            meta.is_ready = true;
        }

        // Classify Log Kind
        let kind = if is_port_reassigned {
            LogKind::Warn
        } else if clean.contains("[Turbopack]") || clean.to_lowercase().contains("turbopack") || self.compiled_regex.is_match(&clean) {
            LogKind::Turbopack
        } else if let Some(caps) = self.http_regex.captures(&clean) {
            let method = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_else(|| "GET".to_string());
            let status = caps.get(3).and_then(|s| s.as_str().parse::<u16>().ok()).unwrap_or(200);
            LogKind::Http { method, status }
        } else if let Some(caps) = self.action_regex.captures(&clean) {
            let action_name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_else(|| "action".to_string());
            LogKind::Action(action_name)
        } else if clean.starts_with("[Server]") || clean.contains("DB Query") || clean.contains("prisma:") {
            LogKind::Server
        } else if clean.contains("ERROR") || clean.contains("Error:") || clean.contains("Failed to") {
            LogKind::Error
        } else if clean.contains("WARN") || clean.contains("Warning:") {
            LogKind::Warn
        } else if clean.starts_with("[INFO]") || clean.contains("Payload:") {
            LogKind::Info
        } else {
            LogKind::Raw
        };

        let log = ParsedLog {
            timestamp,
            kind,
            message: clean,
            raw: raw_line.to_string(),
        };

        (log, meta)
    }
}
