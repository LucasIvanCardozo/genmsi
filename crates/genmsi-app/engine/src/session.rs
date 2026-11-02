//! The editing session: the model in memory, its undo stack and its save.
//!
//! The session is the app's single source of truth while a model is open. The
//! model lives in the facade, not in the frontend: the frontend sends
//! commands and draws what comes back, and every mutation goes through the
//! additive API of `genmsi-format`. The undo stack lives with the model, so
//! one operation is one undo entry no matter which backend sent it.
//!
//! Both backends share this code: the dev bridge keeps one session in a
//! long-lived process, and the desktop shell keeps the same session in memory.

use std::path::{Path, PathBuf};

use genmsi_core::documents::{lint_document, model_document, ModelDocument};
use genmsi_core::live::{LiveBody, LiveRun, model_value, report_value};
use genmsi_core::proposal::{ProposalBody, ProposalDocument, ProposalState};
use genmsi_format::edit::{EditCommand, EditSession};
use genmsi_format::model::{Block, ContainerHeader, GmdlModel, SectionKind};

use crate::{AppError, RunDocuments, model_error, run_model_documents_bytes, simulate_model_bytes};

/// The version code the original editor wrote in a new model.
const NEW_MODEL_VERSION: &str = "1.00.2008.03.23";

/// The generation preferences the original wrote in the model root.
const ROOT_PREFERENCES: &str = "1,1,1,1,1,0,0,0,0,0,0,0,0";

/// What the model in memory compiles to.
///
/// The diagnostics are continuous: they are computed again after every
/// mutation, and a model that does not compile is a state the app shows, not
/// an error that stops the editing.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    /// `ok` when the compiler was silent, `diagnostics` when it had
    /// observations, `error` when the model does not compile.
    pub status: &'static str,
    /// The compiler's observations and the format's own findings (the
    /// entities a block uses that the model does not declare).
    pub diagnostics: Vec<String>,
    /// The engine's message, when the model does not compile.
    pub message: Option<String>,
    /// The executable statements the model compiles to.
    pub blocks: Option<usize>,
    /// The definitions the model declares.
    pub definitions: Option<genmsi_core::documents::DefinitionCounts>,
}

/// The state of the session: what the app draws.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionState {
    /// Path of the model, as the caller named it; empty for a new model.
    pub path: Option<String>,
    /// The graph of the model.
    pub model: ModelDocument,
    /// What the model compiles to.
    pub diagnostics: Diagnostics,
    /// Whether the model has changes that were not saved.
    pub dirty: bool,
    /// Whether an operation can be undone.
    pub can_undo: bool,
    /// Whether an operation can be redone.
    pub can_redo: bool,
    /// The proposal the person has to decide, or the one they decided.
    ///
    /// `None` when there is no proposal on the table.
    pub proposal: Option<ProposalBody>,
    /// Whether the proposal is **on the canvas**.
    ///
    /// A proposal that was undone is still the proposal of the session, but
    /// it is not on the canvas: there is nothing to accept or to discard
    /// until it is adopted again.
    pub proposal_on_canvas: bool,
}

/// The last run of the session: the parameters it ran with and the documents
/// it gave.
///
/// The run stays with the session, because the app publishes it: an agent
/// that talks to the engine through the MCP has to see the run the reader
/// is looking at, with the parameters that produced it.
#[derive(Debug, Clone)]
pub struct LastRun {
    /// The seed of the run: the same seed reproduces the same run.
    pub seed: u64,
    /// The horizon of the run.
    pub until: Option<f64>,
    /// The `START` of the run: how many terminations end it.
    pub start: u64,
    /// The report of the run, byte for byte.
    pub report: Option<String>,
    /// The trace of the run, byte for byte.
    pub trace: Option<String>,
}

/// The outcome of a save.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveOutcome {
    /// Where the model was written.
    pub path: String,
    /// How many bytes the file has.
    pub bytes: usize,
}

/// An editor the session left behind, and what left it there.
///
/// A reload and the adoption of a proposal both leave an editor behind:
/// undoing them brings it back. The session knows which one was the
/// adoption, because the proposal is no longer on the canvas when the
/// adoption comes undone.
struct Previous {
    /// The editor that was there.
    editor: EditSession,
    /// Whether a proposal left it there.
    proposal: bool,
}

