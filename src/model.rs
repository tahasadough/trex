use serde::{Deserialize, Serialize};

pub const SESSIONS_FILE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sessions {
    pub version: u32,
    pub sessions: Vec<SavedSession>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSession {
    pub name: String,
    pub path: String,
    pub windows: Vec<SavedWindow>,
    pub session_options: Vec<String>,
    pub window_options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedWindow {
    pub index: u32,
    pub name: String,
    pub layout: String,
    pub active_pane: u32,
    pub path: String,
    pub panes: Vec<SavedPane>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedPane {
    pub index: u32,
    pub path: String,
    pub active: bool,
    pub command: Option<String>,
}

impl Sessions {
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: SESSIONS_FILE_VERSION,
            sessions: Vec::new(),
        }
    }

    #[must_use]
    pub fn session_names(&self) -> Vec<&str> {
        self.sessions.iter().map(|s| s.name.as_str()).collect()
    }

    pub fn remove_session(&mut self, name: &str) {
        self.sessions.retain(|s| s.name != name);
    }
}

impl Default for Sessions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sessions_new_creates_empty() {
        let data = Sessions::new();
        assert_eq!(data.version, SESSIONS_FILE_VERSION);
        assert!(data.sessions.is_empty());
    }

    #[test]
    fn sessions_session_names_returns_names() {
        let mut data = Sessions::new();
        data.sessions.push(SavedSession {
            name: "dev".into(),
            path: "/home".into(),
            windows: vec![],
            session_options: vec![],
            window_options: vec![],
        });
        data.sessions.push(SavedSession {
            name: "work".into(),
            path: "/work".into(),
            windows: vec![],
            session_options: vec![],
            window_options: vec![],
        });
        let names = data.session_names();
        assert_eq!(names, vec!["dev", "work"]);
    }

    #[test]
    fn sessions_remove_session_removes_by_name() {
        let mut data = Sessions::new();
        data.sessions.push(SavedSession {
            name: "dev".into(),
            path: "/home".into(),
            windows: vec![],
            session_options: vec![],
            window_options: vec![],
        });
        data.sessions.push(SavedSession {
            name: "work".into(),
            path: "/work".into(),
            windows: vec![],
            session_options: vec![],
            window_options: vec![],
        });
        data.remove_session("dev");
        assert_eq!(data.sessions.len(), 1);
        assert_eq!(data.sessions[0].name, "work");
    }

    #[test]
    fn sessions_serialize_roundtrip() {
        let mut data = Sessions::new();
        data.sessions.push(SavedSession {
            name: "test".into(),
            path: "/tmp".into(),
            windows: vec![SavedWindow {
                index: 0,
                name: "main".into(),
                layout: "even-horizontal".into(),
                active_pane: 0,
                path: "/tmp".into(),
                panes: vec![SavedPane {
                    index: 0,
                    path: "/tmp".into(),
                    active: true,
                    command: Some("nvim".into()),
                }],
            }],
            session_options: vec!["default-command \"\"".into()],
            window_options: vec![],
        });
        let json = serde_json::to_string(&data).unwrap();
        let deserialized: Sessions = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.sessions.len(), 1);
        assert_eq!(deserialized.sessions[0].name, "test");
        assert_eq!(
            deserialized.sessions[0].windows[0].panes[0]
                .command
                .as_deref(),
            Some("nvim")
        );
    }
}
