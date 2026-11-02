//! The engine adapter: the only place of the crate that calls the engine.
//!
//! The crate carries no simulation logic (decision D2): it loads a model,
//! asks the engine to compile and run it, and renders the documents the
//! engine already produces. The CLI does exactly the same through the same
//! API, so both clients show the same numbers.

use std::path::PathBuf;

use genmsi_core::{Program, RunResult, SimConfig, Simulation};
use genmsi_format::{EditCommand, EditSession, GmdlModel};
use rmcp::schemars;
use rmcp::schemars::JsonSchema;
use serde::Deserialize;

use crate::errors::{Failure, ToolError};

/// The name a model gets when it never touched the disk.
pub const MEMORY: &str = "<memory>";

/// Where a model comes from.
#[derive(Debug, Clone)]
pub enum ModelSource {
    /// A `.gmdl` file on disk.
    File(PathBuf),
    /// The `.gmdl` document as text.
    Inline(String),
}

impl ModelSource {
    /// The name the agent used for the model.
    pub fn name(&self) -> String {
        match self {
            Self::File(path) => path.display().to_string(),
            Self::Inline(_) => MEMORY.to_string(),
        }
    }
}

/// A model that loaded, validated and compiled.
#[derive(Debug)]
pub struct Loaded {
    /// The compiled program.
    pub program: Program,
    /// The model, kept to locate the blocks a failure points at.
    pub model: GmdlModel,
    /// The name the agent used for the model.
    pub name: String,
}

/// Load, validate and compile a model.
///
/// A model that does not load, does not validate or does not compile fails
/// with the block that caused it, never with a panic and never with an empty
/// document.
pub fn load(source: &ModelSource) -> Result<Loaded, Failure> {
    let name = source.name();
    let model = match source {
        ModelSource::File(path) => {
            // The CLI loads through `parse_file` too, so both clients
            // classify a file the same way: a missing file is its own class,
            // and every other failure to read or parse it is the model
            // failure the CLI prints.
            genmsi_format::parse_file(path).map_err(|error| {
                if error.is_missing_file() {
                    ToolError::not_found(name.clone())
                } else {
                    ToolError::from_format(&error, Some(&name), None)
                }
            })?
        }
        ModelSource::Inline(text) => genmsi_format::parse_str(text)
            .map_err(|error| ToolError::from_format(&error, Some(&name), None))?,
    };
    // The structural rules are checked here, with the model in hand, so a
    // failure can point at the block that breaks them.
    genmsi_format::validate(&model)
        .map_err(|error| ToolError::from_format(&error, Some(&name), Some(&model)))?;
    let program = genmsi_core::compile_model(&model)
        .map_err(|error| ToolError::from_engine(&error, &model, &name))?;
    Ok(Loaded {
        program,
        model,
        name,
    })
}

/// A model that was written to a file.
#[derive(Debug)]
pub struct Saved {
    /// The compiled program: the model compiles, that is why it was written.
    pub program: Program,
    /// The model as it was written.
    pub model: GmdlModel,
    /// Where it was written.
    pub path: PathBuf,
    /// How many bytes the file has.
    pub bytes: u64,
}

/// A model an edit changed.
#[derive(Debug)]
pub struct Edited {
    /// The compiled program: the model compiles, that is why it was written.
    pub program: Program,
    /// The model as it was edited.
    pub model: GmdlModel,
    /// The name the agent used for the model.
    pub name: String,
    /// Whether the model was written to the file it was named by.
    pub written: bool,
    /// The ids of the blocks the batch created.
    pub created: Vec<u32>,
    /// The model as text, when there was no file to write it to.
    pub source: Option<String>,
}

/// Where the blocks a batch adds go.
///
/// The canvas belongs to the engine (decision D18): the place of a block that
/// an agent adds is the free spot of its container, the same place the desktop
/// app gives a block that is dropped. An agent that wants a block somewhere
/// particular says so with [`Layout::Explicit`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Layout {
    /// The engine places the block where there is room (the default).
    #[default]
    Auto,
    /// The coordinates of the command are used, as they were sent.
    Explicit,
}

