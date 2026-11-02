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
    AppError, RunDocuments, catalogue_document_bytes, lint_document_bytes, model_document_bytes,
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
    /// Reload the model from the disk.
    ///
    /// The model is compiled before it replaces the one in memory: a model
    /// that does not compile leaves the session as it was, and the answer
    /// says what the compiler said.
    Reload {
        /// Path of the model.
        path: String,
    },
    /// Apply one editing command.
    Apply {
        /// The command to apply.
        command: EditCommand,
    },
    /// Compose one scope of the model on the canvas.
    ///
    /// The request carries the **scope**: the container the reader is
    /// looking at, `0` for the root of the model. The reader sees one
    /// level at a time, so the request composes that level and nothing
    /// else: the blocks of the scope move and no block of any other scope
    /// does.
    ///
    /// The positions come from the engine's composition of that scope, so
    /// the app and the engine agree on what the scope looks like.
    Relayout {
        /// The scope to compose: the container the reader is in.
        scope: u32,
    },
    /// Undo the last operation.
    Undo,
    /// Redo the last undone operation.
    Redo,
    /// The state of the session, without changing it.
    State,
    /// The proposal that waits for the person, if there is one.
    ///
    /// The answer carries the document of the proposal —the
    /// `genmsi.proposal/1` document of the core— or nothing at all when
    /// there is no proposal waiting.
    PendingProposal,
    /// Adopt the proposal that waits for the person.
    ///
    /// The model of the proposal enters the session dirty and undoable:
    /// the file the app has open does not move. A session with changes that
    /// are not saved refuses the adoption.
    AdoptProposal,
    /// Accept the proposal the session holds.
    ///
    /// The model in memory is written to the file the app has open —the
    /// normal save of the app— and the proposal becomes `accepted`.
    AcceptProposal,
    /// Discard the proposal the session holds.
    ///
    /// The model that was there comes back to the canvas, and the proposal
    /// becomes `rejected`, with the note the person wrote.
    DiscardProposal {
        /// What the person answered.
        note: Option<String>,
    },
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
        /// The `START` of the run: how many termination units end it.
        #[serde(default)]
        start: Option<u64>,
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
        /// The `START` of the run: how many termination units end it.
        #[serde(default)]
        start: Option<u64>,
    },
    /// The trace of a run of a file on disk.
    Trace {
        /// Path of the model.
        path: String,
        /// Seed of the run.
        seed: Option<u64>,
        /// Stop the run at this simulation time.
        until: Option<f64>,
        /// The `START` of the run: how many termination units end it.
        #[serde(default)]
        start: Option<u64>,
        /// Maximum number of trace records kept.
        limit: Option<usize>,
    },
    /// The trace of a run of the model in memory.
    TraceCurrent {
        /// Seed of the run.
        seed: Option<u64>,
        /// Stop the run at this simulation time.
        until: Option<f64>,
        /// The `START` of the run: how many termination units end it.
        #[serde(default)]
        start: Option<u64>,
        /// Maximum number of trace records kept.
        limit: Option<usize>,
    },
    /// The report **and** the trace of one run of the model in memory.
    ///
    /// The engine runs once, with the trace on: the two documents come from
    /// the same run.
    RunCurrent {
        /// Seed of the run.
        seed: Option<u64>,
        /// Stop the run at this simulation time.
        until: Option<f64>,
        /// The `START` of the run: how many termination units end it.
        #[serde(default)]
        start: Option<u64>,
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
    /// The two documents of one run: the report and the trace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documents: Option<RunDocuments>,
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
            documents: None,
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
            documents: None,
            path: None,
            bytes: None,
        }
    }

    /// The answer that carries the two documents of one run.
    pub fn documents(documents: RunDocuments) -> Self {
        Self {
            ok: true,
            error: None,
            state: None,
            document: None,
            documents: Some(documents),
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
            documents: None,
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
            documents: None,
            path: None,
            bytes: None,
        }
    }

    /// The answer of a request that has nothing to say.
    ///
    /// The request that asks for the proposal that waits answers this way
    /// when there is no proposal: there is nothing to decide, and that is
    /// not a failure.
    pub fn ok() -> Self {
        Self {
            ok: true,
            error: None,
            state: None,
            document: None,
            documents: None,
            path: None,
            bytes: None,
        }
    }
}

/// Answer one request.
///
/// The editing operations need an open model; the read-only ones work
/// without one, because they answer about a file on disk.
///
/// The proposals live in the data directory of the app: the requests that
/// ask for one, that adopt one or that resolve one need to know where they
/// are. The directory the core resolves is the one the desktop shell
/// resolves too.
pub fn answer(session: &mut Option<Session>, request: SessionRequest) -> SessionResponse {
    let directory = genmsi_core::app_data_dir();
    answer_in(session, request, directory.as_deref())
}

