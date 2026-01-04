use crate::models::{ManagedProcess, ProcessSource, Script};
use crate::process::ProcessManager;
use anyhow::{Context, Result};
use std::process::{Command, Stdio};

pub struct ScriptRunner;

impl ScriptRunner {
    pub fn run_script(
        script: &Script,
        custom_name: Option<String>,
        custom_port: Option<u16>,
        extra_args: Vec<String>,
    ) -> Result<ManagedProcess> {
        // Determine the command to run
        let (cmd, args) = Self::parse_command(&script.command, &extra_args)?;

        // Detect port from command if not provided
        let full_command = format!("{} {}", script.command, extra_args.join(" "));
        let port = custom_port.or_else(|| ProcessManager::detect_port(&full_command));

        // Generate name
        let name = custom_name.unwrap_or_else(|| {
            ProcessManager::generate_name(&script.name, &script.project_dir, port)
        });

        // Start the process
        let child = Command::new(&cmd)
            .args(&args)
            .current_dir(&script.project_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn process")?;

        let pid = child.id();

        // Create ManagedProcess
        let process = ManagedProcess::new(
            name,
            pid,
            cmd,
            args,
            script.project_dir.clone(),
            port,
            ProcessSource::Script {
                file: script
                    .source_file
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                name: script.name.clone(),
            },
        );

        Ok(process)
    }

    fn parse_command(command: &str, extra_args: &[String]) -> Result<(String, Vec<String>)> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            anyhow::bail!("Empty command");
        }

        let cmd = parts[0].to_string();
        let mut args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
        args.extend(extra_args.iter().cloned());

        Ok((cmd, args))
    }

    pub fn run_manual_command(
        command: &str,
        cwd: std::path::PathBuf,
        custom_name: Option<String>,
    ) -> Result<ManagedProcess> {
        let (cmd, args) = Self::parse_command(command, &[])?;

        // Detect port
        let port = ProcessManager::detect_port(command);

        // Generate name
        let name = custom_name.unwrap_or_else(|| {
            let script_name = cmd.clone();
            ProcessManager::generate_name(&script_name, &cwd, port)
        });

        // Start the process
        let child = Command::new(&cmd)
            .args(&args)
            .current_dir(&cwd)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn process")?;

        let pid = child.id();

        // Create ManagedProcess
        let process = ManagedProcess::new(
            name,
            pid,
            cmd,
            args,
            cwd,
            port,
            ProcessSource::Manual {
                command: command.to_string(),
            },
        );

        Ok(process)
    }
}
