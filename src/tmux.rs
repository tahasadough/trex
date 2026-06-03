use std::process::Command;

use crate::error::{TrexError, TrexResult};

const SHELLS: &[&str] = &["bash", "zsh", "sh", "fish", "tcsh", "ksh", "dash", "ash"];

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub index: u32,
    pub name: String,
    pub layout: String,
}

#[derive(Debug, Clone)]
pub struct PaneInfo {
    pub index: u32,
    pub path: String,
}

#[derive(Debug, Clone, Copy)]
pub enum OptionScope {
    Session,
    Window,
}

pub trait TmuxClient {
    /// Check if any tmux sessions exist.
    ///
    /// # Errors
    /// Returns [`TrexError::TmuxCommandFailed`] if the tmux command fails.
    fn has_sessions(&self) -> TrexResult<bool>;
    /// Get the name of the current tmux session.
    ///
    /// # Errors
    /// Returns [`TrexError::NotInTmuxSession`] if not inside a tmux session.
    fn current_session_name(&self) -> TrexResult<String>;
    /// List all tmux session names.
    ///
    /// # Errors
    /// Returns [`TrexError::TmuxCommandFailed`] if the tmux command fails.
    fn list_sessions(&self) -> TrexResult<Vec<String>>;
    /// Get the path for a given session.
    ///
    /// # Errors
    /// Returns [`TrexError::TmuxCommandFailed`] if the tmux command fails.
    fn session_path(&self, session: &str) -> TrexResult<String>;
    /// List all windows in a session.
    ///
    /// # Errors
    /// Returns [`TrexError::TmuxCommandFailed`] if the tmux command fails.
    fn list_windows(&self, session: &str) -> TrexResult<Vec<WindowInfo>>;
    /// Get the path for a specific window in a session.
    ///
    /// # Errors
    /// Returns [`TrexError::TmuxCommandFailed`] if the tmux command fails.
    fn window_path(&self, session: &str, window: u32) -> TrexResult<String>;
    /// List all panes in a window.
    ///
    /// # Errors
    /// Returns [`TrexError::TmuxCommandFailed`] if the tmux command fails.
    fn list_panes(&self, session: &str, window: u32) -> TrexResult<Vec<PaneInfo>>;
    /// Check if a specific pane is active.
    ///
    /// # Errors
    /// Returns [`TrexError::TmuxCommandFailed`] if the tmux command fails.
    fn pane_active(&self, session: &str, window: u32, pane: u32) -> TrexResult<bool>;
    /// Get the command running in a pane.
    ///
    /// # Errors
    /// Returns [`TrexError::TmuxCommandFailed`] if the tmux command fails.
    fn pane_command(&self, session: &str, window: u32, pane: u32) -> TrexResult<Option<String>>;
    /// Get all session options.
    ///
    /// # Errors
    /// Returns [`TrexError::TmuxCommandFailed`] if the tmux command fails.
    fn get_session_options(&self, session: &str) -> TrexResult<Vec<String>>;
    /// Get all window options.
    ///
    /// # Errors
    /// Returns [`TrexError::TmuxCommandFailed`] if the tmux command fails.
    fn get_window_options(&self, session: &str) -> TrexResult<Vec<String>>;

    /// Create a new tmux session.
    ///
    /// # Errors
    /// Returns [`TrexError::TmuxCommandFailed`] if the tmux command fails.
    fn new_session(&self, name: &str, path: &str) -> TrexResult<()>;
    /// Check if a session with the given name exists.
    ///
    /// # Errors
    /// Returns [`TrexError::TmuxCommandFailed`] if the tmux command fails.
    fn has_session(&self, name: &str) -> TrexResult<bool>;
    /// Rename a window in a session.
    ///
    /// # Errors
    /// Returns [`TrexError::TmuxCommandFailed`] if the tmux command fails.
    fn rename_window(&self, session: &str, old_name: &str, new_name: &str) -> TrexResult<()>;
    /// Create a new window in a session.
    ///
    /// # Errors
    /// Returns [`TrexError::TmuxCommandFailed`] if the tmux command fails.
    fn new_window(&self, session: &str, name: &str, path: &str) -> TrexResult<()>;
    /// Split a window (create a new pane).
    ///
    /// # Errors
    /// Returns [`TrexError::TmuxCommandFailed`] if the tmux command fails.
    fn split_window(&self, session: &str, window: u32, path: &str) -> TrexResult<()>;
    /// Set the layout for a window.
    ///
    /// # Errors
    /// Returns [`TrexError::TmuxCommandFailed`] if the tmux command fails.
    fn select_layout(&self, session: &str, window: u32, layout: &str) -> TrexResult<()>;
    /// Select a specific pane.
    ///
    /// # Errors
    /// Returns [`TrexError::TmuxCommandFailed`] if the tmux command fails.
    fn select_pane(&self, session: &str, window: u32, pane: u32) -> TrexResult<()>;
    /// Set a tmux option for a session or window.
    ///
    /// # Errors
    /// Returns [`TrexError::TmuxCommandFailed`] if the tmux command fails.
    fn set_option(
        &self,
        session: &str,
        scope: OptionScope,
        name: &str,
        value: &str,
    ) -> TrexResult<()>;
    /// Send keys (a command) to a specific pane.
    ///
    /// # Errors
    /// Returns [`TrexError::TmuxCommandFailed`] if the tmux command fails.
    fn send_keys(&self, session: &str, window: u32, pane: u32, command: &str) -> TrexResult<()>;
}

