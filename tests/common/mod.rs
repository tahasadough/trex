#![expect(dead_code, reason = "Helper functions used by integration tests")]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{Arc, Barrier};
use std::thread;
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

/// Starts a simple HTTP server that responds with the given body and status code.
/// The `f` closure is called with the server URL after it is ready to accept connections.
pub fn with_http_server<F>(body: &str, status: u16, f: F)
where
    F: FnOnce(&str),
{
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let url = format!("http://127.0.0.1:{port}");
    let body = body.to_owned();

    let barrier = Arc::new(Barrier::new(2));
    let b = Arc::clone(&barrier);

    thread::spawn(move || {
        b.wait();
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0; 4096];
            let _ = stream.read(&mut buf);
            let response = format!(
                "HTTP/1.1 {status} OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body,
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });

    barrier.wait();
    f(&url);
}
