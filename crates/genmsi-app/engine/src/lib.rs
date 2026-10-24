//! The engine facade both app backends share.
//!
//! The facade is a thin client (decision D2: no simulation logic here): it
//! loads a model, asks `genmsi-core` for one of the engine's documents and
//! returns the document's bytes. The dev bridge spawns the `genmsi-app-engine`
//! binary and moves its stdout verbatim; the Tauri backend calls these
//! functions directly. Both paths carry the same bytes for the same model.
//!
//! The facade also owns the *editing session*: the model in memory, its undo
//! stack and the save. Every mutation goes through the additive API of
//! `genmsi-format`, so the app never touches the XML of a model.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// The protocol the app's backends speak with the facade.
pub mod protocol;
/// The editing session: the model in memory, its undo stack and its save.
pub mod session;

pub use protocol::{SessionRequest, SessionResponse, answer};
pub use session::{Diagnostics, SaveOutcome, Session, SessionState};

/// How many runs the facade has made.
///
/// Every run of the engine goes through [`run_once`], so this number is the
/// number of times the engine ran: a test can prove that the app's run is
/// one run of the engine and not two.
static RUNS: AtomicUsize = AtomicUsize::new(0);

/// How many runs the facade has made since the process started.
pub fn runs_made() -> usize {
    RUNS.load(Ordering::Relaxed)
}

/// The two documents of one run: the report and the trace.
///
/// One run with the trace on gives both: the report comes from the result
/// and the trace from the events of the same run. The engine is
/// deterministic, so the trace is the one a run without the trace records.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunDocuments {
    /// The `genmsi.report/1` document, byte for byte.
    pub report: String,
    /// The `genmsi.trace/1` document, byte for byte.
    pub trace: String,
}

/// Schema id of the app's own model list; an application document, not an
/// engine document.
pub const MODELS_SCHEMA: &str = "genmsi.models/1";

/// The flag that stops a run.
///
/// The reader raises it and the engine reads it, once per event: the run
/// ends with the reason of the cancellation and gives the partial result of
/// the work it managed to do. A run without the flag is the ordinary run, the
/// one the CLI and the oracle make: it cannot be stopped.
pub type StopFlag = Arc<AtomicBool>;

/// A failure of the facade.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// The model did not load, validate, compile or run.
    ///
    /// The path comes first so a log line says which model failed; the engine
    /// message keeps the source line and the node it carries.
    #[error("{path}: {message}")]
    Model {
        /// Model path, as the caller named it.
        path: String,
        /// Engine message.
        message: String,
    },
    /// An argument is well-formed but cannot be used.
    #[error("{0}")]
    Usage(String),
}

/// A `.gmdl` file of a directory, named for the app's model picker.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ModelFile {
    /// File stem, decoded for display.
    pub name: String,
    /// Full path, as the facade would open it.
    pub path: String,
}

/// A model failure, with the path the caller asked for.
///
/// The reader attaches the file to its own location, so an engine message can
/// already start with the path (`model.gmdl:9 (node 3): ...`); stripping that
/// prefix keeps the path from appearing twice, exactly like the CLI does.
fn model_error(path: &Path, error: impl std::fmt::Display) -> AppError {
    let path_text = path.display().to_string();
    let message = error.to_string();
    let message = message
        .strip_prefix(&format!("{path_text}:"))
        .map(|rest| rest.trim_start().to_string())
        .unwrap_or(message);
    AppError::Model {
        path: path_text,
        message,
    }
}

/// One document of the engine, rendered the way the CLI prints it: pretty
/// JSON under its schema id, with the trailing newline the CLI adds.
fn document_bytes(schema: &str, body: &impl serde::Serialize) -> String {
    format!(
        "{}\n",
        genmsi_core::documents::to_pretty_json(schema, body)
    )
}

/// Body of the `genmsi.models/1` list document.
#[derive(Debug, serde::Serialize)]
struct ModelsBody<'a> {
    /// Directory as the caller named it.
    dir: &'a str,
    /// Models, sorted by file name.
    models: &'a [ModelFile],
}

/// The `genmsi.models/1` document: the `.gmdl` files of the given directories.
pub fn models_document_bytes(dirs: &[PathBuf]) -> Result<String, AppError> {
    let mut models = Vec::new();
    for dir in dirs {
        models.extend(list_models(dir)?);
    }
    models.sort_by(|a, b| {
        let name = |model: &ModelFile| {
            Path::new(&model.path)
                .file_name()
                .map(|name| name.to_os_string())
        };
        name(a).cmp(&name(b))
    });
    let body = ModelsBody {
        dir: &dirs
            .iter()
            .map(|dir| dir.display().to_string())
            .collect::<Vec<_>>()
            .join(", "),
        models: &models,
    };
    Ok(format!(
        "{}\n",
        genmsi_core::documents::to_pretty_json(MODELS_SCHEMA, &body)
    ))
}

