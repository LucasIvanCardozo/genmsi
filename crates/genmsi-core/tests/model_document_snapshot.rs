//! Snapshot of the published `genmsi.model/1` document.
//!
//! The model document is the contract every client (the app, the MCP server)
//! renders as a graph, so its shape cannot drift silently: one reviewed model
//! with blocks that carry geometry, properties and edges pins it.

use genmsi_core::documents::{MODEL_SCHEMA, model_document, to_pretty_json};

/// The reviewed model with a container, annotated blocks and edges.
const MODEL: &str = "../../fixtures/models/TP6__Clase 6_1_PuertaBanco.gmdl";

#[test]
fn model_document_snapshot() {
    let model = genmsi_format::load(std::path::Path::new(MODEL)).expect("the fixture parses");
    let document = model_document(&model, MODEL);
    let text = to_pretty_json(MODEL_SCHEMA, &document);
    insta::assert_snapshot!("model_document__tp6_puerta_banco", text);
}
