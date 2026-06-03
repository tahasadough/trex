use tempfile::TempDir;

/// # Panics
/// Panics if a temporary directory cannot be created.
pub fn with_trex_dir<F>(f: F)
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

/// # Panics
/// Panics if a temporary directory cannot be created.
pub fn with_trex_dir_path<F>(f: F)
where
    F: FnOnce(&std::path::Path),
{
    let tmp = TempDir::new().unwrap();
    let prev = std::env::var("TREX_DIR").ok();
    std::env::set_var("TREX_DIR", tmp.path());
    f(tmp.path());
    match prev {
        Some(v) => std::env::set_var("TREX_DIR", v),
        None => std::env::remove_var("TREX_DIR"),
    }
    let _ = tmp;
}

/// # Panics
/// Panics if a temporary directory cannot be created.
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
