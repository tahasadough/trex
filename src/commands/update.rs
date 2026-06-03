use std::env;
use std::io::{Read, Write};

use crate::error::{TrexError, TrexResult};

const INSTALL_URL: &str = "https://raw.githubusercontent.com/tahasadough/trex/main/install.sh";

pub(crate) fn install_url() -> String {
    env::var("TREX_UPDATE_URL")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| INSTALL_URL.to_string())
}

/// # Errors
/// Returns [`TrexError::UpdateFailed`] if downloading or executing
/// the install script fails.
pub fn execute() -> TrexResult<String> {
    let url = install_url();
    eprintln!("Checking for updates...");
    download_and_run(&url)
}

fn download_and_run(url: &str) -> TrexResult<String> {
    let response = ureq::get(url)
        .call()
        .map_err(|e| TrexError::UpdateFailed(format!("Failed to fetch install script: {e}")))?;

    let mut reader = response.into_reader();
    let mut script = Vec::new();
    reader
        .read_to_end(&mut script)
        .map_err(|e| TrexError::UpdateFailed(format!("Failed to read install script: {e}")))?;

    run_script(&script)
}

fn run_script(script: &[u8]) -> TrexResult<String> {
    let mut tmp = tempfile::NamedTempFile::new()
        .map_err(|e| TrexError::UpdateFailed(format!("Temp file failed: {e}")))?;
    tmp.write_all(script)
        .map_err(|e| TrexError::UpdateFailed(format!("Write failed: {e}")))?;

    let mut child = std::process::Command::new("bash")
        .arg(tmp.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .map_err(|e| TrexError::UpdateFailed(format!("Failed to spawn install script: {e}")))?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"n\n");
    }

    let status = child
        .wait()
        .map_err(|e| TrexError::UpdateFailed(format!("Install script failed: {e}")))?;

    if status.success() {
        eprintln!("trex has been updated successfully!");
        Ok("Update complete.".to_string())
    } else {
        Err(TrexError::UpdateFailed(
            "Install script exited with an error. Check output above.".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn install_url_default_is_https() {
        assert!(INSTALL_URL.starts_with("https://"));
    }

    #[test]
    #[serial]
    fn install_url_returns_default_when_no_env() {
        let prev = env::var("TREX_UPDATE_URL").ok();
        env::remove_var("TREX_UPDATE_URL");
        assert_eq!(install_url(), INSTALL_URL);
        if let Some(v) = prev {
            env::set_var("TREX_UPDATE_URL", v);
        }
    }

    #[test]
    #[serial]
    fn install_url_uses_env_var_when_set() {
        let prev = env::var("TREX_UPDATE_URL").ok();
        env::set_var("TREX_UPDATE_URL", "http://localhost:9999/test.sh");
        assert_eq!(install_url(), "http://localhost:9999/test.sh");
        match prev {
            Some(v) => env::set_var("TREX_UPDATE_URL", v),
            None => env::remove_var("TREX_UPDATE_URL"),
        }
    }

    #[test]
    #[serial]
    fn install_url_falls_back_when_env_is_empty() {
        let prev = env::var("TREX_UPDATE_URL").ok();
        env::set_var("TREX_UPDATE_URL", "");
        assert_eq!(install_url(), INSTALL_URL);
        match prev {
            Some(v) => env::set_var("TREX_UPDATE_URL", v),
            None => env::remove_var("TREX_UPDATE_URL"),
        }
    }

    #[test]
    fn run_script_success_returns_ok() {
        let script = b"#!/usr/bin/env bash\nexit 0\n";
        let result = run_script(script);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Update complete.");
    }

    #[test]
    fn run_script_failure_returns_error() {
        let script = b"#!/usr/bin/env bash\nexit 1\n";
        let result = run_script(script);
        assert!(result.is_err());
        match result {
            Err(TrexError::UpdateFailed(msg)) => {
                assert!(msg.contains("exited with an error"), "msg: {msg}");
            }
            _ => panic!("Expected UpdateFailed error"),
        }
    }

    #[test]
    fn download_and_run_returns_error_for_unreachable_url() {
        let result = download_and_run("http://127.0.0.1:1/");
        assert!(result.is_err());
        match result {
            Err(TrexError::UpdateFailed(msg)) => {
                assert!(msg.contains("Failed to fetch"), "msg: {msg}");
            }
            _ => panic!("Expected UpdateFailed error"),
        }
    }
}
