//! The catalogue tool: what a client reads before it writes a model.
//!
//! The test below is the one that matters: an agent asks for the catalogue,
//! asks for a class, writes a model with the template it got, lints it and
//! runs it. A model built that way is a model the engine accepts, without
//! the agent guessing an element name or an attribute.

use rmcp::model::{CallToolRequestParams, ContentBlock};
use rmcp::ServiceExt;
use serde_json::{json, Value};

use genmsi_mcp::GenmsiServer;

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

/// The block of a class, with the id the model gives it.
///
/// The template carries `moduleid="0"`, the id of the model root: whoever
/// drops the block into a model renumbers it.
fn block(template: &str, id: u32) -> String {
    template.replace("moduleid=\"0\"", &format!("moduleid=\"{id}\""))
}

/// A model the agent writes: the skeleton, the blocks and the links.
///
/// The links live in the `<links>` section of the model root, and they name
/// the two ends of the connection: the module and the port it leaves from,
/// and the module and the port it arrives at.
fn model_of(skeleton: &str, blocks: &[(u32, &str)], links: &[(u32, u32, u32, u32)]) -> String {
    let modules: String = blocks
        .iter()
        .map(|(id, template)| block(template, *id))
        .collect();
    let links: String = links
        .iter()
        .map(|(from, from_ioid, to, to_ioid)| {
            format!(
                "<link frommoduleid=\"{from}\" fromioid=\"{from_ioid}\" \
                 tomoduleid=\"{to}\" toioid=\"{to_ioid}\"/>"
            )
        })
        .collect();
    let with_links = skeleton.replace("<links/>", &format!("<links>{links}</links>"));
    with_links.replace("</modules>", &format!("{modules}</modules>"))
}

#[tokio::test]
async fn the_catalogue_lists_the_classes_and_the_skeleton() {
    let (client, _server) = pair().await;
    let listed = call(&client, "catalogue", json!({})).await;
    assert_eq!(listed["schema"], "genmsi.classlist/1", "{listed}");
    let classes = listed["classes"].as_array().expect("the classes");
    assert_eq!(classes.len(), 66, "the classes the editor offers");
    let generate = classes
        .iter()
        .find(|class| class["class"] == "Generate")
        .expect("the Generate class");
    assert_eq!(generate["title"], "Generar");
    let skeleton = listed["skeleton"].as_str().expect("the skeleton");
    assert!(skeleton.contains("<gmodel"), "{skeleton}");
    assert!(skeleton.contains("moduleclass=\"main\""), "{skeleton}");
}

#[tokio::test]
async fn a_class_carries_its_ports_its_properties_and_its_template() {
    let (client, _server) = pair().await;
    let class = call(&client, "catalogue", json!({ "class": "Generate" })).await;
    assert_eq!(class["schema"], "genmsi.class/1", "{class}");
    assert_eq!(class["class"], "Generate");
    assert_eq!(class["title"], "Generar");
    let outputs = class["outputs"].as_array().expect("the outputs");
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0]["name"], "Salida1");
    let properties = class["properties"].as_array().expect("the properties");
    let media = properties
        .iter()
        .find(|property| property["name"] == "Media")
        .expect("the Media property");
    assert_eq!(media["ty"], "TSNA");
    let template = class["template"].as_str().expect("the template");
    assert!(template.starts_with("<module moduleid=\"0\""), "{template}");
    assert!(template.contains("moduleclass=\"Generate\""), "{template}");
    assert!(template.contains("value=\"60\""), "{template}");
}

#[tokio::test]
async fn a_class_that_does_not_exist_is_its_own_failure() {
    let (client, _server) = pair().await;
    let error = call(&client, "catalogue", json!({ "class": "NotAClass" })).await;
    assert_eq!(error["error"], "class_not_found", "{error}");
    assert_eq!(error["class"], "NotAClass", "{error}");
}

/// The test that matters: an agent builds a model with the template.
///
/// It asks for the catalogue, asks for a class, writes the model out of the
/// skeleton and the two templates, lints it and runs it. Nothing here knows
/// the shape of the file: everything comes from the tool.
#[tokio::test]
async fn an_agent_builds_a_model_with_the_templates() {
    let (client, _server) = pair().await;

    // (a) The catalogue: the classes and the skeleton of a model.
    let listed = call(&client, "catalogue", json!({})).await;
    assert_eq!(listed["schema"], "genmsi.classlist/1");
    let skeleton = listed["skeleton"].as_str().expect("the skeleton").to_string();

    // (b) The class the agent wants: the generator.
    let generate = call(&client, "catalogue", json!({ "class": "Generate" })).await;
    assert_eq!(generate["schema"], "genmsi.class/1");
    let generate_template = generate["template"].as_str().expect("the template").to_string();

    // (b) And the class every model ends with.
    let terminate = call(&client, "catalogue", json!({ "class": "Terminate" })).await;
    assert_eq!(terminate["schema"], "genmsi.class/1");
    let terminate_template = terminate["template"].as_str().expect("the template").to_string();

    // (c) The model: the skeleton, the generator, the terminator and the
    // link between them. The terminator takes one unit from the `START` of
    // the run, so the run ends with the first transaction. The class carries
    // no default for that operand, so the agent fills it in.
    let terminate_template = terminate_template.replace("value=\"\"", "value=\"1\"");
    let model = model_of(
        &skeleton,
        &[(1, &generate_template), (2, &terminate_template)],
        &[(1, 1, 2, 1)],
    );

    // (d) The lint: the model compiles.
    let linted = call(&client, "lint", json!({ "model": model })).await;
    assert_eq!(linted["schema"], "genmsi.lint/1", "{linted}");
    assert_eq!(linted["status"], "ok", "{linted}");
    assert_eq!(linted["blocks"], 2, "{linted}");

    // (e) The run: the model the agent wrote is a model the engine runs.
    let simulated = call(&client, "simulate", json!({ "model": model, "seed": 7 })).await;
    assert_eq!(simulated["schema"], "genmsi.report/1", "{simulated}");
    assert_eq!(simulated["end_reason"], "terminated", "{simulated}");
    assert_eq!(simulated["transactions"], 1, "{simulated}");
    let terminated = simulated["blocks"]
        .as_array()
        .expect("the blocks of the report")
        .iter()
        .find(|block| block["class"] == "Terminate")
        .expect("the terminator of the report");
    assert_eq!(terminated["entries"], 1, "{simulated}");
}
