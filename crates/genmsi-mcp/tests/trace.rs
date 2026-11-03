//! The trace the agent can filter, the summary of a run and the map of the
//! statements.
//!
//! The trace of a run is a hose: a model of five hundred blocks makes
//! thousands of events, and the agent that wants the events of one class has
//! to dump them all and filter them on its side. The engine knows the class,
//! the block of the document and the time of every event as it records it,
//! so the filter belongs there: the filter is applied **when the event is
//! recorded**, and the limit of the trace counts the events that pass it.
//!
//! The summary answers the question the agent asks the ninety percent of the
//! time ("how many times did the flow go through here?"): how many entries a
//! block had, and the first and the last time it had them.
//!
//! The map joins the statements of the compiled program with the blocks of
//! the document, so "what block is the statement 218" has an answer of one
//! line.

use genmsi_mcp::GenmsiServer;
use rmcp::model::{CallToolRequestParams, ContentBlock};
use rmcp::ServiceExt;
use serde_json::{json, Value};

/// A model with a generator, a queue, an assignment and a termination.
///
/// The four modules are the ones of the document: the generator is the
/// module `1`, the queue the module `2`, the assignment the module `3` and
/// the termination the module `4`. The clock of the model is the module `5`.
/// Every module is on a line of its own, so the position of every block in
/// the file is known.
const MODEL: &str = r#"<gmodel versioncode="1"><gclasses/><modules>
<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C"><links><link frommoduleid="1" fromioid="1" tomoduleid="2" toioid="1"/><link frommoduleid="2" fromioid="2" tomoduleid="3" toioid="1"/><link frommoduleid="3" fromioid="2" tomoduleid="4" toioid="1"/></links><inputs/><outputs/></module>
<module moduleid="1" moduleclass="Generate" owner="0" caption="g" name="g" kind="A" width="120" height="63" x="0" y="0"><inputs/><outputs><output ioid="1" name="Salida1" caption="" mylabel="Q1"/></outputs><props><property name="Media" hint="" value="10" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Cantidad" hint="" value="4" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="2" moduleclass="Queue" owner="0" caption="q" name="q" kind="A" width="97" height="32" x="0" y="0"><inputs><input ioid="1" name="Entrada1" caption="" mylabel="Q1"/></inputs><outputs><output ioid="2" name="Salida1" caption="" mylabel="A1"/></outputs><props><property name="Queue" hint="" value="cola" type="EntityUse" subType="Queue" visible="1" unique="0"/></props></module>
<module moduleid="3" moduleclass="Assign" owner="0" caption="a" name="a" kind="A" width="120" height="63" x="0" y="0"><inputs><input ioid="1" name="Entrada1" caption="" mylabel="A1"/></inputs><outputs><output ioid="2" name="Salida1" caption="" mylabel="T1"/></outputs><props><property name="Parametro" hint="" value="p" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Valor" hint="" value="7" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="4" moduleclass="Terminate" owner="0" caption="t" name="t" kind="A" width="64" height="64" x="0" y="0"><inputs><input ioid="1" name="Entrada1" caption="" mylabel="T1"/></inputs><outputs/><props><property name="Unidades" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="5" moduleclass="Clock" owner="0" caption="c" name="c" kind="A" width="97" height="32" x="0" y="0"><inputs/><outputs/><props><property name="Frecuencia" hint="" value="1000" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
</modules></gmodel>"#;

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

/// The trace of the model, with the arguments the test asks for.
async fn trace(
    client: &rmcp::service::RunningService<rmcp::RoleClient, ()>,
    arguments: Value,
) -> Value {
    let mut map = match arguments {
        Value::Object(map) => map,
        other => panic!("arguments must be an object, got {other}"),
    };
    map.insert("model".to_string(), json!(MODEL));
    map.insert("seed".to_string(), json!(1));
    map.insert("start".to_string(), json!(4));
    map.insert("until".to_string(), json!(1000.0));
    call(client, "get_trace", Value::Object(map)).await
}

/// The events of a trace document.
fn events(document: &Value) -> Vec<&Value> {
    document["events"]
        .as_array()
        .expect("the events of the trace")
        .iter()
        .collect()
}

/// The class of every event of a trace document.
fn classes(document: &Value) -> Vec<String> {
    events(document)
        .iter()
        .map(|event| event["class"].as_str().expect("a class").to_string())
        .collect()
}

