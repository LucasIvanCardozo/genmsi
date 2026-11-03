//! The file that says which model the app has open.
//!
//! The file is the contract between the app and the agents: the app writes
//! it, the MCP server reads it, and the agent writes the model where it
//! says. These tests pin the format, the name of the file and the rules of
//! the directory it lives in.

use std::path::{Path, PathBuf};

use genmsi_core::open_model::{
    APP_IDENTIFIER, OPEN_MODEL_FILE, OpenModelDocument, app_data_dir, open_model_document_in,
    process_is_running, read_open_model_in, write_open_model,
};

/// A directory of its own for the files the test writes.
///
/// The directory is outside the repository and the test removes it when it
/// is done.
fn scratch(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!("genmsi-openmodel-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("the scratch directory is created");
    directory
}

/// The pid of a process that is not running any more.
///
/// The process is started and waited for: its pid is free again, and the
/// machine does not hand it to anybody else in the meantime.
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
fn the_file_is_named_open_model_json() {
    assert_eq!(OPEN_MODEL_FILE, "open-model.json");
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
fn the_document_names_its_schema() {
    let directory = scratch("schema");
    let document = OpenModelDocument::of(&directory.join("modelo.gmdl"), 42);
    let text = genmsi_core::to_pretty_json(genmsi_core::OPEN_MODEL_SCHEMA, &document);
    assert!(
        text.starts_with("{\n  \"schema\": \"genmsi.openmodel/1\",\n"),
        "{text}"
    );
    assert!(text.contains("\"path\""), "{text}");
    assert!(text.contains("\"pid\": 42"), "{text}");
    std::fs::remove_dir_all(&directory).expect("the scratch directory is removed");
}

#[test]
fn the_file_says_which_model_the_app_has_open() {
    let directory = scratch("open");
    let model = directory.join("modelo.gmdl");
    write_open_model(&directory, Some(&model), 4242).expect("the file is written");

    let document = read_open_model_in(&directory).expect("the file says something");
    assert_eq!(document.path.as_deref(), Some(model.to_str().unwrap()));
    assert_eq!(document.pid, Some(4242));

    // The file lives in the directory of the data, with the name of the
    // document of the app.
    let file = directory.join(OPEN_MODEL_FILE);
    assert!(file.is_file(), "the file is not there: {}", file.display());
    let text = std::fs::read_to_string(&file).expect("the file is readable");
    assert!(text.contains("\"schema\": \"genmsi.openmodel/1\""), "{text}");

    std::fs::remove_dir_all(&directory).expect("the scratch directory is removed");
}

#[test]
fn a_model_that_is_not_there_says_nothing() {
    let directory = scratch("missing");
    assert!(
        read_open_model_in(&directory).is_none(),
        "a file that is not there says something"
    );
    std::fs::remove_dir_all(&directory).expect("the scratch directory is removed");
}

#[test]
fn a_file_that_is_broken_says_nothing() {
    let directory = scratch("broken");
    std::fs::write(directory.join(OPEN_MODEL_FILE), "{ this is not json").expect("the file is written");
    assert!(
        read_open_model_in(&directory).is_none(),
        "a broken file says something"
    );
    std::fs::remove_dir_all(&directory).expect("the scratch directory is removed");
}

#[test]
fn a_new_model_removes_the_file() {
    let directory = scratch("new");
    let model = directory.join("modelo.gmdl");
    write_open_model(&directory, Some(&model), 7).expect("the file is written");
    assert!(directory.join(OPEN_MODEL_FILE).is_file(), "the file is there");

    // The app has no model open any more: the file goes away, so nobody
    // reads it and believes there is a model open.
    write_open_model(&directory, None, 7).expect("the file is removed");
    assert!(
        !directory.join(OPEN_MODEL_FILE).exists(),
        "the file is still there"
    );
    assert!(read_open_model_in(&directory).is_none(), "the file says something");

    std::fs::remove_dir_all(&directory).expect("the scratch directory is removed");
}

#[test]
fn the_path_of_the_model_is_absolute() {
    let directory = scratch("absolute");
    write_open_model(&directory, Some(Path::new("modelo.gmdl")), 7).expect("the file is written");
    let document = read_open_model_in(&directory).expect("the file says something");
    let path = document.path.expect("the document names the model");
    assert!(Path::new(&path).is_absolute(), "the path is not absolute: {path}");
    std::fs::remove_dir_all(&directory).expect("the scratch directory is removed");
}

#[test]
fn the_document_of_an_app_that_is_running_names_the_model() {
    let directory = scratch("running");
    let model = directory.join("modelo.gmdl");
    write_open_model(&directory, Some(&model), std::process::id()).expect("the file is written");

    let text = open_model_document_in(&directory);
    assert!(text.contains("\"schema\": \"genmsi.openmodel/1\""), "{text}");
    assert!(
        text.contains(model.to_str().unwrap()),
        "the document does not name the model: {text}"
    );
    assert!(
        text.contains(&format!("\"pid\": {}", std::process::id())),
        "the document does not name the process: {text}"
    );

    std::fs::remove_dir_all(&directory).expect("the scratch directory is removed");
}

#[test]
fn the_document_of_an_app_that_is_not_running_has_no_path() {
    let directory = scratch("dead");
    let model = directory.join("modelo.gmdl");
    let pid = dead_pid();
    assert!(!process_is_running(pid), "the process is running");
    write_open_model(&directory, Some(&model), pid).expect("the file is written");

    // The file is there, but the app that wrote it is not: the document says
    // that there is no model open.
    let text = open_model_document_in(&directory);
    assert!(text.contains("\"schema\": \"genmsi.openmodel/1\""), "{text}");
    assert!(text.contains("\"path\": null"), "{text}");
    assert!(text.contains("\"pid\": null"), "{text}");

    std::fs::remove_dir_all(&directory).expect("the scratch directory is removed");
}

#[test]
fn the_data_directory_is_the_one_of_the_app() {
    let directory = app_data_dir().expect("the machine says where its data lives");
    assert_eq!(
        directory.file_name().and_then(|name| name.to_str()),
        Some(APP_IDENTIFIER),
        "the directory is not the one of the app: {}",
        directory.display()
    );
}
