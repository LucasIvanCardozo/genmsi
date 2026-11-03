//! The live state of the desktop application.
//!
//! The app publishes what it is showing, so that an agent that talks to the
//! engine through the MCP can see what the reader sees: the model it has
//! open, whether it has changes that are not saved, what it found in the
//! model, and the last run it made.
//!
//! The document lives in the data directory of the app, beside the file that
//! says which model is open. The app writes it; the MCP reads it.
//!
//! Two things do **not** travel in it, on purpose:
//!
//! - the model in memory travels only when it differs from the disk (when the
//!   reader has changes that are not saved): when it does not, the file *is*
//!   the model and the agent reads it from there;
//! - the trace of the last run does not travel: it is enormous and the agent
//!   asks for it when it wants it.

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;

use crate::app_data::{AppDataError, app_data_dir, process_is_running};
use crate::proposal::ProposalBody;

/// The schema of the live document.
pub const LIVE_SCHEMA: &str = "genmsi.live/1";

/// The file the live document lives in, in the data directory of the app.
pub const LIVE_FILE: &str = "live.json";

/// What the app is showing.
#[derive(Debug, Clone, Serialize)]
pub struct LiveBody {
    /// The absolute path of the model the app has open.
    pub path: Option<String>,
    /// The pid of the process that has it open.
    pub pid: Option<u32>,
    /// Whether the reader has changes that are not saved.
    pub dirty: bool,
    /// What the app found in the model.
    pub diagnostics: Vec<String>,
    /// The model as it is in memory, when it differs from the disk.
    ///
    /// `None` when the model of the app is the model of the disk: the agent
    /// reads that one from the file, with the tools it already has.
    pub model: Option<Value>,
    /// The last run the app made.
    pub run: Option<LiveRun>,
    /// The proposal the person has to decide, or the one they decided.
    ///
    /// `None` when there is no proposal on the table.
    pub proposal: Option<ProposalBody>,
}

/// The last run of the app.
#[derive(Debug, Clone, Serialize)]
pub struct LiveRun {
    /// The seed of the run: the same seed reproduces the same run.
    pub seed: u64,
    /// The horizon of the run.
    pub until: Option<f64>,
    /// The `START` of the run: how many terminations end it.
    pub start: u64,
    /// The report of the run, the `genmsi.report/1` document.
    pub report: Option<Value>,
}

impl LiveBody {
    /// The state of an app with no model open and nothing to show.
    pub fn empty() -> Self {
        Self {
            path: None,
            pid: None,
            dirty: false,
            diagnostics: Vec::new(),
            model: None,
            run: None,
            proposal: None,
        }
    }
}

/// The live document, as the text the app writes.
pub fn live_document(body: &LiveBody) -> String {
    crate::documents::to_pretty_json(LIVE_SCHEMA, body)
}

/// The model as the app has it, as the `genmsi.model/1` document.
///
/// The value is the document with its envelope, so the reader of the live
/// state sees the same document the app publishes anywhere else.
pub fn model_value(model: &crate::documents::ModelDocument) -> Value {
    let text = crate::documents::to_pretty_json(crate::documents::MODEL_SCHEMA, model);
    serde_json::from_str(&text).unwrap_or(Value::Null)
}

/// The report of a run, as the value of the document it is.
///
/// The bytes are the ones the app already has: the report is published with
/// its envelope, and the live state carries it as it is.
pub fn report_value(bytes: &str) -> Option<Value> {
    serde_json::from_str(bytes).ok()
}

/// The file the live document lives in.
pub fn live_file() -> Option<PathBuf> {
    app_data_dir().map(|directory| directory.join(LIVE_FILE))
}

