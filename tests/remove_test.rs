#[path = "common/mod.rs"]
mod common;

use common::with_temp_trex_dir;
use serial_test::serial;
use trex::model::{SavedSession, Sessions};
use trex::storage;

#[serial]
#[test]
fn remove_session_by_name() {
    with_temp_trex_dir(|| {
        let mut data = Sessions::new();
        data.sessions.push(SavedSession {
            name: "dev".to_string(),
            path: "/home".to_string(),
            windows: vec![],
            session_options: vec![],
            window_options: vec![],
        });
        storage::save_sessions(&data).unwrap();

        let result = trex::commands::remove::execute(Some("dev")).unwrap();
        assert_eq!(result, "Removed session 'dev'.");

        let loaded = storage::load_sessions().unwrap();
        assert!(loaded.sessions.is_empty());
    });
}

#[serial]
#[test]
fn remove_all_sessions() {
    with_temp_trex_dir(|| {
        let mut data = Sessions::new();
        data.sessions.push(SavedSession {
            name: "dev".to_string(),
            path: "/home".to_string(),
            windows: vec![],
            session_options: vec![],
            window_options: vec![],
        });
        data.sessions.push(SavedSession {
            name: "work".to_string(),
            path: "/office".to_string(),
            windows: vec![],
            session_options: vec![],
            window_options: vec![],
        });
        storage::save_sessions(&data).unwrap();

        let result = trex::commands::remove::execute(None).unwrap();
        assert_eq!(result, "Removed all saved sessions.");

        let loaded = storage::load_sessions().unwrap();
        assert!(loaded.sessions.is_empty());
    });
}

#[serial]
#[test]
fn remove_nonexistent_session() {
    with_temp_trex_dir(|| {
        let result = trex::commands::remove::execute(Some("nonexistent"));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No saved session named 'nonexistent'"));
    });
}

#[serial]
#[test]
fn remove_multiple_times_with_add_in_between() {
    with_temp_trex_dir(|| {
        let mut data = Sessions::new();
        data.sessions.push(SavedSession {
            name: "a".to_string(),
            path: "/a".to_string(),
            windows: vec![],
            session_options: vec![],
            window_options: vec![],
        });
        data.sessions.push(SavedSession {
            name: "b".into(),
            path: "/b".into(),
            windows: vec![],
            session_options: vec![],
            window_options: vec![],
        });
        storage::save_sessions(&data).unwrap();

        trex::commands::remove::execute(Some("a")).unwrap();
        let loaded = storage::load_sessions().unwrap();
        assert_eq!(loaded.sessions.len(), 1);
        assert_eq!(loaded.sessions[0].name, "b");

        let mut data = storage::load_sessions().unwrap();
        data.sessions.push(SavedSession {
            name: "c".into(),
            path: "/c".into(),
            windows: vec![],
            session_options: vec![],
            window_options: vec![],
        });
        storage::save_sessions(&data).unwrap();

        let loaded = storage::load_sessions().unwrap();
        assert_eq!(loaded.sessions.len(), 2);
    });
}
