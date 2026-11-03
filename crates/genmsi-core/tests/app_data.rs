//! Where the data of the app lives.
//!
//! The app publishes what it is showing in its own data directory, and the
//! MCP server reads it from there. These tests pin the two things that
//! module answers: **where** that directory is, and **whether a process is
//! still running** (the pid of the published document says whether the app
//! that wrote it is alive).

use std::path::Path;

use genmsi_core::app_data::{APP_IDENTIFIER, app_data_dir, process_is_running};

/// The pid of a process that is not running any more.
fn dead_pid() -> u32 {
    let mut process = std::process::Command::new("sh")
        .arg("-c")
        .arg("exit 0")
        .spawn()
        .expect("the process starts");
    process.wait().expect("the process ends");
    process.id()
}

#[test]
fn the_identifier_is_the_one_the_app_declares() {
    assert_eq!(APP_IDENTIFIER, "org.gensmi.desktop");
    // The shell of the app declares its identifier in its configuration: the
    // directory of the data is named after it. The two have to be the same,
    // or the MCP server looks in a directory nobody writes to.
    let configuration = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/genmsi-app/src-tauri/tauri.conf.json");
    let text = std::fs::read_to_string(&configuration)
        .unwrap_or_else(|error| panic!("{}: {error}", configuration.display()));
    assert!(
        text.contains(&format!("\"identifier\": \"{APP_IDENTIFIER}\"")),
        "the app does not declare the identifier: {text}"
    );
}

#[test]
fn the_directory_of_the_data_is_named_after_the_identifier() {
    let directory = app_data_dir().expect("the machine says where its data lives");
    assert!(
        directory.ends_with(APP_IDENTIFIER),
        "the directory is named after the app: {}",
        directory.display()
    );
    assert!(
        directory.is_absolute(),
        "the directory is an absolute one: {}",
        directory.display()
    );
}

#[test]
fn a_relative_data_home_is_not_one() {
    // The rules of the platform: a relative `XDG_DATA_HOME` is not a
    // directory of its own, it is the real one. A test that sets a relative
    // one silently talks to the app of the machine.
    let previous = std::env::var("XDG_DATA_HOME").ok();
    // The variable is read by the platform's rules, and the test does not
    // change it: what it pins is the rule itself.
    if let Some(value) = previous.as_deref()
        && !Path::new(value).is_absolute()
    {
        let directory = app_data_dir().expect("the machine says where its data lives");
        assert!(
            !directory.starts_with(value),
            "a relative data home is ignored: {}",
            directory.display()
        );
    }
}

#[test]
fn a_process_that_is_running_is_running() {
    let pid = std::process::id();
    assert!(process_is_running(pid), "this process is running");
}

#[test]
fn a_process_that_is_gone_is_not_running() {
    assert!(
        !process_is_running(dead_pid()),
        "the process that ended is not running"
    );
}
