use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub memory_percent: f32,
    pub threads: u32,
    pub status: ProcessStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessStatus {
    Running,
    Sleeping,
    Zombie,
    Unknown,
}

impl Default for ProcessMetrics {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            memory_bytes: 0,
            memory_percent: 0.0,
            threads: 0,
            status: ProcessStatus::Unknown,
        }
    }
}
