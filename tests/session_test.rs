#[path = "common/mod.rs"]
mod common;

use trex::model::Sessions;

#[test]
fn session_new_is_empty() {
    let data = Sessions::new();
    assert_eq!(data.version, 1);
    assert!(data.sessions.is_empty());
}

#[test]
fn session_roundtrip_serialization() {
    let mut data = Sessions::new();
    data.sessions.push(trex::model::SavedSession {
        name: "test".into(),
        path: "/tmp".into(),
        windows: vec![trex::model::SavedWindow {
            index: 0,
            name: "main".into(),
            layout: "tiled".into(),
            active_pane: 0,
            path: "/tmp".into(),
            panes: vec![trex::model::SavedPane {
                index: 0,
                path: "/tmp".into(),
                active: true,
                command: Some("htop".into()),
            }],
        }],
        session_options: vec![],
        window_options: vec![],
    });

    let json = serde_json::to_string_pretty(&data).unwrap();
    let deserialized: Sessions = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.sessions.len(), 1);
    assert_eq!(deserialized.sessions[0].name, "test");
    assert_eq!(
        deserialized.sessions[0].windows[0].panes[0]
            .command
            .as_deref(),
        Some("htop")
    );
}

#[test]
fn session_remove_by_name() {
    let mut data = Sessions::new();
    data.sessions.push(trex::model::SavedSession {
        name: "a".into(),
        path: "/a".into(),
        windows: vec![],
        session_options: vec![],
        window_options: vec![],
    });
    data.sessions.push(trex::model::SavedSession {
        name: "b".into(),
        path: "/b".into(),
        windows: vec![],
        session_options: vec![],
        window_options: vec![],
    });
    data.remove_session("a");
    assert_eq!(data.sessions.len(), 1);
    assert_eq!(data.sessions[0].name, "b");
}

#[test]
fn session_names() {
    let mut data = Sessions::new();
    data.sessions.push(trex::model::SavedSession {
        name: "x".into(),
        path: "/x".into(),
        windows: vec![],
        session_options: vec![],
        window_options: vec![],
    });
    let names = data.session_names();
    assert_eq!(names, vec!["x"]);
}

#[test]
fn session_empty_names() {
    let data = Sessions::new();
    let names: Vec<&str> = data.session_names();
    assert!(names.is_empty());
}