/// Answer one request, with the data directory of the app.
///
/// The directory is where the proposals live. `None` when the machine does
/// not say where the data of the app lives: the requests that need it say
/// so, instead of looking for the proposals in the wrong place.
pub fn answer_in(
    session: &mut Option<Session>,
    request: SessionRequest,
    directory: Option<&Path>,
) -> SessionResponse {
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
        SessionRequest::Reload { path } => {
            with_session(session, |session| session.reload(Path::new(&path)))
        }
        SessionRequest::Apply { command } => with_session(session, |session| session.apply(command)),
        SessionRequest::Relayout { scope } => {
            with_session(session, |session| session.relayout(scope))
        }
        SessionRequest::Undo => with_session(session, |session| session.undo()),
        SessionRequest::Redo => with_session(session, |session| session.redo()),
        SessionRequest::State => with_session(session, |session| Ok(session.state())),
        SessionRequest::PendingProposal => {
            let Some(directory) = directory else {
                return SessionResponse::error(AppError::Usage(no_data_dir()));
            };
            // The proposal that waits is the one about the model the app
            // has open: a proposal about another file is not this app's
            // business, and it does not hold the queue.
            let open = session.as_ref().and_then(|session| session.path());
            match crate::proposal::pending_document_for(directory, open) {
                Some(document) => SessionResponse::document(document),
                // There is no proposal waiting: there is nothing to decide,
                // and that is not a failure.
                None => SessionResponse::ok(),
            }
        }
        SessionRequest::AdoptProposal => match session.as_mut() {
            Some(session) => {
                let Some(directory) = directory else {
                    return SessionResponse::error(AppError::Usage(no_data_dir()));
                };
                match crate::proposal::adopt_in(session, directory) {
                    Ok(state) => SessionResponse::state(state),
                    Err(error) => SessionResponse::error(error),
                }
            }
            None => SessionResponse::error(AppError::Usage("no model is open".to_string())),
        },
        SessionRequest::AcceptProposal => match session.as_mut() {
            Some(session) => {
                let Some(directory) = directory else {
                    return SessionResponse::error(AppError::Usage(no_data_dir()));
                };
                match crate::proposal::accept_in(session, directory) {
                    Ok((state, outcome)) => SessionResponse::saved(outcome, state),
                    Err(error) => SessionResponse::error(error),
                }
            }
            None => SessionResponse::error(AppError::Usage("no model is open".to_string())),
        },
        SessionRequest::DiscardProposal { note } => match session.as_mut() {
            Some(session) => {
                let Some(directory) = directory else {
                    return SessionResponse::error(AppError::Usage(no_data_dir()));
                };
                match crate::proposal::discard_in(session, directory, note.as_deref()) {
                    Ok(state) => SessionResponse::state(state),
                    Err(error) => SessionResponse::error(error),
                }
            }
            None => SessionResponse::error(AppError::Usage("no model is open".to_string())),
        },
        SessionRequest::Save { path } => match session.as_mut() {
            Some(session) => match session.save(path.as_deref().map(Path::new)) {
                Ok(outcome) => SessionResponse::saved(outcome, session.state()),
                Err(error) => SessionResponse::error(error),
            },
            None => SessionResponse::error(AppError::Usage("no model is open".to_string())),
        },
        SessionRequest::SimulateCurrent { seed, until, start } => match session.as_mut() {
            Some(session) => match session.simulate(seed.unwrap_or(1), until, start_of(start)) {
                Ok(document) => SessionResponse::document(document),
                Err(error) => SessionResponse::error(error),
            },
            None => SessionResponse::error(AppError::Usage("no model is open".to_string())),
        },
        SessionRequest::Lint { path } => match lint_document_bytes(Path::new(&path)) {
            Ok(document) => SessionResponse::document(document),
            Err(error) => SessionResponse::error(error),
        },
        SessionRequest::Simulate { path, seed, until, start } => {
            match simulate_document_bytes(
                Path::new(&path),
                seed.unwrap_or(1),
                until,
                start_of(start),
                None,
            ) {
                Ok(document) => SessionResponse::document(document),
                Err(error) => SessionResponse::error(error),
            }
        }
        SessionRequest::Trace {
            path,
            seed,
            until,
            start,
            limit,
        } => match trace_document_bytes(
            Path::new(&path),
            seed.unwrap_or(1),
            until,
            start_of(start),
            limit.unwrap_or(100),
            None,
        ) {
            Ok(document) => SessionResponse::document(document),
            Err(error) => SessionResponse::error(error),
        },
        SessionRequest::TraceCurrent {
            seed,
            until,
            start,
            limit,
        } => match session.as_mut() {
            Some(session) => match trace_model_bytes(
                session.model(),
                seed.unwrap_or(1),
                until,
                start_of(start),
                limit.unwrap_or(100),
                None,
            ) {
                Ok(document) => SessionResponse::document(document),
                Err(error) => SessionResponse::error(error),
            },
            None => SessionResponse::error(AppError::Usage("no model is open".to_string())),
        },
        SessionRequest::RunCurrent {
            seed,
            until,
            start,
            limit,
        } => match session.as_mut() {
            Some(session) => match session.run(
                seed.unwrap_or(1),
                until,
                start_of(start),
                limit.unwrap_or(100),
            ) {
                Ok(documents) => SessionResponse::documents(documents),
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

/// The message of a request that needs the data directory of the app and
/// the machine does not say where it lives.
///
/// The proposals live there: without the directory the app cannot look for
/// them, and it says so instead of looking in the wrong place.
fn no_data_dir() -> String {
    "the app does not know where its data lives: the proposals cannot be read".to_string()
}

/// The `START` of a run, or the default when the request does not name one.
///
/// The `START` is the counter of terminations of the run: the original has it
/// in the panel of its simulator ("Start:", one by default) and the model
/// does not declare it, so it travels with the request.
fn start_of(start: Option<u64>) -> u64 {
    start.unwrap_or(genmsi_core::DEFAULT_START)
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
