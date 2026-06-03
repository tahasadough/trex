use crate::error::{TrexError, TrexResult};
use crate::model::Sessions;
use crate::storage;

/// # Errors
/// Returns [`TrexError::SessionNotFound`] if the named session does not exist,
/// or [`TrexError`] if the sessions file cannot be read or written.
pub fn execute(name: Option<&str>) -> TrexResult<String> {
    let mut data = storage::load_sessions()?;

    if let Some(n) = name {
        if !data.sessions.iter().any(|s| s.name == n) {
            return Err(TrexError::SessionNotFound(n.to_string()));
        }
        data.remove_session(n);
        storage::save_sessions(&data)?;
        Ok(format!("Removed session '{n}'."))
    } else {
        storage::save_sessions(&Sessions::new())?;
        Ok("Removed all saved sessions.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{SavedSession, Sessions};
    use crate::storage;
    use serial_test::serial;

    fn with_trex_dir<F>(f: F)
    where
        F: FnOnce(),
    {
        crate::test_helpers::with_trex_dir(f);
    }

    #[serial]
    #[test]
    fn remove_session_by_name() {
        with_trex_dir(|| {
            let mut data = Sessions::new();
            data.sessions.push(SavedSession {
                name: "dev".into(),
                path: "/home".into(),
                windows: vec![],
                session_options: vec![],
                window_options: vec![],
            });
            storage::save_sessions(&data).unwrap();

            let result = execute(Some("dev")).unwrap();
            assert_eq!(result, "Removed session 'dev'.");
            let data = storage::load_sessions().unwrap();
            assert!(data.sessions.is_empty());
        });
    }

    #[serial]
    #[test]
    fn remove_nonexistent_session_returns_error() {
        with_trex_dir(|| {
            let mut data = Sessions::new();
            data.sessions.push(SavedSession {
                name: "dev".into(),
                path: "/home".into(),
                windows: vec![],
                session_options: vec![],
                window_options: vec![],
            });
            storage::save_sessions(&data).unwrap();

            let result = execute(Some("other"));
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("No saved session named 'other'"));
        });
    }

    #[serial]
    #[test]
    fn remove_all_sessions() {
        with_trex_dir(|| {
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
                path: "/office".into(),
                windows: vec![],
                session_options: vec![],
                window_options: vec![],
            });
            storage::save_sessions(&data).unwrap();

            let result = execute(None).unwrap();
            assert_eq!(result, "Removed all saved sessions.");
            let loaded = storage::load_sessions().unwrap();
            assert!(loaded.sessions.is_empty());
        });
    }
}
