use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "trex", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Save tmux sessions to disk
    #[command(visible_alias = "s")]
    Save {
        /// Session name to save (default: all sessions)
        name: Option<String>,
        #[arg(short, long)]
        current: bool,
    },
    /// Restore saved tmux sessions
    #[command(visible_alias = "r")]
    Restore {
        #[arg(short, long)]
        quiet: bool,
    },
    /// List saved sessions
    #[command(visible_alias = "l")]
    Ls,
    /// Show session info and timestamps
    #[command(visible_alias = "st")]
    Status,
    /// Remove saved session(s)
    #[command(visible_alias = "rm")]
    Remove {
        name: Option<String>,
        #[arg(short, long)]
        all: bool,
    },
    /// Exclude session from saves
    #[command(visible_alias = "ig")]
    Ignore {
        name: Option<String>,
        #[arg(short, long)]
        list: bool,
    },
    /// Stop ignoring a session
    #[command(visible_alias = "uig")]
    Unignore { name: String },
    /// Configure auto-restore on shell start
    #[command(visible_alias = "a")]
    Auto {
        #[command(subcommand)]
        action: AutoAction,
    },
    /// Update trex to latest version
    #[command(visible_alias = "up")]
    Update,
}

#[derive(Debug, Subcommand)]
pub enum AutoAction {
    /// Enable auto-restore on shell start
    Enable,
    /// Disable auto-restore
    Disable,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parse_save_command() {
        let cli = Cli::try_parse_from(["trex", "save"]).unwrap();
        match cli.command {
            Command::Save { name, current } => {
                assert!(name.is_none());
                assert!(!current);
            }
            _ => panic!("Expected Save"),
        }
    }

    #[test]
    fn parse_save_current_command() {
        let cli = Cli::try_parse_from(["trex", "save", "--current"]).unwrap();
        match cli.command {
            Command::Save { name, current } => {
                assert!(name.is_none());
                assert!(current);
            }
            _ => panic!("Expected Save"),
        }
    }

    #[test]
    fn parse_save_with_name() {
        let cli = Cli::try_parse_from(["trex", "save", "my-session"]).unwrap();
        match cli.command {
            Command::Save { name, current } => {
                assert_eq!(name.as_deref(), Some("my-session"));
                assert!(!current);
            }
            _ => panic!("Expected Save"),
        }
    }

    #[test]
    fn parse_save_alias() {
        let cli = Cli::try_parse_from(["trex", "s"]).unwrap();
        assert!(matches!(cli.command, Command::Save { .. }));
    }

    #[test]
    fn parse_restore_alias() {
        let cli = Cli::try_parse_from(["trex", "r"]).unwrap();
        assert!(matches!(cli.command, Command::Restore { .. }));
    }

    #[test]
    fn parse_restore_quiet() {
        let cli = Cli::try_parse_from(["trex", "restore", "--quiet"]).unwrap();
        match cli.command {
            Command::Restore { quiet } => assert!(quiet),
            _ => panic!("Expected Restore"),
        }
    }

    #[test]
    fn parse_ls() {
        let cli = Cli::try_parse_from(["trex", "ls"]).unwrap();
        assert!(matches!(cli.command, Command::Ls));
    }

    #[test]
    fn parse_ls_alias() {
        let cli = Cli::try_parse_from(["trex", "l"]).unwrap();
        assert!(matches!(cli.command, Command::Ls));
    }

    #[test]
    fn parse_status() {
        let cli = Cli::try_parse_from(["trex", "status"]).unwrap();
        assert!(matches!(cli.command, Command::Status));
    }

    #[test]
    fn parse_remove_with_name() {
        let cli = Cli::try_parse_from(["trex", "remove", "dev"]).unwrap();
        match cli.command {
            Command::Remove { name, all } => {
                assert_eq!(name.as_deref(), Some("dev"));
                assert!(!all);
            }
            _ => panic!("Expected Remove"),
        }
    }

    #[test]
    fn parse_remove_all() {
        let cli = Cli::try_parse_from(["trex", "remove", "--all"]).unwrap();
        match cli.command {
            Command::Remove { name, all } => {
                assert!(name.is_none());
                assert!(all);
            }
            _ => panic!("Expected Remove"),
        }
    }

    #[test]
    fn parse_ignore() {
        let cli = Cli::try_parse_from(["trex", "ignore", "dev"]).unwrap();
        match cli.command {
            Command::Ignore { name, list } => {
                assert_eq!(name.as_deref(), Some("dev"));
                assert!(!list);
            }
            _ => panic!("Expected Ignore"),
        }
    }

    #[test]
    fn parse_ignore_list() {
        let cli = Cli::try_parse_from(["trex", "ignore", "--list"]).unwrap();
        match cli.command {
            Command::Ignore { name, list } => {
                assert!(name.is_none());
                assert!(list);
            }
            _ => panic!("Expected Ignore"),
        }
    }

    #[test]
    fn parse_unignore() {
        let cli = Cli::try_parse_from(["trex", "unignore", "dev"]).unwrap();
        match cli.command {
            Command::Unignore { name } => assert_eq!(name, "dev"),
            _ => panic!("Expected Unignore"),
        }
    }

    #[test]
    fn parse_auto_enable() {
        let cli = Cli::try_parse_from(["trex", "auto", "enable"]).unwrap();
        match cli.command {
            Command::Auto { action } => assert!(matches!(action, AutoAction::Enable)),
            _ => panic!("Expected Auto"),
        }
    }

    #[test]
    fn parse_auto_disable() {
        let cli = Cli::try_parse_from(["trex", "auto", "disable"]).unwrap();
        match cli.command {
            Command::Auto { action } => assert!(matches!(action, AutoAction::Disable)),
            _ => panic!("Expected Auto"),
        }
    }

    #[test]
    fn parse_update() {
        let cli = Cli::try_parse_from(["trex", "update"]).unwrap();
        assert!(matches!(cli.command, Command::Update));
    }

    #[test]
    fn parse_unknown_command_fails() {
        let result = Cli::try_parse_from(["trex", "unknown"]);
        assert!(result.is_err());
    }
}
