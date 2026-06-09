use std::env;
use std::fs;
use std::io::Read;

use crate::error::{TrexError, TrexResult};

const RELEASES_URL: &str = "https://github.com/tahasadough/trex/releases/latest/download";

fn download_url() -> String {
    env::var("TREX_UPDATE_URL")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| RELEASES_URL.to_string())
}

fn detect_target() -> TrexResult<String> {
    let os = if cfg!(target_os = "linux") {
        "unknown-linux-gnu"
    } else if cfg!(target_os = "macos") {
        "apple-darwin"
    } else {
        return Err(TrexError::UpdateFailed("unsupported platform".into()));
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        return Err(TrexError::UpdateFailed("unsupported architecture".into()));
    };

    Ok(format!("{arch}-{os}"))
}

/// # Errors
/// Returns [`TrexError::UpdateFailed`] if downloading or replacing the binary fails.
pub fn execute() -> TrexResult<String> {
    let current_exe = env::current_exe()
        .map_err(|e| TrexError::UpdateFailed(format!("Cannot get current exe path: {e}")))?;

    let target = detect_target()?;
    let base_url = download_url();
    let tarball_url = format!("{base_url}/trex-{target}.tar.gz");

    eprintln!("Checking for updates...");

    let response = ureq::get(&tarball_url)
        .call()
        .map_err(|e| TrexError::UpdateFailed(format!("Failed to download release: {e}")))?;

    let mut reader = response.into_reader();
    let mut archive_bytes = Vec::new();
    reader
        .read_to_end(&mut archive_bytes)
        .map_err(|e| TrexError::UpdateFailed(format!("Failed to read response: {e}")))?;

    let parent = current_exe
        .parent()
        .ok_or_else(|| TrexError::UpdateFailed("Cannot determine binary directory".into()))?;

    let tmp_dir = tempfile::tempdir()
        .map_err(|e| TrexError::UpdateFailed(format!("Temp dir failed: {e}")))?;
    let tarball_path = tmp_dir.path().join("trex.tar.gz");
    fs::write(&tarball_path, &archive_bytes)
        .map_err(|e| TrexError::UpdateFailed(format!("Failed to write tarball: {e}")))?;

    let extract_status = std::process::Command::new("tar")
        .arg("-xzf")
        .arg(&tarball_path)
        .arg("-C")
        .arg(tmp_dir.path())
        .arg("trex")
        .status()
        .map_err(|e| TrexError::UpdateFailed(format!("Failed to run tar: {e}")))?;

    if !extract_status.success() {
        return Err(TrexError::UpdateFailed(
            "Failed to extract binary from tarball".into(),
        ));
    }

    let extracted = tmp_dir.path().join("trex");
    if !extracted.exists() {
        return Err(TrexError::UpdateFailed(
            "Extracted binary not found in tarball".into(),
        ));
    }

    // Place the new binary on the same filesystem as the target to allow atomic rename
    let staging = parent.join("trex.new");
    fs::copy(&extracted, &staging)
        .map_err(|e| TrexError::UpdateFailed(format!("Failed to stage new binary: {e}")))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&staging, fs::Permissions::from_mode(0o755))
            .map_err(|e| TrexError::UpdateFailed(format!("Failed to set executable bit: {e}")))?;
    }

    fs::rename(&staging, &current_exe)
        .map_err(|e| TrexError::UpdateFailed(format!("Failed to replace binary: {e}")))?;

    eprintln!("trex has been updated successfully!");
    Ok("Update complete.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn download_url_default_is_https() {
        assert!(RELEASES_URL.starts_with("https://"));
    }

    #[test]
    #[serial]
    fn download_url_returns_default_when_no_env() {
        let prev = env::var("TREX_UPDATE_URL").ok();
        env::remove_var("TREX_UPDATE_URL");
        assert_eq!(download_url(), RELEASES_URL);
        if let Some(v) = prev {
            env::set_var("TREX_UPDATE_URL", v);
        }
    }

    #[test]
    #[serial]
    fn download_url_uses_env_var_when_set() {
        let prev = env::var("TREX_UPDATE_URL").ok();
        env::set_var("TREX_UPDATE_URL", "http://localhost:9999/");
        assert_eq!(download_url(), "http://localhost:9999/");
        match prev {
            Some(v) => env::set_var("TREX_UPDATE_URL", v),
            None => env::remove_var("TREX_UPDATE_URL"),
        }
    }

    #[test]
    #[serial]
    fn download_url_falls_back_when_env_is_empty() {
        let prev = env::var("TREX_UPDATE_URL").ok();
        env::set_var("TREX_UPDATE_URL", "");
        assert_eq!(download_url(), RELEASES_URL);
        match prev {
            Some(v) => env::set_var("TREX_UPDATE_URL", v),
            None => env::remove_var("TREX_UPDATE_URL"),
        }
    }

    #[test]
    fn detect_target_returns_known_format() {
        let target = detect_target().unwrap();
        // Two valid patterns: x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu,
        // x86_64-apple-darwin, aarch64-apple-darwin
        assert!(
            target == "x86_64-unknown-linux-gnu"
                || target == "aarch64-unknown-linux-gnu"
                || target == "x86_64-apple-darwin"
                || target == "aarch64-apple-darwin",
            "unexpected target: {target}"
        );
    }

    #[test]
    fn execute_fails_on_unreachable_url() {
        let prev = env::var("TREX_UPDATE_URL").ok();
        env::set_var("TREX_UPDATE_URL", "http://127.0.0.1:1/");
        let result = execute();
        match prev {
            Some(v) => env::set_var("TREX_UPDATE_URL", v),
            None => env::remove_var("TREX_UPDATE_URL"),
        }
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to download"),
            "expected 'Failed to download' in error message"
        );
    }
}
