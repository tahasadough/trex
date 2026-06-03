use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrexError {
    #[error("No active tmux sessions")]
    NoTmuxSessions,
    #[error("Not inside a tmux session")]
    NotInTmuxSession,
    #[error("No saved sessions")]
    NoSavedSessions,
    #[error("No saved session named '{0}'")]
    SessionNotFound(String),
    #[error("Session '{0}' is already ignored")]
    AlreadyIgnored(String),
    #[error("No ignored sessions")]
    NoIgnoredSessions,
    #[error("Failed to execute tmux command: {0}")]
    TmuxCommandFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Update failed: {0}")]
    UpdateFailed(String),
    #[error("No write permission to {0}")]
    NoWritePermission(String),
    #[error("{0}")]
    Generic(String),
}

pub type TrexResult<T> = Result<T, TrexError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_no_tmux_sessions_message() {
        let err = TrexError::NoTmuxSessions;
        assert_eq!(err.to_string(), "No active tmux sessions");
    }

    #[test]
    fn error_not_in_tmux_session_message() {
        let err = TrexError::NotInTmuxSession;
        assert_eq!(err.to_string(), "Not inside a tmux session");
    }

    #[test]
    fn error_session_not_found_message() {
        let err = TrexError::SessionNotFound("dev".into());
        assert_eq!(err.to_string(), "No saved session named 'dev'");
    }

    #[test]
    fn error_tmux_command_failed_message() {
        let err = TrexError::TmuxCommandFailed("tmux not found".into());
        assert_eq!(
            err.to_string(),
            "Failed to execute tmux command: tmux not found"
        );
    }

    #[test]
    fn error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err: TrexError = io_err.into();
        assert!(err.to_string().contains("file not found"));
    }
}