#[tokio::test]
async fn the_trace_can_be_filtered_by_class() {
    let (client, _server) = pair().await;
    let document = trace(&client, json!({ "classes": ["Queue"] })).await;
    assert_eq!(document["schema"], "genmsi.trace/1", "{document}");
    let classes = classes(&document);
    assert!(!classes.is_empty(), "the filter keeps the queue events: {document}");
    assert!(
        classes.iter().all(|class| class == "Queue"),
        "the filter keeps only the events of the class: {classes:?}"
    );
    // The document says what was filtered, so the agent knows what it is not
    // seeing.
    assert_eq!(document["filter"]["classes"], json!(["Queue"]), "{document}");
    assert!(document["filter"]["nodes"].as_array().expect("the nodes").is_empty());
}

#[tokio::test]
async fn the_trace_can_be_filtered_by_node() {
    let (client, _server) = pair().await;
    // The queue is the module `2` of the document.
    let document = trace(&client, json!({ "nodes": [2] })).await;
    assert_eq!(document["schema"], "genmsi.trace/1", "{document}");
    let events = events(&document);
    assert!(!events.is_empty(), "the filter keeps the events of the block");
    assert!(
        events
            .iter()
            .all(|event| event["node"] == json!(2)),
        "the filter keeps only the events of the block: {events:?}"
    );
    assert!(
        classes(&document).iter().all(|class| class == "Queue"),
        "the block of the document is the queue"
    );
}

#[tokio::test]
async fn the_trace_can_be_filtered_by_transaction() {
    let (client, _server) = pair().await;
    // The transaction of the first event of the queue.
    let whole = trace(&client, json!({})).await;
    let xact = events(&whole)
        .iter()
        .find(|event| event["class"] == json!("Queue"))
        .expect("the run passes through the queue")["xact"]
        .clone();
    let document = trace(&client, json!({ "xact": xact })).await;
    let events = events(&document);
    assert!(
        !events.is_empty(),
        "the filter keeps the events of the transaction: {document}"
    );
    assert!(
        events.iter().all(|event| event["xact"] == xact),
        "the filter keeps only the events of the transaction: {events:?}"
    );
}

#[tokio::test]
async fn the_trace_can_be_filtered_by_window_of_time() {
    let (client, _server) = pair().await;
    let document = trace(&client, json!({ "from": 20.0, "to": 30.0 })).await;
    let events = events(&document);
    assert!(
        !events.is_empty(),
        "the filter keeps the events of the window: {document}"
    );
    assert!(
        events.iter().all(|event| {
            let time = event["time"].as_f64().expect("a time");
            (20.0..=30.0).contains(&time)
        }),
        "the filter keeps only the events of the window: {events:?}"
    );
}

#[tokio::test]
async fn the_limit_counts_the_events_that_pass_the_filter() {
    let (client, _server) = pair().await;
    // The first events of the run are the ones of the generator: a limit of
    // two records must keep the first two *terminations*, and not the first
    // two events of the run.
    let document = trace(&client, json!({ "classes": ["Terminate"], "limit": 2 })).await;
    let events = events(&document);
    assert_eq!(events.len(), 2, "the limit counts the events that pass");
    assert!(
        classes(&document).iter().all(|class| class == "Terminate"),
        "the events that pass are the ones that were asked for: {document}"
    );
    let whole = trace(&client, json!({ "limit": 2 })).await;
    assert!(
        classes(&whole).iter().all(|class| class != "Terminate"),
        "the first events of the run are not the ones that were asked for: {whole}"
    );
}

#[tokio::test]
async fn a_filter_that_keeps_everything_is_the_same_as_no_filter() {
    let (client, _server) = pair().await;
    let without = trace(&client, json!({})).await;
    let with = trace(
        &client,
        json!({ "classes": [], "nodes": [], "xact": null, "from": null, "to": null }),
    )
    .await;
    assert_eq!(
        without, with,
        "a filter that keeps everything keeps the same trace"
    );
}

