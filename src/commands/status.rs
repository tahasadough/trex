use crate::error::TrexResult;
use crate::storage;

pub struct StatusInfo {
    pub count: usize,
    pub sessions: Vec<(String, String)>,
    pub modified: String,
}

/// # Errors
/// Returns [`TrexError`] if the sessions file cannot be read or deserialized.
pub fn execute() -> TrexResult<StatusInfo> {
    let data = storage::load_sessions()?;
    let path = storage::sessions_file();

    let modified = if !path.exists() {
        String::new()
    } else if let Ok(metadata) = std::fs::metadata(&path) {
        if let Ok(modified_time) = metadata.modified() {
            let duration = modified_time
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            let secs = duration.as_secs();
            #[expect(clippy::cast_possible_wrap)]
            chrono::DateTime::from_timestamp(secs as i64, 0).map_or_else(
                || "unknown".to_string(),
                |dt| dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            )
        } else {
            "unknown".to_string()
        }
    } else {
        "unknown".to_string()
    };

    let sessions: Vec<(String, String)> = data
        .sessions
        .iter()
        .map(|s| (s.name.clone(), s.path.clone()))
        .collect();

    let count = sessions.len();

    Ok(StatusInfo {
        count,
        sessions,
        modified,
    })
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
    fn status_empty_when_no_sessions() {
        with_trex_dir(|| {
            let info = execute().unwrap();
            assert_eq!(info.count, 0);
            assert!(info.sessions.is_empty());
        });
    }

    #[serial]
    #[test]
    fn status_returns_session_info() {
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

            let info = execute().unwrap();
            assert_eq!(info.count, 1);
            assert_eq!(info.sessions[0], ("dev".to_string(), "/home".to_string()));
        });
    }
}
