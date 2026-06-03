#[path = "common/mod.rs"]
mod common;

use common::with_temp_trex_dir;
use serial_test::serial;

#[serial]
#[test]
fn ignore_crud_integration() {
    with_temp_trex_dir(|| {
        trex::commands::ignore::execute_add("dev").unwrap();
        trex::commands::ignore::execute_add("work").unwrap();

        let list = trex::commands::ignore::execute_list().unwrap();
        assert!(list.contains("Ignored sessions: 2"));
        assert!(list.contains("* dev"));
        assert!(list.contains("* work"));

        trex::commands::ignore::execute_remove("dev").unwrap();
        let list = trex::commands::ignore::execute_list().unwrap();
        assert!(list.contains("Ignored sessions: 1"));

        let result = trex::commands::ignore::execute_remove("nonexistent");
        assert!(result.is_err());
    });
}

#[serial]
#[test]
fn ignore_duplicate_returns_error() {
    with_temp_trex_dir(|| {
        trex::commands::ignore::execute_add("dev").unwrap();
        let result = trex::commands::ignore::execute_add("dev");
        assert!(result.is_err());
    });
}

#[serial]
#[test]
fn ignore_list_empty() {
    with_temp_trex_dir(|| {
        let list = trex::commands::ignore::execute_list().unwrap();
        assert_eq!(list, "No ignored sessions.");
    });
}