/// An editing session: one model in memory and its undo stack.
pub struct Session {
    /// The model and its undo stack.
    editor: EditSession,
    /// The sessions the reloads left behind.
    ///
    /// A reload is an operation of the session, like an edit: undoing it
    /// brings the session back, with the undo stack it had. The reader's
    /// work is therefore not lost when the model of the disk arrives.
    previous: Vec<Previous>,
    /// Where the model was opened from, as the caller named it.
    name: Option<String>,
    /// Where the model is written when it is saved.
    path: Option<PathBuf>,
    /// The last run the session made.
    last_run: Option<LastRun>,
    /// The proposal the session is holding: the one on the canvas, or the
    /// one the person already decided.
    ///
    /// The session keeps it until another proposal arrives: the verdict
    /// travels in the live state, and the agent has to be able to read it.
    proposal: Option<ProposalBody>,
    /// Whether the model of the proposal is the one on the canvas.
    ///
    /// It stops being there when the adoption comes undone or when the
    /// model of the disk is reloaded over it.
    adopted: bool,
}

impl Session {
    /// Open a model for editing.
    ///
    /// The model is read once and kept in memory: the edits happen there and
    /// reach the disk when the user saves.
    pub fn open(path: &Path, name: &str) -> Result<Self, AppError> {
        let model = genmsi_format::load(path).map_err(|error| model_error(path, error))?;
        Ok(Session {
            editor: EditSession::new(model),
            previous: Vec::new(),
            name: Some(name.to_string()),
            path: Some(path.to_path_buf()),
            last_run: None,
            proposal: None,
            adopted: false,
        })
    }

    /// Start a new, empty model: the root container and nothing else.
    ///
    /// The root is the container the original editor creates with a new file:
    /// it carries the generation preferences and no blocks.
    pub fn new_model() -> Self {
        let root = Block {
            id: 0,
            class: "main".to_string(),
            kind: 'C',
            owner: 0,
            name: "Modelo1".to_string(),
            caption: "Modelo".to_string(),
            layout: None,
            header: Some(ContainerHeader {
                genprefer: ROOT_PREFERENCES.to_string(),
                exehead: String::new(),
                nonexehead: String::new(),
                exefoot: String::new(),
                nonexefoot: String::new(),
            }),
            inputs: Vec::new(),
            outputs: Vec::new(),
            links: Vec::new(),
            props: Vec::new(),
            sections: vec![SectionKind::Links, SectionKind::Inputs, SectionKind::Outputs],
            source: None,
        };
        let model = GmdlModel {
            version: NEW_MODEL_VERSION.to_string(),
            has_classes: true,
            classes_self_closing: true,
            spaced_self_closing: false,
            classes: Vec::new(),
            blocks: vec![root],
            trailing: String::new(),
        };
        Session {
            editor: EditSession::new(model),
            previous: Vec::new(),
            name: None,
            path: None,
            last_run: None,
            proposal: None,
            adopted: false,
        }
    }

    /// Reload the model from the disk.
    ///
    /// The model is read and **compiled** before it replaces the one in
    /// memory: a model that does not compile leaves the session as it was,
    /// so the app keeps the model it had and shows the compiler's message.
    ///
    /// The session that was there does not go away: it goes to the history
    /// of the session, and undoing the reload brings it back with the undo
    /// stack it had. The reload does not take the reader's work away.
    pub fn reload(&mut self, path: &Path) -> Result<SessionState, AppError> {
        let model = genmsi_format::load(path).map_err(|error| model_error(path, error))?;
        genmsi_core::compile_model(&model).map_err(|error| model_error(path, error))?;
        let editor = std::mem::replace(&mut self.editor, EditSession::new(model));
        self.previous.push(Previous {
            editor,
            proposal: false,
        });
        // The model of the disk is on the canvas: the proposal, if there
        // was one, is no longer there.
        self.adopted = false;
        self.name = Some(path.display().to_string());
        self.path = Some(path.to_path_buf());
        Ok(self.state())
    }

