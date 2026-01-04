use crate::cli::load_manager_state;
use crate::process::ProcessManager;
use crate::script::ScriptDetector;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::io::{self, BufRead, Write};

#[derive(Debug, Serialize, Deserialize)]
struct McpRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct McpResponse {
    jsonrpc: String,
    id: Value,
    result: Option<Value>,
    error: Option<McpError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct McpError {
    code: i32,
    message: String,
}

pub struct McpServer {
    manager: ProcessManager,
}

impl McpServer {
    pub fn new() -> Result<Self> {
        let manager = load_manager_state()?;
        Ok(Self { manager })
    }

    pub fn run(&mut self) -> Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let request: McpRequest = serde_json::from_str(&line)?;
            let response = self.handle_request(request);

            let response_json = serde_json::to_string(&response)?;
            writeln!(stdout, "{}", response_json)?;
            stdout.flush()?;
        }

        Ok(())
    }

    fn handle_request(&mut self, request: McpRequest) -> McpResponse {
        let result = match request.method.as_str() {
            "list_processes" => self.list_processes(),
            "list_scripts" => self.list_scripts(request.params),
            "start_script" => self.start_script(request.params),
            "kill_process" => self.kill_process(request.params),
            "get_process_detail" => self.get_process_detail(request.params),
            _ => Err(anyhow::anyhow!("Unknown method: {}", request.method)),
        };

        match result {
            Ok(value) => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(value),
                error: None,
            },
            Err(e) => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(McpError {
                    code: -32603,
                    message: e.to_string(),
                }),
            },
        }
    }

    fn list_processes(&mut self) -> Result<Value> {
        self.manager.refresh();
        let processes = self.manager.list_processes();

        let mut result = Vec::new();
        for process in processes {
            let metrics = self.manager.get_metrics(process).unwrap_or_default();
            result.push(serde_json::json!({
                "id": process.id,
                "name": process.name,
                "pid": process.pid,
                "command": process.command,
                "args": process.args,
                "port": process.port,
                "started_at": process.started_at,
                "cpu_percent": metrics.cpu_percent,
                "memory_bytes": metrics.memory_bytes,
            }));
        }

        Ok(serde_json::to_value(result)?)
    }

    fn list_scripts(&self, params: Option<Value>) -> Result<Value> {
        let dir = if let Some(params) = params {
            if let Some(dir_str) = params.get("dir").and_then(|v| v.as_str()) {
                std::path::PathBuf::from(dir_str)
            } else {
                env::current_dir()?
            }
        } else {
            env::current_dir()?
        };

        let scripts = ScriptDetector::detect_scripts(&dir)?;
        let result: Vec<_> = scripts
            .iter()
            .map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "command": s.command,
                    "source_file": s.source_file,
                })
            })
            .collect();

        Ok(serde_json::to_value(result)?)
    }

    fn start_script(&mut self, params: Option<Value>) -> Result<Value> {
        let params = params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?;
        let script_name = params
            .get("script")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing script name"))?;

        let cwd = env::current_dir()?;
        let scripts = ScriptDetector::detect_scripts(&cwd)?;

        let script = scripts
            .iter()
            .find(|s| s.name == script_name)
            .ok_or_else(|| anyhow::anyhow!("Script not found"))?;

        let port = params.get("port").and_then(|v| v.as_u64()).map(|p| p as u16);
        let name = params.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());

        let process = crate::script::ScriptRunner::run_script(script, name, port, Vec::new())?;
        let process_id = process.id;
        self.manager.add_process(process);

        Ok(serde_json::json!({
            "success": true,
            "process_id": process_id,
        }))
    }

    fn kill_process(&mut self, params: Option<Value>) -> Result<Value> {
        let params = params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?;
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing process name"))?;

        let process = self
            .manager
            .get_process_by_name(name)
            .ok_or_else(|| anyhow::anyhow!("Process not found"))?;

        let id = process.id;
        self.manager.kill_process(&id, 15)?;

        Ok(serde_json::json!({
            "success": true,
        }))
    }

    fn get_process_detail(&mut self, params: Option<Value>) -> Result<Value> {
        let params = params.ok_or_else(|| anyhow::anyhow!("Missing parameters"))?;
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing process name"))?;

        self.manager.refresh();
        let process = self
            .manager
            .get_process_by_name(name)
            .ok_or_else(|| anyhow::anyhow!("Process not found"))?;

        let metrics = self.manager.get_metrics(process).unwrap_or_default();

        Ok(serde_json::json!({
            "id": process.id,
            "name": process.name,
            "pid": process.pid,
            "command": process.command,
            "args": process.args,
            "cwd": process.cwd,
            "port": process.port,
            "started_at": process.started_at,
            "cpu_percent": metrics.cpu_percent,
            "memory_bytes": metrics.memory_bytes,
            "memory_percent": metrics.memory_percent,
            "status": format!("{:?}", metrics.status),
        }))
    }
}
