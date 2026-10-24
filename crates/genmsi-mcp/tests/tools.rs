//! The tools over the wire: an MCP client drives the server and every
//! reference model goes through the tools.
//!
//! The server is started in process over a duplex transport, so the test
//! speaks the same protocol an agent speaks: `tools/list`, then `tools/call`.

use std::path::{Path, PathBuf};

use genmsi_mcp::GenmsiServer;
use rmcp::model::{CallToolRequestParams, ContentBlock};
use rmcp::ServiceExt;
use serde_json::{json, Value};

/// The five models the original refuses, with the class of the engine message
/// each one must produce (the same table `docs/cli.md` publishes).
const REFUSED: [(&str, &str); 5] = [
    ("Parcial 8-11-21__ParcialMikelModelos", "invalid block"),
    ("Problema 2__puenteLevadizo", "no TERMINATE block"),
    ("TP6__Clase 6_0_TestGate", "no TERMINATE block"),
    ("TP9__hack", "no TERMINATE block"),
    ("TP9__tp9", "the operand B must not be null"),
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn models() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(repo_root().join("fixtures/models"))
        .expect("fixtures/models")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "gmdl"))
        .collect();
    paths.sort();
    paths
}

/// The path of a reference model, as the agent would name it.
///
/// The path is absolute: the test runs from the crate directory, and an agent
/// may name a model anywhere.
fn model_path(stem: &str) -> String {
    repo_root()
        .join("fixtures/models")
        .join(format!("{stem}.gmdl"))
        .display()
        .to_string()
}

/// A client and a server that talk over a duplex transport.
///
/// The server runs in its own task: it waits for the client's handshake, so
/// it cannot be started before the client is.
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

#[tokio::test]
async fn the_server_offers_the_four_tools() {
    let (client, _server) = pair().await;
    let listed = client.list_tools(None).await.expect("tools are listed");
    let mut names: Vec<String> = listed
        .tools
        .iter()
        .map(|tool| tool.name.to_string())
        .collect();
    names.sort();
    assert_eq!(
        names,
        ["explain_model", "get_trace", "lint", "simulate"],
        "the four tools of the roadmap, and nothing else"
    );
    // Every tool tells the agent what it takes and what it returns.
    for tool in &listed.tools {
        assert!(tool.description.is_some(), "{} has no description", tool.name);
        assert_eq!(
            tool.input_schema.get("type").and_then(|t| t.as_str()),
            Some("object"),
            "{} has no object schema",
            tool.name
        );
    }
}

#[tokio::test]
async fn every_reference_model_reaches_the_tools() {
    let (client, _server) = pair().await;
    let mut accepted = 0;
    let mut refused = 0;
    for path in models() {
        let stem = path.file_stem().expect("stem").to_string_lossy().to_string();
        let name = model_path(&stem);
        let linted = call(&client, "lint", json!({ "path": name })).await;
        match REFUSED.iter().find(|(model, _)| *model == stem) {
            Some((_, expected)) => {
                refused += 1;
                assert_eq!(
                    linted["error"], "invalid_model",
                    "{stem}: the engine refuses it"
                );
                let message = linted["message"].as_str().expect("a message");
                assert!(
                    message.contains(expected),
                    "{stem}: {message} does not name the class {expected}"
                );
                // The failure points at a block of the model.
                assert!(linted["node"].is_number(), "{stem}: no node");
                assert!(linted["line"].is_number(), "{stem}: no line");
                assert!(linted["column"].is_number(), "{stem}: no column");
            }
            None => {
                accepted += 1;
                assert_eq!(linted["status"], "ok", "{stem}: {linted}");
                let simulated = call(&client, "simulate", json!({ "path": name })).await;
                assert_eq!(
                    simulated["schema"], "genmsi.report/1",
                    "{stem}: the report document"
                );
                assert!(
                    simulated["end_reason"].is_string(),
                    "{stem}: the run did not end"
                );
            }
        }
    }
    assert_eq!(accepted, 20, "the models the original accepts");
    assert_eq!(refused, 5, "the models the original refuses");
}

#[tokio::test]
async fn an_inline_model_needs_no_file() {
    let (client, _server) = pair().await;
    let document = "<gmodel versioncode=\"1\"><gclasses/><modules>\
        <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"m\" name=\"m\" kind=\"C\"><links>\
        <link frommoduleid=\"1\" fromioid=\"1\" tomoduleid=\"2\" toioid=\"1\"/>\
        </links><inputs/><outputs/></module>\
        <module moduleid=\"1\" moduleclass=\"Generate\" owner=\"0\" caption=\"g\" name=\"g\" kind=\"A\"><outputs><output ioid=\"1\" name=\"Salida1\" caption=\"\" mylabel=\"L1\"/></outputs>\
        <props><property name=\"Media\" hint=\"\" value=\"10\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/></props></module>\
        <module moduleid=\"2\" moduleclass=\"Terminate\" owner=\"0\" caption=\"t\" name=\"t\" kind=\"A\"><inputs><input ioid=\"1\" name=\"i\" caption=\"\" mylabel=\"L1\"/></inputs>\
        <props><property name=\"Unidades\" hint=\"\" value=\"1\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/></props></module>\
        </modules></gmodel>";
    let arguments = json!({ "model": document });

    let linted = call(&client, "lint", arguments.clone()).await;
    assert_eq!(linted["schema"], "genmsi.lint/1");
    assert_eq!(linted["status"], "ok");
    assert_eq!(linted["path"], "<memory>", "a model without a file");

    let simulated = call(&client, "simulate", arguments.clone()).await;
    assert_eq!(simulated["schema"], "genmsi.report/1");
    assert_eq!(simulated["end_reason"], "terminated");

    let traced = call(&client, "get_trace", arguments.clone()).await;
    assert_eq!(traced["schema"], "genmsi.trace/1");
    let events: Vec<&str> = traced["events"]
        .as_array()
        .expect("the events")
        .iter()
        .map(|event| event["event"].as_str().expect("an event name"))
        .collect();
    assert!(
        events.contains(&"generate") && events.contains(&"terminate"),
        "the trace holds the events of the run: {events:?}"
    );

    let explained = call(&client, "explain_model", arguments).await;
    assert_eq!(explained["schema"], "genmsi.explain/1");
    assert!(
        explained["glossary"]["Generate"].is_string(),
        "the glossary names the statements: {explained}"
    );
}