/// Apply a batch of editing commands to a model.
///
/// The batch is **atomic**: a command that fails leaves the model exactly as it
/// was —the commands that were applied are undone— and the failure says which
/// command failed and why.
///
/// The model is checked before it is written: one that does not load, does not
/// validate or does not compile is **not** written, and the failure names the
/// block that caused it, exactly like the other tools. A model that carries no
/// canvas geometry is **composed** before it is written, so the editor can
/// draw it.
///
/// A model given as text has no file to write to: the text of the result
/// travels back with the answer, and that text is the only place the edit
/// lives.
pub fn apply_edits(
    source: &ModelSource,
    commands: &[EditCommand],
    layout: Layout,
) -> Result<Edited, Failure> {
    let name = source.name();
    // The model is parsed, but neither validated nor compiled: the commands
    // of the batch may be what makes it valid.
    let model = match source {
        ModelSource::File(path) => match genmsi_format::parse_file(path) {
            Ok(model) => model,
            // A model that is not on the disk yet is a model the batch
            // creates: it starts from the skeleton, the same one the
            // catalogue publishes. `save_model` writes a path that is not
            // there yet too; the verb creates it the same way, and the
            // agent does not have to build the model in memory and write it
            // somewhere else.
            Err(error) if error.is_missing_file() => genmsi_format::parse_str(
                genmsi_format::SKELETON_MODEL,
            )
            .map_err(|error| ToolError::from_format(&error, Some(&name), None))?,
            Err(error) => {
                return Err(Box::new(ToolError::from_format(
                    &error,
                    Some(&name),
                    None,
                )));
            }
        },
        ModelSource::Inline(text) => genmsi_format::parse_str(text)
            .map_err(|error| ToolError::from_format(&error, Some(&name), None))?,
    };
    let mut session = EditSession::new(model);
    let mut created: Vec<u32> = Vec::new();
    for (index, command) in commands.iter().enumerate() {
        let op = op_of(command);
        let command = prepared(command, session.model(), layout);
        match session.apply(command) {
            Ok(ids) => created.extend(ids),
            Err(error) => {
                // The failure is built before the rollback, so the position it
                // carries is the one of the model that failed.
                let failure = ToolError::command_failed(
                    index,
                    &op,
                    &error,
                    Some(&name),
                    Some(session.model()),
                );
                // The batch is atomic: the commands that were applied go back.
                // The failure is built before the rollback, so the position it
                // carries is the one of the model that failed; the rollback
                // leaves the session with the model it started with, so a
                // half-applied batch can never travel anywhere.
                for _ in 0..index {
                    let _ = session.undo();
                }
                return Err(Box::new(failure));
            }
        }
    }
    let mut model = session.model().clone();
    // The composition: the model is composed only when its geometry is
    // incomplete, so a model its author placed keeps the disposition it was
    // given (decision D18).
    genmsi_format::compose_incomplete(&mut model);
    genmsi_format::validate(&model)
        .map_err(|error| ToolError::from_format(&error, Some(&name), Some(&model)))?;
    let program = genmsi_core::compile_model(&model)
        .map_err(|error| ToolError::from_engine(&error, &model, &name))?;
    let (written, text) = match source {
        ModelSource::File(path) => {
            write_atomically(&model, path)?;
            (true, None)
        }
        ModelSource::Inline(_) => (false, Some(genmsi_format::to_string(&model))),
    };
    Ok(Edited {
        program,
        model,
        name,
        written,
        created,
        source: text,
    })
}

/// The command to apply, with the place of the block it adds.
///
/// With [`Layout::Auto`] the block lands in the free spot of its container:
/// the same place the desktop app gives a block that is dropped, computed
/// with the engine's own primitive. With [`Layout::Explicit`] the coordinates
/// of the command are used as they were sent.
fn prepared(command: &EditCommand, model: &GmdlModel, layout: Layout) -> EditCommand {
    match (layout, command) {
        (Layout::Auto, EditCommand::AddBlock { class, container, .. }) => {
            let (x, y) = free_spot(model, *container);
            EditCommand::AddBlock {
                class: class.clone(),
                container: *container,
                x,
                y,
            }
        }
        _ => command.clone(),
    }
}

/// The free spot of a container: where a block dropped there would land.
///
/// The primitive is the engine's own, and it is the twin of the one the
/// desktop app uses (`freeSpotOf`): the left of the blocks of the container
/// and below the lowest of them.
fn free_spot(model: &GmdlModel, container: u32) -> (i32, i32) {
    let children: Vec<u32> = model
        .blocks
        .iter()
        .filter(|block| block.owner == container && !block.is_root())
        .map(|block| block.id)
        .collect();
    genmsi_format::layout::free_spot_of_scope(model, &children)
}

