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

        // Try to find build.gradle or build.gradle.kts
        if let Ok(gradle_scripts) = Self::detect_gradle(dir) {
            scripts.extend(gradle_scripts);
        }

        // Try to find Makefile
        if let Ok(make_scripts) = Self::detect_makefile(dir) {
            scripts.extend(make_scripts);
        }

        // Try to find Cargo.toml
        if let Ok(cargo_scripts) = Self::detect_cargo(dir) {
            scripts.extend(cargo_scripts);
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

    fn detect_gradle(dir: &Path) -> Result<Vec<Script>> {
        let gradle_path = if dir.join("build.gradle").exists() {
            dir.join("build.gradle")
        } else if dir.join("build.gradle.kts").exists() {
            dir.join("build.gradle.kts")
        } else {
            return Ok(Vec::new());
        };

        let content = fs::read_to_string(&gradle_path)?;
        let mut scripts = Vec::new();

        // Common Gradle tasks
        let common_tasks = vec![
            "build", "clean", "test", "bootRun", "run", "assemble", "check",
        ];

        for task in common_tasks {
            // Check if task is mentioned in the file
            if content.contains(&format!("task {}", task)) || content.contains(&format!("'{}'", task)) {
                scripts.push(Script::new(
                    task.to_string(),
                    format!("gradle {}", task),
                    gradle_path.clone(),
                    dir.to_path_buf(),
                ));
            }
        }

        // Always add basic tasks even if not explicitly defined
        if scripts.is_empty() {
            for task in &["build", "clean", "test"] {
                scripts.push(Script::new(
                    task.to_string(),
                    format!("gradle {}", task),
                    gradle_path.clone(),
                    dir.to_path_buf(),
                ));
            }
        }

        Ok(scripts)
    }

    fn detect_makefile(dir: &Path) -> Result<Vec<Script>> {
        let makefile_path = if dir.join("Makefile").exists() {
            dir.join("Makefile")
        } else if dir.join("makefile").exists() {
            dir.join("makefile")
        } else {
            return Ok(Vec::new());
        };

        let content = fs::read_to_string(&makefile_path)?;
        let mut scripts = Vec::new();

        // Parse Makefile targets
        for line in content.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }

            if let Some(target) = line.split(':').next() {
                let target = target.trim();
                // Skip special targets and variables
                if target.starts_with('.') || target.contains('=') || target.is_empty() {
                    continue;
                }

                scripts.push(Script::new(
                    target.to_string(),
                    format!("make {}", target),
                    makefile_path.clone(),
                    dir.to_path_buf(),
                ));
            }
        }

        Ok(scripts)
    }

    fn detect_cargo(dir: &Path) -> Result<Vec<Script>> {
        let cargo_path = dir.join("Cargo.toml");
        if !cargo_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&cargo_path)?;
        let mut scripts = Vec::new();

        // Parse TOML to find bin targets
        if let Ok(toml_value) = content.parse::<toml::Value>() {
            // Check for [[bin]] sections
            if let Some(bins) = toml_value.get("bin").and_then(|v| v.as_array()) {
                for bin in bins {
                    if let Some(name) = bin.get("name").and_then(|v| v.as_str()) {
                        scripts.push(Script::new(
                            format!("run-{}", name),
                            format!("cargo run --bin {}", name),
                            cargo_path.clone(),
                            dir.to_path_buf(),
                        ));
                    }
                }
            }
        }

        // Always add common cargo commands
        let common_commands = vec![
            ("build", "cargo build"),
            ("run", "cargo run"),
            ("test", "cargo test"),
            ("check", "cargo check"),
            ("clean", "cargo clean"),
        ];

        for (name, cmd) in common_commands {
            scripts.push(Script::new(
                name.to_string(),
                cmd.to_string(),
                cargo_path.clone(),
                dir.to_path_buf(),
            ));
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