/// The `genmsi.catalogue/1` document: the block classes the editor offers.
pub fn catalogue_document_bytes() -> String {
    let body = genmsi_core::documents::catalogue_document();
    format!(
        "{}\n",
        genmsi_core::documents::to_pretty_json(
            genmsi_core::documents::CATALOGUE_SCHEMA,
            &body,
        )
    )
}

/// The `genmsi.model/1` document of a model: its blocks and its links.
///
/// The document is built from the file's semantic model, so a model that does
/// not compile still has a graph; only a file the reader refuses fails here.
pub fn model_document_bytes(path: &Path) -> Result<String, AppError> {
    let model = genmsi_format::load(path).map_err(|error| model_error(path, error))?;
    let body = genmsi_core::documents::model_document(&model, &path.display().to_string());
    Ok(document_bytes(genmsi_core::documents::MODEL_SCHEMA, &body))
}

/// The `genmsi.lint/1` document of a model: what it compiles to.
pub fn lint_document_bytes(path: &Path) -> Result<String, AppError> {
    let model = genmsi_format::load(path).map_err(|error| model_error(path, error))?;
    let program =
        genmsi_core::compile_model(&model).map_err(|error| model_error(path, error))?;
    let body = genmsi_core::documents::lint_document(&program, &model, &path.display().to_string());
    Ok(document_bytes(genmsi_core::documents::LINT_SCHEMA, &body))
}

/// The `genmsi.report/1` document of a finished run.
///
/// The flag, when there is one, is the flag of the cancellation: the reader
/// raises it and the engine stops the run.
pub fn simulate_document_bytes(
    path: &Path,
    seed: u64,
    until: Option<f64>,
    stop: Option<StopFlag>,
) -> Result<String, AppError> {
    let model = genmsi_format::load(path).map_err(|error| model_error(path, error))?;
    simulate_bytes(&model, path, seed, until, stop)
}

/// The `genmsi.report/1` document of a run of a model that is in memory.
///
/// The bytes are the ones the CLI prints for the same model and the same
/// seed: the document does not know where the model came from.
pub fn simulate_model_bytes(
    model: &genmsi_format::GmdlModel,
    seed: u64,
    until: Option<f64>,
    stop: Option<StopFlag>,
) -> Result<String, AppError> {
    simulate_bytes(model, Path::new(""), seed, until, stop)
}

/// The report of a run, with the path the failure names.
fn simulate_bytes(
    model: &genmsi_format::GmdlModel,
    path: &Path,
    seed: u64,
    until: Option<f64>,
    stop: Option<StopFlag>,
) -> Result<String, AppError> {
    let program = genmsi_core::compile_model(model).map_err(|error| model_error(path, error))?;
    let config = genmsi_core::SimConfig {
        seed,
        stop_time: until,
        ..genmsi_core::SimConfig::default()
    };
    let result = run_once(&program, config, path, stop)?;
    let body = genmsi_core::json_report(&result, &program);
    Ok(document_bytes(genmsi_core::documents::REPORT_SCHEMA, &body))
}

/// The report and the trace of one run of a model that is in memory.
///
/// The engine runs **once**, with the trace on: the report comes from the
/// result of that run and the trace from its events. The engine is
/// deterministic, so the two documents are the ones the two separate runs
/// used to give.
pub fn run_model_documents_bytes(
    model: &genmsi_format::GmdlModel,
    seed: u64,
    until: Option<f64>,
    limit: usize,
    stop: Option<StopFlag>,
) -> Result<RunDocuments, AppError> {
    run_documents_of(model, Path::new(""), seed, until, limit, stop)
}

/// The report and the trace of one run of a model on disk.
///
/// The engine runs **once**, with the trace on: the report comes from the
/// result of that run and the trace from its events.
pub fn run_documents_bytes(
    path: &Path,
    seed: u64,
    until: Option<f64>,
    limit: usize,
    stop: Option<StopFlag>,
) -> Result<RunDocuments, AppError> {
    let model = genmsi_format::load(path).map_err(|error| model_error(path, error))?;
    run_documents_of(&model, path, seed, until, limit, stop)
}

