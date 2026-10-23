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

fn models() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(repo_root().join("fixtures/models"))
        .expect("fixtures/models")
        .map(|entry| entry.expect("entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "gmdl"))
        .collect();
    paths.sort();
    paths
}

fn stem(path: &Path) -> String {
    path.file_stem().expect("stem").to_string_lossy().to_string()
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

/// A model that uses a storage no block declares.
///
/// The original opens a model like this (its `StorageUse` says the storage has
/// to be declared, and the editor's picker only offers the declared ones), so
/// the lint does not refuse it: it says what it found.
const UNDECLARED_STORAGE: &str = "<gmodel versioncode=\"1\"><gclasses/><modules>
  <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"m\" name=\"m\" kind=\"C\">
    <links/><inputs/><outputs/>
  </module>
  <module moduleid=\"1\" moduleclass=\"StorageUse\" owner=\"0\" caption=\"c\" name=\"n\" kind=\"A\">
    <props>
      <property name=\"Media\" hint=\"\" value=\"10\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/>
      <property name=\"Storage\" hint=\"\" value=\"Almacen\" type=\"EntityUse\" subType=\"Storage\" visible=\"1\" unique=\"0\"/>
    </props>
  </module>
  <module moduleid=\"2\" moduleclass=\"Terminate\" owner=\"0\" caption=\"c\" name=\"n\" kind=\"A\">
    <props>
      <property name=\"Unidades\" hint=\"\" value=\"1\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/>
    </props>
  </module>
</modules></gmodel>";

/// Write a model to a file of its own, so the CLI can read it from the disk.
fn write_model(name: &str, text: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("genmsi-cli-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the temporary directory");
    let path = dir.join(format!("{name}.gmdl"));
    std::fs::write(&path, text).expect("the model is written");
    path
}

#[test]
fn lint_says_the_warning_of_the_format() {
    let path = write_model("undeclared-storage", UNDECLARED_STORAGE);
    let output = genmsi(&["lint", path.to_str().expect("a path")]);
    // A warning is not a failure: the model loads, and the lint says what it
    // found with the code it uses for that.
    assert_eq!(code(&output), 3, "{}", stderr(&output));
    assert!(stderr(&output).is_empty());
    let text = stdout(&output);
    assert!(text.contains("status: diagnostics"), "{text}");
    let findings: Vec<&str> = text
        .lines()
        .filter(|line| line.starts_with("  - "))
        .collect();
    // One finding, one text: the compiler reports the storage on its own, and
    // the format's warning about the same storage is not said again.
    assert_eq!(findings.len(), 1, "{text}");
    assert!(findings[0].contains("Almacen"), "{text}");
    assert!(findings[0].contains("no capacity"), "{text}");
}

#[test]
fn lint_json_carries_the_warning_of_the_format() {
    let path = write_model("undeclared-storage-json", UNDECLARED_STORAGE);
    let output = genmsi(&["lint", path.to_str().expect("a path"), "--json"]);
    assert_eq!(code(&output), 3, "{}", stderr(&output));
    let document = json(&output);
    assert_eq!(document["schema"], "genmsi.lint/1");
    assert_eq!(document["status"], "diagnostics");
    let diagnostics = document["diagnostics"]
        .as_array()
        .expect("an array of diagnostics");
    assert_eq!(diagnostics.len(), 1, "{document}");
    assert!(
        diagnostics[0]
            .as_str()
            .expect("a message")
            .contains("Almacen"),
        "{document}"
    );
}

#[test]
fn simulate_prints_the_gpss_report() {
    let output = genmsi(&["simulate", SMALL, "--seed", "1"]);
    assert_eq!(code(&output), 0, "{}", stderr(&output));
    insta::assert_snapshot!("simulate_report", stdout(&output));
}

#[test]
fn simulate_json_carries_the_engine_document() {
    let output = genmsi(&["simulate", SMALL, "--json", "--seed", "1"]);
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
    // Without a seed the run draws one, and the document says which: the
    // reader can repeat the run with that number.
    let drawn = default["seed"].as_u64().expect("the seed of the run");
    assert!(drawn > 0, "the run says the seed it used: {drawn}");
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
fn two_runs_without_a_seed_are_not_the_same_run() {
    let first = genmsi(&["simulate", "fixtures/models/TP3__aeropuerto.gmdl"]);
    let second = genmsi(&["simulate", "fixtures/models/TP3__aeropuerto.gmdl"]);
    assert_ne!(
        str::from_utf8(&first.stdout),
        str::from_utf8(&second.stdout),
        "two runs that draw their own seed are two different runs"
    );
}

#[test]
fn the_same_seed_reproduces_the_same_bytes() {
    let first = genmsi(&["simulate", "fixtures/models/TP3__aeropuerto.gmdl", "--seed", "1"]);
    let second = genmsi(&["simulate", "fixtures/models/TP3__aeropuerto.gmdl", "--seed", "1"]);
    assert_eq!(first.status, second.status);
    assert_eq!(str::from_utf8(&first.stdout), str::from_utf8(&second.stdout));
}

#[test]
fn trace_prints_events_and_reports_the_limit() {
    let output = genmsi(&["trace", SMALL, "--limit", "8", "--seed", "1"]);
    assert_eq!(code(&output), 0, "{}", stderr(&output));
    insta::assert_snapshot!("trace_events", stdout(&output));
}

#[test]
fn trace_json_is_versioned() {
    let output = genmsi(&["trace", SMALL, "--limit", "3", "--json", "--seed", "1"]);
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
    let output = genmsi(&["run-all", "fixtures/models", "--seed", "1"]);
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
    let output = genmsi(&["run-all", "fixtures/models", "--json", "--seed", "1"]);
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

#[test]
fn a_missing_model_file_exits_with_one() {
    let output = genmsi(&["simulate", "fixtures/models/does-not-exist.gmdl"]);
    assert_eq!(code(&output), 1);
    let message = stderr(&output);
    assert!(
        message.starts_with("error: fixtures/models/does-not-exist.gmdl: "),
        "{message}"
    );
}

#[test]
fn a_usage_error_exits_with_two() {
    let missing_argument = genmsi(&["simulate"]);
    assert_eq!(code(&missing_argument), 2);
    assert!(
        stderr(&missing_argument).contains("Usage"),
        "{}",
        stderr(&missing_argument)
    );

    let unknown_option = genmsi(&["simulate", SMALL, "--nope"]);
    assert_eq!(code(&unknown_option), 2);
    assert!(
        stderr(&unknown_option).contains("--nope"),
        "{}",
        stderr(&unknown_option)
    );
}

#[test]
fn every_reference_model_is_runnable_through_the_cli() {
    let paths = models();
    assert_eq!(paths.len(), 25, "the fixture set holds 25 models");
    let mut ran = 0;
    let mut refused = 0;
    for path in &paths {
        let name = stem(path);
        let shown = path
            .strip_prefix(repo_root())
            .expect("inside the repository")
            .display()
            .to_string();
        let output = genmsi(&["simulate", &shown]);
        match REFUSED.iter().find(|(model, _)| *model == name) {
            Some((_, class)) => {
                refused += 1;
                assert_eq!(code(&output), 1, "{name}: must be refused");
                let message = stderr(&output);
                assert!(message.contains(class), "{name}: expected {class:?}, got {message}");
                assert!(stdout(&output).is_empty(), "{name}: a failure prints no report");
            }
            None => {
                ran += 1;
                assert_eq!(code(&output), 0, "{name}: {}", stderr(&output));
                assert!(
                    stdout(&output).contains("Reporte MicroVisualSIS"),
                    "{name}: no report"
                );
            }
        }
    }
    assert_eq!((ran, refused), (20, 5));
}
