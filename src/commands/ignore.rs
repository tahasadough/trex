use crate::error::TrexResult;
use crate::storage;
#[cfg(test)]
use crate::test_helpers::with_trex_dir_path;

/// # Errors
/// Returns [`TrexError::AlreadyIgnored`] if the session is already ignored,
/// or a filesystem error if the ignore file cannot be written.
pub fn execute_add(name: &str) -> TrexResult<String> {
    storage::add_to_ignore(name)?;
    Ok(format!("Added '{name}' to ignore list."))
}

/// # Errors
/// Returns [`TrexError::NoIgnoredSessions`] if no ignore file exists,
/// or [`TrexError::Generic`] if the name is not in the ignore list.
pub fn execute_remove(name: &str) -> TrexResult<String> {
    storage::remove_from_ignore(name)?;
    Ok(format!("Removed '{name}' from ignore list."))
}

/// # Errors
/// Returns [`TrexError`] if the ignore file cannot be read.
pub fn execute_list() -> TrexResult<String> {
    let ignored = storage::load_ignore_list();
    if ignored.is_empty() {
        return Ok("No ignored sessions.".to_string());
    }
    let count = ignored.len();
    let sessions = ignored
        .iter()
        .map(|s| format!("  * {s}"))
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!("Ignored sessions: {count}\n{sessions}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[serial]
    #[test]
    fn add_ignore() {
        with_trex_dir_path(|_| {
            let result = execute_add("dev").unwrap();
            assert_eq!(result, "Added 'dev' to ignore list.");
        });
    }

    #[serial]
    #[test]
    fn add_duplicate_ignore_returns_error() {
        with_trex_dir_path(|_| {
            execute_add("dev").unwrap();
            let result = execute_add("dev");
            assert!(result.is_err());
        });
    }

    #[serial]
    #[test]
    fn remove_ignore() {
        with_trex_dir_path(|_| {
            execute_add("dev").unwrap();
            let result = execute_remove("dev").unwrap();
            assert_eq!(result, "Removed 'dev' from ignore list.");
        });
    }

    #[serial]
    #[test]
    fn list_ignored_empty() {
        with_trex_dir_path(|_| {
            let result = execute_list().unwrap();
            assert_eq!(result, "No ignored sessions.");
        });
    }

    #[serial]
    #[test]
    fn list_ignored_with_sessions() {
        with_trex_dir_path(|_| {
            execute_add("dev").unwrap();
            execute_add("work").unwrap();
            let result = execute_list().unwrap();
            assert!(result.contains("Ignored sessions: 2"));
            assert!(result.contains("* dev"));
            assert!(result.contains("* work"));
        });
    }
}