    /// Adopt the model of a proposal: it enters the session **dirty** and
    /// **undoable**.
    ///
    /// The model of the proposal is read and **compiled** before it
    /// replaces the one in memory: a model that does not compile leaves the
    /// session as it was, like a reload does.
    ///
    /// The file the app has open does not move: a proposal does not change
    /// the model of the app, it offers another one. The editor that was
    /// there goes to the history of the session, and undoing the adoption
    /// brings it back with the undo stack it had.
    ///
    /// A session with changes that are not saved refuses the adoption: the
    /// work of the person is not stepped on, and the proposal stays
    /// `pending` until the model is saved. It is the rule of the brake,
    /// from the other side.
    ///
    /// The proposal is adopted only when it is about the model the app has
    /// open: the model of a proposal is the version of **that** file, and
    /// accepting a proposal about another one would put an unrelated model
    /// where the reader is working. A proposal about another file stays
    /// `pending`, and the app says why.
    pub fn adopt_proposal(&mut self, document: &ProposalDocument) -> Result<SessionState, AppError> {
        if self.editor.dirty() {
            return Err(AppError::Usage(
                "the model has changes that are not saved: the proposal waits until they are saved"
                    .to_string(),
            ));
        }
        if let Some(target) = document.target.as_deref() {
            let Some(path) = self.path.as_deref() else {
                return Err(AppError::Usage(
                    "the app has no model open: the proposal stays pending".to_string(),
                ));
            };
            if !genmsi_core::same_file(Path::new(target), path) {
                return Err(AppError::Usage(
                    "the proposal is about another model: it stays pending".to_string(),
                ));
            }
        }
        let path = Path::new(&document.model);
        let model = genmsi_format::load(path).map_err(|error| model_error(path, error))?;
        genmsi_core::compile_model(&model).map_err(|error| model_error(path, error))?;
        // The model that was there is the checkpoint of the one that
        // arrives: the session is dirty, and the adoption is one step of
        // the history.
        let adopted = EditSession::adopted(model, self.editor.model());
        let previous = std::mem::replace(&mut self.editor, adopted);
        self.previous.push(Previous {
            editor: previous,
            proposal: true,
        });
        self.adopted = true;
        // The session remembers the proposal: the person decides about it,
        // and the live state carries the verdict.
        self.proposal = Some(ProposalBody::from(document));
        Ok(self.state())
    }

    /// The proposal the session is holding.
    ///
    /// `None` when there is no proposal on the table.
    pub fn proposal(&self) -> Option<&ProposalBody> {
        self.proposal.as_ref()
    }

    /// Whether the model of the proposal is the one on the canvas.
    ///
    /// It stops being there when the adoption comes undone, when the model
    /// of the disk is reloaded over it and when the person decides.
    pub fn adopted(&self) -> bool {
        self.adopted
    }

    /// Go back to the model that was there before the adoption.
    ///
    /// Everything the person did after the adoption goes undone, and the
    /// editor that was there comes back with the undo stack it had: the
    /// model of the disk is on the canvas again, and the session is clean.
    pub fn discard_proposal(&mut self) -> Result<SessionState, AppError> {
        if !self.adopted {
            // The proposal is not on the canvas: there is nothing to go
            // back to.
            return Ok(self.state());
        }
        while self.editor.can_undo() {
            self.editor.undo().map_err(|error| AppError::Model {
                path: self.name.clone().unwrap_or_default(),
                message: error.to_string(),
            })?;
        }
        if let Some(previous) = self.previous.pop() {
            self.editor = previous.editor;
        }
        self.adopted = false;
        Ok(self.state())
    }

    /// The verdict the person gave to the proposal the session holds.
    ///
    /// The verdict travels in the live state: the agent reads it there and
    /// does not have to ask again.
    pub fn resolve_proposal(&mut self, state: ProposalState, note: Option<&str>) {
        if let Some(proposal) = self.proposal.as_mut() {
            proposal.state = state;
            proposal.note = note.map(|note| note.to_string());
        }
        // The person decided: the proposal is no longer on the canvas.
        self.adopted = false;
    }

    /// The model in memory.
    pub fn model(&self) -> &GmdlModel {
        self.editor.model()
    }

