//! Black-box test of `genmsi mcp`: the CLI hosting the MCP server.
//!
//! The test speaks the protocol itself with a hand-written JSON-RPC request,
//! through the public interface only: one `initialize` request on stdin, one
//! response line on stdout, then a clean exit when stdin closes.

use std::io::Write as _;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// One `initialize` request: a stable, well-known protocol version the server
/// must echo back, and an identity for this test client.
const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"genmsi-cli-test","version":"0"}}}"#;

/// How long the server may take to answer and exit before the test gives up.
const BUDGET: Duration = Duration::from_secs(10);

#[test]
fn mcp_answers_the_initialize_handshake_and_exits_cleanly() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_genmsi"))
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the genmsi binary runs");

    // The server reads line-delimited JSON-RPC; closing stdin after the
    // request lets it answer and then disconnect.
    {
        let mut stdin = child.stdin.take().expect("stdin is piped");
        stdin
            .write_all(INITIALIZE.as_bytes())
            .and_then(|()| stdin.write_all(b"\n"))
            .and_then(|()| stdin.flush())
            .expect("the request is written");
    }

    // Bounded wait: the server must answer and exit within the budget, or it
    // is killed and the captured stderr explains why.
    let deadline = Instant::now() + BUDGET;
    let status = loop {
        match child.try_wait().expect("the child is pollable") {
            Some(status) => break status,
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let output = child.wait_with_output().expect("the child is read");
                panic!(
                    "`genmsi mcp` did not exit within {BUDGET:?}; stderr: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    };

    // The response is small enough for the pipe buffer, so it is read after
    // the exit: stdout is one JSON-RPC response line, stderr stays empty.
    let output = child.wait_with_output().expect("the child is read");
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");

    assert!(status.success(), "exit {status:?}; stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr must stay empty: {stderr}");
    let mut lines = stdout.lines();
    let response: serde_json::Value =
        serde_json::from_str(lines.next().unwrap_or_default()).expect("stdout is one JSON line");
    assert!(lines.next().is_none(), "stdout is one line, not: {stdout}");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert_eq!(response["result"]["protocolVersion"], "2025-06-18");
    let name = response["result"]["serverInfo"]["name"]
        .as_str()
        .expect("the server advertises its name");
    assert!(!name.is_empty(), "the server name is non-empty");
    assert!(
        response["result"]["capabilities"]["tools"].is_object(),
        "the server offers tools"
    );
}
