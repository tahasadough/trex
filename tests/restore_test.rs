#[path = "common/mod.rs"]
mod common;

use common::with_temp_trex_dir;
use serial_test::serial;
use trex::model::{SavedPane, SavedSession, SavedWindow, Sessions};
use trex::storage;
use trex::tmux::MockTmux;

#[serial]
#[test]
fn restore_full_flow_integration() {
    with_temp_trex_dir(|| {
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
                panes: vec![SavedPane {
                    index: 0,
                    path: "/home/project".into(),
                    active: true,
                    command: Some("nvim".into()),
                }],
            }],
            session_options: vec!["default-command \"\"".into()],
            window_options: vec![],
        });
        storage::save_sessions(&data).unwrap();

        let mock = MockTmux::new();
        let result = trex::commands::restore::execute(&mock, false).unwrap();
        assert_eq!(result, "Sessions restored.");
    });
}

#[serial]
#[test]
fn restore_quiet_mode_integration() {
    with_temp_trex_dir(|| {
        let mut data = Sessions::new();
        data.sessions.push(SavedSession {
            name: "dev".into(),
            path: "/home".into(),
            windows: vec![],
            session_options: vec![],
            window_options: vec![],
        });
        storage::save_sessions(&data).unwrap();

        let mock = MockTmux::new();
        let result = trex::commands::restore::execute(&mock, true).unwrap();
        assert!(result.is_empty());
    });
}

#[serial]
#[test]
fn restore_no_data_integration() {
    with_temp_trex_dir(|| {
        let mock = MockTmux::new();
        let result = trex::commands::restore::execute(&mock, false);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No saved sessions"));
    });
}

#[serial]
#[test]
fn restore_skips_existing_session() {
    with_temp_trex_dir(|| {
        let mut data = Sessions::new();
        data.sessions.push(SavedSession {
            name: "dev".into(),
            path: "/home".into(),
            windows: vec![],
            session_options: vec![],
            window_options: vec![],
        });
        storage::save_sessions(&data).unwrap();

        let mut mock = MockTmux::new();
        mock.existing_sessions = vec!["dev".into()];
        let result = trex::commands::restore::execute(&mock, false).unwrap();
        assert_eq!(result, "Sessions restored.");
    });
}

#[serial]
#[test]
fn restore_applies_layouts() {
    with_temp_trex_dir(|| {
        let mut data = Sessions::new();
        data.sessions.push(SavedSession {
            name: "dev".into(),
            path: "/home".into(),
            windows: vec![SavedWindow {
                index: 0,
                name: "main".into(),
                layout: "tiled".into(),
                active_pane: 0,
                path: "/home".into(),
                panes: vec![],
            }],
            session_options: vec![],
            window_options: vec![],
        });
        storage::save_sessions(&data).unwrap();

        let mock = MockTmux::new();
        let result = trex::commands::restore::execute(&mock, false).unwrap();
        assert_eq!(result, "Sessions restored.");
    });
}
