//! The document of the server must describe the server.
//!
//! The tools are listed over the wire, the same way an agent sees them, and
//! the table of `docs/mcp.md` is checked against that list: a tool the
//! document does not mention fails here, instead of failing in the reader's
//! head. The document each tool returns is checked too, because a tool whose
//! schema moved is a contract that moved.

use std::path::{Path, PathBuf};

use genmsi_mcp::GenmsiServer;
use rmcp::ServiceExt;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn document() -> String {
    std::fs::read_to_string(repo_root().join("docs/mcp.md")).expect("docs/mcp.md is readable")
}

/// The row of the table of tools that belongs to a tool.
fn row(doc: &str, tool: &str) -> Option<String> {
    doc.lines()
        .find(|line| line.starts_with(&format!("| `{tool}`")))
        .map(|line| line.to_string())
}

/// The table of tools: the section that starts with the heading of the tools.
fn tools_table() -> String {
    let doc = document();
    let start = doc
        .find("## The tools")
        .expect("docs/mcp.md has a section of tools");
    let rest = &doc[start..];
    // The table of tools is what follows the heading; the sub-sections of
    // each tool come after it, with their own tables of fields.
    let end = rest[3..]
        .find("\n### ")
        .map(|at| at + 3)
        .unwrap_or(rest.len());
    rest[..end].to_string()
}

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

/// The document each tool returns: the contract the server publishes.
const RETURNS: [(&str, &str); 8] = [
    ("catalogue", "genmsi.classlist/1"),
    ("lint", "genmsi.lint/1"),
    ("simulate", "genmsi.report/1"),
    ("get_trace", "genmsi.trace/1"),
    ("explain_model", "genmsi.explain/1"),
    ("save_model", "genmsi.lint/1"),
    ("apply_edit", "genmsi.edit/1"),
    ("app_state", "genmsi.live/1"),
];

#[tokio::test]
async fn the_document_names_every_tool_the_server_offers() {
    let (client, _server) = pair().await;
    let listed = client.list_tools(None).await.expect("the tools are listed");
    let doc = document();
    for tool in &listed.tools {
        assert!(
            row(&doc, &tool.name).is_some(),
            "docs/mcp.md does not describe the tool `{}`",
            tool.name
        );
    }
}

#[tokio::test]
async fn the_document_names_no_tool_the_server_does_not_offer() {
    let (client, _server) = pair().await;
    let listed = client.list_tools(None).await.expect("the tools are listed");
    let doc = document();
    for (tool, _) in RETURNS {
        assert!(
            listed.tools.iter().any(|offered| offered.name.as_ref() == tool),
            "docs/mcp.md describes the tool `{tool}`, which the server does not offer"
        );
    }
    // And nothing else: the table has one row per tool.
    let rows = tools_table()
        .lines()
        .filter(|line| line.starts_with("| `"))
        .count();
    assert_eq!(
        rows,
        listed.tools.len(),
        "the table of tools of docs/mcp.md has a row that is not a tool"
    );
}

#[tokio::test]
async fn the_document_names_the_schema_each_tool_returns() {
    let doc = document();
    for (tool, schema) in RETURNS {
        let row = row(&doc, tool)
            .unwrap_or_else(|| panic!("docs/mcp.md does not describe the tool `{tool}`"));
        assert!(
            row.contains(schema),
            "the row of `{tool}` in docs/mcp.md does not name the document it returns ({schema})"
        );
    }
}