/// Write the live state of the app.
///
/// The write is atomic: the text goes to a temporary file next to the target
/// and that file is renamed over it, so a reader never sees half of one.
pub fn write_live(directory: &Path, body: &LiveBody) -> Result<(), AppDataError> {
    let file = directory.join(LIVE_FILE);
    let text = live_document(body);
    std::fs::create_dir_all(directory).map_err(|source| AppDataError::Write {
        path: directory.to_path_buf(),
        source,
    })?;
    let pid = body.pid.unwrap_or(0);
    let temporary = directory.join(format!("{LIVE_FILE}.{pid}.tmp"));
    std::fs::write(&temporary, text).map_err(|source| AppDataError::Write {
        path: temporary.clone(),
        source,
    })?;
    std::fs::rename(&temporary, &file).map_err(|source| {
        // The temporary file is not left behind: the directory of the app
        // does not fill with the remains of a failed write.
        let _ = std::fs::remove_file(&temporary);
        AppDataError::Write {
            path: file.clone(),
            source,
        }
    })
}

/// Read the live document of a directory.
///
/// Tolerant: a file that cannot be read is no live state, not a failure.
pub fn read_live_in(directory: &Path) -> Option<Value> {
    let text = std::fs::read_to_string(directory.join(LIVE_FILE)).ok()?;
    serde_json::from_str(&text).ok()
}

/// The live state of the app, when the app is still running.
///
/// A file left behind by an app that is gone says nothing: the pid of the
/// document says whether the process is alive.
pub fn current_live() -> Option<Value> {
    current_live_in(&app_data_dir()?)
}

/// The live state of the app, from a directory.
pub fn current_live_in(directory: &Path) -> Option<Value> {
    let document = read_live_in(directory)?;
    match document.get("pid").and_then(|pid| pid.as_u64()) {
        Some(pid) if !process_is_running(pid as u32) => None,
        _ => Some(document),
    }
}

/// The model the app has open, read from the live document.
///
/// The live document is the **only** publication of the app's state: it
/// carries the model it has open and the pid of the process that has it.
/// A document left behind by an app that is gone says nothing, and the pid
/// is what says so.
///
/// The document of the open model (`genmsi.openmodel/1`) used to be a second
/// file with the same two facts, and two files could disagree: an agent
/// read the path from one and the `dirty` from the other. There is one file
/// now, and this is how it is read.
pub fn open_model_from_live() -> Option<OpenModel> {
    open_model_from_live_in(&app_data_dir()?)
}

/// The model the app has open, read from the live document of a directory.
pub fn open_model_from_live_in(directory: &Path) -> Option<OpenModel> {
    let document = current_live_in(directory)?;
    let pid = document.get("pid").and_then(|pid| pid.as_u64())? as u32;
    let path = document
        .get("path")
        .and_then(|path| path.as_str())
        .map(str::to_string);
    Some(OpenModel { path, pid: Some(pid) })
}

/// The model the app has open, as the live document says it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenModel {
    /// The absolute path of the model the app has open.
    pub path: Option<String>,
    /// The pid of the process that has it open.
    pub pid: Option<u32>,
}

/// Whether the app has changes that are not saved on the model at `path`.
///
/// The agent asks this before it writes: a model the app has open with
/// changes that are not saved must not be written, because the app would not
/// reload it and the work would stay invisible on the canvas.
///
/// The paths are compared as they are on the disk: the same file named two
/// ways is the same file.
pub fn unsaved_changes_on(path: &Path) -> bool {
    match app_data_dir() {
        Some(directory) => unsaved_changes_in(&directory, path),
        None => false,
    }
}

/// Whether the app has changes that are not saved on the model at `path`,
/// reading the state from a directory.
pub fn unsaved_changes_in(directory: &Path, path: &Path) -> bool {
    let Some(document) = current_live_in(directory) else {
        return false;
    };
    if !document
        .get("dirty")
        .and_then(|dirty| dirty.as_bool())
        .unwrap_or(false)
    {
        return false;
    }
    let Some(open) = document.get("path").and_then(|path| path.as_str()) else {
        return false;
    };
    same_file(Path::new(open), path)
}

/// Whether two paths name the same file.
///
/// The paths are compared after resolving them: a relative path, a symlink and
/// an absolute path name the same file. A file that cannot be resolved is
/// compared as it was named: the agent may name a file that is not there yet.
pub fn same_file(left: &Path, right: &Path) -> bool {
    match (std::fs::canonicalize(left), std::fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => absolute(left) == absolute(right),
    }
}

