//! The engine adapter: the only place of the crate that calls the engine.
//!
//! The crate carries no simulation logic (decision D2): it loads a model,
//! asks the engine to compile and run it, and renders the documents the
//! engine already produces. The CLI does exactly the same through the same
//! API, so both clients show the same numbers.

use std::path::PathBuf;

use genmsi_core::{Program, RunResult, SimConfig, Simulation};
use genmsi_format::GmdlModel;

use crate::errors::ToolError;

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
pub fn load(source: &ModelSource) -> Result<Loaded, ToolError> {
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

/// Save a model to a file.
///
/// The model is checked **before** it is written: a model that does not
/// load, does not validate or does not compile is **not** written, and the
/// failure names the block that caused it, exactly like the other tools.
///
/// The write is atomic: the text goes to a temporary file next to the target
/// and that file is renamed over it. A reader that watches the model (the
/// desktop application does) never sees half a model.
pub fn save_model(path: &std::path::Path, text: &str) -> Result<Saved, ToolError> {
    let name = path.display().to_string();
    let model = genmsi_format::parse_str(text)
        .map_err(|error| ToolError::from_format(&error, Some(&name), None))?;
    genmsi_format::validate(&model)
        .map_err(|error| ToolError::from_format(&error, Some(&name), Some(&model)))?;
    let program = genmsi_core::compile_model(&model)
        .map_err(|error| ToolError::from_engine(&error, &model, &name))?;
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
fn write_atomically(model: &GmdlModel, path: &std::path::Path) -> Result<u64, ToolError> {
    let name = path.display().to_string();
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "model.gmdl".to_string());
    let temporary = path.with_file_name(format!(".{file_name}.tmp"));
    genmsi_format::save(model, &temporary).map_err(|error| {
        ToolError::cannot_write(&name, format!("{error}"))
    })?;
    let bytes = std::fs::metadata(&temporary).map(|meta| meta.len()).unwrap_or(0);
    if let Err(error) = std::fs::rename(&temporary, path) {
        let _ = std::fs::remove_file(&temporary);
        return Err(ToolError::cannot_write(&name, error));
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
pub fn run(loaded: &Loaded, config: SimConfig) -> Result<RunResult, ToolError> {
    Simulation::new(&loaded.program, config)
        .run()
        .map_err(|error| ToolError::from_engine(&error, &loaded.model, &loaded.name))
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