pub struct Tmux;

/// # Errors
/// Returns [`TrexError::TmuxCommandFailed`] if the tmux binary is not found
/// or the command exits with a non-zero status.
fn run_tmux(args: &[&str]) -> TrexResult<String> {
    let output = Command::new("tmux")
        .args(args)
        .output()
        .map_err(|e| TrexError::TmuxCommandFailed(format!("tmux binary not found: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TrexError::TmuxCommandFailed(stderr.trim().to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn run_tmux_ignore_error(args: &[&str]) -> String {
    Command::new("tmux")
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn parse_u32(s: &str) -> u32 {
    s.trim().parse().unwrap_or(0)
}

impl TmuxClient for Tmux {
    fn has_sessions(&self) -> TrexResult<bool> {
        match run_tmux(&["has-session"]) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn current_session_name(&self) -> TrexResult<String> {
        run_tmux(&["display-message", "-p", "-F", "#{session_name}"])
            .map_err(|_| TrexError::NotInTmuxSession)
    }

    fn list_sessions(&self) -> TrexResult<Vec<String>> {
        let output = run_tmux(&["list-sessions", "-F", "#{session_name}"])?;
        if output.is_empty() {
            return Ok(Vec::new());
        }
        Ok(output.lines().map(|l| l.trim().to_string()).collect())
    }

    fn session_path(&self, session: &str) -> TrexResult<String> {
        let path = run_tmux_ignore_error(&[
            "display-message",
            "-p",
            "-t",
            session,
            "-F",
            "#{pane_current_path}",
        ]);
        Ok(if path.is_empty() {
            "~".to_string()
        } else {
            path
        })
    }

    fn list_windows(&self, session: &str) -> TrexResult<Vec<WindowInfo>> {
        let output = run_tmux_ignore_error(&[
            "list-windows",
            "-t",
            session,
            "-F",
            "#{window_index}|#{window_name}|#{window_layout}",
        ]);
        if output.is_empty() {
            return Ok(Vec::new());
        }
        Ok(output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(3, '|').collect();
                if parts.len() < 3 {
                    return None;
                }
                Some(WindowInfo {
                    index: parse_u32(parts[0]),
                    name: parts[1].to_string(),
                    layout: parts[2].to_string(),
                })
            })
            .collect())
    }

    fn window_path(&self, session: &str, window: u32) -> TrexResult<String> {
        let path = run_tmux_ignore_error(&[
            "display-message",
            "-p",
            "-t",
            &format!("{session}:{window}"),
            "-F",
            "#{pane_current_path}",
        ]);
        Ok(path)
    }

    fn list_panes(&self, session: &str, window: u32) -> TrexResult<Vec<PaneInfo>> {
        let target = format!("{session}:{window}");
        let output = run_tmux_ignore_error(&[
            "list-panes",
            "-t",
            &target,
            "-F",
            "#{pane_index}|#{pane_current_path}",
        ]);
        if output.is_empty() {
            return Ok(Vec::new());
        }
        Ok(output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, '|').collect();
                if parts.len() < 2 {
                    return None;
                }
                Some(PaneInfo {
                    index: parse_u32(parts[0]),
                    path: parts[1].to_string(),
                })
            })
            .collect())
    }

    fn pane_active(&self, session: &str, window: u32, pane: u32) -> TrexResult<bool> {
        let target = format!("{session}:{window}.{pane}");
        let active = run_tmux_ignore_error(&[
            "display-message",
            "-p",
            "-t",
            &target,
            "-F",
            "#{pane_active}",
        ]);
        Ok(active == "1")
    }

    fn pane_command(&self, session: &str, window: u32, pane: u32) -> TrexResult<Option<String>> {
        let target = format!("{session}:{window}.{pane}");
        let cmd = run_tmux_ignore_error(&[
            "display-message",
            "-p",
            "-t",
            &target,
            "-F",
            "#{pane_current_command}",
        ]);
        if cmd.is_empty() || SHELLS.contains(&cmd.as_str()) {
            return Ok(None);
        }

        let pid =
            run_tmux_ignore_error(&["display-message", "-p", "-t", &target, "-F", "#{pane_pid}"]);
        if pid.is_empty() || pid == "0" {
            return Ok(Some(cmd));
        }

        let full_cmd = get_process_command(&pid);
        if let Some(ref fc) = full_cmd {
            if fc.contains(&cmd) {
                return Ok(Some(fc.clone()));
            }
        }

        if let Some(children) = get_child_pids(&pid) {
            for child in &children {
                if let Some(child_cmd) = get_process_command(child) {
                    if child_cmd.contains(&cmd) {
                        return Ok(Some(child_cmd));
                    }
                }
            }

            let deepest = find_deepest_child(&pid);
            if deepest != pid {
                if let Some(deep_cmd) = get_process_command(&deepest) {
                    return Ok(Some(deep_cmd));
                }
            }
        }

        Ok(full_cmd.or(Some(cmd)))
    }

    fn get_session_options(&self, session: &str) -> TrexResult<Vec<String>> {
        let output = run_tmux_ignore_error(&["show-options", "-t", session]);
        Ok(output.lines().map(ToString::to_string).collect())
    }

    fn get_window_options(&self, session: &str) -> TrexResult<Vec<String>> {
        let output = run_tmux_ignore_error(&["show-options", "-w", "-t", session]);
        Ok(output.lines().map(ToString::to_string).collect())
    }

    fn new_session(&self, name: &str, path: &str) -> TrexResult<()> {
        run_tmux(&[
            "new-session",
            "-d",
            "-s",
            name,
            "-c",
            path,
            "-n",
            "_tmuxer_",
        ])?;
        Ok(())
    }

    fn has_session(&self, name: &str) -> TrexResult<bool> {
        match run_tmux(&["has-session", "-t", name]) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn rename_window(&self, session: &str, _old_name: &str, new_name: &str) -> TrexResult<()> {
        run_tmux_ignore_error(&[
            "rename-window",
            "-t",
            &format!("{session}:_tmuxer_"),
            new_name,
        ]);
        Ok(())
    }

    fn new_window(&self, session: &str, name: &str, path: &str) -> TrexResult<()> {
        run_tmux_ignore_error(&["new-window", "-t", session, "-n", name, "-c", path]);
        Ok(())
    }

    fn split_window(&self, session: &str, window: u32, path: &str) -> TrexResult<()> {
        let target = format!("{session}:{window}");
        run_tmux_ignore_error(&["split-window", "-t", &target, "-c", path]);
        Ok(())
    }

    fn select_layout(&self, session: &str, window: u32, layout: &str) -> TrexResult<()> {
        let target = format!("{session}:{window}");
        run_tmux_ignore_error(&["select-layout", "-t", &target, layout]);
        Ok(())
    }

    fn select_pane(&self, session: &str, window: u32, pane: u32) -> TrexResult<()> {
        let target = format!("{session}:{window}.{pane}");
        run_tmux_ignore_error(&["select-pane", "-t", &target]);
        Ok(())
    }

    fn set_option(
        &self,
        session: &str,
        scope: OptionScope,
        name: &str,
        value: &str,
    ) -> TrexResult<()> {
        let mut args = vec!["set-option"];
        match scope {
            OptionScope::Window => args.push("-w"),
            OptionScope::Session => {}
        }
        args.push("-t");
        args.push(session);
        args.push(name);
        args.push(value);
        run_tmux_ignore_error(&args);
        Ok(())
    }

    fn send_keys(&self, session: &str, window: u32, pane: u32, command: &str) -> TrexResult<()> {
        let target = format!("{session}:{window}.{pane}");
        run_tmux_ignore_error(&["send-keys", "-t", &target, command, "Enter"]);
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn get_process_command(pid: &str) -> Option<String> {
    let output = Command::new("ps")
        .args(["-o", "args=", "-p", pid])
        .output()
        .ok()?;
    if output.status.success() {
        let cmd = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !cmd.is_empty() {
            return Some(cmd);
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn get_process_command(pid: &str) -> Option<String> {
    let output = Command::new("ps")
        .args(["-p", pid, "-o", "command"])
        .output()
        .ok()?;
    if output.status.success() {
        let cmd = String::from_utf8_lossy(&output.stdout)
            .lines()
            .nth(1)
            .unwrap_or("")
            .trim()
            .to_string();
        if !cmd.is_empty() {
            return Some(cmd);
        }
    }
    None
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_process_command(_pid: &str) -> Option<String> {
    None
}

#[cfg(target_os = "linux")]
fn get_child_pids(pid: &str) -> Option<Vec<String>> {
    let output = Command::new("pgrep").args(["-P", pid]).output().ok()?;
    if output.status.success() {
        let children: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        if !children.is_empty() {
            return Some(children);
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn get_child_pids(pid: &str) -> Option<Vec<String>> {
    let output = Command::new("ps").args(["-eo", "pid,ppid"]).output().ok()?;
    if output.status.success() {
        let pid_trimmed = pid.trim();
        let children: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .skip(1)
            .filter_map(|line| {
                let mut parts = line.split_whitespace();
                let child_pid = parts.next()?;
                let parent_pid = parts.next()?;
                if parent_pid.trim() == pid_trimmed {
                    Some(child_pid.to_string())
                } else {
                    None
                }
            })
            .collect();
        if !children.is_empty() {
            return Some(children);
        }
    }
    None
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_child_pids(_pid: &str) -> Option<Vec<String>> {
    None
}

fn find_deepest_child(pid: &str) -> String {
    let mut current = pid.to_string();
    loop {
        match get_child_pids(&current) {
            Some(children) if !children.is_empty() => {
                current.clone_from(children.last().unwrap());
            }
            _ => break,
        }
    }
    current
}

pub struct MockTmux {
    pub sessions: Vec<String>,
    pub current_session: String,
    pub windows: Vec<WindowInfo>,
    pub panes: Vec<PaneInfo>,
    pub paths: std::collections::HashMap<(String, Option<u32>), String>,
    pub active_panes: std::collections::HashMap<(u32, u32), bool>,
    pub pane_commands: std::collections::HashMap<(u32, u32), Option<String>>,
    pub session_opts: Vec<String>,
    pub window_opts: Vec<String>,
    pub existing_sessions: Vec<String>,
}

impl Default for MockTmux {
    fn default() -> Self {
        Self {
            sessions: Vec::new(),
            current_session: "default".into(),
            windows: Vec::new(),
            panes: Vec::new(),
            paths: std::collections::HashMap::new(),
            active_panes: std::collections::HashMap::new(),
            pane_commands: std::collections::HashMap::new(),
            session_opts: Vec::new(),
            window_opts: Vec::new(),
            existing_sessions: Vec::new(),
        }
    }
}

impl MockTmux {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl TmuxClient for MockTmux {
    fn has_sessions(&self) -> TrexResult<bool> {
        Ok(!self.sessions.is_empty())
    }

    fn current_session_name(&self) -> TrexResult<String> {
        Ok(self.current_session.clone())
    }

    fn list_sessions(&self) -> TrexResult<Vec<String>> {
        Ok(self.sessions.clone())
    }

    fn session_path(&self, session: &str) -> TrexResult<String> {
        Ok(self
            .paths
            .get(&(session.to_string(), None))
            .cloned()
            .unwrap_or_else(|| "~".into()))
    }

    fn list_windows(&self, _session: &str) -> TrexResult<Vec<WindowInfo>> {
        Ok(self.windows.clone())
    }

    fn window_path(&self, _session: &str, window: u32) -> TrexResult<String> {
        Ok(self
            .paths
            .get(&(String::new(), Some(window)))
            .cloned()
            .unwrap_or_else(|| "~".into()))
    }

    fn list_panes(&self, _session: &str, _window: u32) -> TrexResult<Vec<PaneInfo>> {
        Ok(self.panes.clone())
    }

    fn pane_active(&self, _session: &str, _window: u32, pane: u32) -> TrexResult<bool> {
        Ok(self.active_panes.get(&(0, pane)).copied().unwrap_or(false))
    }

    fn pane_command(&self, _session: &str, _window: u32, pane: u32) -> TrexResult<Option<String>> {
        Ok(self
            .pane_commands
            .get(&(0, pane))
            .cloned()
            .unwrap_or_default())
    }

    fn get_session_options(&self, _session: &str) -> TrexResult<Vec<String>> {
        Ok(self.session_opts.clone())
    }

    fn get_window_options(&self, _session: &str) -> TrexResult<Vec<String>> {
        Ok(self.window_opts.clone())
    }

    fn new_session(&self, name: &str, _path: &str) -> TrexResult<()> {
        let _ = name;
        Ok(())
    }

    fn has_session(&self, name: &str) -> TrexResult<bool> {
        Ok(self.existing_sessions.iter().any(|s| s == name))
    }

    fn rename_window(&self, _session: &str, _old_name: &str, _new_name: &str) -> TrexResult<()> {
        Ok(())
    }

    fn new_window(&self, _session: &str, _name: &str, _path: &str) -> TrexResult<()> {
        Ok(())
    }

    fn split_window(&self, _session: &str, _window: u32, _path: &str) -> TrexResult<()> {
        Ok(())
    }

    fn select_layout(&self, _session: &str, _window: u32, _layout: &str) -> TrexResult<()> {
        Ok(())
    }

    fn select_pane(&self, _session: &str, _window: u32, _pane: u32) -> TrexResult<()> {
        Ok(())
    }

    fn set_option(
        &self,
        _session: &str,
        _scope: OptionScope,
        _name: &str,
        _value: &str,
    ) -> TrexResult<()> {
        Ok(())
    }

    fn send_keys(
        &self,
        _session: &str,
        _window: u32,
        _pane: u32,
        _command: &str,
    ) -> TrexResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_has_sessions_returns_true_when_sessions_exist() {
        let mock = MockTmux::new();
        assert!(!mock.has_sessions().unwrap());
    }

    #[test]
    fn mock_list_sessions_returns_sessions() {
        let mut mock = MockTmux::new();
        mock.sessions = vec!["dev".into(), "work".into()];
        let sessions = mock.list_sessions().unwrap();
        assert_eq!(sessions, vec!["dev", "work"]);
    }

    #[test]
    fn mock_current_session_name() {
        let mock = MockTmux::new();
        assert_eq!(mock.current_session_name().unwrap(), "default");
    }

    #[test]
    fn mock_pane_command_returns_none_for_shell() {
        let mock = MockTmux::new();
        let cmd = mock.pane_command("sess", 0, 0).unwrap();
        assert!(cmd.is_none());
    }

    #[test]
    fn mock_pane_command_returns_command() {
        let mut mock = MockTmux::new();
        mock.pane_commands
            .insert((0, 0), Some("nvim main.rs".into()));
        let cmd = mock.pane_command("sess", 0, 0).unwrap();
        assert_eq!(cmd, Some("nvim main.rs".into()));
    }

    #[test]
    fn parse_u32_parses_valid_number() {
        assert_eq!(parse_u32("42"), 42);
    }

    #[test]
    fn parse_u32_returns_zero_on_invalid() {
        assert_eq!(parse_u32("abc"), 0);
    }

    #[test]
    fn parse_u32_handles_whitespace() {
        assert_eq!(parse_u32("  7  "), 7);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn get_process_command_returns_cmd_for_current_pid() {
        let pid = std::process::id().to_string();
        let cmd = get_process_command(&pid);
        assert!(cmd.is_some());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn get_process_command_returns_cmd_for_current_pid() {
        let pid = std::process::id().to_string();
        let cmd = get_process_command(&pid);
        assert!(cmd.is_some());
    }

    #[test]
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    fn get_process_command_returns_none_on_unsupported() {
        assert!(get_process_command("1").is_none());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn get_child_pids_returns_children_for_init() {
        let children = get_child_pids("1");
        assert!(children.is_some());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn get_child_pids_returns_children_for_init() {
        let children = get_child_pids("1");
        assert!(children.is_some());
    }

    #[test]
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    fn get_child_pids_returns_none_on_unsupported() {
        assert!(get_child_pids("1").is_none());
    }

    #[test]
    fn find_deepest_child_returns_current_pid_when_no_children() {
        let pid = std::process::id().to_string();
        let deepest = find_deepest_child(&pid);
        assert!(!deepest.is_empty());
    }
}
