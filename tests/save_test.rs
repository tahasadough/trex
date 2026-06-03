#[path = "common/mod.rs"]
mod common;

use common::with_temp_trex_dir;
use serial_test::serial;
use trex::storage;
use trex::tmux::MockTmux;

#[serial]
#[test]
fn save_and_load_sessions_integration() {
    with_temp_trex_dir(|| {
        let mut mock = MockTmux::new();
        mock.sessions = vec!["dev".into()];
        mock.windows = vec![trex::tmux::WindowInfo {
            index: 0,
            name: "main".into(),
            layout: "even-horizontal".into(),
        }];
        mock.panes = vec![trex::tmux::PaneInfo {
            index: 0,
            path: "/home".into(),
        }];

        let result = trex::commands::save::execute(&mock, None, false).unwrap();
        assert!(result.contains("Saved 1 session(s)"));

        let loaded = storage::load_sessions().unwrap();
        assert_eq!(loaded.sessions.len(), 1);
        assert_eq!(loaded.sessions[0].name, "dev");
    });
}

#[serial]
#[test]
fn save_with_ignore_integration() {
    with_temp_trex_dir(|| {
        storage::add_to_ignore("dev").unwrap();

        let mut mock = MockTmux::new();
        mock.sessions = vec!["dev".into(), "work".into()];

        let result = trex::commands::save::execute(&mock, None, false).unwrap();
        assert!(result.contains("Saved 1 session(s)"));

        let loaded = storage::load_sessions().unwrap();
        assert_eq!(loaded.sessions.len(), 1);
        assert_eq!(loaded.sessions[0].name, "work");
    });
}

#[serial]
#[test]
fn save_current_session_integration() {
    with_temp_trex_dir(|| {
        let mut mock = MockTmux::new();
        mock.sessions = vec!["dev".into(), "work".into()];
        mock.current_session = "dev".into();

        let result = trex::commands::save::execute(&mock, None, true).unwrap();
        assert!(result.contains("Saved 1 session(s)"));

        let loaded = storage::load_sessions().unwrap();
        assert_eq!(loaded.sessions[0].name, "dev");
    });
}

#[serial]
#[test]
fn save_multiple_windows_and_panes() {
    with_temp_trex_dir(|| {
        let mut mock = MockTmux::new();
        mock.sessions = vec!["dev".into()];
        mock.windows = vec![
            trex::tmux::WindowInfo {
                index: 0,
                name: "main".into(),
                layout: "even-horizontal".into(),
            },
            trex::tmux::WindowInfo {
                index: 1,
                name: "logs".into(),
                layout: "tiled".into(),
            },
        ];
        mock.panes = vec![trex::tmux::PaneInfo {
            index: 0,
            path: "/home".into(),
        }];

        let result = trex::commands::save::execute(&mock, None, false).unwrap();
        assert!(result.contains("Saved 1 session(s)"));

        let loaded = storage::load_sessions().unwrap();
        assert_eq!(loaded.sessions[0].windows.len(), 2);
    });
}