    /// The file the session has open.
    ///
    /// `None` when no model is open. It is the model a proposal has to be
    /// about for the app to adopt it.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// The state the app draws.
    pub fn state(&self) -> SessionState {
        let path = self.name.clone().unwrap_or_default();
        SessionState {
            path: self.name.clone(),
            model: model_document(self.editor.model(), &path),
            diagnostics: diagnostics_of(self.editor.model()),
            dirty: self.editor.dirty(),
            can_undo: self.can_undo(),
            can_redo: self.editor.can_redo(),
            proposal: self.proposal.clone(),
            proposal_on_canvas: self.adopted,
        }
    }

    /// Whether an operation can be undone.
    ///
    /// A reload is an operation: undoing it brings the session back, with
    /// the undo stack it had.
    pub fn can_undo(&self) -> bool {
        self.editor.can_undo() || !self.previous.is_empty()
    }

    /// Apply one editing command.
    ///
    /// The command may leave the model in a state the engine refuses: the
    /// editing continues and the diagnostics say what is wrong.
    pub fn apply(&mut self, command: EditCommand) -> Result<SessionState, AppError> {
        self.editor
            .apply(command)
            .map_err(|error| AppError::Model {
                path: self.name.clone().unwrap_or_default(),
                message: error.to_string(),
            })?;
        Ok(self.state())
    }

    /// Compose one scope of the model on the canvas.
    ///
    /// The reader sees one level of the model at a time —the root, or the
    /// content of a container— so the composition is the one of that scope:
    /// the blocks the scope owns and nothing else. The scope is the id of
    /// the container the reader is in; `0` is the root of the model.
    ///
    /// The positions come from the engine's composition of the scope
    /// (`genmsi_format::layout::relayout_scope`), the same algorithm the
    /// CLI applies to the whole model: there is one composition in the
    /// engine, and the app asks it for the scope the reader is looking at.
    ///
    /// The composition is applied as **one** command of positions, so it is
    /// one entry of the undo history, like every other edit. A scope with
    /// nothing to compose changes nothing: the model stays as it was and
    /// the undo stack does not grow.
    pub fn relayout(&mut self, scope: u32) -> Result<SessionState, AppError> {
        let composition = genmsi_format::layout::relayout_scope(self.editor.model(), scope);
        if composition.positions.is_empty() && composition.ports.is_empty() {
            return Ok(self.state());
        }
        self.apply(EditCommand::SetLayouts {
            placements: composition.placements(),
        })
    }

    /// Undo the last operation.
    ///
    /// When the model in memory has nothing to undo, the last reload is
    /// what comes undone: the session it left behind comes back, with the
    /// model and the undo stack it had.
    pub fn undo(&mut self) -> Result<SessionState, AppError> {
        if !self.editor.can_undo()
            && let Some(previous) = self.previous.pop()
        {
            // The adoption of a proposal is one step of the history: when
            // it comes undone, the proposal is no longer on the canvas.
            if previous.proposal {
                self.adopted = false;
            }
            self.editor = previous.editor;
            return Ok(self.state());
        }
        self.editor.undo().map_err(|error| AppError::Model {
            path: self.name.clone().unwrap_or_default(),
            message: error.to_string(),
        })?;
        Ok(self.state())
    }

    /// Redo the last undone operation.
    pub fn redo(&mut self) -> Result<SessionState, AppError> {
        self.editor.redo().map_err(|error| AppError::Model {
            path: self.name.clone().unwrap_or_default(),
            message: error.to_string(),
        })?;
        Ok(self.state())
    }

    /// Save the model, to the path it was opened from or to a new one.
    ///
    /// The write goes through the writer of `genmsi-format`, the same one the
    /// CLI uses, so the parts of the file that were not edited stay
    /// byte-identical.
    pub fn save(&mut self, path: Option<&Path>) -> Result<SaveOutcome, AppError> {
        let target = match path {
            Some(path) => path.to_path_buf(),
            None => self.path.clone().ok_or_else(|| {
                AppError::Usage(
                    "the model has no path yet: choose one with `Guardar como…`".to_string(),
                )
            })?,
        };
        let text = genmsi_format::to_string(self.editor.model());
        let bytes = text.len();
        std::fs::write(&target, text).map_err(|error| AppError::Model {
            path: target.display().to_string(),
            message: format!("cannot write model: {error}"),
        })?;
        self.path = Some(target.clone());
        self.name = Some(target.display().to_string());
        self.editor.mark_saved();
        Ok(SaveOutcome {
            path: target.display().to_string(),
            bytes,
        })
    }

