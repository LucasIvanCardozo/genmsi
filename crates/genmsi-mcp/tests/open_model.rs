//! The model the app has open: the default path of the tools.
//!
//! The app publishes **one** document about itself (`live.json`, the
//! `genmsi.live/1`), and that document carries the model it has open and
//! the pid of the process that has it. A tool that takes a model and is
//! given neither a path nor a document works on that model: the agent
//! works on what the reader is looking at instead of guessing a
//! destination.
//!
//! The server runs as a process with a data directory of its own: the
//! directory of the app's data comes from the environment, so the test
//! gives the server one of its own and never touches the data of the
//! machine.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use genmsi_core::live::{LiveBody, write_live};
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

    /// Call a tool and return the document it answered with.
    fn call(&mut self, tool: &str, arguments: Value) -> Value {
        let answer = self.request("tools/call", json!({ "name": tool, "arguments": arguments }));
        if let Some(error) = answer.get("error") {
            return error.clone();
        }
        let content = &answer["result"]["content"][0]["text"];
        serde_json::from_str(content.as_str().unwrap_or_default()).unwrap_or(Value::Null)
    }

    /// Send a message to the server.
    fn send(&mut self, message: &Value) {
        let text = serde_json::to_string(message).expect("the message serializes");
        self.stdin.write_all(text.as_bytes()).expect("the server reads");
        self.stdin.write_all(b"\n").expect("the server reads");
        self.stdin.flush().expect("the server reads");
    }

    /// Read a message of the server.
    fn read(&mut self) -> Value {
        let mut line = String::new();
        self.stdout.read_line(&mut line).expect("the server writes");
        serde_json::from_str(&line).expect("the server writes JSON")
    }
}

impl Drop for Server {
    /// The server of the test does not outlive it.
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

/// A scratch directory that cleans itself up.
fn scratch(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "genmsi-open-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("the scratch directory is created");
    path
}

/// The directory of the app's data, inside the scratch one.
fn data_dir(data_home: &Path) -> PathBuf {
    data_home.join(genmsi_core::app_data::APP_IDENTIFIER)
}

/// The pid of a process that is not running any more.
fn dead_pid() -> u32 {
    let mut process = Command::new("sh")
        .arg("-c")
        .arg("exit 0")
        .spawn()
        .expect("the process starts");
    process.wait().expect("the process ends");
    process.id()
}

/// Publish the live state of an app that has the model open.
fn publish_live(directory: &Path, model: &Path, pid: u32) {
    let body = LiveBody {
        path: Some(model.display().to_string()),
        pid: Some(pid),
        dirty: false,
        diagnostics: Vec::new(),
        model: None,
        run: None,
        proposal: None,
    };
    write_live(directory, &body).expect("the state is written");
}

/// The default of a tool is the model the app has open.
///
/// The agent names neither a file nor a document, and the tool works on
/// the model the reader is looking at.
#[test]
fn the_default_path_is_the_model_the_app_has_open() {
    let home = scratch("open");
    let model = home.join("modelo.gmdl");
    std::fs::write(&model, WORKING_MODEL).expect("the model is on the disk");
    publish_live(&data_dir(&home), &model, std::process::id());

    let mut server = Server::start(&home);
    let document = server.call("lint", json!({}));
    assert_eq!(document["schema"], "genmsi.lint/1", "{document}");
    assert_eq!(
        document["path"].as_str(),
        Some(model.to_str().unwrap()),
        "the tool worked on the model the app has open: {document}"
    );

    std::fs::remove_dir_all(&home).expect("the scratch directory is removed");
}

/// An app that is not running says nothing.
///
/// The live document of an app that is gone says nothing: the pid says
/// whether the process is alive. Without a model open there is no
/// destination to guess, and the failure says what to do.
#[test]
fn the_default_path_says_what_to_do_when_the_app_is_not_running() {
    let home = scratch("dead");
    let model = home.join("modelo.gmdl");
    std::fs::write(&model, WORKING_MODEL).expect("the model is on the disk");
    publish_live(&data_dir(&home), &model, dead_pid());

    let mut server = Server::start(&home);
    let failure = server.call("lint", json!({}));
    assert_eq!(failure["error"], "no_open_model", "{failure}");
    let message = failure["message"].as_str().expect("a message");
    assert!(
        message.contains("give `path`") && message.contains("open a model in the app"),
        "the message says what to do: {message}"
    );

    std::fs::remove_dir_all(&home).expect("the scratch directory is removed");
}

/// The tool that said which model the app has open is gone.
///
/// It answered a subset of what `app_state` answers, and the app published
/// the same fact in two files that could disagree. There is one document
/// and one tool now.
#[test]
fn the_tool_that_said_which_model_is_open_is_not_offered() {
    let home = scratch("listed");
    let mut server = Server::start(&home);
    let listed = server.request("tools/list", json!({}));
    let tools = listed["result"]["tools"]
        .as_array()
        .unwrap_or_else(|| panic!("the tools are listed: {listed}"));
    let names: Vec<&str> = tools
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect();
    assert!(
        !names.contains(&"open_model"),
        "the tool is gone: {names:?}"
    );
    assert_eq!(names.len(), 8, "the tools are eight: {names:?}");

    std::fs::remove_dir_all(&home).expect("the scratch directory is removed");
}
