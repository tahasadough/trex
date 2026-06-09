use crate::error::{TrexError, TrexResult};
use crate::model::{SavedPane, SavedSession, SavedWindow, Sessions};
use crate::storage;
use crate::tmux::{TmuxClient, WindowInfo};

/// # Errors
/// Returns [`TrexError::NoTmuxSessions`] if no tmux sessions exist,
/// or [`TrexError`] if any tmux command or file operation fails.
pub fn execute(
    tmux: &dyn TmuxClient,
    name: Option<&str>,
    current_only: bool,
) -> TrexResult<String> {
    if !tmux.has_sessions()? {
        return Err(TrexError::NoTmuxSessions);
    }

    storage::ensure_trex_dir()?;
    let ignored = storage::load_ignore_list();
    // Preserve last-known commands from previous save so that panes whose
    // application was closed before a re-save still get their commands restored.
    let prev = storage::load_sessions().unwrap_or_default();

    let session_names: Vec<String> = if let Some(n) = name {
        if !tmux.has_session(n)? {
            return Err(TrexError::Generic(format!(
                "No active tmux session named '{n}'"
            )));
        }
        vec![n.to_string()]
    } else if current_only {
        let n = tmux.current_session_name()?;
        vec![n]
    } else {
        tmux.list_sessions()?
    };

    let mut data = Sessions::new();

    for name in &session_names {
        if ignored.contains(name) {
            continue;
        }

        let path = tmux.session_path(name)?;
        let windows = tmux.list_windows(name)?;
        let session_options = tmux.get_session_options(name)?;
        let window_options = tmux.get_window_options(name)?;

        let saved_windows: Vec<SavedWindow> = windows
            .iter()
            .map(|win| save_window(tmux, name, win, &prev))
            .collect::<TrexResult<_>>()?;

        data.sessions.push(SavedSession {
            name: name.clone(),
            path,
            windows: saved_windows,
            session_options,
            window_options,
        });
    }

    storage::save_sessions(&data)?;
    let count = data.sessions.len();
    Ok(format!(
        "Saved {count} session(s) to {}",
        storage::sessions_file().display()
    ))
}

/// # Errors
/// Returns [`TrexError`] if any tmux command fails.
fn save_window(
    tmux: &dyn TmuxClient,
    session: &str,
    win: &WindowInfo,
    prev: &Sessions,
) -> TrexResult<SavedWindow> {
    let path = tmux.window_path(session, win.index)?;
    let panes = tmux.list_panes(session, win.index)?;

    let saved_panes: Vec<SavedPane> = panes
        .iter()
        .map(|p| {
            let active = tmux
                .pane_active(session, win.index, p.index)
                .unwrap_or(false);
            let command = tmux
                .pane_command(session, win.index, p.index)
                .unwrap_or(None);
            let command = command.or_else(|| {
                previous_pane_command(prev, session, win.index, p.index)
            });
            SavedPane {
                index: p.index,
                path: p.path.clone(),
                active,
                command,
            }
        })
        .collect();

    let active_pane = saved_panes.iter().find(|p| p.active).map_or(0, |p| p.index);

    Ok(SavedWindow {
        index: win.index,
        name: win.name.clone(),
        layout: win.layout.clone(),
        active_pane,
        path,
        panes: saved_panes,
    })
}

