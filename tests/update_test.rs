#[path = "common/mod.rs"]
mod common;

use common::with_http_server;
use serial_test::serial;

#[serial]
#[test]
fn update_succeeds_with_valid_script() {
    with_http_server("#!/usr/bin/env bash\nexit 0\n", 200, |url| {
        let prev = std::env::var("TREX_UPDATE_URL").ok();
        std::env::set_var("TREX_UPDATE_URL", url);
        let result = trex::commands::update::execute();
        match prev {
            Some(v) => std::env::set_var("TREX_UPDATE_URL", v),
            None => std::env::remove_var("TREX_UPDATE_URL"),
        }
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
        assert_eq!(result.unwrap(), "Update complete.");
    });
}

#[serial]
#[test]
fn update_fails_when_script_exits_with_error() {
    with_http_server("#!/usr/bin/env bash\nexit 1\n", 200, |url| {
        let prev = std::env::var("TREX_UPDATE_URL").ok();
        std::env::set_var("TREX_UPDATE_URL", url);
        let result = trex::commands::update::execute();
        match prev {
            Some(v) => std::env::set_var("TREX_UPDATE_URL", v),
            None => std::env::remove_var("TREX_UPDATE_URL"),
        }
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("exited with an error"),
            "expected 'exited with an error' in error message"
        );
    });
}

#[serial]
#[test]
fn update_fails_when_url_is_unreachable() {
    let prev = std::env::var("TREX_UPDATE_URL").ok();
    std::env::set_var("TREX_UPDATE_URL", "http://127.0.0.1:1/");
    let result = trex::commands::update::execute();
    match prev {
        Some(v) => std::env::set_var("TREX_UPDATE_URL", v),
        None => std::env::remove_var("TREX_UPDATE_URL"),
    }
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Failed to fetch"),
        "expected 'Failed to fetch' in error message"
    );
}
