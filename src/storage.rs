use std::fs;
use std::path::PathBuf;

use crate::error::{TrexError, TrexResult};
use crate::model::Sessions;

const TREX_DIR_ENV: &str = "TREX_DIR";

#[must_use]
pub fn trex_dir() -> PathBuf {
    if let Ok(dir) = std::env::var(TREX_DIR_ENV) {
        return PathBuf::from(dir);
    }
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("trex")
}

#[must_use]
pub fn sessions_file() -> PathBuf {
    let mut p = trex_dir();
    p.push("sessions.json");
    p
}

#[must_use]
pub fn ignore_file() -> PathBuf {
    let mut p = trex_dir();
    p.push("ignore");
    p
}

/// # Errors
/// Returns [`TrexError`] if the directory cannot be created.
pub fn ensure_trex_dir() -> TrexResult<()> {
    let dir = trex_dir();
    fs::create_dir_all(&dir)?;
    Ok(())
}

/// # Errors
/// Returns [`TrexError`] if the file cannot be read or deserialized.
pub fn load_sessions() -> TrexResult<Sessions> {
    let path = sessions_file();
    if !path.exists() {
        return Ok(Sessions::new());
    }
    let content = fs::read_to_string(&path)?;
    if content.trim().is_empty() {
        return Ok(Sessions::new());
    }
    let data: Sessions = serde_json::from_str(&content)?;
    Ok(data)
}

/// # Errors
/// Returns [`TrexError`] if the directory cannot be created or the file cannot be written.
pub fn save_sessions(data: &Sessions) -> TrexResult<()> {
    ensure_trex_dir()?;
    let path = sessions_file();
    let content = serde_json::to_string_pretty(data)?;
    fs::write(&path, content)?;
    Ok(())
}

#[must_use]
pub fn load_ignore_list() -> Vec<String> {
    let path = ignore_file();
    if !path.exists() {
        return Vec::new();
    }
    let content = fs::read_to_string(&path).unwrap_or_default();
    content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

/// # Errors
/// Returns [`TrexError::AlreadyIgnored`] if the session is already ignored,
/// or [`TrexError`] if the file cannot be written.
pub fn add_to_ignore(name: &str) -> TrexResult<()> {
    ensure_trex_dir()?;
    let ignored = load_ignore_list();
    if ignored.iter().any(|s| s == name) {
        return Err(TrexError::AlreadyIgnored(name.to_string()));
    }
    let path = ignore_file();
    let mut content = fs::read_to_string(&path).unwrap_or_default();
    if !content.ends_with('\n') && !content.is_empty() {
        content.push('\n');
    }
    content.push_str(name);
    content.push('\n');
    fs::write(&path, content)?;
    Ok(())
}

/// # Errors
/// Returns [`TrexError::NoIgnoredSessions`] if the ignore file does not exist,
/// or [`TrexError::Generic`] if the name is not in the ignore list,
/// or [`TrexError`] if the file cannot be read or written.
pub fn remove_from_ignore(name: &str) -> TrexResult<()> {
    let path = ignore_file();
    if !path.exists() {
        return Err(TrexError::NoIgnoredSessions);
    }
    let content = fs::read_to_string(&path)?;
    let filtered: Vec<&str> = content.lines().filter(|l| l.trim() != name).collect();
    if filtered.len() == content.lines().count() {
        return Err(TrexError::Generic(format!(
            "'{name}' is not in the ignore list"
        )));
    }
    fs::write(&path, filtered.join("\n") + "\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    fn with_temp_env<F>(dir: &TempDir, f: F)
    where
        F: FnOnce(),
    {
        let prev = std::env::var(TREX_DIR_ENV).ok();
        std::env::set_var(TREX_DIR_ENV, dir.path());
        f();
        match prev {
            Some(v) => std::env::set_var(TREX_DIR_ENV, v),
            None => std::env::remove_var(TREX_DIR_ENV),
        }
    }

    #[serial]
    #[test]
    fn trex_dir_uses_env_var() {
        let tmp = TempDir::new().unwrap();
        std::env::set_var(TREX_DIR_ENV, tmp.path());
        let dir = trex_dir();
        assert_eq!(dir, tmp.path());
        std::env::remove_var(TREX_DIR_ENV);
    }

    #[serial]
    #[test]
    fn save_and_load_sessions_roundtrip() {
        let tmp = TempDir::new().unwrap();
        with_temp_env(&tmp, || {
            let mut data = Sessions::new();
            data.sessions.push(crate::model::SavedSession {
                name: "test".into(),
                path: "/tmp".into(),
                windows: vec![],
                session_options: vec![],
                window_options: vec![],
            });
            save_sessions(&data).unwrap();
            let loaded = load_sessions().unwrap();
            assert_eq!(loaded.sessions.len(), 1);
            assert_eq!(loaded.sessions[0].name, "test");
        });
    }

    #[serial]
    #[test]
    fn load_sessions_nonexistent_returns_empty() {
        let tmp = TempDir::new().unwrap();
        with_temp_env(&tmp, || {
            let data = load_sessions().unwrap();
            assert!(data.sessions.is_empty());
        });
    }

    #[serial]
    #[test]
    fn load_ignore_list_nonexistent_returns_empty() {
        let tmp = TempDir::new().unwrap();
        with_temp_env(&tmp, || {
            let list = load_ignore_list();
            assert!(list.is_empty());
        });
    }

    #[serial]
    #[test]
    fn add_and_load_ignore_list() {
        let tmp = TempDir::new().unwrap();
        with_temp_env(&tmp, || {
            add_to_ignore("dev").unwrap();
            let list = load_ignore_list();
            assert_eq!(list, vec!["dev"]);
        });
    }

    #[serial]
    #[test]
    fn add_duplicate_ignore_returns_error() {
        let tmp = TempDir::new().unwrap();
        with_temp_env(&tmp, || {
            add_to_ignore("dev").unwrap();
            let result = add_to_ignore("dev");
            assert!(result.is_err());
        });
    }

    #[serial]
    #[test]
    fn remove_ignore_item() {
        let tmp = TempDir::new().unwrap();
        with_temp_env(&tmp, || {
            add_to_ignore("dev").unwrap();
            add_to_ignore("work").unwrap();
            super::remove_from_ignore("dev").unwrap();
            let list = load_ignore_list();
            assert_eq!(list, vec!["work"]);
        });
    }

    #[serial]
    #[test]
    fn remove_nonexistent_ignore_returns_error() {
        let tmp = TempDir::new().unwrap();
        with_temp_env(&tmp, || {
            add_to_ignore("dev").unwrap();
            let result = super::remove_from_ignore("other");
            assert!(result.is_err());
        });
    }
}
