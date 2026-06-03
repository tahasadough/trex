use crate::error::{TrexError, TrexResult};
use crate::model::Sessions;
use crate::storage;
use crate::tmux::{OptionScope, TmuxClient};

/// # Errors
/// Returns [`TrexError::NoSavedSessions`] if no sessions are saved (and not quiet),
/// or [`TrexError`] if any tmux command fails.
pub fn execute(tmux: &dyn TmuxClient, quiet: bool) -> TrexResult<String> {
    let data = storage::load_sessions()?;
    if data.sessions.is_empty() {
        if quiet {
            return Ok(String::new());
        }
        return Err(TrexError::NoSavedSessions);
    }

    let restored = pass1_create_sessions(tmux, &data)?;

    // Non-fatal: layout/option/command failures should not abort the restore
    pass2_apply_layouts_and_options(tmux, &data, &restored);

    pass3_send_commands(tmux, &data, &restored);

    if quiet {
        Ok(String::new())
    } else {
        Ok("Sessions restored.".to_string())
    }
}

/// # Errors
/// Returns [`TrexError`] if any tmux command fails.
fn pass1_create_sessions(tmux: &dyn TmuxClient, data: &Sessions) -> TrexResult<Vec<String>> {
    let mut restored = Vec::new();

    for session in &data.sessions {
        if tmux.has_session(&session.name)? {
            continue;
        }

        tmux.new_session(&session.name, &session.path)?;
        restored.push(session.name.clone());

        let mut windows = session.windows.iter();
        if let Some(first) = windows.next() {
            tmux.rename_window(&session.name, "_tmuxer_", &first.name)?;
            for pane in first.panes.iter().skip(1) {
                tmux.split_window(&session.name, first.index, &pane.path)?;
            }
        }
        for win in windows {
            tmux.new_window(&session.name, &win.name, &win.path)?;
            for pane in win.panes.iter().skip(1) {
                tmux.split_window(&session.name, win.index, &pane.path)?;
            }
        }
    }

    Ok(restored)
}

fn pass2_apply_layouts_and_options(tmux: &dyn TmuxClient, data: &Sessions, restored: &[String]) {
    for session in &data.sessions {
        if !restored.contains(&session.name) {
            continue;
        }

        for win in &session.windows {
            if !win.layout.is_empty() {
                let _ = tmux.select_layout(&session.name, win.index, &win.layout);
            }
            let _ = tmux.select_pane(&session.name, win.index, win.active_pane);
        }

        for opt in &session.session_options {
            if let Some((name, value)) = parse_option(opt) {
                let _ = tmux.set_option(&session.name, OptionScope::Session, &name, &value);
            }
        }

        for opt in &session.window_options {
            if let Some((name, value)) = parse_option(opt) {
                let _ = tmux.set_option(&session.name, OptionScope::Window, &name, &value);
            }
        }
    }
}

fn pass3_send_commands(tmux: &dyn TmuxClient, data: &Sessions, restored: &[String]) {
    for session in &data.sessions {
        if !restored.contains(&session.name) {
            continue;
        }

        for win in &session.windows {
            for pane in &win.panes {
                if let Some(ref cmd) = pane.command {
                    if !cmd.is_empty() {
                        let _ = tmux.send_keys(&session.name, win.index, pane.index, cmd);
                    }
                }
            }
        }
    }
}

fn parse_option(opt: &str) -> Option<(String, String)> {
    let trimmed = opt.trim();
    if trimmed.is_empty() {
        return None;
    }
    let space_pos = trimmed.find(' ')?;
    let name = trimmed[..space_pos].to_string();
    let value = trimmed[space_pos + 1..].trim().to_string();
    Some((name, value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{SavedPane, SavedSession, SavedWindow};
    use crate::test_helpers::with_trex_dir;
    use crate::tmux::MockTmux;
    use serial_test::serial;

    fn create_test_data() -> Sessions {
        let mut data = Sessions::new();
        data.sessions.push(SavedSession {
            name: "dev".into(),
            path: "/home".into(),
            windows: vec![SavedWindow {
                index: 0,
                name: "editor".into(),
                layout: "even-horizontal".into(),
                active_pane: 0,
                path: "/home/project".into(),
                panes: vec![
                    SavedPane {
                        index: 0,
                        path: "/home/project".into(),
                        active: true,
                        command: Some("nvim".into()),
                    },
                    SavedPane {
                        index: 1,
                        path: "/home/project".into(),
                        active: false,
                        command: Some("cargo build".into()),
                    },
                ],
            }],
            session_options: vec!["default-command \"\"".into()],
            window_options: vec![],
        });
        data
    }

    #[serial]
    #[test]
    fn restore_with_no_data_returns_error() {
        with_trex_dir(|| {
            let mock = MockTmux::new();
            let result = execute(&mock, false);
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("No saved sessions"));
        });
    }

    #[serial]
    #[test]
    fn restore_quiet_with_no_data_returns_ok() {
        with_trex_dir(|| {
            let mock = MockTmux::new();
            let result = execute(&mock, true).unwrap();
            assert!(result.is_empty());
        });
    }

    #[serial]
    #[test]
    fn restore_creates_sessions() {
        let mock = MockTmux::new();
        let data = create_test_data();

        with_trex_dir(|| {
            crate::storage::save_sessions(&data).unwrap();
            let result = execute(&mock, false).unwrap();
            assert_eq!(result, "Sessions restored.");
        });
    }

    #[serial]
    #[test]
    fn restore_skips_existing_sessions() {
        let mut mock = MockTmux::new();
        mock.existing_sessions = vec!["dev".into()];

        let data = create_test_data();

        with_trex_dir(|| {
            crate::storage::save_sessions(&data).unwrap();
            let result = execute(&mock, false).unwrap();
            assert_eq!(result, "Sessions restored.");
        });
    }

    #[test]
    fn parse_option_valid() {
        let result = parse_option("default-command \"\"");
        assert_eq!(result, Some(("default-command".into(), "\"\"".into())));
    }

    #[test]
    fn parse_option_empty_returns_none() {
        assert!(parse_option("").is_none());
        assert!(parse_option("   ").is_none());
    }

    #[test]
    fn parse_option_no_space_returns_none() {
        assert!(parse_option("default-command").is_none());
    }

    #[serial]
    #[test]
    fn restore_multiple_sessions() {
        let mock = MockTmux::new();
        let mut data = Sessions::new();
        data.sessions.push(SavedSession {
            name: "dev".into(),
            path: "/home".into(),
            windows: vec![SavedWindow {
                index: 0,
                name: "main".into(),
                layout: String::new(),
                active_pane: 0,
                path: "/home".into(),
                panes: vec![],
            }],
            session_options: vec![],
            window_options: vec![],
        });
        data.sessions.push(SavedSession {
            name: "work".into(),
            path: "/work".into(),
            windows: vec![SavedWindow {
                index: 0,
                name: "code".into(),
                layout: String::new(),
                active_pane: 0,
                path: "/work".into(),
                panes: vec![],
            }],
            session_options: vec![],
            window_options: vec![],
        });

        with_trex_dir(|| {
            crate::storage::save_sessions(&data).unwrap();
            let result = execute(&mock, false).unwrap();
            assert_eq!(result, "Sessions restored.");
        });
    }
}