    /// The last run the session made.
    ///
    /// `None` when the session did not run the model yet.
    pub fn last_run(&self) -> Option<&LastRun> {
        self.last_run.as_ref()
    }

    /// Keep the documents of a run and the parameters it ran with.
    ///
    /// The run may have happened off the thread of the window: the app
    /// keeps the model in memory free while the engine works, and the
    /// documents of the run come back when it is done. The session keeps
    /// them, so the app can publish what the reader is seeing.
    pub fn remember_run(
        &mut self,
        seed: u64,
        until: Option<f64>,
        start: u64,
        report: Option<String>,
        trace: Option<String>,
    ) {
        self.last_run = Some(LastRun {
            seed,
            until,
            start,
            report,
            trace,
        });
    }

    /// Run the model in memory and keep the documents of the run.
    ///
    /// The report and the trace come from **one** run, with the trace on:
    /// the engine is deterministic, so the two documents are the ones two
    /// separate runs would have given.
    pub fn run(
        &mut self,
        seed: u64,
        until: Option<f64>,
        start: u64,
        limit: usize,
    ) -> Result<RunDocuments, AppError> {
        let documents =
            run_model_documents_bytes(self.editor.model(), seed, until, start, limit, None)?;
        self.remember_run(
            seed,
            until,
            start,
            Some(documents.report.clone()),
            Some(documents.trace.clone()),
        );
        Ok(documents)
    }

    /// Run the model in memory for its report and keep it.
    pub fn simulate(
        &mut self,
        seed: u64,
        until: Option<f64>,
        start: u64,
    ) -> Result<String, AppError> {
        let document = simulate_model_bytes(self.editor.model(), seed, until, start, None)?;
        self.remember_run(seed, until, start, Some(document.clone()), None);
        Ok(document)
    }

    /// The live state of the app, as the core publishes it.
    ///
    /// The model in memory travels **only** when the reader has changes
    /// that are not saved: when it does not, the file *is* the model and
    /// the agent reads it from there.
    pub fn live_body(&self) -> LiveBody {
        self.live_body_in(None)
    }

    /// The live state of the app, with the directory the proposals live in.
    ///
    /// The proposal the person is deciding travels in it: the one the
    /// session holds (with its verdict) or, when there is none, the one that
    /// waits for the person. An agent that left a proposal always finds out
    /// what happened to it: it is waiting, or it was accepted, or it was
    /// discarded with a note.
    pub fn live_body_in(&self, directory: Option<&Path>) -> LiveBody {
        let state = self.state();
        let pending = directory
            .and_then(|directory| crate::proposal::pending_for(directory, self.path()))
            .map(|document| ProposalBody::from(&document));
        LiveBody {
            path: state.path.as_deref().map(absolute_path),
            pid: Some(std::process::id()),
            dirty: state.dirty,
            diagnostics: state.diagnostics.diagnostics.clone(),
            model: state.dirty.then(|| model_value(&state.model)),
            run: self.last_run.as_ref().map(|run| LiveRun {
                seed: run.seed,
                until: run.until,
                start: run.start,
                report: run.report.as_deref().and_then(report_value),
            }),
            proposal: self.proposal.clone().or(pending),
        }
    }
}

/// The live state of an app that has no model open.
///
/// The app is running: the document says so, with the pid of the process,
/// and it says that no model is open.
pub fn empty_live_body() -> LiveBody {
    LiveBody {
        path: None,
        pid: Some(std::process::id()),
        dirty: false,
        diagnostics: Vec::new(),
        model: None,
        run: None,
        proposal: None,
    }
}

/// The absolute path of a model, as the machine sees it.
///
/// The agent that reads the live state has to be able to find the model
/// without knowing where the app was started from.
fn absolute_path(path: &str) -> String {
    std::path::absolute(path)
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| path.to_string())
}

