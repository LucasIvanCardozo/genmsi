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

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use genmsi_app_engine::{
    ModelFile, Session, SessionRequest, answer, lint_document_bytes,
    list_models as list_model_files, model_document_bytes, simulate_document_bytes,
    trace_document_bytes,
};
use tauri::Manager;

/// The directory the shell offers as models.
const MODELS_DIR: &str = "fixtures/models";

/// How many models the list of recents keeps.
const RECENT_MODELS: usize = 8;

/// The name of the file the recents live in, inside the app's data
/// directory.
const RECENTS_FILE: &str = "recent-models.json";

/// Say what the shell did.
///
/// The shell has no console of its own, so the line goes to the log the
/// smoke check reads: it is the proof that the `invoke` path of the
/// frontend reaches the facade.
fn log(command: &str) {
    eprintln!("[genmsi-app] {command}");
}

/// The editing session the shell keeps open.
static SESSION: Mutex<Option<Session>> = Mutex::new(None);

/// The models the app offers.
#[tauri::command]
pub fn list_models() -> Result<Vec<ModelFile>, String> {
    let models = list_model_files(Path::new(MODELS_DIR)).map_err(|error| error.to_string())?;
    log(&format!("list_models: {} modelos", models.len()));
    Ok(models)
}

/// The model's graph (`genmsi.model/1`).
#[tauri::command]
pub fn load_model(path: String) -> Result<String, String> {
    log(&format!("load_model: {path}"));
    model_document_bytes(Path::new(&path)).map_err(|error| error.to_string())
}

/// The model's lint (`genmsi.lint/1`).
#[tauri::command]
pub fn lint_model(path: String) -> Result<String, String> {
    log(&format!("lint_model: {path}"));
    lint_document_bytes(Path::new(&path)).map_err(|error| error.to_string())
}

/// The run's report (`genmsi.report/1`).
#[tauri::command]
pub fn simulate_model(
    path: String,
    seed: Option<u64>,
    until: Option<f64>,
) -> Result<String, String> {
    log(&format!("simulate_model: {path}"));
    simulate_document_bytes(Path::new(&path), seed.unwrap_or(1), until)
        .map_err(|error| error.to_string())
}

/// The run's trace (`genmsi.trace/1`).
#[tauri::command]
pub fn trace_model(
    path: String,
    seed: Option<u64>,
    until: Option<f64>,
    limit: Option<usize>,
) -> Result<String, String> {
    log(&format!("trace_model: {path}"));
    trace_document_bytes(
        Path::new(&path),
        seed.unwrap_or(1),
        until,
        limit.unwrap_or(100),
    )
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
    log("session: una petición del frontend");
    let response = answer(&mut guard, parsed);
    serde_json::to_string(&response).map_err(|error| error.to_string())
}

/// The file the recents live in, inside the app's data directory.
///
/// The path comes from the resolver of Tauri: the app's data directory is
/// where the shell may write without asking.
fn recents_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("no se pudo encontrar el directorio de datos: {error}"))?;
    Ok(dir.join(RECENTS_FILE))
}

/// The models opened lately, the most recent first.
///
/// A file that is not there, or that cannot be read, is a list without
/// recents: the app opens models anyway.
fn read_recents(file: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(file) else {
        return Vec::new();
    };
    let Ok(models) = serde_json::from_str::<Vec<String>>(&text) else {
        return Vec::new();
    };
    models
}

/// Write the recents, creating the app's data directory if it is not there.
fn write_recents(file: &Path, models: &[String]) -> Result<(), String> {
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|error| format!("no se pudo crear {}: {error}", dir.display()))?;
    }
    let text = serde_json::to_string_pretty(models).map_err(|error| error.to_string())?;
    std::fs::write(file, text)
        .map_err(|error| format!("no se pudo escribir {}: {error}", file.display()))
}

/// The models opened lately, the most recent first.
#[tauri::command]
pub fn recent_models(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let file = recents_file(&app)?;
    log("recent_models: los modelos recientes");
    Ok(read_recents(&file))
}

/// Remember a model as one of the last opened ones.
///
/// The list is short and the last one is the first: the model the reader
/// comes back to is the one he opened last.
#[tauri::command]
pub fn remember_model(app: tauri::AppHandle, path: String) -> Result<Vec<String>, String> {
    let file = recents_file(&app)?;
    log(&format!("remember_model: {path}"));
    let mut models = read_recents(&file);
    models.retain(|model| model != &path);
    models.insert(0, path);
    models.truncate(RECENT_MODELS);
    write_recents(&file, &models)?;
    Ok(models)
}
