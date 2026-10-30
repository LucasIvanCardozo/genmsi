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
use genmsi_format::edit::{EditCommand, EditSession};
use genmsi_format::model::{Block, ContainerHeader, GmdlModel, SectionKind};

use crate::{AppError, model_error};

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

/// An editing session: one model in memory and its undo stack.
pub struct Session {
    /// The model and its undo stack.
    editor: EditSession,
    /// The sessions the reloads left behind.
    ///
    /// A reload is an operation of the session, like an edit: undoing it
    /// brings the session back, with the undo stack it had. The reader's
    /// work is therefore not lost when the model of the disk arrives.
    previous: Vec<EditSession>,
    /// Where the model was opened from, as the caller named it.
    name: Option<String>,
    /// Where the model is written when it is saved.
    path: Option<PathBuf>,
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
        self.previous.push(editor);
        self.name = Some(path.display().to_string());
        self.path = Some(path.to_path_buf());
        Ok(self.state())
    }

    /// The model in memory.
    pub fn model(&self) -> &GmdlModel {
        self.editor.model()
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
            self.editor = previous;
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