/// What the model compiles to.
///
/// A model that does not compile is a state the app shows: the engine's
/// message, without the path (the app shows the path on its own).
///
/// The format's findings come along: a block that uses an entity the model
/// does not declare (a storage nobody declares, for instance) is a warning
/// the app shows next to the compiler's observations. It is a warning and
/// not an error: the model still compiles, and the original opens a model
/// like that too.
///
/// The engine reports the storages itself, because a storage nobody declares
/// has no capacity and the run stops when a transaction reaches it. The app
/// shows that finding and does not repeat the format's warning about the same
/// storage.
///
/// The findings are the ones the engine's lint document carries, so the app
/// and the CLI say the same thing about the same model.
pub fn diagnostics_of(model: &GmdlModel) -> Diagnostics {
    let warnings = genmsi_format::declaration_warnings(model);
    match genmsi_core::compile_model(model) {
        Ok(program) => {
            let lint = lint_document(&program, model, "");
            Diagnostics {
                status: lint.status,
                diagnostics: lint.diagnostics,
                message: None,
                blocks: Some(lint.blocks),
                definitions: Some(lint.definitions.clone()),
            }
        }
        Err(error) => Diagnostics {
            status: "error",
            diagnostics: warnings,
            message: Some(error.to_string()),
            blocks: None,
            definitions: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use genmsi_core::proposal::{ProposalDocument, ProposalState};

    /// The reference model of the app's slice.
    fn reference() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../fixtures/models/TP6__Clase 6_1_PuertaBanco.gmdl")
    }

    /// A directory that cleans itself up.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "genmsi-app-session-{tag}-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("the temporary directory is created");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// The model the app has open: a copy of the reference, in a directory
    /// of its own, so the tests never write in the repository.
    fn open_model(directory: &Path) -> PathBuf {
        let text = std::fs::read_to_string(reference()).expect("the reference model is there");
        let path = directory.join("open.gmdl");
        std::fs::write(&path, text).expect("the model is written");
        path
    }

    /// The model an agent proposes: the same model, with another caption.
    fn proposed_model(directory: &Path) -> PathBuf {
        let mut model = genmsi_format::load(reference()).expect("the reference model loads");
        model.blocks[0].caption = "Modelo propuesto".to_string();
        let path = directory.join("proposed.gmdl");
        std::fs::write(&path, genmsi_format::to_string(&model)).expect("the model is written");
        path
    }

    /// The caption of the model that is on the canvas.
    fn caption(session: &Session) -> String {
        session.model().blocks[0].caption.clone()
    }

    /// A session with the model of the reference open.
    fn session(directory: &Path) -> Session {
        let open = open_model(directory);
        Session::open(&open, &open.display().to_string()).expect("the model opens")
    }

    #[test]
    fn adopting_a_proposal_leaves_the_session_dirty_and_the_path_unchanged() {
        let directory = TempDir::new("adopt");
        let open = open_model(directory.path());
        let proposed = proposed_model(directory.path());
        let mut session = Session::open(&open, &open.display().to_string()).expect("the model opens");
        let document = ProposalDocument::pending("1700000000000-ab", &proposed, Some(&open))
            .with_summary(Some("the bus loop"));

        let state = session.adopt_proposal(&document).expect("the proposal is adopted");
        // The model of the proposal is on the canvas.
        assert_eq!(caption(&session), "Modelo propuesto");
        // The session is dirty: the model differs from the disk.
        assert!(state.dirty, "the session is not dirty: {state:?}");
        // The file the app has open does not move: a proposal does not
        // change the model of the app.
        assert_eq!(state.path.as_deref(), Some(open.display().to_string().as_str()));
        // The session remembers the proposal.
        let proposal = session.proposal().expect("the session holds the proposal");
        assert_eq!(proposal.id, "1700000000000-ab");
        assert_eq!(proposal.summary.as_deref(), Some("the bus loop"));
        assert_eq!(proposal.state, ProposalState::Pending);
    }

    #[test]
    fn undoing_the_adoption_brings_the_model_that_was_there_back() {
        let directory = TempDir::new("undo");
        let open = open_model(directory.path());
        let proposed = proposed_model(directory.path());
        let mut session = Session::open(&open, &open.display().to_string()).expect("the model opens");
        let document = ProposalDocument::pending("1700000000000-ab", &proposed, Some(&open));
        session.adopt_proposal(&document).expect("the proposal is adopted");

        let state = session.undo().expect("the adoption is undone");
        assert_eq!(caption(&session), "Modelo", "the model that was there is not back");
        assert!(!state.dirty, "the session is still dirty: {state:?}");
    }

    #[test]
    fn a_proposal_about_another_model_is_not_adopted() {
        let directory = TempDir::new("other-target");
        let open = open_model(directory.path());
        let other = proposed_model(directory.path());
        let mut session = Session::open(&open, &open.display().to_string()).expect("the model opens");
        // The proposal is about another file: the model it carries is the
        // version of that file, not of the one the app has open.
        let document = ProposalDocument::pending("1700000000000-ab", &other, Some(&other));

        let error = session
            .adopt_proposal(&document)
            .expect_err("the adoption is refused");
        assert!(
            error.to_string().contains("about another model"),
            "the message does not say why: {error}"
        );
        // The canvas did not move.
        assert_eq!(caption(&session), "Modelo");
        assert!(session.proposal().is_none(), "the session holds a proposal");
    }

    #[test]
    fn a_proposal_without_a_target_is_adopted() {
        let directory = TempDir::new("no-target");
        let open = open_model(directory.path());
        let proposed = proposed_model(directory.path());
        let mut session = Session::open(&open, &open.display().to_string()).expect("the model opens");
        // A proposal without a target does not constrain the adoption.
        let document = ProposalDocument::pending("1700000000000-ab", &proposed, None);

        let state = session.adopt_proposal(&document).expect("the proposal is adopted");
        assert!(state.dirty, "the session is not dirty: {state:?}");
        assert_eq!(caption(&session), "Modelo propuesto");
    }

    #[test]
    fn a_dirty_session_refuses_the_adoption() {
        let directory = TempDir::new("dirty");
        let open = open_model(directory.path());
        let proposed = proposed_model(directory.path());
        let mut session = Session::open(&open, &open.display().to_string()).expect("the model opens");
        session
            .apply(EditCommand::SetCaption {
                id: 0,
                caption: "otro".to_string(),
            })
            .expect("the caption changes");

        let document = ProposalDocument::pending("1700000000000-ab", &proposed, Some(&open));
        let error = session
            .adopt_proposal(&document)
            .expect_err("the adoption is refused");
        assert!(
            error.to_string().contains("changes that are not saved"),
            "the message does not say why: {error}"
        );
        // The model of the person is untouched.
        assert_eq!(caption(&session), "otro");
        assert!(session.proposal().is_none(), "the session holds a proposal");
    }

    #[test]
    fn the_live_body_carries_the_proposal_and_its_verdict() {
        let directory = TempDir::new("live");
        let open = open_model(directory.path());
        let proposed = proposed_model(directory.path());
        let mut session = Session::open(&open, &open.display().to_string()).expect("the model opens");
        let document = ProposalDocument::pending("1700000000000-ab", &proposed, Some(&open))
            .with_summary(Some("the bus loop"));
        session.adopt_proposal(&document).expect("the proposal is adopted");

        let body = session.live_body();
        let proposal = body.proposal.expect("the proposal travels");
        assert_eq!(proposal.id, "1700000000000-ab");
        assert_eq!(proposal.state, ProposalState::Pending);
        assert_eq!(proposal.summary.as_deref(), Some("the bus loop"));

        // The verdict travels too.
        session.resolve_proposal(ProposalState::Rejected, Some("not yet"));
        let body = session.live_body();
        let proposal = body.proposal.expect("the verdict travels");
        assert_eq!(proposal.state, ProposalState::Rejected);
        assert_eq!(proposal.note.as_deref(), Some("not yet"));
    }

    #[test]
    fn a_session_without_a_proposal_publishes_none() {
        let directory = TempDir::new("no-proposal");
        let session = session(directory.path());
        assert!(session.live_body().proposal.is_none());
        assert!(session.state().proposal.is_none());
    }
}