/// The absolute path of a file, when it can be resolved.
fn absolute(path: &Path) -> PathBuf {
    match std::env::current_dir() {
        Ok(current) if path.is_relative() => current.join(path),
        _ => path.to_path_buf(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A directory that cleans itself up.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "genmsi-live-{tag}-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("the temporary directory is created");
            Self(path)
        }

        fn file(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// A body of an app with a model open and changes that are not saved.
    fn body(path: &Path, dirty: bool) -> LiveBody {
        LiveBody {
            path: Some(path.display().to_string()),
            pid: Some(std::process::id()),
            dirty,
            diagnostics: vec!["el modelo usa una entidad que no declara".to_string()],
            model: None,
            run: None,
            proposal: None,
        }
    }

    #[test]
    fn the_document_is_versioned_and_says_what_the_app_shows() {
        let dir = TempDir::new("document");
        let model = dir.file("modelo.gmdl");
        std::fs::write(&model, "<gmodel/>").expect("the model is there");
        let text = live_document(&body(&model, true));
        let document: Value = serde_json::from_str(&text).expect("the document is JSON");
        assert_eq!(document["schema"], LIVE_SCHEMA, "{text}");
        assert_eq!(document["dirty"], true, "{text}");
        assert_eq!(document["pid"], std::process::id(), "{text}");
        assert!(document["path"].is_string(), "{text}");
        assert_eq!(document["diagnostics"].as_array().map(|d| d.len()), Some(1), "{text}");
        assert!(document["model"].is_null(), "sin cambios, el modelo no viaja: {text}");
        assert!(document["run"].is_null(), "sin corrida, no hay informe: {text}");
    }

    #[test]
    fn what_is_written_can_be_read_back() {
        let dir = TempDir::new("roundtrip");
        let model = dir.file("modelo.gmdl");
        std::fs::write(&model, "<gmodel/>").expect("the model is there");
        write_live(&dir.0, &body(&model, true)).expect("the state is written");

        let document = read_live_in(&dir.0).expect("the state is read back");
        assert_eq!(document["dirty"], true, "{document}");
        assert_eq!(document["path"], model.display().to_string(), "{document}");
    }

    #[test]
    fn a_broken_file_is_no_state_and_not_a_failure() {
        let dir = TempDir::new("broken");
        std::fs::write(dir.file(LIVE_FILE), "{ esto no es json").expect("the file is there");
        assert!(read_live_in(&dir.0).is_none(), "un archivo roto no es estado");
        assert!(
            !unsaved_changes_in(&dir.0, Path::new("modelo.gmdl")),
            "sin estado, no hay cambios sin guardar"
        );
    }

    #[test]
    fn the_changes_without_saving_are_the_ones_of_the_model_of_the_app() {
        let dir = TempDir::new("dirty");
        let model = dir.file("modelo.gmdl");
        let other = dir.file("otro.gmdl");
        std::fs::write(&model, "<gmodel/>").expect("the model is there");
        std::fs::write(&other, "<gmodel/>").expect("the other model is there");

        write_live(&dir.0, &body(&model, true)).expect("the state is written");
        assert!(
            unsaved_changes_in(&dir.0, &model),
            "el modelo de la app tiene cambios sin guardar"
        );
        assert!(
            !unsaved_changes_in(&dir.0, &other),
            "el otro modelo no lo tiene"
        );

        // Sin cambios sin guardar, ningún modelo está trabado.
        write_live(&dir.0, &body(&model, false)).expect("the state is written");
        assert!(
            !unsaved_changes_in(&dir.0, &model),
            "sin cambios sin guardar se puede escribir"
        );
    }

    #[test]
    fn the_same_file_named_two_ways_is_the_same_file() {
        let dir = TempDir::new("same");
        let model = dir.file("modelo.gmdl");
        std::fs::write(&model, "<gmodel/>").expect("the model is there");
        let relative = dir.0.join(".");
        assert!(same_file(&model, &model), "la misma ruta");
        assert!(
            same_file(&model, &relative.join("modelo.gmdl")),
            "la misma ruta dicha de otra manera"
        );
        assert!(!same_file(&model, &dir.file("otro.gmdl")), "otro archivo");
    }
}