/// The report and the trace of one run, with the path the failure names.
fn run_documents_of(
    model: &genmsi_format::GmdlModel,
    path: &Path,
    seed: u64,
    until: Option<f64>,
    limit: usize,
    stop: Option<StopFlag>,
) -> Result<RunDocuments, AppError> {
    let program = genmsi_core::compile_model(model).map_err(|error| model_error(path, error))?;
    let config = genmsi_core::SimConfig {
        seed,
        stop_time: until,
        trace: true,
        trace_limit: limit,
        ..genmsi_core::SimConfig::default()
    };
    let result = run_once(&program, config, path, stop)?;
    let report = genmsi_core::json_report(&result, &program);
    let trace = genmsi_core::documents::trace_document(&result, limit);
    Ok(RunDocuments {
        report: document_bytes(genmsi_core::documents::REPORT_SCHEMA, &report),
        trace: document_bytes(genmsi_core::documents::TRACE_SCHEMA, &trace),
    })
}

/// Run the program once and give its result.
///
/// Every run of the facade goes through here: the counter of the runs is the
/// number of times the engine ran.
///
/// The flag, when there is one, is the flag of the cancellation: the engine
/// reads it once per event and stops the run when it finds it up.
fn run_once(
    program: &genmsi_core::Program,
    mut config: genmsi_core::SimConfig,
    path: &Path,
    stop: Option<StopFlag>,
) -> Result<genmsi_core::RunResult, AppError> {
    RUNS.fetch_add(1, Ordering::Relaxed);
    // The flag that stops the run travels with the configuration, next to the
    // horizon that ends it: the two are the same kind of thing.
    config.stop = stop;
    genmsi_core::Simulation::new(program, config)
        .run()
        .map_err(|error| model_error(path, error))
}

/// The `genmsi.trace/1` document of a finished run.
pub fn trace_document_bytes(
    path: &Path,
    seed: u64,
    until: Option<f64>,
    limit: usize,
    stop: Option<StopFlag>,
) -> Result<String, AppError> {
    let model = genmsi_format::load(path).map_err(|error| model_error(path, error))?;
    trace_bytes(&model, path, seed, until, limit, stop)
}

/// The `genmsi.trace/1` document of a run of a model that is in memory.
///
/// The bytes are the ones the CLI prints for the same model, seed and limit:
/// the document does not know where the model came from.
pub fn trace_model_bytes(
    model: &genmsi_format::GmdlModel,
    seed: u64,
    until: Option<f64>,
    limit: usize,
    stop: Option<StopFlag>,
) -> Result<String, AppError> {
    trace_bytes(model, Path::new(""), seed, until, limit, stop)
}

/// The trace of a run, with the path the failure names.
fn trace_bytes(
    model: &genmsi_format::GmdlModel,
    path: &Path,
    seed: u64,
    until: Option<f64>,
    limit: usize,
    stop: Option<StopFlag>,
) -> Result<String, AppError> {
    let program = genmsi_core::compile_model(model).map_err(|error| model_error(path, error))?;
    let config = genmsi_core::SimConfig {
        seed,
        stop_time: until,
        trace: true,
        trace_limit: limit,
        ..genmsi_core::SimConfig::default()
    };
    let result = run_once(&program, config, path, stop)?;
    Ok(document_bytes(
        genmsi_core::documents::TRACE_SCHEMA,
        &genmsi_core::documents::trace_document(&result, limit),
    ))
}

