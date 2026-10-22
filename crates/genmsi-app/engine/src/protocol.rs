//! The protocol the app's backends speak with the facade.
//!
//! One request is one JSON object and one answer is one JSON object. The dev
//! bridge forwards the bytes of the request to a long-lived session process
//! and returns the bytes of the answer; the desktop shell calls the same
//! functions. Neither backend builds a document of its own: the answers carry
//! the engine's bytes, or the state of the session.

use std::path::Path;

use genmsi_format::edit::EditCommand;

use crate::session::{SaveOutcome, Session, SessionState};
use crate::{
    AppError, catalogue_document_bytes, lint_document_bytes, model_document_bytes,
    models_document_bytes, simulate_document_bytes, trace_document_bytes, trace_model_bytes,
};

/// One request of the protocol.
#[derive(Debug, serde::Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum SessionRequest {
    /// Open a model for editing.
    Open {
        /// Path of the model.
        path: String,
    },
    /// Start a new, empty model.
    New,
    /// Apply one editing command.
    Apply {
        /// The command to apply.
        command: EditCommand,
    },
    /// Undo the last operation.
    Undo,
    /// Redo the last undone operation.
    Redo,
    /// The state of the session, without changing it.
    State,
    /// Save the model; without a path it goes to the one it was opened from.
    Save {
        /// Where to write the model.
        path: Option<String>,
    },
    /// The report of a run of the model in memory.
    SimulateCurrent {
        /// Seed of the run.
        seed: Option<u64>,
        /// Stop the run at this simulation time.
        until: Option<f64>,
    },
    /// The lint of a file on disk.
    Lint {
        /// Path of the model.
        path: String,
    },
    /// The report of a run of a file on disk.
    Simulate {
        /// Path of the model.
        path: String,
        /// Seed of the run.
        seed: Option<u64>,
        /// Stop the run at this simulation time.
        until: Option<f64>,
    },
    /// The trace of a run of a file on disk.
    Trace {
        /// Path of the model.
        path: String,
        /// Seed of the run.
        seed: Option<u64>,
        /// Stop the run at this simulation time.
        until: Option<f64>,
        /// Maximum number of trace records kept.
        limit: Option<usize>,
    },
    /// The trace of a run of the model in memory.
    TraceCurrent {
        /// Seed of the run.
        seed: Option<u64>,
        /// Stop the run at this simulation time.
        until: Option<f64>,
        /// Maximum number of trace records kept.
        limit: Option<usize>,
    },
    /// The graph of a file on disk.
    Model {
        /// Path of the model.
        path: String,
    },
    /// The models of the given directories.
    Models {
        /// Directories to scan.
        dirs: Vec<String>,
    },
    /// The catalogue of the block classes.
    Catalogue,
}

/// The answer of the facade to one request.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    /// Whether the request succeeded.
    pub ok: bool,
    /// The message of a failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// The state of the session, for the editing operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<SessionState>,
    /// A document of the engine, byte for byte as the CLI prints it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<String>,
    /// Where a model was written.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// How many bytes the written model has.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<usize>,
}

impl SessionResponse {
    /// The answer that carries the state of the session.
    pub fn state(state: SessionState) -> Self {
        Self {
            ok: true,
            error: None,
            state: Some(state),
            document: None,
            path: None,
            bytes: None,
        }
    }

    /// The answer that carries a document of the engine.
    pub fn document(document: String) -> Self {
        Self {
            ok: true,
            error: None,
            state: None,
            document: Some(document),
            path: None,
            bytes: None,
        }
    }

    /// The answer that carries the outcome of a save.
    pub fn saved(outcome: SaveOutcome, state: SessionState) -> Self {
        Self {
            ok: true,
            error: None,
            state: Some(state),
            document: None,
            path: Some(outcome.path),
            bytes: Some(outcome.bytes),
        }
    }

    /// The answer that carries a failure.
    pub fn error(error: AppError) -> Self {
        Self {
            ok: false,
            error: Some(error.to_string()),
            state: None,
            document: None,
            path: None,
            bytes: None,
        }
    }
}

