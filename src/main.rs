mod cli;
mod models;
mod process;
mod script;
mod tui;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, load_manager_state};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::List { format }) => {
            let mut manager = load_manager_state()?;
            cli::commands::handle_list(&format, &mut manager)?;
        }
        Some(Commands::Scripts { dir }) => {
            cli::commands::handle_scripts(dir)?;
        }
        Some(Commands::Start {
            script,
            port,
            name,
            args,
        }) => {
            let mut manager = load_manager_state()?;
            cli::commands::handle_start(&script, port, name, args, &mut manager)?;
        }
        Some(Commands::Run { command, name }) => {
            let mut manager = load_manager_state()?;
            cli::commands::handle_run(&command, name, &mut manager)?;
        }
        Some(Commands::Kill { target, signal }) => {
            let mut manager = load_manager_state()?;
            cli::commands::handle_kill(&target, signal, &mut manager)?;
        }
        None => {
            // No command specified, launch TUI
            let mut app = tui::App::new()?;
            app.run()?;
        }
    }

    Ok(())
}
