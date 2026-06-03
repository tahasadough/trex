#[path = "common/mod.rs"]
mod common;

use common::with_temp_trex_dir;
use serial_test::serial;
use trex::model::{SavedSession, Sessions};
use trex::storage;

#[serial]
#[test]
fn status_with_no_sessions() {
    with_temp_trex_dir(|| {
        let info = trex::commands::status::execute().unwrap();
        assert_eq!(info.count, 0);
    });
}

#[serial]
#[test]
fn status_with_sessions() {
    with_temp_trex_dir(|| {
        let mut data = Sessions::new();
        data.sessions.push(SavedSession {
            name: "dev".into(),
            path: "/home/dev".into(),
            windows: vec![],
            session_options: vec![],
            window_options: vec![],
        });
        storage::save_sessions(&data).unwrap();

        let info = trex::commands::status::execute().unwrap();
        assert_eq!(info.count, 1);
        assert_eq!(
            info.sessions[0],
            ("dev".to_string(), "/home/dev".to_string())
        );
    });
}

#[serial]
#[test]
fn status_multiple_sessions() {
    with_temp_trex_dir(|| {
        let mut data = Sessions::new();
        data.sessions.push(SavedSession {
            name: "alpha".into(),
            path: "/a".into(),
            windows: vec![],
            session_options: vec![],
            window_options: vec![],
        });
        data.sessions.push(SavedSession {
            name: "beta".into(),
            path: "/b".into(),
            windows: vec![],
            session_options: vec![],
            window_options: vec![],
        });
        storage::save_sessions(&data).unwrap();

        let info = trex::commands::status::execute().unwrap();
        assert_eq!(info.count, 2);
        assert_eq!(info.sessions[0], ("alpha".to_string(), "/a".to_string()));
        assert_eq!(info.sessions[1], ("beta".to_string(), "/b".to_string()));
    });
}

#[serial]
#[test]
fn list_sessions() {
    with_temp_trex_dir(|| {
        let names = trex::commands::list::execute().unwrap();
        assert!(names.is_empty());

        let mut data = Sessions::new();
        data.sessions.push(SavedSession {
            name: "alpha".into(),
            path: "/a".into(),
            windows: vec![],
            session_options: vec![],
            window_options: vec![],
        });
        data.sessions.push(SavedSession {
            name: "beta".into(),
            path: "/b".into(),
            windows: vec![],
            session_options: vec![],
            window_options: vec![],
        });
        storage::save_sessions(&data).unwrap();

        let names = trex::commands::list::execute().unwrap();
        assert_eq!(names, vec!["alpha", "beta"]);
    });
}
