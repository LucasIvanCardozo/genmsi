//! The set of error classes is closed and pinned.
//!
//! `error` is a `&'static str` an agent switches on, so a renamed or added
//! class must fail here instead of reaching an agent silently.

use std::collections::BTreeSet;
use std::path::Path;

use genmsi_mcp::errors::ToolError;

/// Every class `docs/mcp.md` documents, in full.
const DOCUMENTED: [&str; 12] = [
    "class_not_found",
    "invalid_arguments",
    "invalid_block",
    "invalid_expression",
    "invalid_graph",
    "invalid_model",
    "invalid_xml",
    "model_not_found",
    "model_unreadable",
    "model_unwritable",
    "runtime_error",
    "unsupported_block",
];

/// The string literals of `errors.rs` that follow either needle, so every
/// place the code emits a class name is seen.
fn literals_after(source: &str, needle: &str, found: &mut BTreeSet<String>) {
    let mut rest = source;
    while let Some(index) = rest.find(needle) {
        let after = &rest[index + needle.len()..];
        if let Some(end) = after.find('"') {
            found.insert(after[..end].to_string());
        }
        rest = after;
    }
}

#[test]
fn the_pinned_set_is_the_documented_set() {
    assert_eq!(ToolError::CLASSES, DOCUMENTED, "the documented set, in full");
}

#[test]
fn the_source_emits_exactly_the_pinned_set() {
    let source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/errors.rs"),
    )
    .expect("errors.rs is readable");
    let mut emitted = BTreeSet::new();
    literals_after(&source, "error: \"", &mut emitted);
    literals_after(&source, "=> (\"", &mut emitted);
    let pinned: BTreeSet<String> = ToolError::CLASSES.iter().map(|c| c.to_string()).collect();
    assert_eq!(
        emitted, pinned,
        "errors.rs emits exactly the pinned classes: adding, renaming or \
         dropping one must fail here"
    );
}