#[tokio::test]
async fn el_start_de_la_corrida_llega_al_motor() {
    let (client, _server) = pair().await;
    // A generator of one transaction every ten units and a terminate that
    // takes one unit from the `START` of the run.
    let document = "<gmodel versioncode=\"1\"><gclasses/><modules>\
        <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"m\" name=\"m\" kind=\"C\"><links>\
        <link frommoduleid=\"1\" fromioid=\"1\" tomoduleid=\"2\" toioid=\"1\"/>\
        </links><inputs/><outputs/></module>\
        <module moduleid=\"1\" moduleclass=\"Generate\" owner=\"0\" caption=\"g\" name=\"g\" kind=\"A\"><outputs><output ioid=\"1\" name=\"Salida1\" caption=\"\" mylabel=\"L1\"/></outputs>\
        <props><property name=\"Media\" hint=\"\" value=\"10\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/></props></module>\
        <module moduleid=\"2\" moduleclass=\"Terminate\" owner=\"0\" caption=\"t\" name=\"t\" kind=\"A\"><inputs><input ioid=\"1\" name=\"i\" caption=\"\" mylabel=\"L1\"/></inputs>\
        <props><property name=\"Unidades\" hint=\"\" value=\"1\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/></props></module>\
        </modules></gmodel>";

    // The default of the original: the run ends with the first transaction.
    let one = call(&client, "simulate", json!({ "model": document })).await;
    assert_eq!(one["end_reason"], "terminated");
    assert_eq!(one["end_time"], 10.0);

    // A `START` of three: the run ends with the third transaction.
    let three = call(&client, "simulate", json!({ "model": document, "start": 3 })).await;
    assert_eq!(three["end_reason"], "terminated");
    assert_eq!(three["end_time"], 30.0);

    // The original refuses a `Start` of zero ("El Start debe ser mayor a
    // cero."), and so does the server.
    let zero = call(&client, "simulate", json!({ "model": document, "start": 0 })).await;
    assert_eq!(zero["error"], "invalid_arguments");
    assert!(
        zero["message"]
            .as_str()
            .unwrap_or_default()
            .contains("the start must be greater than zero"),
        "{zero}"
    );
}

#[tokio::test]
async fn the_lint_says_the_warning_of_the_format() {
    let (client, _server) = pair().await;
    // A model that uses a storage no block declares: the original opens it, so
    // the lint says what it found instead of refusing the model.
    let document = "<gmodel versioncode=\"1\"><gclasses/><modules>\
        <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"m\" name=\"m\" kind=\"C\"><links/><inputs/><outputs/></module>\
        <module moduleid=\"1\" moduleclass=\"StorageUse\" owner=\"0\" caption=\"c\" name=\"n\" kind=\"A\">\
        <props>\
        <property name=\"Media\" hint=\"\" value=\"10\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/>\
        <property name=\"Storage\" hint=\"\" value=\"Almacen\" type=\"EntityUse\" subType=\"Storage\" visible=\"1\" unique=\"0\"/>\
        </props></module>\
        <module moduleid=\"2\" moduleclass=\"Terminate\" owner=\"0\" caption=\"t\" name=\"t\" kind=\"A\">\
        <props><property name=\"Unidades\" hint=\"\" value=\"1\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/></props></module>\
        </modules></gmodel>";

    let linted = call(&client, "lint", json!({ "model": document })).await;
    assert_eq!(linted["schema"], "genmsi.lint/1");
    assert_eq!(linted["status"], "diagnostics", "{linted}");
    let diagnostics = linted["diagnostics"]
        .as_array()
        .expect("an array of diagnostics");
    // One finding, one text: the compiler reports the storage on its own, and
    // the format's warning about the same storage is not said again.
    assert_eq!(diagnostics.len(), 1, "{linted}");
    assert!(
        diagnostics[0]
            .as_str()
            .expect("a message")
            .contains("Almacen"),
        "{linted}"
    );
}

#[tokio::test]
async fn a_refused_model_points_at_the_block_to_fix() {
    let (client, _server) = pair().await;
    let linted = call(
        &client,
        "lint",
        json!({ "path": model_path("TP9__tp9") }),
    )
    .await;
    assert_eq!(linted["error"], "invalid_model");
    assert_eq!(linted["node"], 68);
    assert_eq!(linted["class"], "Unlink");
    assert_eq!(linted["line"], 1, "the reference models are one long line");
    assert!(
        linted["column"].as_u64().expect("a column") > 0,
        "the column points at the block: {linted}"
    );
}

#[tokio::test]
async fn a_call_without_a_model_is_an_argument_error() {
    let (client, _server) = pair().await;
    let result = call(&client, "lint", json!({})).await;
    assert_eq!(result["error"], "invalid_arguments");
}
