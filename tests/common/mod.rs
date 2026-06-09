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

/// Starts a simple HTTP server that responds with the given bytes and status code.
/// The `f` closure is called with the server URL after it is ready to accept connections.
pub fn with_http_server<F>(data: &[u8], status: u16, f: F)
where
    F: FnOnce(&str),
{
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let url = format!("http://127.0.0.1:{port}");
    let data = data.to_vec();

    let barrier = Arc::new(Barrier::new(2));
    let b = Arc::clone(&barrier);

    thread::spawn(move || {
        b.wait();
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0; 4096];
            let _ = stream.read(&mut buf);
            let header = format!(
                "HTTP/1.1 {status} OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                data.len(),
            );
            let _ = stream.write_all(header.as_bytes());
            let _ = stream.write_all(&data);
            let _ = stream.flush();
        }
    });

    barrier.wait();
    f(&url);
}

/// Creates a gzipped tarball containing a single file named `trex` with the given content.
/// Uses the system `tar` command (available on Linux and macOS).
pub fn create_test_tarball(binary_content: &[u8]) -> Vec<u8> {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("trex"), binary_content).unwrap();
    let output = std::process::Command::new("tar")
        .arg("-czf")
        .arg("-")
        .arg("-C")
        .arg(dir.path())
        .arg("trex")
        .output()
        .unwrap();
    output.stdout
}
