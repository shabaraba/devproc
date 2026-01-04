use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedProcess {
    pub id: Uuid,
    pub name: String,
    pub pid: u32,
    pub command: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub port: Option<u16>,
    pub started_at: DateTime<Utc>,
    pub source: ProcessSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProcessSource {
    Script { file: String, name: String },
    Manual { command: String },
    External,
}

impl ManagedProcess {
    pub fn new(
        name: String,
        pid: u32,
        command: String,
        args: Vec<String>,
        cwd: PathBuf,
        port: Option<u16>,
        source: ProcessSource,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            pid,
            command,
            args,
            cwd,
            port,
            started_at: Utc::now(),
            source,
        }
    }

    pub fn full_command(&self) -> String {
        if self.args.is_empty() {
            self.command.clone()
        } else {
            format!("{} {}", self.command, self.args.join(" "))
        }
    }
}
