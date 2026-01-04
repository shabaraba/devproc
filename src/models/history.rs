use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: Uuid,
    pub command: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub executed_at: DateTime<Utc>,
    pub exit_code: Option<i32>,
    pub duration_secs: Option<u64>,
}

impl HistoryEntry {
    #[allow(dead_code)]
    pub fn new(command: String, args: Vec<String>, cwd: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            command,
            args,
            cwd,
            executed_at: Utc::now(),
            exit_code: None,
            duration_secs: None,
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
