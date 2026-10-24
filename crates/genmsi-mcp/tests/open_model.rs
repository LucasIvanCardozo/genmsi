//! The tool that says which model the app has open.
//!
//! The server runs as a process with a data directory of its own: the
//! directory of the app's data comes from the environment, so the test gives
//! the server one of its own and never touches the data of the machine.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use genmsi_core::open_model::{APP_IDENTIFIER, write_open_model};
use serde_json::{Value, json};

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
        let result = answer
            .get("result")
            .unwrap_or_else(|| panic!("the call failed: {answer}"));
        assert_eq!(
            result.get("isError").and_then(Value::as_bool),
            Some(false),
            "the tool failed: {result}"
        );
        let text = result["content"][0]["text"]
            .as_str()
            .unwrap_or_else(|| panic!("the tool answered with no text: {result}"));
        serde_json::from_str(text).unwrap_or_else(|error| panic!("the document parses: {error}"))
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
    let directory = std::env::temp_dir().join(format!("genmsi-mcp-openmodel-{name}-{}", std::process::id()));
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

#[test]
fn the_tool_says_that_there_is_no_model_open() {
    let home = scratch("none");
    let mut server = Server::start(&home);

    let document = server.call("open_model", json!({}));
    assert_eq!(document["schema"], "genmsi.openmodel/1", "{document}");
    assert!(document["path"].is_null(), "{document}");
    assert!(document["pid"].is_null(), "{document}");

    std::fs::remove_dir_all(&home).expect("the scratch directory is removed");
}

#[test]
fn the_tool_says_which_model_the_app_has_open() {
    let home = scratch("open");
    let model = home.join("modelo.gmdl");
    let directory = data_dir(&home);
    write_open_model(&directory, Some(&model), std::process::id()).expect("the file is written");

    let mut server = Server::start(&home);
    let document = server.call("open_model", json!({}));
    assert_eq!(document["schema"], "genmsi.openmodel/1", "{document}");
    assert_eq!(
        document["path"].as_str(),
        Some(model.to_str().unwrap()),
        "{document}"
    );
    assert_eq!(
        document["pid"].as_u64(),
        Some(std::process::id() as u64),
        "{document}"
    );

    std::fs::remove_dir_all(&home).expect("the scratch directory is removed");
}

#[test]
fn the_tool_says_nothing_when_the_app_is_not_running() {
    let home = scratch("dead");
    let model = home.join("modelo.gmdl");
    let directory = data_dir(&home);
    let pid = dead_pid();
    write_open_model(&directory, Some(&model), pid).expect("the file is written");

    // The file is there, but the app that wrote it is not: the document says
    // that there is no model open.
    let mut server = Server::start(&home);
    let document = server.call("open_model", json!({}));
    assert_eq!(document["schema"], "genmsi.openmodel/1", "{document}");
    assert!(document["path"].is_null(), "{document}");
    assert!(document["pid"].is_null(), "{document}");

    std::fs::remove_dir_all(&home).expect("the scratch directory is removed");
}

#[test]
fn the_tool_is_offered_and_says_what_it_is_for() {
    let home = scratch("listed");
    let mut server = Server::start(&home);
    let listed = server.request("tools/list", json!({}));    let tools = listed["result"]["tools"]
        .as_array()
        .unwrap_or_else(|| panic!("the tools are listed: {listed}"));
    let tool = tools
        .iter()
        .find(|tool| tool["name"] == "open_model")
        .unwrap_or_else(|| panic!("the tool is offered: {listed}"));
    let description = tool["description"]
        .as_str()
        .unwrap_or_else(|| panic!("the tool has a description: {tool}"));
    // The description says what the tool is for: which file the app has
    // open, so the agent writes the model there.
    assert!(description.contains("open"), "{description}");
    assert!(description.contains("path"), "{description}");
    assert!(description.contains("genmsi.openmodel/1"), "{description}");

    std::fs::remove_dir_all(&home).expect("the scratch directory is removed");
}
