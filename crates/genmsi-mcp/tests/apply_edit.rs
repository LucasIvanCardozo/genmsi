//! The verb of the MCP: the tool that adds, edits and connects the blocks.
//!
//! The tool speaks the engine's own grammar (`EditCommand`), so a batch of
//! commands is the same thing the desktop application speaks by wire. The
//! batch is **atomic**: a command that fails leaves the model as it was. The
//! model is checked **before** it is written: one that does not compile is
//! not written at all.
//!
//! The test below talks the protocol an agent talks: the server runs in
//! process over a duplex transport and the test calls `tools/call` for real.

use std::path::PathBuf;

use genmsi_mcp::GenmsiServer;
use rmcp::model::{CallToolRequestParams, ContentBlock};
use rmcp::ServiceExt;
use serde_json::{json, Value};

/// A model that works: a generator, a terminator and the link between them.
///
/// It carries the geometry of its blocks, so the engine composes nothing and
/// the places a batch gives are the places the file keeps.
const WORKING_MODEL: &str = "<gmodel versioncode=\"1\"><gclasses/><modules>\
<module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"Modelo\" name=\"Modelo1\" kind=\"C\" genprefer=\"\" exehead=\"\" nonexehead=\"\" exefoot=\"\" nonexefoot=\"\">\
<links><link frommoduleid=\"1\" fromioid=\"1\" tomoduleid=\"2\" toioid=\"1\"/></links><inputs/><outputs/></module>\
<module moduleid=\"1\" moduleclass=\"Generate\" owner=\"0\" caption=\"Generate\" name=\"Generate\" kind=\"A\" width=\"120\" height=\"48\" x=\"0\" y=\"0\">\
<outputs><output ioid=\"1\" name=\"Salida1\" caption=\"\" mylabel=\"L1\"/></outputs>\
<props><property name=\"Media\" hint=\"\" value=\"10\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/></props></module>\
<module moduleid=\"2\" moduleclass=\"Terminate\" owner=\"0\" caption=\"Terminate\" name=\"Terminate\" kind=\"A\" width=\"120\" height=\"48\" x=\"180\" y=\"0\">\
<inputs><input ioid=\"1\" name=\"Entrada1\" caption=\"\" mylabel=\"L1\"/></inputs>\
<props><property name=\"Unidades\" hint=\"\" value=\"1\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/></props></module>\
</modules></gmodel>";

/// A client and a server that talk over a duplex transport.
async fn pair() -> (
    rmcp::service::RunningService<rmcp::RoleClient, ()>,
    tokio::task::JoinHandle<()>,
) {
    let (client_transport, server_transport) = tokio::io::duplex(4 * 1024 * 1024);
    let server = tokio::spawn(async move {
        let service = GenmsiServer::new()
            .serve(server_transport)
            .await
            .expect("the server starts");
        service.waiting().await.expect("the server ends");
    });
    let client = ().serve(client_transport).await.expect("the client starts");
    (client, server)
}

/// Call a tool and return its content as JSON.
async fn call(
    client: &rmcp::service::RunningService<rmcp::RoleClient, ()>,
    tool: &str,
    arguments: Value,
) -> Value {
    let arguments = match arguments {
        Value::Object(map) => map,
        other => panic!("arguments must be an object, got {other}"),
    };
    let result = client
        .call_tool(CallToolRequestParams::new(tool.to_string()).with_arguments(arguments))
        .await
        .expect("the call returns");
    let block = &result.content[0];
    let ContentBlock::Text(content) = block else {
        panic!("expected text content, got {block:?}");
    };
    serde_json::from_str(&content.text).expect("the tool returns one JSON document")
}

