use crate::{
    cli::{Cli, Command},
    commands,
    error::TrexResult,
    tmux::TmuxClient,
};

pub struct App<T: TmuxClient> {
    pub tmux: T,
}

impl<T: TmuxClient> App<T> {
    pub fn new(tmux: T) -> Self {
        Self { tmux }
    }

    /// Parse and dispatch a CLI command.
    ///
    /// # Errors
    /// Delegates to the underlying command's error type.
    pub fn run(&self, cli: Cli) -> TrexResult<String> {
        match cli.command {
            Command::Save { name, current } => {
                commands::save::execute(&self.tmux, name.as_deref(), current)
            }
            Command::Restore { quiet } => commands::restore::execute(&self.tmux, quiet),
            Command::Ls => {
                let names = commands::list::execute()?;
                if names.is_empty() {
                    Ok(String::new())
                } else {
                    Ok(names.join("\n"))
                }
            }
            Command::Status => {
                use std::fmt::Write;
                let info = commands::status::execute()?;
                if info.count == 0 {
                    return Ok(String::from("No saved sessions. Run 'trex save' first."));
                }
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
                crate::cli::AutoAction::Enable => commands::auto::execute_enable(),
                crate::cli::AutoAction::Disable => commands::auto::execute_disable(),
            },
            Command::Update => commands::update::execute(),
        }
    }
}
