use crate::process::ProcessManager;
use crate::script::{ScriptDetector, ScriptRunner};
use anyhow::Result;
use chrono::Local;
use clap::{Parser, Subcommand};
use std::env;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "devproc")]
#[command(about = "Development process manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all managed processes
    List {
        /// Output format (table or json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    /// List available scripts
    Scripts {
        /// Directory to search for scripts
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },
    /// Start a script
    Start {
        /// Script name to start
        script: String,
        /// Custom port
        #[arg(short, long)]
        port: Option<u16>,
        /// Custom name
        #[arg(short, long)]
        name: Option<String>,
        /// Extra arguments to pass to the script
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Run a custom command
    Run {
        /// Command to run
        command: String,
        /// Custom name
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Kill a process
    Kill {
        /// Process name or ID
        target: String,
        /// Signal to send (default: 15/SIGTERM)
        #[arg(short, long, default_value = "15")]
        signal: i32,
    },
}

pub fn handle_list(format: &str, manager: &mut ProcessManager) -> Result<()> {
    manager.refresh();
    let processes = manager.list_processes();

    if format == "json" {
        let mut output = Vec::new();
        for process in processes {
            let metrics = manager.get_metrics(process).unwrap_or_default();
            output.push(serde_json::json!({
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
                "status": format!("{:?}", metrics.status),
            }));
        }
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        // Table format
        println!(
            "{:<30} {:<8} {:<6} {:<10} {:<6} {:<10}",
            "NAME", "PID", "CPU", "MEM", "PORT", "STARTED"
        );
        println!("{}", "-".repeat(80));

        for process in processes {
            let metrics = manager.get_metrics(process).unwrap_or_default();
            let duration = Local::now().signed_duration_since(process.started_at);
            let started = if duration.num_hours() > 0 {
                format!("{}h ago", duration.num_hours())
            } else if duration.num_minutes() > 0 {
                format!("{}m ago", duration.num_minutes())
            } else {
                format!("{}s ago", duration.num_seconds())
            };

            let memory_mb = metrics.memory_bytes / 1024 / 1024;
            let port_str = process.port.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string());

            println!(
                "{:<30} {:<8} {:>5.1}% {:>7}MB {:<6} {:<10}",
                process.name,
                process.pid,
                metrics.cpu_percent,
                memory_mb,
                port_str,
                started
            );
        }
    }

    Ok(())
}

pub fn handle_scripts(dir: Option<PathBuf>) -> Result<()> {
    let search_dir = dir.unwrap_or_else(|| env::current_dir().unwrap());
    let scripts = ScriptDetector::detect_scripts(&search_dir)?;

    println!("{:<20} {:<50}", "NAME", "COMMAND");
    println!("{}", "-".repeat(70));

    for script in scripts {
        println!("{:<20} {:<50}", script.name, script.command);
    }

    Ok(())
}

pub fn handle_start(
    script_name: &str,
    port: Option<u16>,
    name: Option<String>,
    args: Vec<String>,
    manager: &mut ProcessManager,
) -> Result<()> {
    let cwd = env::current_dir()?;
    let scripts = ScriptDetector::detect_scripts(&cwd)?;

    let script = scripts
        .iter()
        .find(|s| s.name == script_name)
        .ok_or_else(|| anyhow::anyhow!("Script '{}' not found", script_name))?;

    let process = ScriptRunner::run_script(script, name, port, args)?;
    println!("Started: {} (PID: {})", process.name, process.pid);

    manager.add_process(process);
    save_manager_state(manager)?;

    Ok(())
}

pub fn handle_run(
    command: &str,
    name: Option<String>,
    manager: &mut ProcessManager,
) -> Result<()> {
    let cwd = env::current_dir()?;
    let process = ScriptRunner::run_manual_command(command, cwd, name)?;
    println!("Started: {} (PID: {})", process.name, process.pid);

    manager.add_process(process);
    save_manager_state(manager)?;

    Ok(())
}

pub fn handle_kill(target: &str, signal: i32, manager: &mut ProcessManager) -> Result<()> {
    manager.refresh();

    // Try to find by name first
    let process = manager
        .get_process_by_name(target)
        .or_else(|| {
            // Try to parse as PID
            target.parse::<u32>().ok().and_then(|pid| manager.get_process_by_pid(pid))
        })
        .ok_or_else(|| anyhow::anyhow!("Process '{}' not found", target))?;

    let id = process.id;
    let name = process.name.clone();

    manager.kill_process(&id, signal)?;
    println!("Killed: {}", name);

    save_manager_state(manager)?;

    Ok(())
}

fn get_state_file() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("devproc")
        .join("processes.json")
}

pub fn load_manager_state() -> Result<ProcessManager> {
    let mut manager = ProcessManager::new();
    let state_file = get_state_file();

    if let Some(parent) = state_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let _ = manager.load_state(&state_file);
    Ok(manager)
}

pub fn save_manager_state(manager: &ProcessManager) -> Result<()> {
    let state_file = get_state_file();
    if let Some(parent) = state_file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    manager.save_state(&state_file)
}
