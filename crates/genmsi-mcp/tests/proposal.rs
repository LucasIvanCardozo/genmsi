//! The loop of the proposal: the agent proposes, the person decides.
//!
//! Two things happen here, and both of them are about where the work of the
//! agent goes:
//!
//! - the **default** of a tool that takes a model is the model the app has
//!   open: the agent works on what the reader is looking at, and without an
//!   app there is no destination to guess (the failure is `no_open_model`);
//! - a tool that takes a `target` of `proposal` leaves its result in the
//!   staging area of the app, without touching the disk, and describes it
//!   with a `genmsi.proposal/1` document: the person sees the work and
//!   decides.
//!
//! The server runs as a process of its own: the data directory of the app
//! comes from the environment, and a server in process would share the data
//! directory of the machine the test runs on.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use genmsi_core::live::{LiveBody, write_live};
use genmsi_core::app_data::APP_IDENTIFIER;
use serde_json::{Value, json};

/// A model that works: a generator, a terminator and the link between them.
const WORKING_MODEL: &str = "<gmodel versioncode=\"1\"><gclasses/><modules>\
<module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"Modelo\" name=\"Modelo1\" kind=\"C\">\
<links><link frommoduleid=\"1\" fromioid=\"1\" tomoduleid=\"2\" toioid=\"1\"/></links><inputs/><outputs/></module>\
<module moduleid=\"1\" moduleclass=\"Generate\" owner=\"0\" caption=\"Generate\" name=\"Generate\" kind=\"A\">\
<outputs><output ioid=\"1\" name=\"Salida1\" caption=\"\" mylabel=\"L1\"/></outputs>\
<props><property name=\"Media\" hint=\"\" value=\"10\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/></props></module>\
<module moduleid=\"2\" moduleclass=\"Terminate\" owner=\"0\" caption=\"Terminate\" name=\"Terminate\" kind=\"A\">\
<inputs><input ioid=\"1\" name=\"Entrada1\" caption=\"\" mylabel=\"L1\"/></inputs>\
<props><property name=\"Unidades\" hint=\"\" value=\"1\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/></props></module>\
</modules></gmodel>";

