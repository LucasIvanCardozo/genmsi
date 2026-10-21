//! Black-box tests of the `genmsi` binary.
//!
//! Everything here goes through the public interface only: the process exits
//! with the documented code, writes the document on stdout and the failure on
//! stderr. Snapshots pin one output per subcommand; the matrix at the end runs
//! every reference model through the CLI.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};


/// A model small enough to snapshot whole and rich enough to exercise queues,
/// facilities, gates and two generators.
const SMALL: &str = "fixtures/models/TP6__Clase 6_1_PuertaBanco.gmdl";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Run the binary from the repository root so every path in a document is
/// relative and the snapshots stay machine independent.
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

fn json(output: &Output) -> serde_json::Value {
    serde_json::from_str(&stdout(output)).expect("stdout is one JSON document")
}



#[test]
fn lint_reports_a_clean_model() {
    let output = genmsi(&["lint", "fixtures/models/TP2__problema1.gmdl"]);
    assert_eq!(code(&output), 0, "{}", stderr(&output));
    assert!(stderr(&output).is_empty());
    insta::assert_snapshot!("lint_clean", stdout(&output));
}

#[test]
fn lint_json_is_versioned() {
    let output = genmsi(&["lint", SMALL, "--json"]);
    assert_eq!(code(&output), 0, "{}", stderr(&output));
    let document = json(&output);
    assert_eq!(document["schema"], "genmsi.lint/1");
    assert_eq!(document["status"], "ok");
    assert_eq!(document["blocks"], 18);
    insta::assert_snapshot!("lint_json", stdout(&output));
}

#[test]
fn lint_refuses_the_structurally_invalid_model() {
    let output = genmsi(&["lint", "fixtures/models/Parcial 8-11-21__ParcialMikelModelos.gmdl"]);
    assert_eq!(code(&output), 1);
    assert!(stdout(&output).is_empty());
    insta::assert_snapshot!("lint_invalid_model", stderr(&output));
}

#[test]
fn lint_refuses_a_model_the_original_refuses() {
    let output = genmsi(&["lint", "fixtures/models/TP9__tp9.gmdl"]);
    assert_eq!(code(&output), 1);
    let message = stderr(&output);
    assert!(message.contains("the operand B must not be null"), "{message}");
    assert!(message.starts_with("error: fixtures/models/TP9__tp9.gmdl: "), "{message}");
}












