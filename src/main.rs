use std::fmt::Write;

use clap::Parser;
use trex::cli::{AutoAction, Cli, Command};
use trex::commands;
use trex::error::TrexResult;
use trex::tmux::Tmux;

/// # Errors
/// Delegates to the underlying command's error type.
fn run() -> TrexResult<String> {
    let cli = Cli::parse();
    let tmux = Tmux;

    match cli.command {
        Command::Save { name, current } => commands::save::execute(&tmux, name.as_deref(), current),
        Command::Restore { quiet } => commands::restore::execute(&tmux, quiet),
        Command::Ls => {
            let names = commands::list::execute()?;
            if names.is_empty() {
                Ok(String::new())
            } else {
                Ok(names.join("\n"))
            }
        }
        Command::Status => {
            let info = commands::status::execute()?;
            if info.count == 0 {
                Ok(String::from("No saved sessions. Run 'trex save' first."))
            } else {
                let mut out = format!("Saved sessions: {}\n", info.count);
                if !info.modified.is_empty() {
                    let _ = writeln!(out, "Last saved:     {}", info.modified);
                }
                out.push('\n');
                for (name, path) in &info.sessions {
                    let _ = writeln!(out, "  * {name}  ({path})");
                }
                Ok(out.trim().to_string())
            }
        }
        Command::Remove { name, all } => {
            if all {
                commands::remove::execute(None)
            } else {
                commands::remove::execute(name.as_deref())
            }
        }
        Command::Ignore { name, list } => {
            if list {
                commands::ignore::execute_list()
            } else if let Some(n) = name {
                commands::ignore::execute_add(&n)
            } else {
                commands::ignore::execute_list()
            }
        }
        Command::Unignore { name } => commands::ignore::execute_remove(&name),
        Command::Auto { action } => match action {
            AutoAction::Enable => commands::auto::execute_enable(),
            AutoAction::Disable => commands::auto::execute_disable(),
        },
        Command::Update => commands::update::execute(),
    }
}

fn main() {
    match run() {
        Ok(msg) => {
            if !msg.is_empty() {
                println!("{msg}");
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
