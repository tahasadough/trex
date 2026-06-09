use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{TrexError, TrexResult};

#[cfg(target_os = "macos")]
const LAUNCHD_LABEL: &str = "com.tahasadough.trex";

const HOOK_COMMENT: &str = "# trex: auto-restore tmux sessions";

fn hook_lines(exe_path: &Path) -> [String; 2] {
    let exe = exe_path.to_string_lossy();
    [
        HOOK_COMMENT.to_string(),
        format!("[ -x \"{exe}\" ] && \"{exe}\" restore --quiet"),
    ]
}

fn is_hook_line(line: &str) -> bool {
    line == HOOK_COMMENT || (line.contains("restore --quiet") && line.contains("trex"))
}

fn shell_rc_files() -> Vec<PathBuf> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let zdotdir = std::env::var("ZDOTDIR")
        .ok()
        .map_or_else(|| home.clone(), PathBuf::from);

    vec![
        zdotdir.join(".zshrc"),
        home.join(".bashrc"),
        home.join(".profile"),
    ]
}

fn detect_rc() -> PathBuf {
    for rc in shell_rc_files() {
        if !rc.exists() {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&rc) {
            if content.contains("trex") {
                return rc;
            }
        }
    }
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    let shell_name = std::path::Path::new(&shell)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("sh");
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    match shell_name {
        "zsh" => {
            let zdotdir = std::env::var("ZDOTDIR").ok().map(PathBuf::from);
            zdotdir.unwrap_or(home).join(".zshrc")
        }
        _ => home.join(".bashrc"),
    }
}

/// # Errors
/// Returns [`TrexError`] if the rc file cannot be read or appended to,
/// or if the trex binary path cannot be resolved.
pub fn execute_enable() -> TrexResult<String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| TrexError::Generic(format!("Cannot determine trex binary path: {e}")))?;
    let [comment_line, command_line] = hook_lines(&exe_path);

    let rc_file = detect_rc();
    let content = fs::read_to_string(&rc_file).unwrap_or_default();

    if content.contains(&comment_line) {
        return Ok(format!(
            "Auto-restore already configured in {}",
            rc_file.display()
        ));
    }

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&rc_file)?;

    writeln!(file)?;
    writeln!(file, "{comment_line}")?;
    writeln!(file, "{command_line}")?;

    let msg = format!("Auto-restore enabled in {}", rc_file.display());

    #[cfg(target_os = "macos")]
    let msg = format!("{}\n{}", msg, enable_launchd_agent(&exe_path));

    Ok(msg)
}

/// # Errors
/// Returns [`TrexError`] if any rc file cannot be read or written.
pub fn execute_disable() -> TrexResult<String> {
    let mut messages = Vec::new();

    for rc_file in shell_rc_files() {
        if !rc_file.exists() {
            continue;
        }
        let content = fs::read_to_string(&rc_file)?;
        if content.lines().any(is_hook_line) {
            let filtered: Vec<&str> = content.lines().filter(|l| !is_hook_line(l)).collect();
            fs::write(&rc_file, filtered.join("\n") + "\n")?;
            messages.push(format!(
                "Auto-restore disabled (removed from {})",
                rc_file.display()
            ));
        }
    }

    disable_systemd_service(&mut messages);

    #[cfg(target_os = "macos")]
    disable_launchd_agent(&mut messages);

    if messages.is_empty() {
        Ok("No auto-restore configuration found.".to_string())
    } else {
        Ok(messages.join("\n"))
    }
}

#[must_use]
#[cfg(target_os = "linux")]
pub fn systemd_service_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".config/systemd/user/trex.service")
}

#[cfg(target_os = "linux")]
fn disable_systemd_service(messages: &mut Vec<String>) {
    let service_path = systemd_service_path();
    if service_path.exists() {
        let _ = fs::remove_file(&service_path);
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .output();
        messages.push("Disabled systemd auto-restore service".to_string());
    }
}

#[cfg(not(target_os = "linux"))]
fn disable_systemd_service(_messages: &mut Vec<String>) {}

#[cfg(target_os = "macos")]
fn launchd_plist_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join("Library/LaunchAgents/com.tahasadough.trex.plist")
}

#[cfg(target_os = "macos")]
fn plist_content(exe_path: &Path) -> String {
    let exe = exe_path.to_string_lossy();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{LAUNCHD_LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
        <string>restore</string>
        <string>--quiet</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
</dict>
</plist>
"#,
    )
}