/// The server, with the data directory of its own.
struct Server {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl Server {
    /// Start the server with the data directory of the test.
    fn start(data_home: &Path) -> Self {
        let mut process = Command::new(env!("CARGO_BIN_EXE_genmsi-mcp"))
            .env("XDG_DATA_HOME", data_home)
            .env("HOME", data_home)
            .env("APPDATA", data_home)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("the server starts");
        let stdin = process.stdin.take().expect("the server reads");
        let stdout = BufReader::new(process.stdout.take().expect("the server writes"));
        let mut server = Self {
            process,
            stdin,
            stdout,
            next_id: 0,
        };
        server.request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "genmsi-test", "version": "1" },
            }),
        );
        server.notify("notifications/initialized");
        server
    }

    /// Send a request and wait for its answer.
    fn request(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let message = json!({
            "jsonrpc": "2.0",
            "id": self.next_id,
            "method": method,
            "params": params,
        });
        self.send(&message);
        loop {
            let answer = self.read();
            if answer.get("id").and_then(Value::as_u64) == Some(self.next_id) {
                return answer;
            }
        }
    }

    /// Send a notification: the server does not answer it.
    fn notify(&mut self, method: &str) {
        let message = json!({ "jsonrpc": "2.0", "method": method });
        self.send(&message);
    }

    /// Call a tool and return the document it answered with, and whether the
    /// tool failed.
    fn call(&mut self, tool: &str, arguments: Value) -> (Value, bool) {
        let answer = self.request("tools/call", json!({ "name": tool, "arguments": arguments }));
        let result = answer
            .get("result")
            .unwrap_or_else(|| panic!("the call failed: {answer}"));
        let failed = result.get("isError").and_then(Value::as_bool).unwrap_or(false);
        let text = result["content"][0]["text"]
            .as_str()
            .unwrap_or_else(|| panic!("the tool answered with no text: {result}"));
        let document =
            serde_json::from_str(text).unwrap_or_else(|error| panic!("the document parses: {error}"));
        (document, failed)
    }

    /// Write one message to the server.
    fn send(&mut self, message: &Value) {
        let line = serde_json::to_string(message).expect("the message serializes");
        self.stdin.write_all(line.as_bytes()).expect("the server reads");
        self.stdin.write_all(b"\n").expect("the server reads");
        self.stdin.flush().expect("the server reads");
    }

    /// Read one message from the server.
    fn read(&mut self) -> Value {
        let mut line = String::new();
        self.stdout.read_line(&mut line).expect("the server writes");
        assert!(!line.is_empty(), "the server closed the connection");
        serde_json::from_str(&line).unwrap_or_else(|error| panic!("the message parses: {error}"))
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

/// A directory of its own for the files the test writes.
///
/// The directory is outside the repository and the test removes it when it
/// is done.
fn scratch(name: &str) -> PathBuf {
    let directory = std::env::temp_dir()
        .join(format!("genmsi-mcp-proposal-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("the scratch directory is created");
    directory
}

/// The directory the app's data lives in, for the data home of the test.
///
/// The rule is the platform's, written down again on purpose: a test that
/// repeats the rule of the code is the one that checks it.
fn data_dir(data_home: &Path) -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        data_home
            .join("Library")
            .join("Application Support")
            .join(APP_IDENTIFIER)
    }
    #[cfg(not(target_os = "macos"))]
    {
        data_home.join(APP_IDENTIFIER)
    }
}

/// The staging area of the app, for the data home of the test.
fn staging_dir(data_home: &Path) -> PathBuf {
    data_dir(data_home).join("staging")
}

/// The files of the staging area, sorted.
fn staged_files(data_home: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(staging_dir(data_home)) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    names
}

/// The state of an app that has a model open.
///
/// The pid is the pid of the test: the process is alive, so the document
/// says that the app is running.
fn publish_open_model(data_home: &Path, model: &Path) {
    // The app publishes ONE document about itself, and it carries the
    // model it has open.
    let body = LiveBody {
        path: Some(model.display().to_string()),
        pid: Some(std::process::id()),
        dirty: false,
        diagnostics: Vec::new(),
        model: None,
        run: None,
        proposal: None,
    };
    write_live(&data_dir(data_home), &body).expect("the live state is published");
}

/// The default of a tool: the model the app has open.
///
/// The agent does not name a file, and the tool works on the model the
/// reader is looking at.
#[test]
fn a_tool_without_a_path_works_on_the_model_the_app_has_open() {
    let home = scratch("default");
    let model = home.join("modelo.gmdl");
    std::fs::write(&model, WORKING_MODEL).expect("the model is on the disk");
    publish_open_model(&home, &model);

    let mut server = Server::start(&home);

    // The lint says which model it linted: the one the app has open.
    let (linted, failed) = server.call("lint", json!({}));
    assert!(!failed, "the tool worked: {linted}");
    assert_eq!(linted["schema"], "genmsi.lint/1", "{linted}");
    assert_eq!(
        linted["path"].as_str(),
        Some(model.to_str().unwrap()),
        "the model the app has open is the one that is linted: {linted}"
    );

    // The verb edits the same model: the batch is applied to the file the
    // app has open.
    let (edited, failed) = server.call(
        "apply_edit",
        json!({
            "commands": [
                { "op": "setCaption", "id": 1, "caption": "Entrada" }
            ]
        }),
    );
    assert!(!failed, "the verb worked: {edited}");
    assert_eq!(edited["schema"], "genmsi.edit/1", "{edited}");
    assert_eq!(edited["written"], true, "{edited}");
    assert_eq!(
        edited["path"].as_str(),
        Some(model.to_str().unwrap()),
        "the model the app has open is the one that is edited: {edited}"
    );
    let written = std::fs::read_to_string(&model).expect("the model is readable");
    assert!(
        written.contains("Entrada"),
        "the batch was applied to the model the app has open: {written}"
    );

    std::fs::remove_dir_all(&home).expect("the scratch directory is removed");
}

/// Without an app there is no destination to guess.
///
/// The failure is its own class and says what to do: give a path, or open a
/// model in the app.
#[test]
fn without_an_app_a_tool_without_a_path_says_what_to_do() {
    let home = scratch("no-app");
    let mut server = Server::start(&home);

    for tool in ["lint", "simulate", "get_trace", "explain_model", "apply_edit"] {
        let arguments = if tool == "apply_edit" {
            json!({ "commands": [] })
        } else {
            json!({})
        };
        let (answer, failed) = server.call(tool, arguments);
        assert!(failed, "the tool refused: {tool}: {answer}");
        assert_eq!(answer["error"], "no_open_model", "{tool}: {answer}");
        let message = answer["message"].as_str().expect("a message");
        assert!(
            message.contains("give `path`") && message.contains("open a model in the app"),
            "{tool}: the message says what to do: {message}"
        );
    }

    // The verb that writes says the same thing.
    let (answer, failed) = server.call("save_model", json!({}));
    assert!(failed, "the tool refused: {answer}");
    assert_eq!(answer["error"], "no_open_model", "{answer}");

    std::fs::remove_dir_all(&home).expect("the scratch directory is removed");
}

/// An app that is not running any more does not hold a model.
///
/// The file the app left behind says nothing: the pid of the document says
/// whether the process is alive.
#[test]
fn an_app_that_died_does_not_hold_a_model() {
    let home = scratch("dead-app");
    let model = home.join("modelo.gmdl");
    std::fs::write(&model, WORKING_MODEL).expect("the model is on the disk");

    let mut process = Command::new("sh")
        .arg("-c")
        .arg("exit 0")
        .spawn()
        .expect("the process starts");
    process.wait().expect("the process ends");
    // The app that died published its state: the pid says that the process
    // is not alive, so the model it had open is nobody's destination.
    let body = LiveBody {
        path: Some(model.display().to_string()),
        pid: Some(process.id()),
        dirty: false,
        diagnostics: Vec::new(),
        model: None,
        run: None,
        proposal: None,
    };
    write_live(&data_dir(&home), &body).expect("the live state is published");

    let mut server = Server::start(&home);
    let (answer, failed) = server.call("lint", json!({}));
    assert!(failed, "the tool refused: {answer}");
    assert_eq!(answer["error"], "no_open_model", "{answer}");

    std::fs::remove_dir_all(&home).expect("the scratch directory is removed");
}

/// A proposal does not touch the disk.
///
/// The batch is applied to the model the agent named, the result waits in
/// the staging area of the app, and the file is byte for byte the file that
/// was there.
#[test]
fn a_proposal_leaves_the_model_in_the_staging_area() {
    let home = scratch("proposal");
    let model = home.join("modelo.gmdl");
    std::fs::write(&model, WORKING_MODEL).expect("the model is on the disk");
    let before = std::fs::read(&model).expect("the model is readable");
    publish_open_model(&home, &model);

    let mut server = Server::start(&home);
    let (edited, failed) = server.call(
        "apply_edit",
        json!({
            "path": model.to_string_lossy(),
            "target": "proposal",
            "summary": "el bus del anillo",
            "commands": [
                { "op": "setCaption", "id": 1, "caption": "Entrada" }
            ]
        }),
    );

    assert!(!failed, "the verb worked: {edited}");
    assert_eq!(edited["schema"], "genmsi.edit/1", "{edited}");
    assert_eq!(edited["written"], false, "a proposal is not written: {edited}");
    assert_eq!(edited["applied"], 1, "{edited}");
    assert!(
        edited["source"].is_null(),
        "the source keeps its meaning: the model was named by a file: {edited}"
    );

    // The answer carries the identity of the proposal and the path of the
    // model it carries.
    let proposal = &edited["proposal"];
    let id = proposal["id"].as_str().expect("the identity of the proposal");
    assert!(!id.is_empty(), "the proposal has an identity: {edited}");
    let staged = proposal["model"].as_str().expect("the model of the proposal");
    assert!(
        staged.ends_with(&format!("{id}.gmdl")),
        "the staged model is named after the proposal: {edited}"
    );
    assert!(
        Path::new(staged).is_absolute(),
        "the staged model has an absolute path: {edited}"
    );

    // The file on the disk is byte for byte the file that was there.
    let after = std::fs::read(&model).expect("the model is readable");
    assert!(after == before, "the proposal changed the file");

    // The staging area holds the model of the proposal and the document that
    // describes it.
    let files = staged_files(&home);
    assert_eq!(
        files,
        vec![format!("{id}.gmdl"), format!("{id}.json")],
        "the staging area holds the model and its document: {files:?}"
    );

    // The model that waits is the model the batch built.
    let staged_text = std::fs::read_to_string(staged).expect("the staged model is readable");
    assert!(
        staged_text.contains("Entrada"),
        "the staged model carries the batch: {staged_text}"
    );

    // The document says that the proposal waits for the person, and says
    // what the agent did.
    let document: Value = serde_json::from_str(
        &std::fs::read_to_string(staging_dir(&home).join(format!("{id}.json")))
            .expect("the document is readable"),
    )
    .expect("the document parses");
    assert_eq!(document["schema"], "genmsi.proposal/1", "{document}");
    assert_eq!(document["id"], id, "{document}");
    assert_eq!(document["state"], "pending", "{document}");
    assert_eq!(document["summary"], "el bus del anillo", "{document}");
    assert_eq!(document["model"], staged, "{document}");
    assert_eq!(
        document["target"].as_str(),
        Some(model.to_str().unwrap()),
        "the proposal proposes a change to the model the app has open: {document}"
    );

    std::fs::remove_dir_all(&home).expect("the scratch directory is removed");
}

/// The proposal is about the model the agent worked on.
///
/// The agent names a file that is not the one the app has open: the
/// proposal says so, because the model it carries is that file's model. The
/// app does not adopt a proposal about another model (it waits), and a
/// proposal that lied about its target would replace the model the reader
/// is looking at with an unrelated one.
#[test]
fn the_proposal_is_about_the_model_the_agent_named() {
    let home = scratch("proposal-target");
    let open = home.join("abierto.gmdl");
    let other = home.join("otro.gmdl");
    std::fs::write(&open, WORKING_MODEL).expect("the model is on the disk");
    std::fs::write(&other, WORKING_MODEL).expect("the model is on the disk");
    publish_open_model(&home, &open);

    let mut server = Server::start(&home);
    let (edited, failed) = server.call(
        "apply_edit",
        json!({
            "path": other.to_string_lossy(),
            "target": "proposal",
            "commands": [
                { "op": "setCaption", "id": 1, "caption": "Entrada" }
            ]
        }),
    );
    assert!(!failed, "the verb worked: {edited}");

    let id = edited["proposal"]["id"]
        .as_str()
        .expect("the identity of the proposal");
    let document: Value = serde_json::from_str(
        &std::fs::read_to_string(staging_dir(&home).join(format!("{id}.json")))
            .expect("the document is readable"),
    )
    .expect("the document parses");
    assert_eq!(
        document["target"].as_str(),
        Some(other.to_str().unwrap()),
        "the proposal is about the model the agent named: {document}"
    );
    assert_ne!(
        document["target"].as_str(),
        Some(open.to_str().unwrap()),
        "the proposal does not pretend to be about the model the app has open: {document}"
    );

    std::fs::remove_dir_all(&home).expect("the scratch directory is removed");
}

/// A proposal of a model given as text is the same proposal.
///
/// The source of the batch is what the agent named: the text of the model.
#[test]
fn a_proposal_of_a_model_given_as_text() {
    let home = scratch("proposal-inline");
    let model = home.join("modelo.gmdl");
    std::fs::write(&model, WORKING_MODEL).expect("the model is on the disk");
    publish_open_model(&home, &model);

    let mut server = Server::start(&home);
    let (edited, failed) = server.call(
        "apply_edit",
        json!({
            "model": WORKING_MODEL,
            "target": "proposal",
            "commands": [
                { "op": "setCaption", "id": 2, "caption": "Salida" }
            ]
        }),
    );

    assert!(!failed, "the verb worked: {edited}");
    assert_eq!(edited["written"], false, "a proposal is not written: {edited}");
    let proposal = &edited["proposal"];
    let id = proposal["id"].as_str().expect("the identity of the proposal");
    let staged = proposal["model"].as_str().expect("the model of the proposal");
    assert!(
        staged.ends_with(&format!("{id}.gmdl")),
        "the staged model is named after the proposal: {edited}"
    );
    assert_eq!(staged_files(&home).len(), 2, "the staging area holds two files");

    std::fs::remove_dir_all(&home).expect("the scratch directory is removed");
}

/// The batch is still atomic when it goes to the staging area.
///
/// A command that fails leaves the model as it was, and leaves nothing in
/// the staging area: a half-applied batch can never travel anywhere.
#[test]
fn a_failing_batch_leaves_nothing_behind() {
    let home = scratch("proposal-fail");
    let model = home.join("modelo.gmdl");
    std::fs::write(&model, WORKING_MODEL).expect("the model is on the disk");
    let before = std::fs::read(&model).expect("the model is readable");
    publish_open_model(&home, &model);

    let mut server = Server::start(&home);
    let (answer, failed) = server.call(
        "apply_edit",
        json!({
            "path": model.to_string_lossy(),
            "target": "proposal",
            "commands": [
                { "op": "setCaption", "id": 1, "caption": "Entrada" },
                { "op": "connect", "from": 99, "fromIoid": 1, "to": 2, "toIoid": 1 }
            ]
        }),
    );

    assert!(failed, "the batch failed: {answer}");
    assert_eq!(answer["error"], "invalid_command", "{answer}");
    assert_eq!(answer["command"], 1, "{answer}");

    let after = std::fs::read(&model).expect("the model is readable");
    assert!(after == before, "the failed batch changed the file");
    assert!(
        staged_files(&home).is_empty(),
        "the failed batch left nothing in the staging area: {:?}",
        staged_files(&home)
    );

    std::fs::remove_dir_all(&home).expect("the scratch directory is removed");
}

/// The verb that writes can propose too.
///
/// The model does not touch the disk: it waits in the staging area, and the
/// answer names the model that waits there.
#[test]
fn save_model_can_leave_a_proposal() {
    let home = scratch("save-proposal");
    let model = home.join("modelo.gmdl");
    std::fs::write(&model, WORKING_MODEL).expect("the model is on the disk");
    let before = std::fs::read(&model).expect("the model is readable");
    publish_open_model(&home, &model);

    let mut server = Server::start(&home);
    let (linted, failed) = server.call(
        "save_model",
        json!({
            "model": WORKING_MODEL,
            "target": "proposal",
            "summary": "el modelo completo"
        }),
    );

    assert!(!failed, "the tool worked: {linted}");
    assert_eq!(linted["schema"], "genmsi.lint/1", "{linted}");
    let after = std::fs::read(&model).expect("the model is readable");
    assert!(after == before, "the proposal changed the file");

    let files = staged_files(&home);
    assert_eq!(files.len(), 2, "the staging area holds two files: {files:?}");
    let document: Value = serde_json::from_str(
        &std::fs::read_to_string(staging_dir(&home).join(&files[1]))
            .expect("the document is readable"),
    )
    .expect("the document parses");
    assert_eq!(document["schema"], "genmsi.proposal/1", "{document}");
    assert_eq!(document["state"], "pending", "{document}");
    assert_eq!(document["summary"], "el modelo completo", "{document}");

    // The answer names the model that waits in the staging area.
    let staged = linted["path"].as_str().expect("the path of the model");
    assert!(
        staged.starts_with(staging_dir(&home).to_str().unwrap()),
        "the answer names the staged model: {linted}"
    );

    std::fs::remove_dir_all(&home).expect("the scratch directory is removed");
}

/// Without a path the verb writes the model the app has open.
///
/// The path may be left out: the model the reader is looking at is the one
/// that is written.
#[test]
fn save_model_without_a_path_writes_the_model_the_app_has_open() {
    let home = scratch("save-default");
    let model = home.join("modelo.gmdl");
    std::fs::write(&model, WORKING_MODEL).expect("the model is on the disk");
    publish_open_model(&home, &model);

    let mut server = Server::start(&home);
    let (linted, failed) = server.call(
        "save_model",
        json!({ "model": WORKING_MODEL }),
    );

    assert!(!failed, "the tool worked: {linted}");
    assert_eq!(linted["schema"], "genmsi.lint/1", "{linted}");
    assert_eq!(
        linted["path"].as_str(),
        Some(model.to_str().unwrap()),
        "the model the app has open is the one that is written: {linted}"
    );

    std::fs::remove_dir_all(&home).expect("the scratch directory is removed");
}

/// The tools still say what they are for.
///
/// The description is what the agent reads: it has to say that the path may
/// be left out, what a proposal is, and where the verdict comes back.
#[test]
fn the_descriptions_say_what_the_new_fields_do() {
    let home = scratch("descriptions");
    let mut server = Server::start(&home);
    let listed = server.request("tools/list", json!({}));
    let tools = listed["result"]["tools"]
        .as_array()
        .unwrap_or_else(|| panic!("the tools are listed: {listed}"));

    for name in ["apply_edit", "save_model"] {
        let tool = tools
            .iter()
            .find(|tool| tool["name"] == name)
            .unwrap_or_else(|| panic!("the tool is offered: {listed}"));
        let description = tool["description"]
            .as_str()
            .unwrap_or_else(|| panic!("the tool has a description: {tool}"));
        assert!(
            description.contains("`path` may be left out"),
            "{name}: the description says the path may be left out: {description}"
        );
        assert!(
            description.contains("proposal"),
            "{name}: the description says what a proposal is: {description}"
        );
        assert!(
            description.contains("app_state"),
            "{name}: the description says where the verdict comes back: {description}"
        );
    }

    std::fs::remove_dir_all(&home).expect("the scratch directory is removed");
}
