//! Black-box tests of the `genmsi` binary.
//!
//! Everything here goes through the public interface only: the process exits
//! with the documented code, writes the document on stdout and the failure on
//! stderr. Snapshots pin one output per subcommand; the matrix at the end runs
//! every reference model through the CLI.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The five models whose oracle baseline is an error instead of a report, and
/// the engine message class each must produce.
const REFUSED: [(&str, &str); 5] = [
    (
        "Parcial 8-11-21__ParcialMikelModelos",
        "invalid block",
    ),
    ("Problema 2__puenteLevadizo", "no TERMINATE block"),
    ("TP6__Clase 6_0_TestGate", "no TERMINATE block"),
    ("TP9__hack", "no TERMINATE block"),
    ("TP9__tp9", "the operand B must not be null"),
];

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

#[test]
fn simulate_prints_the_gpss_report() {
    let output = genmsi(&["simulate", SMALL]);
    assert_eq!(code(&output), 0, "{}", stderr(&output));
    insta::assert_snapshot!("simulate_report", stdout(&output));
}

#[test]
fn simulate_json_carries_the_engine_document() {
    let output = genmsi(&["simulate", SMALL, "--json"]);
    assert_eq!(code(&output), 0, "{}", stderr(&output));
    let document = json(&output);
    assert_eq!(document["schema"], "genmsi.report/1");
    assert_eq!(document["end_time"], 3600.0);
    assert_eq!(document["end_reason"], "terminated");
    assert_eq!(document["seed"], 1);
    assert_eq!(document["blocks"].as_array().expect("blocks").len(), 18);
    insta::assert_snapshot!("simulate_json", stdout(&output));
}

#[test]
fn simulate_honours_the_seed_and_the_horizon() {
    let default = json(&genmsi(&["simulate", "fixtures/models/TP4__supermercado.gmdl", "--json"]));
    let other = json(&genmsi(&[
        "simulate",
        "fixtures/models/TP4__supermercado.gmdl",
        "--json",
        "--seed",
        "99",
    ]));
    assert_eq!(default["seed"], 1);
    assert_eq!(other["seed"], 99);
    assert_ne!(default["blocks"], other["blocks"], "a new seed must change the run");

    let bounded = json(&genmsi(&[
        "simulate",
        "fixtures/models/TP4__supermercado.gmdl",
        "--json",
        "--until",
        "600",
    ]));
    assert_eq!(bounded["end_reason"], "time_limit");
    // The engine stops at the first event after the horizon, so it may overshoot.
    let end_time = bounded["end_time"].as_f64().expect("end_time");
    assert!(end_time >= 600.0, "{end_time}");
    assert!(end_time < 700.0, "{end_time}");
}


#[test]
fn trace_prints_events_and_reports_the_limit() {
    let output = genmsi(&["trace", SMALL, "--limit", "8"]);
    assert_eq!(code(&output), 0, "{}", stderr(&output));
    insta::assert_snapshot!("trace_events", stdout(&output));
}

#[test]
fn trace_json_is_versioned() {
    let output = genmsi(&["trace", SMALL, "--limit", "3", "--json"]);
    assert_eq!(code(&output), 0, "{}", stderr(&output));
    let document = json(&output);
    assert_eq!(document["schema"], "genmsi.trace/1");
    assert_eq!(document["limit"], 3);
    assert_eq!(document["records"], 3);
    assert_eq!(document["truncated"], true);
    insta::assert_snapshot!("trace_json", stdout(&output));
}

#[test]
fn run_all_sweeps_the_whole_fixture_set() {
    let output = genmsi(&["run-all", "fixtures/models"]);
    assert_eq!(code(&output), 1, "five models are refused: {}", stdout(&output));
    assert!(
        stdout(&output).ends_with("25 models: 20 ok, 5 failed\n"),
        "{}",
        stdout(&output)
    );
    insta::assert_snapshot!("run_all_summary", stdout(&output));
}

#[test]
fn run_all_json_marks_every_refused_model() {
    let output = genmsi(&["run-all", "fixtures/models", "--json"]);
    assert_eq!(code(&output), 1);
    let document = json(&output);
    assert_eq!(document["schema"], "genmsi.run-all/1");
    assert_eq!(document["models_total"], 25);
    assert_eq!(document["models_ok"], 20);
    assert_eq!(document["models_failed"], 5);
    let models = document["models"].as_array().expect("models");
    let failed: Vec<&serde_json::Value> = models
        .iter()
        .filter(|model| model["status"] == "failed")
        .collect();
    assert_eq!(failed.len(), 5);
    for (name, class) in REFUSED {
        let model = failed
            .iter()
            .find(|model| model["name"] == name)
            .unwrap_or_else(|| panic!("{name} is missing from the sweep"));
        let error = model["error"].as_str().expect("error text");
        assert!(error.contains(class), "{name}: {error}");
        assert!(model["end_time"].is_null(), "{name}: a failed model has no end_time");
    }
}

#[test]
fn run_all_on_a_file_is_a_usage_error() {
    let output = genmsi(&["run-all", SMALL]);
    assert_eq!(code(&output), 2);
    assert!(stderr(&output).contains("not a directory"), "{}", stderr(&output));
}



