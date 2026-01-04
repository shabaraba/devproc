use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub name: String,
    pub command: String,
    pub source_file: PathBuf,
    pub project_dir: PathBuf,
    pub is_running: bool,
    pub running_process_id: Option<Uuid>,
}

impl Script {
    pub fn new(
        name: String,
        command: String,
        source_file: PathBuf,
        project_dir: PathBuf,
    ) -> Self {
        Self {
            name,
            command,
            source_file,
            project_dir,
            is_running: false,
            running_process_id: None,
        }
    }
}
