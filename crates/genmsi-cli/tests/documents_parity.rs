//! Byte parity between the CLI documents and the engine's documents.
//!
//! `genmsi lint --json` and `genmsi trace --json` must print exactly the bytes
//! `genmsi_core::documents` produces, so the CLI and the MCP server (which
//! builds its documents through that module) can never drift apart. A
//! failure here names which document moved.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use genmsi_core::documents::{
    LINT_SCHEMA, TRACE_SCHEMA, lint_document, to_pretty_json, trace_document,
};

/// A model small enough to be fast and rich enough to exercise the documents.
const MODEL: &str = "fixtures/models/TP6__Clase 6_1_PuertaBanco.gmdl";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Run the binary from the repository root, as the CLI tests do, so the path
/// inside a document is the relative one.
fn genmsi(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_genmsi"))
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("the genmsi binary runs")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout is UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr is UTF-8")
}

fn code(output: &Output) -> i32 {
    output.status.code().expect("the process exits with a code")
}

/// Load and compile the fixture the way the CLI does.
fn program() -> genmsi_core::Program {
    let path = repo_root().join(MODEL);
    let model = genmsi_format::load(&path).expect("the fixture parses");
    genmsi_core::compile_model(&model).expect("the fixture compiles")
}

#[test]
fn the_cli_lint_document_is_the_engine_document() {
    let output = genmsi(&["lint", MODEL, "--json"]);
    assert_eq!(code(&output), 0, "{}", stderr(&output));
    let cli_bytes = stdout(&output);
    let compiled = program();
    let document = lint_document(&compiled, MODEL);
    // The CLI adds the trailing newline its stdout helper appends.
    let engine_bytes = format!("{}\n", to_pretty_json(LINT_SCHEMA, &document));
    assert_eq!(
        cli_bytes, engine_bytes,
        "the genmsi.lint/1 document drifted between the CLI and genmsi_core::documents"
    );
}

#[test]
fn the_cli_trace_document_is_the_engine_document() {
    let output = genmsi(&["trace", MODEL, "--json", "--limit", "3", "--seed", "1"]);
    assert_eq!(code(&output), 0, "{}", stderr(&output));
    let cli_bytes = stdout(&output);
    // The same knobs the CLI was given (`--seed 1`, no `--until`) with the
    // trace on.
    let config = genmsi_core::SimConfig {
        seed: 1,
        trace: true,
        trace_limit: 3,
        ..genmsi_core::SimConfig::default()
    };
    let compiled = program();
    let result = genmsi_core::Simulation::new(&compiled, config)
        .run()
        .expect("the fixture runs");
    let engine_bytes = format!("{}\n", to_pretty_json(TRACE_SCHEMA, &trace_document(&result, 3)));
    assert_eq!(
        cli_bytes, engine_bytes,
        "the genmsi.trace/1 document drifted between the CLI and genmsi_core::documents"
    );
}
