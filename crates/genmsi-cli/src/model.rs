//! The engine boundary: everything the CLI asks of the two engine crates.
//!
//! Keeping the calls here means the subcommand modules stay readable and the
//! CLI never re-implements a simulation rule (decision D2: thin layer).

use std::path::Path;

use genmsi_core::{Program, SimConfig};
use genmsi_format::GmdlModel;

use crate::error::CliError;

/// A model that loaded, validated and compiled, ready to run.
#[derive(Debug)]
pub struct Loaded {
    /// Compiled program.
    pub program: Program,
    /// The model, kept to read what the format says about it (the entities it
    /// uses without declaring them).
    pub model: GmdlModel,
}

/// Load, validate and compile one model file.
///
/// The three failures the engine distinguishes (invalid XML, structural
/// problem, modelling problem) all come back as [`CliError::Model`] with the
/// file, line and node the engine attached.
pub fn load(path: &Path) -> Result<Loaded, CliError> {
    let model: GmdlModel =
        genmsi_format::load(path).map_err(|error| CliError::model(path, error))?;
    let program =
        genmsi_core::compile_model(&model).map_err(|error| CliError::model(path, error))?;
    Ok(Loaded { program, model })
}

/// Engine configuration for one CLI run.
///
/// The seed is random when the caller does not choose one, and the run says
/// which it used: the reader can repeat it with the same number.
///
/// The `start` is the `START` of the original: the counter of terminations of
/// the run, one by default.
pub fn config(seed: Option<u64>, until: Option<f64>, start: Option<u64>) -> SimConfig {
    SimConfig {
        seed: seed.unwrap_or_else(genmsi_core::random_seed),
        stop_time: until,
        start: start.unwrap_or(genmsi_core::DEFAULT_START),
        ..SimConfig::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn loads_a_reference_model() {
        let loaded = load(&repo_root().join("fixtures/models/TP2__problema1.gmdl")).expect("loads");
        assert!(!loaded.program.blocks.is_empty());
    }

    #[test]
    fn keeps_the_engine_context_on_failure() {
        let path = repo_root().join("fixtures/models/Parcial 8-11-21__ParcialMikelModelos.gmdl");
        let error = load(&path).expect_err("invalid block name");
        let text = error.to_string();
        assert!(text.contains("invalid block"), "{text}");
        assert!(text.contains("(node "), "{text}");
        // The CLI names the path exactly once, even though the engine's own
        // location already carries the file.
        let path_text = path.display().to_string();
        assert!(text.starts_with(&path_text), "{text}");
        assert_eq!(text.matches(&path_text).count(), 1, "path printed twice: {text}");
        assert_eq!(text.matches("(node ").count(), 1, "node printed twice: {text}");
    }

    #[test]
    fn configuration_carries_seed_and_horizon() {
        let config = config(Some(7), Some(120.0), None);
        assert_eq!(config.seed, 7);
        assert_eq!(config.stop_time, Some(120.0));
        assert!(!config.trace);
    }

    #[test]
    fn the_start_of_a_run_is_one_when_nobody_chooses_one() {
        assert_eq!(config(Some(7), None, None).start, 1);
        assert_eq!(config(Some(7), None, Some(5)).start, 5);
    }
}