/// The name of the operation of a command, as the wire spells it.
///
/// The name is read from the command itself, so it cannot drift away from the
/// `op` the tool receives.
fn op_of(command: &EditCommand) -> String {
    serde_json::to_value(command)
        .ok()
        .and_then(|value| {
            value
                .get("op")
                .and_then(|op| op.as_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "command".to_string())
}

/// Save a model to a file.
///
/// The model is checked **before** it is written: a model that does not
/// load, does not validate or does not compile is **not** written, and the
/// failure names the block that caused it, exactly like the other tools.
///
/// A model that carries no canvas geometry is **composed** before it is
/// written: the geometry is the engine's business, and a model the editor
/// cannot draw is of no use to the reader. A model that already carries its
/// geometry keeps the disposition its author left.
///
/// The write is atomic: the text goes to a temporary file next to the target
/// and that file is renamed over it. A reader that watches the model (the
/// desktop application does) never sees half a model.
pub fn save_model(path: &std::path::Path, text: &str) -> Result<Saved, Failure> {
    let name = path.display().to_string();
    let mut model = genmsi_format::parse_str(text)
        .map_err(|error| ToolError::from_format(&error, Some(&name), None))?;
    genmsi_format::validate(&model)
        .map_err(|error| ToolError::from_format(&error, Some(&name), Some(&model)))?;
    let program = genmsi_core::compile_model(&model)
        .map_err(|error| ToolError::from_engine(&error, &model, &name))?;
    // The composition: the model that is about to be written is composed
    // only when its geometry is incomplete, so a model its author placed
    // keeps the disposition it was given.
    genmsi_format::compose_incomplete(&mut model);
    let bytes = write_atomically(&model, path)?;
    Ok(Saved {
        program,
        model,
        path: path.to_path_buf(),
        bytes,
    })
}

/// Write the model to a temporary file next to the target, then rename it.
///
/// The temporary file is in the same directory, so the rename is atomic: the
/// target is either the old model or the new one, never a half-written one.
fn write_atomically(model: &GmdlModel, path: &std::path::Path) -> Result<u64, Failure> {
    let name = path.display().to_string();
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "model.gmdl".to_string());
    let temporary = path.with_file_name(format!(".{file_name}.tmp"));
    // A write that fails leaves nothing behind: the temporary file is the
    // target's name with a dot in front, and a reader that lists the
    // directory must not find a half-written model in it.
    if let Err(error) = genmsi_format::save(model, &temporary) {
        let _ = std::fs::remove_file(&temporary);
        return Err(Box::new(ToolError::cannot_write(&name, format!("{error}"))));
    }
    let bytes = std::fs::metadata(&temporary).map(|meta| meta.len()).unwrap_or(0);
    if let Err(error) = std::fs::rename(&temporary, path) {
        let _ = std::fs::remove_file(&temporary);
        return Err(Box::new(ToolError::cannot_write(&name, error)));
    }
    Ok(bytes)
}

/// The configuration of a run.
///
/// `trace` and `limit` decide whether the run keeps its event trace and how
/// many records it keeps; the seed makes the run reproducible; `start` is the
/// `START` of the original, the counter of terminations of the run.
pub fn config(
    seed: u64,
    until: Option<f64>,
    start: u64,
    trace: bool,
    limit: usize,
) -> SimConfig {
    SimConfig {
        seed,
        stop_time: until,
        start,
        trace,
        trace_limit: limit,
        ..SimConfig::default()
    }
}

/// Run a compiled model.
pub fn run(loaded: &Loaded, config: SimConfig) -> Result<RunResult, Failure> {
    Ok(Simulation::new(&loaded.program, config)
        .run()
        .map_err(|error| ToolError::from_engine(&error, &loaded.model, &loaded.name))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINI: &str = "<gmodel versioncode=\"1\"><gclasses/><modules>\
        <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"m\" name=\"m\" kind=\"C\"><links/><inputs/><outputs/></module>\
        <module moduleid=\"1\" moduleclass=\"Clock\" owner=\"0\" caption=\"c\" name=\"c\" kind=\"A\"><props>\
        <property name=\"Frecuencia\" hint=\"\" value=\"3600\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/>\
        </props></module>\
        </modules></gmodel>";

    #[test]
    fn an_inline_model_compiles_like_a_file() {
        let loaded = load(&ModelSource::Inline(MINI.to_string())).expect("model compiles");
        assert_eq!(loaded.name, MEMORY);
        assert_eq!(loaded.program.blocks.len(), 2);
    }

    #[test]
    fn a_missing_file_is_reported_as_such() {
        let error = load(&ModelSource::File(PathBuf::from("does-not-exist.gmdl"))).expect_err("must fail");
        assert_eq!(error.error, "model_not_found");
        assert_eq!(error.path.as_deref(), Some("does-not-exist.gmdl"));
    }

    #[test]
    fn the_configuration_carries_the_knobs() {
        let config = config(7, Some(120.0), 5, true, 25);
        assert_eq!(config.seed, 7);
        assert_eq!(config.stop_time, Some(120.0));
        assert_eq!(config.start, 5);
        assert!(config.trace);
        assert_eq!(config.trace_limit, 25);
    }
}
