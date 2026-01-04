use crate::models::Script;
use anyhow::Result;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub struct ScriptDetector;

impl ScriptDetector {
    pub fn detect_scripts(dir: &Path) -> Result<Vec<Script>> {
        let mut scripts = Vec::new();

        // Try to find package.json
        if let Ok(pkg_scripts) = Self::detect_package_json(dir) {
            scripts.extend(pkg_scripts);
        }

        Ok(scripts)
    }

    fn detect_package_json(dir: &Path) -> Result<Vec<Script>> {
        let package_json_path = dir.join("package.json");
        if !package_json_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&package_json_path)?;
        let parsed: Value = serde_json::from_str(&content)?;

        let mut scripts = Vec::new();

        if let Some(scripts_obj) = parsed.get("scripts").and_then(|v| v.as_object()) {
            for (name, command) in scripts_obj {
                if let Some(cmd_str) = command.as_str() {
                    // Skip pre/post scripts
                    if name.starts_with("pre") || name.starts_with("post") {
                        continue;
                    }

                    scripts.push(Script::new(
                        name.clone(),
                        cmd_str.to_string(),
                        package_json_path.clone(),
                        dir.to_path_buf(),
                    ));
                }
            }
        }

        Ok(scripts)
    }

    pub fn find_project_dirs(start_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut project_dirs = Vec::new();

        // Check current directory
        if start_dir.join("package.json").exists() {
            project_dirs.push(start_dir.to_path_buf());
        }

        // Check subdirectories (non-recursive for now)
        if let Ok(entries) = fs::read_dir(start_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    let path = entry.path();
                    // Skip node_modules, .git, etc.
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with('.') || name == "node_modules" {
                            continue;
                        }
                    }

                    if path.join("package.json").exists() {
                        project_dirs.push(path);
                    }
                }
            }
        }

        Ok(project_dirs)
    }
}