/// A temporary directory that cleans itself up.
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "genmsi-mcp-{tag}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("the temporary directory is created");
        Self(path)
    }

    fn file(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The skeleton of a model, as the catalogue publishes it.
///
/// The agent starts from it: it is a model with its root and nothing else.
async fn skeleton(client: &rmcp::service::RunningService<rmcp::RoleClient, ()>) -> String {
    let listed = call(client, "catalogue", json!({})).await;
    assert_eq!(listed["schema"], "genmsi.classlist/1", "{listed}");
    listed["skeleton"]
        .as_str()
        .expect("the skeleton of a model")
        .to_string()
}

/// The place of a block of the model, as the file keeps it.
fn place_of(model: &genmsi_format::GmdlModel, id: u32) -> (i32, i32) {
    let block = model.block(id).expect("the block is in the model");
    let layout = block.layout.expect("the block is placed on the canvas");
    (layout.x, layout.y)
}

/// The flow an agent follows to build a model.
///
/// It asks for the catalogue, takes the skeleton as its starting point and
/// sends a batch: a generator, a terminator, the link between them and the
/// properties the two classes declare. The model that comes out compiles,
/// runs and is on the disk.
#[tokio::test]
async fn a_batch_that_builds_a_model_writes_it() {
    let (client, _server) = pair().await;
    let dir = TempDir::new("edit-build");
    let path = dir.file("modelo.gmdl");
    let skeleton = skeleton(&client).await;
    std::fs::write(&path, &skeleton).expect("the skeleton is on the disk");

    let edited = call(
        &client,
        "apply_edit",
        json!({
            "path": path.to_string_lossy(),
            "commands": [
                { "op": "addBlock", "class": "Generate", "container": 0, "x": 0, "y": 0 },
                { "op": "addBlock", "class": "Terminate", "container": 0, "x": 0, "y": 0 },
                { "op": "connect", "from": 1, "fromIoid": 1, "to": 2, "toIoid": 1 },
                { "op": "setProperty", "id": 1, "property": "Media", "value": "10" },
                { "op": "setProperty", "id": 2, "property": "Unidades", "value": "1" }
            ]
        }),
    )
    .await;

    // The document of the edit says what the batch did.
    assert_eq!(edited["schema"], "genmsi.edit/1", "{edited}");
    assert_eq!(edited["written"], true, "{edited}");
    assert_eq!(edited["applied"], 5, "{edited}");
    assert_eq!(edited["created"], json!([1, 2]), "{edited}");
    assert_eq!(edited["status"], "ok", "{edited}");
    assert_eq!(edited["blocks"], 2, "{edited}");
    assert!(
        edited["source"].is_null(),
        "a model that is on the disk does not travel back as text: {edited}"
    );

    // The file is on the disk and it is the model the batch built.
    assert!(path.exists(), "the model was written");
    let written = std::fs::read_to_string(&path).expect("the model is readable");
    let model = genmsi_format::parse_str(&written).expect("the model that was written parses");
    let classes: Vec<&str> = model
        .blocks
        .iter()
        .filter(|block| !block.is_root())
        .map(|block| block.class.as_str())
        .collect();
    assert_eq!(classes, vec!["Generate", "Terminate"], "{written}");
    let links = &model.block(0).expect("the root").links;
    assert_eq!(links.len(), 1, "the two blocks are connected: {written}");
    assert_eq!(links[0].from_module, 1, "{written}");
    assert_eq!(links[0].from_ioid, 1, "{written}");
    assert_eq!(links[0].to_module, 2, "{written}");
    assert_eq!(links[0].to_ioid, 1, "{written}");
    let media = model
        .block(1)
        .expect("the generator")
        .prop("Media")
        .expect("the Media property");
    assert_eq!(media.raw_value, "10", "{written}");

    // The model that was written is a model the engine accepts: it lints
    // clean and it runs.
    let linted = call(&client, "lint", json!({ "path": path.to_string_lossy() })).await;
    assert_eq!(linted["schema"], "genmsi.lint/1", "{linted}");
    assert_eq!(linted["status"], "ok", "{linted}");
    let simulated = call(&client, "simulate", json!({ "path": path.to_string_lossy() })).await;
    assert_eq!(simulated["schema"], "genmsi.report/1", "{simulated}");
    assert_eq!(simulated["end_reason"], "terminated", "{simulated}");
}

/// A command that fails undoes the batch: nothing of it stays.
///
/// The failure says which command failed, so the agent can correct it and
/// send the batch again.
#[tokio::test]
async fn a_failing_command_leaves_the_file_as_it_was() {
    let (client, _server) = pair().await;
    let dir = TempDir::new("edit-fail");
    let path = dir.file("modelo.gmdl");
    std::fs::write(&path, WORKING_MODEL).expect("the model is on the disk");
    let before = std::fs::read(&path).expect("the model is readable");

    // The first command adds a block, the second connects from a block that
    // does not exist: the batch fails on its second command.
    let failure = call(
        &client,
        "apply_edit",
        json!({
            "path": path.to_string_lossy(),
            "commands": [
                { "op": "addBlock", "class": "Generate", "container": 0, "x": 0, "y": 0 },
                { "op": "connect", "from": 99, "fromIoid": 1, "to": 2, "toIoid": 1 }
            ]
        }),
    )
    .await;

    assert_eq!(failure["error"], "invalid_command", "{failure}");
    assert_eq!(failure["command"], 1, "the command that failed: {failure}");
    assert_eq!(failure["op"], "connect", "the operation that failed: {failure}");
    let message = failure["message"].as_str().expect("a message");
    assert!(
        message.starts_with("command 1 (connect) failed:"),
        "the message names the command: {message}"
    );

    // The file is byte for byte the file that was there.
    let after = std::fs::read(&path).expect("the model is readable");
    assert_eq!(
        after.len(),
        before.len(),
        "the failed batch changed the size of the file"
    );
    assert!(after == before, "the failed batch changed the file");

    // And the batch left no trace in the model either: a batch that comes
    // after it finds the model as it was, without the block the failed
    // batch added.
    let applied = call(
        &client,
        "apply_edit",
        json!({
            "path": path.to_string_lossy(),
            "commands": [
                { "op": "setCaption", "id": 1, "caption": "Entrada" }
            ]
        }),
    )
    .await;
    assert_eq!(applied["written"], true, "{applied}");
    let written = std::fs::read_to_string(&path).expect("the model is readable");
    let model = genmsi_format::parse_str(&written).expect("the model that was written parses");
    let blocks: Vec<u32> = model
        .blocks
        .iter()
        .filter(|block| !block.is_root())
        .map(|block| block.id)
        .collect();
    assert_eq!(
        blocks,
        vec![1, 2],
        "the block of the failed batch is not in the model: {written}"
    );
    assert_eq!(
        model.block(1).expect("the generator").caption,
        "Entrada",
        "{written}"
    );
}

/// A model that does not compile is not written.
///
/// The check comes before the write: the file that was there is still there,
/// byte for byte, so the agent can correct the batch and send it again.
#[tokio::test]
async fn a_model_that_does_not_compile_is_not_written() {
    let (client, _server) = pair().await;
    let dir = TempDir::new("edit-broken");
    let path = dir.file("modelo.gmdl");
    std::fs::write(&path, WORKING_MODEL).expect("the model is on the disk");
    let before = std::fs::read(&path).expect("the model is readable");

    // The batch takes the terminator away: what is left has no TERMINATE
    // and no CLOCK, so the run would never finish.
    let failure = call(
        &client,
        "apply_edit",
        json!({
            "path": path.to_string_lossy(),
            "commands": [
                { "op": "removeBlock", "id": 2 }
            ]
        }),
    )
    .await;

    assert_eq!(failure["error"], "invalid_model", "{failure}");
    let message = failure["message"].as_str().expect("a message");
    assert!(
        message.contains("TERMINATE"),
        "the message of the compiler says what is missing: {message}"
    );

    let after = std::fs::read(&path).expect("the model is readable");
    assert_eq!(
        after.len(),
        before.len(),
        "a model that does not compile is not written"
    );
    assert!(
        after == before,
        "a model that does not compile is not written"
    );
}

/// The place of a block is the engine's business.
///
/// With `auto` (the default) a block lands where there is room in its
/// container, like a drop in the application: the second block of the batch
/// lands below the first. With `explicit` the block lands exactly where the
/// command says.
#[tokio::test]
async fn the_blocks_land_where_there_is_room() {
    let (client, _server) = pair().await;
    let dir = TempDir::new("edit-layout");
    let skeleton = skeleton(&client).await;

    // (a) `auto`: the two blocks of the batch fall one under the other.
    let auto_path = dir.file("auto.gmdl");
    std::fs::write(&auto_path, &skeleton).expect("the skeleton is on the disk");
    let auto = call(
        &client,
        "apply_edit",
        json!({
            "path": auto_path.to_string_lossy(),
            "layout": "auto",
            "commands": [
                { "op": "addBlock", "class": "Generate", "container": 0, "x": 0, "y": 0 },
                { "op": "addBlock", "class": "Terminate", "container": 0, "x": 0, "y": 0 },
                { "op": "setProperty", "id": 2, "property": "Unidades", "value": "1" }
            ]
        }),
    )
    .await;
    assert_eq!(auto["written"], true, "{auto}");
    let written = std::fs::read_to_string(&auto_path).expect("the model is readable");
    let model = genmsi_format::parse_str(&written).expect("the model that was written parses");
    let (first_x, first_y) = place_of(&model, 1);
    let (second_x, second_y) = place_of(&model, 2);
    assert_eq!(
        first_x, second_x,
        "the two blocks fall in the same column: {written}"
    );
    assert!(
        second_y > first_y,
        "the block that came later falls below the one that came first: \
         {first_y} then {second_y}"
    );
    // The coordinates the command carried are ignored: the block did not
    // land on them.
    assert_ne!(
        (first_x, first_y),
        (0, 0),
        "the command does not choose the place: {written}"
    );

    // (b) `explicit`: the block lands exactly where the command says.
    let explicit_path = dir.file("explicit.gmdl");
    std::fs::write(&explicit_path, &skeleton).expect("the skeleton is on the disk");
    let explicit = call(
        &client,
        "apply_edit",
        json!({
            "path": explicit_path.to_string_lossy(),
            "layout": "explicit",
            "commands": [
                { "op": "addBlock", "class": "Generate", "container": 0, "x": 100, "y": 200 },
                { "op": "addBlock", "class": "Terminate", "container": 0, "x": 300, "y": 400 },
                { "op": "setProperty", "id": 2, "property": "Unidades", "value": "1" }
            ]
        }),
    )
    .await;
    assert_eq!(explicit["written"], true, "{explicit}");
    let written = std::fs::read_to_string(&explicit_path).expect("the model is readable");
    let model = genmsi_format::parse_str(&written).expect("the model that was written parses");
    assert_eq!(
        place_of(&model, 1),
        (100, 200),
        "the generator lands where the command said: {written}"
    );
    assert_eq!(
        place_of(&model, 2),
        (300, 400),
        "the terminator lands where the command said: {written}"
    );
}

/// A model that is given as text comes back as text.
///
/// There is no file to write it to, so the text of the result is the only
/// place the edit lives: it travels back with the answer.
#[tokio::test]
async fn an_inline_model_comes_back_as_text() {
    let (client, _server) = pair().await;
    let skeleton = skeleton(&client).await;

    let edited = call(
        &client,
        "apply_edit",
        json!({
            "model": skeleton,
            "commands": [
                { "op": "addBlock", "class": "Generate", "container": 0, "x": 0, "y": 0 },
                { "op": "addBlock", "class": "Terminate", "container": 0, "x": 0, "y": 0 },
                { "op": "connect", "from": 1, "fromIoid": 1, "to": 2, "toIoid": 1 },
                { "op": "setProperty", "id": 2, "property": "Unidades", "value": "1" }
            ]
        }),
    )
    .await;

    assert_eq!(edited["schema"], "genmsi.edit/1", "{edited}");
    assert_eq!(edited["written"], false, "a model of text is not written: {edited}");
    assert_eq!(edited["applied"], 4, "{edited}");
    assert_eq!(edited["created"], json!([1, 2]), "{edited}");
    assert_eq!(edited["path"], "<memory>", "a model without a file: {edited}");

    let source = edited["source"].as_str().expect("the text of the model");
    assert!(source.contains("<gmodel"), "{source}");
    let model = genmsi_format::parse_str(source).expect("the text that came back parses");
    let classes: Vec<&str> = model
        .blocks
        .iter()
        .filter(|block| !block.is_root())
        .map(|block| block.class.as_str())
        .collect();
    assert_eq!(classes, vec!["Generate", "Terminate"], "{source}");
    assert_eq!(
        model.block(0).expect("the root").links.len(),
        1,
        "the two blocks are connected: {source}"
    );
}
