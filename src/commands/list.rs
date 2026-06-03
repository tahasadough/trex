use crate::error::TrexResult;
use crate::storage;

/// # Errors
/// Returns [`TrexError`] if the sessions file cannot be read or deserialized.
pub fn execute() -> TrexResult<Vec<String>> {
    let data = storage::load_sessions()?;
    Ok(data
        .session_names()
        .into_iter()
        .map(ToString::to_string)
        .collect())
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
    fn list_empty_when_no_sessions_saved() {
        with_trex_dir(|| {
            let names = execute().unwrap();
            assert!(names.is_empty());
        });
    }

    #[serial]
    #[test]
    fn list_returns_saved_session_names() {
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

            let names = execute().unwrap();
            assert_eq!(names, vec!["dev", "work"]);
        });
    }
}