#[cfg(target_os = "macos")]
fn enable_launchd_agent(exe_path: &Path) -> String {
    let plist_path = launchd_plist_path();

    if let Some(parent) = plist_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let _ = fs::write(&plist_path, plist_content(exe_path));
    let _ = std::process::Command::new("launchctl")
        .args(["load", &plist_path.to_string_lossy()])
        .output();

    format!("Launchd agent installed at {}", plist_path.display())
}

#[cfg(target_os = "macos")]
fn disable_launchd_agent(messages: &mut Vec<String>) {
    let plist_path = launchd_plist_path();

    if !plist_path.exists() {
        return;
    }

    let _ = std::process::Command::new("launchctl")
        .args(["unload", &plist_path.to_string_lossy()])
        .output();

    let _ = fs::remove_file(&plist_path);

    messages.push("Launchd auto-restore agent disabled".to_string());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::with_temp_home;
    use serial_test::serial;

    #[serial]
    #[test]
    fn detect_rc_returns_existing_file_with_trex() {
        with_temp_home(|| {
            let home = std::env::var("HOME").unwrap();
            let zshrc = std::path::Path::new(&home).join(".zshrc");
            let mut file = fs::File::create(&zshrc).unwrap();
            writeln!(file, "source /some/stuff").unwrap();
            writeln!(file, "trex restore --quiet").unwrap();

            let rc = detect_rc();
            assert_eq!(rc, zshrc);
        });
    }

    #[serial]
    #[test]
    fn detect_rc_falls_back_to_shell() {
        with_temp_home(|| {
            std::env::set_var("SHELL", "/bin/zsh");
            let home = std::env::var("HOME").unwrap();
            let rc = detect_rc();
            assert_eq!(rc, std::path::Path::new(&home).join(".zshrc"));
        });
    }

    #[serial]
    #[test]
    fn auto_enable_appends_to_rc_file() {
        with_temp_home(|| {
            std::env::set_var("SHELL", "/bin/zsh");
            let result = execute_enable().unwrap();
            assert!(result.contains("Auto-restore enabled"));

            let home = std::env::var("HOME").unwrap();
            let rc_path = std::path::Path::new(&home).join(".zshrc");
            let content = fs::read_to_string(&rc_path).unwrap();
            assert!(content.contains(HOOK_COMMENT));
            assert!(content.contains("restore --quiet"));
            assert!(content.lines().any(|l| l.starts_with("[ -x \"")));
        });
    }

    #[serial]
    #[test]
    fn auto_enable_idempotent() {
        with_temp_home(|| {
            std::env::set_var("SHELL", "/bin/bash");
            execute_enable().unwrap();
            let result = execute_enable().unwrap();
            assert!(result.contains("already configured"));
        });
    }

    #[serial]
    #[test]
    fn auto_disable_removes_hooks() {
        with_temp_home(|| {
            std::env::set_var("SHELL", "/bin/bash");
            execute_enable().unwrap();
            let result = execute_disable().unwrap();
            assert!(result.contains("disabled"));

            let home = std::env::var("HOME").unwrap();
            let rc_path = std::path::Path::new(&home).join(".bashrc");
            let content = fs::read_to_string(&rc_path).unwrap_or_default();
            assert!(!content.contains(HOOK_COMMENT));
            assert!(!content.contains("restore --quiet"));
        });
    }

    #[serial]
    #[test]
    #[cfg(target_os = "linux")]
    fn systemd_service_path_returns_correct_path() {
        with_temp_home(|| {
            let home = std::env::var("HOME").unwrap();
            let path = systemd_service_path();
            assert_eq!(
                path,
                std::path::Path::new(&home).join(".config/systemd/user/trex.service")
            );
        });
    }

    #[serial]
    #[test]
    #[cfg(target_os = "macos")]
    fn launchd_plist_path_is_in_launch_agents() {
        with_temp_home(|| {
            let home = std::env::var("HOME").unwrap();
            let path = launchd_plist_path();
            assert_eq!(
                path,
                std::path::Path::new(&home).join("Library/LaunchAgents/com.tahasadough.trex.plist")
            );
        });
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn plist_content_contains_exe_path() {
        let exe = std::path::Path::new("/opt/local/bin/trex");
        let content = plist_content(exe);
        assert!(content.contains("/opt/local/bin/trex"));
        assert!(content.contains("com.tahasadough.trex"));
        assert!(content.contains("restore"));
        assert!(content.contains("--quiet"));
        assert!(content.contains("RunAtLoad"));
    }
}
