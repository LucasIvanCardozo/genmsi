//! The commands the frontend calls.
//!
//! Each command returns one engine document, built by the shared facade
//! (`genmsi-app-engine`): the shell moves the bytes, it never builds a
//! document of its own. The dev bridge's binary runs the same facade, so both
//! backends return the same document for the same model.
//!
//! The editing session lives here too: the shell keeps one model in memory
//! with its undo stack, exactly like the dev bridge keeps one in its session
//! process.

use std::path::Path;
use std::sync::Mutex;

use genmsi_app_engine::{
    ModelFile, Session, SessionRequest, answer, lint_document_bytes,
    list_models as list_model_files, model_document_bytes, simulate_document_bytes,
};

/// The directory the shell offers as models.
const MODELS_DIR: &str = "fixtures/models";

/// The editing session the shell keeps open.
static SESSION: Mutex<Option<Session>> = Mutex::new(None);

/// The models the app offers.
#[tauri::command]
pub fn list_models() -> Result<Vec<ModelFile>, String> {
    list_model_files(Path::new(MODELS_DIR)).map_err(|error| error.to_string())
}

/// The model's graph (`genmsi.model/1`).
#[tauri::command]
pub fn load_model(path: String) -> Result<String, String> {
    model_document_bytes(Path::new(&path)).map_err(|error| error.to_string())
}

/// The model's lint (`genmsi.lint/1`).
#[tauri::command]
pub fn lint_model(path: String) -> Result<String, String> {
    lint_document_bytes(Path::new(&path)).map_err(|error| error.to_string())
}

/// The run's report (`genmsi.report/1`).
#[tauri::command]
pub fn simulate_model(
    path: String,
    seed: Option<u64>,
    until: Option<f64>,
) -> Result<String, String> {
    simulate_document_bytes(Path::new(&path), seed.unwrap_or(1), until)
        .map_err(|error| error.to_string())
}

/// One request of the editing protocol, answered by the same facade the dev
/// bridge's session process runs.
///
/// The request and the answer are JSON: the frontend parses them the same
/// way in both backends.
#[tauri::command]
pub fn session(request: String) -> Result<String, String> {
    let mut guard = SESSION
        .lock()
        .map_err(|_| "la sesión del modelo está ocupada".to_string())?;
    let parsed: SessionRequest =
        serde_json::from_str(&request).map_err(|error| error.to_string())?;
    let response = answer(&mut guard, parsed);
    serde_json::to_string(&response).map_err(|error| error.to_string())
}
