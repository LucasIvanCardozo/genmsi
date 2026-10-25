//! The tool that writes a model: what it writes and what it says about it.
//!
//! The canvas geometry is the engine's business (spec S6): a model an agent
//! builds carries no geometry, so the engine composes it before it is written
//! and the editor can draw it. A model that already carries its geometry is
//! written exactly as its author left it, and the lint document does not
//! mention the composition.

use std::path::{Path, PathBuf};

use genmsi_mcp::GenmsiServer;
use rmcp::model::{CallToolRequestParams, ContentBlock};
use rmcp::ServiceExt;
use serde_json::{json, Value};

/// The path of a reference model, as the agent would name it.
fn model_path(stem: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/models")
        .join(format!("{stem}.gmdl"))
        .display()
        .to_string()
}

/// A model of a generator and a terminate, with no canvas geometry.
///
/// The catalogue templates carry no geometry, so this is what a model an
/// agent builds looks like.
const WITHOUT_GEOMETRY: &str = "<gmodel versioncode=\"1\"><gclasses/><modules>\
<module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"Modelo\" name=\"Modelo1\" kind=\"C\" genprefer=\"\" exehead=\"\" nonexehead=\"\" exefoot=\"\" nonexefoot=\"\">\
<links><link frommoduleid=\"1\" fromioid=\"1\" tomoduleid=\"2\" toioid=\"1\"/></links><inputs/><outputs/></module>\
<module moduleid=\"1\" moduleclass=\"Generate\" owner=\"0\" caption=\"Generate\" name=\"Generate\" kind=\"A\">\
<outputs><output ioid=\"1\" name=\"Salida1\" caption=\"\" mylabel=\"L1\"/></outputs>\
<props><property name=\"Media\" hint=\"\" value=\"10\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/></props></module>\
<module moduleid=\"2\" moduleclass=\"Terminate\" owner=\"0\" caption=\"Terminate\" name=\"Terminate\" kind=\"A\">\
<inputs><input ioid=\"1\" name=\"Entrada1\" caption=\"\" mylabel=\"L1\"/></inputs>\
<props><property name=\"Unidades\" hint=\"\" value=\"1\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/></props></module>\
</modules></gmodel>";

/// The same model, with the geometry its author placed.
///
/// The text is written the way the writer writes it, so the file that comes
/// back is the file that went in.
const WITH_GEOMETRY: &str = "<gmodel versioncode=\"1\"><gclasses/><modules>\
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

/// The diagnostics of a lint document.
fn diagnostics(document: &Value) -> Vec<String> {
    document["diagnostics"]
        .as_array()
        .expect("an array of diagnostics")
        .iter()
        .map(|message| message.as_str().expect("a message").to_string())
        .collect()
}

#[tokio::test]
async fn a_model_without_geometry_is_composed_before_it_is_written() {
    let (client, _server) = pair().await;
    let dir = TempDir::new("compose");
    let path = dir.file("modelo.gmdl");

    let lint = call(
        &client,
        "save_model",
        json!({ "path": path.to_string_lossy(), "model": WITHOUT_GEOMETRY }),
    )
    .await;
    assert_eq!(lint["schema"], "genmsi.lint/1", "{lint}");

    // The file that was written carries the geometry of every block the
    // editor draws: the root is not drawn, so it keeps no geometry.
    let written = std::fs::read_to_string(&path).expect("the model was written");
    let model = genmsi_format::parse_str(&written).expect("the model that was written parses");
    let placed: Vec<u32> = model
        .blocks
        .iter()
        .filter(|block| !block.is_root())
        .map(|block| block.id)
        .collect();
    assert_eq!(placed.len(), 2, "the model holds two blocks to draw");
    for block in model.blocks.iter().filter(|block| !block.is_root()) {
        let layout = block.layout.expect("the block is placed on the canvas");
        assert!(layout.width > 0, "block {} has no width", block.id);
        assert!(layout.height > 0, "block {} has no height", block.id);
    }
    assert!(
        model.root().expect("the root").layout.is_none(),
        "the root is not drawn, so it keeps no geometry"
    );

    // The lint document says what the engine did.
    let findings = diagnostics(&lint);
    assert!(
        findings
            .iter()
            .any(|message| message.contains("the engine composed 2 blocks")),
        "{lint}"
    );
    assert_eq!(lint["status"], "diagnostics", "{lint}");

    // The geometry does not enter the simulation: the composed model runs
    // like the model the agent sent.
    let inline = call(
        &client,
        "simulate",
        json!({ "model": WITHOUT_GEOMETRY, "seed": 1 }),
    )
    .await;
    let composed_run = call(
        &client,
        "simulate",
        json!({ "path": path.to_string_lossy(), "seed": 1 }),
    )
    .await;
    assert_eq!(inline["end_time"], composed_run["end_time"], "{composed_run}");
    assert_eq!(
        inline["end_reason"], composed_run["end_reason"],
        "{composed_run}"
    );
}

#[tokio::test]
async fn a_model_that_carries_its_geometry_is_written_unchanged() {
    let (client, _server) = pair().await;
    let dir = TempDir::new("compose-kept");
    let path = dir.file("modelo.gmdl");

    let lint = call(
        &client,
        "save_model",
        json!({ "path": path.to_string_lossy(), "model": WITH_GEOMETRY }),
    )
    .await;
    assert_eq!(lint["schema"], "genmsi.lint/1", "{lint}");

    // The model is written exactly as it was sent: the composition never
    // moves a model whose author placed it.
    let written = std::fs::read_to_string(&path).expect("the model was written");
    assert_eq!(written, WITH_GEOMETRY, "the model was rewritten");

    // The lint document does not mention the composition.
    let findings = diagnostics(&lint);
    assert!(
        !findings.iter().any(|message| message.contains("composed")),
        "{lint}"
    );
    assert_eq!(lint["status"], "ok", "{lint}");
}

#[tokio::test]
async fn a_model_that_does_not_compile_is_not_written() {
    let (client, _server) = pair().await;
    let dir = TempDir::new("compose-broken");
    let path = dir.file("modelo.gmdl");
    // The file holds a model that works.
    std::fs::write(&path, WITH_GEOMETRY).expect("the model is written");

    // The UNLINK without a label: the model the reference refuses.
    let broken = std::fs::read_to_string(model_path("TP9__tp9"))
        .expect("the reference model is readable");
    let error = call(
        &client,
        "save_model",
        json!({ "path": path.to_string_lossy(), "model": broken }),
    )
    .await;
    assert_eq!(error["error"], "invalid_model", "the failure names the class");
    assert_eq!(error["class"], "Unlink");
    // The file that was there is still there: a model that does not compile
    // is not written.
    let written = std::fs::read_to_string(&path).expect("the model is readable");
    assert_eq!(written, WITH_GEOMETRY, "the file was overwritten");
}
