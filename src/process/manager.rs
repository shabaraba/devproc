use crate::models::{ManagedProcess, ProcessMetrics, ProcessStatus};
use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use sysinfo::{Pid, System};
use uuid::Uuid;

pub struct ProcessManager {
    processes: HashMap<Uuid, ManagedProcess>,
    system: System,
}

impl ProcessManager {
    pub fn new() -> Self {
        let system = System::new_all();
        Self {
            processes: HashMap::new(),
            system,
        }
    }

    pub fn add_process(&mut self, process: ManagedProcess) {
        self.processes.insert(process.id, process);
    }

    pub fn remove_process(&mut self, id: &Uuid) -> Option<ManagedProcess> {
        self.processes.remove(id)
    }

    pub fn get_process(&self, id: &Uuid) -> Option<&ManagedProcess> {
        self.processes.get(id)
    }

    pub fn get_process_by_name(&self, name: &str) -> Option<&ManagedProcess> {
        self.processes.values().find(|p| p.name == name)
    }

    pub fn get_process_by_pid(&self, pid: u32) -> Option<&ManagedProcess> {
        self.processes.values().find(|p| p.pid == pid)
    }

    pub fn list_processes(&self) -> Vec<&ManagedProcess> {
        self.processes.values().collect()
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();

        // Remove processes that are no longer running
        let mut to_remove = Vec::new();
        for (id, process) in &self.processes {
            if !self.system.process(Pid::from_u32(process.pid)).is_some() {
                to_remove.push(*id);
            }
        }
        for id in to_remove {
            self.processes.remove(&id);
        }
    }

    pub fn get_metrics(&self, process: &ManagedProcess) -> Result<ProcessMetrics> {

        let sys_process = self
            .system
            .process(Pid::from_u32(process.pid))
            .context("Process not found")?;

        let status = match sys_process.status().to_string().as_str() {
            "run" | "running" => ProcessStatus::Running,
            "sleep" | "sleeping" => ProcessStatus::Sleeping,
            "zombie" => ProcessStatus::Zombie,
            _ => ProcessStatus::Unknown,
        };

        Ok(ProcessMetrics {
            cpu_percent: sys_process.cpu_usage(),
            memory_bytes: sys_process.memory(),
            memory_percent: sys_process.memory() as f32 / self.system.total_memory() as f32 * 100.0,
            threads: sys_process.tasks().map(|t| t.len() as u32).unwrap_or(0),
            status,
        })
    }

    pub fn kill_process(&mut self, id: &Uuid, signal: i32) -> Result<()> {
        let process = self.processes.get(id).context("Process not found")?;
        let pid = process.pid;

        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid as NixPid;

            let sig = match signal {
                9 => Signal::SIGKILL,
                2 => Signal::SIGINT,
                _ => Signal::SIGTERM,
            };

            kill(NixPid::from_raw(pid as i32), sig)
                .context("Failed to send signal to process")?;
        }

        #[cfg(not(unix))]
        {
            // Windows fallback
            Command::new("taskkill")
                .args(&["/PID", &pid.to_string(), "/F"])
                .output()
                .context("Failed to kill process")?;
        }

        self.processes.remove(id);
        Ok(())
    }

    pub fn detect_port(command: &str) -> Option<u16> {
        // Try to detect port from common patterns
        let patterns = vec![
            r"-p\s+(\d+)",           // -p 3000
            r"--port\s+(\d+)",       // --port 3000
            r"--port=(\d+)",         // --port=3000
            r":(\d{4,5})\b",         // :3000
            r"PORT=(\d+)",           // PORT=3000
        ];

        for pattern in patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(command) {
                    if let Some(port_str) = caps.get(1) {
                        if let Ok(port) = port_str.as_str().parse::<u16>() {
                            return Some(port);
                        }
                    }
                }
            }
        }

        None
    }

    pub fn generate_name(
        script_name: &str,
        cwd: &PathBuf,
        port: Option<u16>,
    ) -> String {
        let dir_name = cwd
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        match port {
            Some(p) => format!("{}/{}/{}", dir_name, script_name, p),
            None => format!("{}/{}", dir_name, script_name),
        }
    }

    pub fn save_state(&self, path: &PathBuf) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.processes)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_state(&mut self, path: &PathBuf) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }
        let json = std::fs::read_to_string(path)?;
        self.processes = serde_json::from_str(&json)?;
        Ok(())
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