#[tokio::test]
async fn the_summary_answers_the_summary_document() {
    let (client, _server) = pair().await;
    let document = trace(&client, json!({ "summary": true })).await;
    assert_eq!(
        document["schema"], "genmsi.trace_summary/1",
        "{document}"
    );
    assert!(document["title"].is_string(), "{document}");
    assert!(document["end_reason"].is_string(), "{document}");

    let blocks = document["blocks"].as_array().expect("the blocks");
    let queue = blocks
        .iter()
        .find(|block| block["class"] == json!("Queue"))
        .expect("the summary names the queue");
    assert_eq!(queue["entries"], json!(4), "the four transactions pass: {document}");
    assert_eq!(queue["first"], json!(10.0), "the first one arrives at 10");
    assert_eq!(queue["last"], json!(40.0), "the last one arrives at 40");
    assert_eq!(queue["node"], json!(2), "the queue is the module 2");

    // The clock of the model never ticks: the run ends before it. Its
    // module is the `5`, and it compiles to two statements.
    let clock = blocks
        .iter()
        .find(|block| block["node"] == json!(5))
        .expect("the summary names the clock");
    assert_eq!(clock["entries"], json!(0), "nobody entered the clock");
    assert_eq!(clock["first"], json!(0.0), "a block nobody entered has no first time");
    assert_eq!(clock["last"], json!(0.0), "a block nobody entered has no last time");
}

#[tokio::test]
async fn the_summary_is_the_summary_of_the_whole_run() {
    let (client, _server) = pair().await;
    // The filters cut the events, and not the summary: the summary is the
    // summary of the run.
    let whole = trace(&client, json!({ "summary": true })).await;
    let filtered = trace(&client, json!({ "summary": true, "classes": ["Queue"] })).await;
    assert_eq!(
        whole["blocks"], filtered["blocks"],
        "the summary does not change with the filter"
    );
}

#[tokio::test]
async fn the_explain_document_publishes_the_map_of_the_statements() {
    let (client, _server) = pair().await;
    let document = call(&client, "explain_model", json!({ "model": MODEL })).await;
    assert_eq!(document["schema"], "genmsi.explain/1", "{document}");

    let map = document["map"].as_array().expect("the map");
    let statements = document["program"]["blocks"]
        .as_array()
        .expect("the blocks of the program");
    assert_eq!(
        map.len(),
        statements.len(),
        "the map has one row per statement: {document}"
    );

    // One row per statement, in the order of the program.
    for (index, row) in map.iter().enumerate() {
        assert_eq!(row["statement"], json!(index), "{row}");
        assert_eq!(
            row["node"], statements[index]["node"],
            "the row names the block of the document: {row}"
        );
        assert_eq!(
            row["class"], statements[index]["class"],
            "the row names the class of the statement: {row}"
        );
        assert_eq!(
            row["line"], statements[index]["line"],
            "the row names the line of the block: {row}"
        );
        assert_eq!(
            row["column"], statements[index]["column"],
            "the row names the column of the block: {row}"
        );
    }

    // The generator is the third line of the file, and the queue the fourth.
    let generator = &map[0];
    assert_eq!(generator["node"], json!(1), "{generator}");
    assert_eq!(generator["class"], json!("Generate"), "{generator}");
    assert_eq!(generator["caption"], json!("g"), "{generator}");
    assert_eq!(generator["line"], json!(3), "{generator}");
    assert_eq!(generator["column"], json!(1), "{generator}");

    let queue = map
        .iter()
        .find(|row| row["class"] == json!("Queue"))
        .expect("the map names the queue");
    assert_eq!(queue["node"], json!(2), "{queue}");
    assert_eq!(queue["caption"], json!("q"), "{queue}");
    assert_eq!(queue["line"], json!(4), "{queue}");
}

#[tokio::test]
async fn the_node_of_an_event_and_the_map_say_the_same_thing() {
    let (client, _server) = pair().await;
    // The question the map exists for: "what block is the statement 218?".
    let explained = call(&client, "explain_model", json!({ "model": MODEL })).await;
    let map = explained["map"].as_array().expect("the map");
    let traced = trace(&client, json!({})).await;
    for event in events(&traced) {
        let node = event["node"].as_u64().expect("the node of the event");
        let row = map
            .iter()
            .find(|row| row["node"] == json!(node))
            .expect("the map names the block of the event");
        assert_eq!(
            row["class"], event["class"],
            "the event and the map speak of the same statement: {event} {row}"
        );
    }
}