fn previous_pane_command(
    prev: &Sessions,
    session: &str,
    window_index: u32,
    pane_index: u32,
) -> Option<String> {
    prev.sessions
        .iter()
        .find(|s| s.name == session)
        .and_then(|s| s.windows.iter().find(|w| w.index == window_index))
        .and_then(|w| w.panes.iter().find(|p| p.index == pane_index))
        .and_then(|p| p.command.clone().filter(|c| !c.is_empty()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::with_trex_dir;
    use crate::tmux::MockTmux;
    use serial_test::serial;

    #[serial]
    #[test]
    fn save_no_sessions_returns_error() {
        let mock = MockTmux::new();
        let result = execute(&mock, None, false);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No active tmux sessions"));
    }

    #[serial]
    #[test]
    fn save_all_sessions() {
        let mut mock = MockTmux::new();
        mock.sessions = vec!["dev".into(), "work".into()];
        mock.windows = vec![WindowInfo {
            index: 0,
            name: "main".into(),
            layout: "even-horizontal".into(),
        }];
        mock.panes = vec![crate::tmux::PaneInfo {
            index: 0,
            path: "/home".into(),
        }];

        with_trex_dir(|| {
            let result = execute(&mock, None, false).unwrap();
            assert!(result.contains("Saved 2 session(s)"));
        });
    }

    #[serial]
    #[test]
    fn save_current_session_only() {
        let mut mock = MockTmux::new();
        mock.sessions = vec!["dev".into(), "work".into()];
        mock.current_session = "dev".into();
        mock.windows = vec![WindowInfo {
            index: 0,
            name: "main".into(),
            layout: "even-horizontal".into(),
        }];
        mock.panes = vec![crate::tmux::PaneInfo {
            index: 0,
            path: "/home".into(),
        }];

        with_trex_dir(|| {
            let result = execute(&mock, None, true).unwrap();
            assert!(result.contains("Saved 1 session(s)"));
        });
    }

    #[serial]
    #[test]
    fn save_by_name() {
        let mut mock = MockTmux::new();
        mock.sessions = vec!["dev".into(), "work".into()];
        mock.existing_sessions = vec!["dev".into()];
        mock.windows = vec![WindowInfo {
            index: 0,
            name: "main".into(),
            layout: "even-horizontal".into(),
        }];
        mock.panes = vec![crate::tmux::PaneInfo {
            index: 0,
            path: "/home".into(),
        }];

        with_trex_dir(|| {
            let result = execute(&mock, Some("dev"), false).unwrap();
            assert!(result.contains("Saved 1 session(s)"));
        });
    }

    #[serial]
    #[test]
    fn save_by_name_not_found_returns_error() {
        let mut mock = MockTmux::new();
        mock.sessions = vec!["dev".into()];

        with_trex_dir(|| {
            let result = execute(&mock, Some("nonexistent"), false);
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("nonexistent"));
        });
    }

    #[serial]
    #[test]
    fn save_skips_ignored_sessions() {
        let mut mock = MockTmux::new();
        mock.sessions = vec!["dev".into(), "work".into()];

        with_trex_dir(|| {
            storage::add_to_ignore("dev").unwrap();

            let result = execute(&mock, None, false).unwrap();
            assert!(result.contains("Saved 1 session(s)"));
            assert!(!result.contains("dev"));
        });
    }

    #[serial]
    #[test]
    fn save_with_panes_and_commands() {
        let mut mock = MockTmux::new();
        mock.sessions = vec!["test".into()];
        mock.windows = vec![WindowInfo {
            index: 0,
            name: "code".into(),
            layout: "tiled".into(),
        }];
        mock.panes = vec![
            crate::tmux::PaneInfo {
                index: 0,
                path: "/project".into(),
            },
            crate::tmux::PaneInfo {
                index: 1,
                path: "/project/src".into(),
            },
        ];
        mock.active_panes.insert((0, 0), true);
        mock.active_panes.insert((0, 1), false);
        mock.pane_commands
            .insert((0, 0), Some("nvim main.rs".into()));
        mock.pane_commands.insert((0, 1), None);

        with_trex_dir(|| {
            let result = execute(&mock, None, false).unwrap();
            assert!(result.contains("Saved 1 session(s)"));

            let loaded = crate::storage::load_sessions().unwrap();
            assert_eq!(loaded.sessions[0].windows[0].panes.len(), 2);
            assert_eq!(
                loaded.sessions[0].windows[0].panes[0].command.as_deref(),
                Some("nvim main.rs")
            );
        });
    }
}
