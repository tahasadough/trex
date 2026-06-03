use std::os::unix::fs::PermissionsExt;

use crate::error::{TrexError, TrexResult};

const CARGO_INSTALL_URL: &str = "https://github.com/tahasadough/trex";
const RELEASES_URL: &str = "https://github.com/tahasadough/trex/releases/latest/download/trex";

fn target_triple() -> Option<String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    match (os, arch) {
        ("linux", "x86_64") => Some("x86_64-unknown-linux-gnu".into()),
        ("linux", "aarch64") => Some("aarch64-unknown-linux-gnu".into()),
        ("macos", "x86_64") => Some("x86_64-apple-darwin".into()),
        ("macos", "aarch64") => Some("aarch64-apple-darwin".into()),
        _ => None,
    }
}

fn download_url() -> Option<String> {
    let triple = target_triple()?;
    Some(format!("{RELEASES_URL}-{triple}.tar.gz"))
}

/// # Errors
/// Returns [`TrexError::UpdateFailed`] if downloading, extracting,
/// or replacing the binary fails.
fn update_via_binary() -> TrexResult<String> {
    let url = download_url()
        .ok_or_else(|| TrexError::UpdateFailed("No prebuilt binary for this platform".into()))?;

    let self_path = std::env::current_exe()
        .map_err(|e| TrexError::UpdateFailed(format!("Cannot find own path: {e}")))?;

    let response = ureq::get(&url)
        .call()
        .map_err(|e| TrexError::UpdateFailed(format!("Download failed: {e}")))?;

    let mut reader = response.into_reader();
    let mut decoder = flate2::read::GzDecoder::new(&mut reader);
    let mut archive = tar::Archive::new(&mut decoder);

    let tmp = tempfile::TempDir::new()
        .map_err(|e| TrexError::UpdateFailed(format!("Temp dir failed: {e}")))?;

    archive
        .unpack(tmp.path())
        .map_err(|e| TrexError::UpdateFailed(format!("Extract failed: {e}")))?;

    let binary = tmp.path().join("trex");
    if !binary.exists() {
        return Err(TrexError::UpdateFailed(
            "Binary not found in archive".into(),
        ));
    }

    std::fs::rename(&binary, &self_path)
        .or_else(|_| {
            std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755))
                .and_then(|()| std::fs::copy(&binary, &self_path))
                .map(|_| ())
        })
        .map_err(|e| TrexError::UpdateFailed(format!("Replace failed: {e}")))?;
    Ok("Update complete.".to_string())
}

/// # Errors
/// Returns [`TrexError::UpdateFailed`] if `cargo install` fails or
/// the `cargo` binary is not found.
fn update_via_cargo() -> TrexResult<String> {
    let cargo = std::process::Command::new("cargo")
        .arg("install")
        .arg("--force")
        .arg("--git")
        .arg(CARGO_INSTALL_URL)
        .arg("trex")
        .output()
        .map_err(|e| TrexError::UpdateFailed(format!("cargo not found: {e}")))?;

    if cargo.status.success() {
        Ok("Update complete.".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&cargo.stderr);
        Err(TrexError::UpdateFailed(format!(
            "cargo install failed: {stderr}"
        )))
    }
}

/// # Errors
/// Returns [`TrexError::UpdateFailed`] if downloading, extracting,
/// or replacing the binary fails.
pub fn execute() -> TrexResult<String> {
    eprintln!("Checking for updates...");
    let result = update_via_binary().or_else(|_| update_via_cargo());
    if result.is_ok() {
        eprintln!("trex has been updated successfully!");
    }
    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn find_self_path_returns_valid_path() {
        let path = std::env::current_exe().unwrap();
        assert!(path.exists());
    }
}
