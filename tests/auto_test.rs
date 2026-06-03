#[path = "common/mod.rs"]
mod common;

use common::with_temp_home;
use serial_test::serial;
use std::fs;

#[test]
#[serial]
fn auto_enable_creates_rc_hook() {
    with_temp_home(|| {
        std::env::set_var("SHELL", "/bin/zsh");
        let result = trex::commands::auto::execute_enable().unwrap();
        assert!(result.contains("Auto-restore enabled"));

        let home = dirs::home_dir().unwrap();
        let rc = home.join(".zshrc");
        let content = fs::read_to_string(&rc).unwrap();
        assert!(content.contains("trex restore --quiet"));
    });
}

#[test]
#[serial]
fn auto_disable_removes_rc_hook() {
    with_temp_home(|| {
        std::env::set_var("SHELL", "/bin/bash");
        trex::commands::auto::execute_enable().unwrap();
        let result = trex::commands::auto::execute_disable().unwrap();
        assert!(result.contains("disabled"));

        let home = dirs::home_dir().unwrap();
        let rc = home.join(".bashrc");
        let content = fs::read_to_string(&rc).unwrap_or_default();
        assert!(!content.contains("trex restore --quiet"));
    });
}

#[test]
#[serial]
fn auto_enable_idempotent() {
    with_temp_home(|| {
        std::env::set_var("SHELL", "/bin/zsh");
        trex::commands::auto::execute_enable().unwrap();
        let result = trex::commands::auto::execute_enable().unwrap();
        assert!(result.contains("already configured"));
    });
}

#[test]
#[serial]
fn auto_disable_noop_when_not_configured() {
    with_temp_home(|| {
        std::env::set_var("SHELL", "/bin/zsh");
        let result = trex::commands::auto::execute_disable().unwrap();
        assert!(result.contains("No auto-restore configuration found"));
    });
}

#[serial]
#[test]
fn systemd_service_path_format() {
    with_temp_home(|| {
        let path = trex::commands::auto::systemd_service_path();
        assert!(path.ends_with(".config/systemd/user/trex.service"));
    });
}
