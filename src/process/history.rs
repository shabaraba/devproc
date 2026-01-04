use crate::models::HistoryEntry;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub struct HistoryManager {
    entries: Vec<HistoryEntry>,
    max_entries: usize,
}

impl HistoryManager {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    pub fn add_entry(&mut self, entry: HistoryEntry) {
        self.entries.insert(0, entry);
        if self.entries.len() > self.max_entries {
            self.entries.truncate(self.max_entries);
        }
    }

    pub fn list_entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self.entries)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load(&mut self, path: &PathBuf) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }
        let json = fs::read_to_string(path)?;
        self.entries = serde_json::from_str(&json)?;
        Ok(())
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new(100)
    }
}
