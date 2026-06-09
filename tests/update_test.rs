#[path = "common/mod.rs"]
mod common;

use common::create_test_tarball;
use common::with_http_server;
use serial_test::serial;

#[serial]
#[test]
fn update_succeeds_with_valid_tarball() {
    let tarball = create_test_tarball(b"#!/usr/bin/env bash\nexit 0\n");
    with_http_server(&tarball, 200, |url| {
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
fn update_fails_with_invalid_tarball() {
    with_http_server(b"not a valid tarball", 200, |url| {
        let prev = std::env::var("TREX_UPDATE_URL").ok();
        std::env::set_var("TREX_UPDATE_URL", url);
        let result = trex::commands::update::execute();
        match prev {
            Some(v) => std::env::set_var("TREX_UPDATE_URL", v),
            None => std::env::remove_var("TREX_UPDATE_URL"),
        }
        assert!(result.is_err());
        let err = result.as_ref().unwrap_err().to_string();
        assert!(
            err.contains("Failed to extract"),
            "expected 'Failed to extract' in error message, got: {err}",
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
            .contains("Failed to download"),
        "expected 'Failed to download' in error message"
    );
}