/// Answer one request.
///
/// The editing operations need an open model; the read-only ones work
/// without one, because they answer about a file on disk.
pub fn answer(session: &mut Option<Session>, request: SessionRequest) -> SessionResponse {
    match request {
        SessionRequest::Open { path } => match Session::open(Path::new(&path), &path) {
            Ok(opened) => {
                let state = opened.state();
                *session = Some(opened);
                SessionResponse::state(state)
            }
            Err(error) => SessionResponse::error(error),
        },
        SessionRequest::New => {
            let opened = Session::new_model();
            let state = opened.state();
            *session = Some(opened);
            SessionResponse::state(state)
        }
        SessionRequest::Apply { command } => with_session(session, |session| session.apply(command)),
        SessionRequest::Undo => with_session(session, |session| session.undo()),
        SessionRequest::Redo => with_session(session, |session| session.redo()),
        SessionRequest::State => with_session(session, |session| Ok(session.state())),
        SessionRequest::Save { path } => match session.as_mut() {
            Some(session) => match session.save(path.as_deref().map(Path::new)) {
                Ok(outcome) => SessionResponse::saved(outcome, session.state()),
                Err(error) => SessionResponse::error(error),
            },
            None => SessionResponse::error(AppError::Usage("no model is open".to_string())),
        },
        SessionRequest::SimulateCurrent { seed, until } => match session.as_mut() {
            Some(session) => {
                match crate::simulate_model_bytes(session.model(), seed.unwrap_or(1), until) {
                    Ok(document) => SessionResponse::document(document),
                    Err(error) => SessionResponse::error(error),
                }
            }
            None => SessionResponse::error(AppError::Usage("no model is open".to_string())),
        },
        SessionRequest::Lint { path } => match lint_document_bytes(Path::new(&path)) {
            Ok(document) => SessionResponse::document(document),
            Err(error) => SessionResponse::error(error),
        },
        SessionRequest::Simulate { path, seed, until } => {
            match simulate_document_bytes(Path::new(&path), seed.unwrap_or(1), until) {
                Ok(document) => SessionResponse::document(document),
                Err(error) => SessionResponse::error(error),
            }
        }
        SessionRequest::Trace {
            path,
            seed,
            until,
            limit,
        } => match trace_document_bytes(
            Path::new(&path),
            seed.unwrap_or(1),
            until,
            limit.unwrap_or(100),
        ) {
            Ok(document) => SessionResponse::document(document),
            Err(error) => SessionResponse::error(error),
        },
        SessionRequest::TraceCurrent {
            seed,
            until,
            limit,
        } => match session.as_mut() {
            Some(session) => match trace_model_bytes(
                session.model(),
                seed.unwrap_or(1),
                until,
                limit.unwrap_or(100),
            ) {
                Ok(document) => SessionResponse::document(document),
                Err(error) => SessionResponse::error(error),
            },
            None => SessionResponse::error(AppError::Usage("no model is open".to_string())),
        },
        SessionRequest::Model { path } => match model_document_bytes(Path::new(&path)) {
            Ok(document) => SessionResponse::document(document),
            Err(error) => SessionResponse::error(error),
        },
        SessionRequest::Models { dirs } => {
            let dirs: Vec<std::path::PathBuf> = dirs.iter().map(std::path::PathBuf::from).collect();
            match models_document_bytes(&dirs) {
                Ok(document) => SessionResponse::document(document),
                Err(error) => SessionResponse::error(error),
            }
        }
        SessionRequest::Catalogue => SessionResponse::document(catalogue_document_bytes()),
    }
}

/// Run one operation of the session, or say that no model is open.
fn with_session(
    session: &mut Option<Session>,
    operation: impl FnOnce(&mut Session) -> Result<SessionState, AppError>,
) -> SessionResponse {
    match session.as_mut() {
        Some(session) => match operation(session) {
            Ok(state) => SessionResponse::state(state),
            Err(error) => SessionResponse::error(error),
        },
        None => SessionResponse::error(AppError::Usage("no model is open".to_string())),
    }
}
