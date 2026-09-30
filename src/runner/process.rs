use crate::runner::parser::{ExtractedMetadata, LogParser, ParsedLog};
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnerStatus {
    Starting,
    Online,
    Stopped,
    Failed(Option<i32>),
    Error(String),
}

#[derive(Debug, Clone)]
pub enum RunnerEvent {
    Log(ParsedLog),
    StatusChanged(RunnerStatus),
    Metadata(ExtractedMetadata),
    Pid(u32),
    Exited(Option<i32>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Pnpm,
    Npm,
    Yarn,
    Bun,
}

impl PackageManager {
    pub fn command_name(&self) -> &'static str {
        match self {
            PackageManager::Pnpm => "pnpm",
            PackageManager::Npm => "npm",
            PackageManager::Yarn => "yarn",
            PackageManager::Bun => "bun",
        }
    }

    pub fn dev_args(&self) -> Vec<&'static str> {
        match self {
            PackageManager::Pnpm => vec!["dev"],
            PackageManager::Npm => vec!["run", "dev"],
            PackageManager::Yarn => vec!["dev"],
            PackageManager::Bun => vec!["dev"],
        }
    }
}

pub fn detect_package_manager(base_dir: &Path) -> PackageManager {
    if base_dir.join("pnpm-lock.yaml").exists() {
        PackageManager::Pnpm
    } else if base_dir.join("bun.lockb").exists() || base_dir.join("bun.lock").exists() {
        PackageManager::Bun
    } else if base_dir.join("yarn.lock").exists() {
        PackageManager::Yarn
    } else {
        PackageManager::Npm
    }
}

pub struct ProcessRunner {
    pub pid: Option<u32>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl Default for ProcessRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessRunner {
    pub fn new() -> Self {
        Self {
            pid: None,
            shutdown_tx: None,
        }
    }

    pub async fn spawn(
        &mut self,
        base_dir: &Path,
        tx: UnboundedSender<RunnerEvent>,
    ) -> Result<(), String> {
        let pkg_json = base_dir.join("package.json");

        if !pkg_json.exists() {
            let err_msg = "package.json not found in workspace directory".to_string();
            let _ = tx.send(RunnerEvent::StatusChanged(RunnerStatus::Error(err_msg.clone())));
            return Err(err_msg);
        }

        let pm = detect_package_manager(base_dir);
        let mut cmd = tokio::process::Command::new(pm.command_name());
        cmd.args(pm.dev_args())
            .current_dir(base_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        #[cfg(unix)]
        {
            cmd.process_group(0);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn {} dev: {}", pm.command_name(), e))?;

        let pid = child.id().unwrap_or(0);
        self.pid = Some(pid);
        let _ = tx.send(RunnerEvent::Pid(pid));
        let _ = tx.send(RunnerEvent::StatusChanged(RunnerStatus::Starting));

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let tx_out = tx.clone();
        if let Some(stdout) = stdout {
            tokio::spawn(async move {
                let parser = LogParser::new();
                let mut lines = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let (parsed, meta) = parser.parse_line(&line);
                    if meta.is_ready {
                        let _ = tx_out.send(RunnerEvent::StatusChanged(RunnerStatus::Online));
                    }
                    let _ = tx_out.send(RunnerEvent::Metadata(meta));
                    let _ = tx_out.send(RunnerEvent::Log(parsed));
                }
            });
        }

        let tx_err = tx.clone();
        if let Some(stderr) = stderr {
            tokio::spawn(async move {
                let parser = LogParser::new();
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let (parsed, meta) = parser.parse_line(&line);
                    let _ = tx_err.send(RunnerEvent::Metadata(meta));
                    let _ = tx_err.send(RunnerEvent::Log(parsed));
                }
            });
        }

        // Supervisor task to track process termination and handle shutdown
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);
        let tx_exit = tx.clone();

        tokio::spawn(async move {
            tokio::select! {
                _ = &mut shutdown_rx => {
                    #[cfg(unix)]
                    unsafe { libc_kill_pgid(pid as i32); }
                    let _ = child.kill().await;
                }
                status = child.wait() => {
                    let code = match status {
                        Ok(st) => st.code(),
                        Err(_) => None,
                    };
                    let _ = tx_exit.send(RunnerEvent::Exited(code));
                }
            }
        });

        Ok(())
    }

    pub async fn terminate(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(pid) = self.pid {
            #[cfg(unix)]
            unsafe {
                libc_kill_pgid(pid as i32);
            }
        }
    }
}

#[cfg(unix)]
unsafe fn libc_kill_pgid(pid: i32) {
    extern "C" {
        fn killpg(pgrp: i32, sig: i32) -> i32;
    }
    killpg(pid, 15); // SIGTERM
}

impl Drop for ProcessRunner {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(pid) = self.pid {
            #[cfg(unix)]
            unsafe {
                libc_kill_pgid(pid as i32);
            }
        }
    }
}
