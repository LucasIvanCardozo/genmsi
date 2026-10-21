//! Tool errors carry a usable position: `column` and `node` wherever one
//! exists, and nothing (not a zero, not a null) where one cannot.

use genmsi_mcp::server::ModelInput;
use genmsi_mcp::GenmsiServer;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ContentBlock;
use serde_json::Value;

/// Two ports of node 1 share `ioid="1"`; the module starts on line 2,
/// indented two spaces (the same document `genmsi-format` is tested with).
const DUPLICATE_PORT: &str = "<gmodel versioncode=\"1\"><gclasses/><modules>\n  \
     <module moduleid=\"1\" moduleclass=\"Queue\" owner=\"0\" caption=\"q\" name=\"q\" kind=\"A\">\
     <inputs><input ioid=\"1\" name=\"a\" caption=\"\" mylabel=\"L\"/>\
     <input ioid=\"1\" name=\"b\" caption=\"\" mylabel=\"L\"/></inputs>\
     <outputs/><props/></module>\n</modules></gmodel>";

/// The JSON body `lint` returns for the given model.
fn lint_body(input: ModelInput) -> Value {
    let result = GenmsiServer::new().lint(Parameters(input));
    assert_eq!(result.is_error, Some(true), "lint must refuse it");
    let ContentBlock::Text(content) = &result.content[0] else {
        panic!("expected text content, got {:?}", result.content);
    };
    serde_json::from_str(&content.text).expect("the tool returns one JSON document")
}

#[test]
fn a_duplicate_port_error_carries_column_and_node() {
    let body = lint_body(ModelInput {
        path: None,
        model: Some(DUPLICATE_PORT.to_string()),
    });
    assert_eq!(body["error"], "invalid_model", "{body}");
    // The node travels in its own field, so the message must not repeat it.
    assert_eq!(body["message"], "duplicate port id 1", "{body}");
    assert_eq!(body["node"], 1, "the offending <module>: {body}");
    assert_eq!(body["line"], 2, "{body}");
    assert_eq!(body["column"], 3, "the column of the opening tag: {body}");
    assert_eq!(body["path"], "<memory>", "{body}");
    assert!(
        body.get("class").is_none(),
        "a parse failure names no GPSS class: {body}"
    );
}

#[test]
fn a_position_that_cannot_exist_is_absent_not_zero() {
    // A missing file has no line, column or node in the document at all.
    let body = lint_body(ModelInput {
        path: Some("definitely-missing.gmdl".to_string()),
        model: None,
    });
    assert_eq!(body["error"], "model_not_found", "{body}");
    assert!(body.get("line").is_none(), "{body}");
    assert!(body.get("column").is_none(), "{body}");
    assert!(body.get("node").is_none(), "{body}");
}

#[test]
fn a_file_that_is_not_utf8_is_the_model_error_the_cli_reports() {
    // The CLI loads through `parse_file` and prints "cannot read model: ...";
    // the MCP classifies the same file the same way.
    let dir = std::env::temp_dir().join(format!("genmsi-mcp-not-utf8-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the temp directory is created");
    let path = dir.join("binary.gmdl");
    std::fs::write(&path, [0xFF, 0xFE, b'<', b'g']).expect("the file is written");
    let body = lint_body(ModelInput {
        path: Some(path.display().to_string()),
        model: None,
    });
    std::fs::remove_file(&path).expect("the temp file is removed");
    std::fs::remove_dir(&dir).expect("the temp directory is removed");
    assert_eq!(body["error"], "invalid_model", "{body}");
    assert!(
        body["message"]
            .as_str()
            .unwrap_or_default()
            .starts_with("cannot read model"),
        "{body}"
    );
}
