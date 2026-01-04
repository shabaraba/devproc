use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub ui: UiConfig,
    pub scripts: ScriptsConfig,
    pub naming: NamingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub refresh_interval: u64,
    pub max_history: usize,
    pub detect_external: bool,
    pub external_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub section_order: Vec<String>,
    pub initial_focus: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptsConfig {
    pub detect_files: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingConfig {
    pub format: String,
    pub format_no_port: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                refresh_interval: 1000,
                max_history: 100,
                detect_external: true,
                external_patterns: vec![
                    "node".to_string(),
                    "java".to_string(),
                    "python".to_string(),
                    "ruby".to_string(),
                    "go run".to_string(),
                    "cargo run".to_string(),
                ],
            },
            ui: UiConfig {
                theme: "dark".to_string(),
                section_order: vec![
                    "processes".to_string(),
                    "scripts".to_string(),
                    "history".to_string(),
                ],
                initial_focus: "processes".to_string(),
            },
            scripts: ScriptsConfig {
                detect_files: vec![
                    "package.json".to_string(),
                    "build.gradle".to_string(),
                    "build.gradle.kts".to_string(),
                    "Makefile".to_string(),
                    "Cargo.toml".to_string(),
                ],
                exclude_patterns: vec!["pre.*".to_string(), "post.*".to_string()],
            },
            naming: NamingConfig {
                format: "{dir}/{script}/{port}".to_string(),
                format_no_port: "{dir}/{script}".to_string(),
            },
        }
    }
}
