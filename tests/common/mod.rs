#![expect(dead_code, reason = "Helper functions used by integration tests")]

use std::path::PathBuf;
use tempfile::TempDir;

pub fn with_temp_trex_dir<F>(f: F)
where
    F: FnOnce(),
{
    let tmp = TempDir::new().unwrap();
    let prev = std::env::var("TREX_DIR").ok();
    std::env::set_var("TREX_DIR", tmp.path());
    f();
    match prev {
        Some(v) => std::env::set_var("TREX_DIR", v),
        None => std::env::remove_var("TREX_DIR"),
    }
    let _ = tmp;
}

pub fn with_temp_home<F>(f: F)
where
    F: FnOnce(),
{
    let tmp = TempDir::new().unwrap();
    let prev_home = std::env::var("HOME").ok();
    let prev_shell = std::env::var("SHELL").ok();
    std::env::set_var("HOME", tmp.path());
    f();
    match prev_home {
        Some(v) => std::env::set_var("HOME", v),
        None => std::env::remove_var("HOME"),
    }
    match prev_shell {
        Some(v) => std::env::set_var("SHELL", v),
        None => std::env::remove_var("SHELL"),
    }
    let _ = tmp;
}

pub fn create_test_sessions_file(path: &PathBuf) {
    use std::fs;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let data = r#"{
        "version": 1,
        "sessions": [
            {
                "name": "dev",
                "path": "/home/dev",
                "windows": [],
                "session_options": [],
                "window_options": []
            },
            {
                "name": "work",
                "path": "/office",
                "windows": [],
                "session_options": [],
                "window_options": []
            }
        ]
    }"#;
    fs::write(path, data).unwrap();
}