/// The `.gmdl` files of one directory level, sorted by file name.
pub fn list_models(dir: &Path) -> Result<Vec<ModelFile>, AppError> {
    if !dir.is_dir() {
        return Err(AppError::Usage(format!(
            "{}: not a directory",
            dir.display()
        )));
    }
    let entries =
        std::fs::read_dir(dir).map_err(|error| AppError::Usage(format!("{}: {error}", dir.display())))?;
    let mut models = Vec::new();
    for entry in entries {
        let entry = entry
            .map_err(|error| AppError::Usage(format!("{}: {error}", dir.display())))?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "gmdl") {
            models.push(ModelFile {
                name: path
                    .file_stem()
                    .map(|stem| stem.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                path: path.display().to_string(),
            });
        }
    }
    models.sort_by(|a, b| {
        let name = |model: &ModelFile| {
            Path::new(&model.path)
                .file_name()
                .map(|name| name.to_os_string())
        };
        name(a).cmp(&name(b))
    });
    Ok(models)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// The reference models of the repository.
    fn fixtures() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/models")
    }

    /// The reference model of the app's slice.
    fn reference() -> PathBuf {
        fixtures().join("TP6__Clase 6_1_PuertaBanco.gmdl")
    }

    #[test]
    fn the_lint_bytes_match_the_engine_document() {
        let path = reference();
        let model = genmsi_format::load(&path).expect("the reference model loads");
        let program = genmsi_core::compile_model(&model).expect("the reference model compiles");
        let path_text = path.display().to_string();
        let expected = format!(
            "{}\n",
            genmsi_core::documents::to_pretty_json(
                genmsi_core::documents::LINT_SCHEMA,
                &genmsi_core::documents::lint_document(&program, &model, &path_text),
            )
        );
        assert_eq!(lint_document_bytes(&path).expect("lints"), expected);
    }

    #[test]
    fn the_simulate_bytes_match_the_engine_document() {
        let path = reference();
        let model = genmsi_format::load(&path).expect("the reference model loads");
        let program = genmsi_core::compile_model(&model).expect("the reference model compiles");
        let result = genmsi_core::Simulation::new(&program, genmsi_core::SimConfig {
            seed: 1,
            ..genmsi_core::SimConfig::default()
        })
        .run()
        .expect("the reference model runs");
        let expected = format!(
            "{}\n",
            genmsi_core::documents::to_pretty_json(
                genmsi_core::documents::REPORT_SCHEMA,
                &genmsi_core::json_report(&result, &program),
            )
        );
        assert_eq!(
            simulate_document_bytes(&path, 1, None, None).expect("runs"),
            expected
        );
    }

    #[test]
    fn the_model_bytes_parse_and_name_their_schema() {
        let bytes = model_document_bytes(&reference()).expect("builds");
        let document: serde_json::Value = serde_json::from_str(&bytes).expect("parses");
        assert_eq!(document["schema"], "genmsi.model/1");
    }

    #[test]
    fn a_refused_model_is_a_model_error_naming_it_once() {
        let path = fixtures().join("TP9__tp9.gmdl");
        let error = simulate_document_bytes(&path, 1, None, None).expect_err("the engine refuses it");
        let text = error.to_string();
        let path_text = path.display().to_string();
        assert!(matches!(error, AppError::Model { .. }), "{text}");
        assert!(text.contains(&path_text), "{text}");
        assert_eq!(
            text.matches(&path_text).count(),
            1,
            "path printed twice: {text}"
        );
        assert!(!text.starts_with("error"), "{text}");
    }

    #[test]
    fn the_trace_bytes_parse_and_name_their_schema() {
        let bytes =
            trace_document_bytes(&reference(), 1, None, 100, None).expect("runs");
        let document: serde_json::Value = serde_json::from_str(&bytes).expect("parses");
        assert_eq!(document["schema"], "genmsi.trace/1");
    }

    #[test]
    fn the_same_model_seed_and_limit_give_the_same_trace_bytes() {
        let path = reference();
        let first = trace_document_bytes(&path, 7, None, 100, None).expect("runs");
        let second = trace_document_bytes(&path, 7, None, 100, None).expect("runs");
        assert_eq!(first, second);
    }

    #[test]
    fn a_small_limit_truncates_and_a_big_one_does_not() {
        let path = reference();
        let small = trace_document_bytes(&path, 1, None, 5, None).expect("runs");
        let small: serde_json::Value = serde_json::from_str(&small).expect("parses");
        assert_eq!(small["limit"], 5);
        assert!(small["records"].as_u64().is_some_and(|r| r <= 5));
        assert_eq!(small["truncated"], true);

        let big = trace_document_bytes(&path, 1, None, 200_000, None).expect("runs");
        let big: serde_json::Value = serde_json::from_str(&big).expect("parses");
        assert_eq!(big["truncated"], false);
        assert!(
            big["records"].as_u64().is_some_and(|r| r > 5),
            "the run keeps more than a small limit of records"
        );
    }

    #[test]
    fn the_trace_of_the_model_in_memory_is_the_trace_of_its_file() {
        let path = reference();
        let model = genmsi_format::load(&path).expect("the reference model loads");
        let from_file = trace_document_bytes(&path, 1, None, 100, None).expect("runs");
        let from_memory = trace_model_bytes(&model, 1, None, 100, None).expect("runs");
        assert_eq!(from_file, from_memory);
    }

    #[test]
    fn list_models_returns_the_reference_models_sorted() {
        let models = list_models(&fixtures()).expect("lists");
        assert_eq!(models.len(), 25, "{models:?}");
        let names: Vec<&str> = models.iter().map(|model| model.name.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted, "not sorted: {names:?}");
        let first = &models[0];
        assert_eq!(first.name, "Parcial 6-11-23__Veron - Parcial");
        assert!(first.path.ends_with(".gmdl"), "{}", first.path);
    }
}
