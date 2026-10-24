//! `genmsi run-all <dir>`: run every model of a directory and summarise.
//!
//! One line per model, sorted by file name, so a CI log answers "which models
//! still run" at a glance. A model that fails does not stop the sweep: the
//! summary counts it and the command exits 1, because a directory that holds
//! refused models is a real outcome (the five reference models the original
//! rejects are expected to fail here).

use std::path::{Path, PathBuf};

use genmsi_core::{SimConfig, Simulation};
use serde::Serialize;

use crate::cli::RunAllArgs;
use crate::error::{CliError, ExitCode};
use crate::model;
use crate::output::{self, RUN_ALL_SCHEMA};

/// Outcome of one model inside a `run-all` sweep.
#[derive(Debug, Serialize)]
pub struct ModelOutcome {
    /// File stem of the model, spaces preserved.
    pub name: String,
    /// Path as scanned.
    pub path: String,
    /// `ok` or `failed`.
    pub status: &'static str,
    /// End time of the run, when it ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<f64>,
    /// Why the run ended, when it ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_reason: Option<&'static str>,
    /// Engine message, when it failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ModelOutcome {
    fn ok(name: String, path: String, end_time: f64, end_reason: &'static str) -> Self {
        Self {
            name,
            path,
            status: "ok",
            end_time: Some(end_time),
            end_reason: Some(end_reason),
            error: None,
        }
    }

    fn failed(name: String, path: String, error: String) -> Self {
        Self {
            name,
            path,
            status: "failed",
            end_time: None,
            end_reason: None,
            error: Some(error),
        }
    }
}

/// Body of the `genmsi.run-all/1` document.
#[derive(Debug, Serialize)]
pub struct RunAllReport {
    /// Directory scanned, as the user wrote it.
    pub dir: String,
    /// Seed applied to every model.
    pub seed: u64,
    /// Horizon applied to every model, when one was given.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<f64>,
    /// Models found.
    pub models_total: usize,
    /// Models that ran.
    pub models_ok: usize,
    /// Models that failed.
    pub models_failed: usize,
    /// One outcome per model, in scan order.
    pub models: Vec<ModelOutcome>,
}

impl RunAllReport {
    /// Exit code this sweep implies.
    pub fn exit_code(&self) -> ExitCode {
        if self.models_failed == 0 {
            ExitCode::Success
        } else {
            ExitCode::Failure
        }
    }

    /// Human-readable form.
    pub fn render(&self) -> String {
        let mut out = String::new();
        for model in &self.models {
            let badge = if model.status == "ok" { "OK" } else { "FAIL" };
            let detail = match (model.end_time, model.end_reason, &model.error) {
                (Some(end_time), Some(reason), _) => format!("{end_time:>10.1}  {reason}"),
                (_, _, Some(error)) => error.clone(),
                _ => String::new(),
            };
            let line = format!("{badge:<5} {:<42} {}", model.name, detail);
            out.push_str(line.trim_end());
            out.push('\n');
        }
        out.push_str(&format!(
            "{} models: {} ok, {} failed\n",
            self.models_total, self.models_ok, self.models_failed
        ));
        out
    }
}

/// Run `genmsi run-all`.
pub fn run(args: RunAllArgs) -> Result<ExitCode, CliError> {
    let paths = scan(&args.dir)?;
    let config = model::config(args.run.seed, args.run.until);
    let models: Vec<ModelOutcome> = paths
        .iter()
        .map(|path| sweep(path, config.clone()))
        .collect();
    let models_ok = models.iter().filter(|model| model.status == "ok").count();
    let report = RunAllReport {
        dir: args.dir.display().to_string(),
        seed: config.seed,
        until: config.stop_time,
        models_total: models.len(),
        models_ok,
        models_failed: models.len() - models_ok,
        models,
    };
    if args.output.json {
        output::print_json(RUN_ALL_SCHEMA, &report)?;
    } else {
        output::print(&report.render())?;
    }
    Ok(report.exit_code())
}

/// Every `.gmdl` file of one directory level, sorted by path.
fn scan(dir: &Path) -> Result<Vec<PathBuf>, CliError> {
    if !dir.is_dir() {
        return Err(CliError::Usage(format!(
            "{}: not a directory",
            dir.display()
        )));
    }
    let entries = std::fs::read_dir(dir).map_err(|error| CliError::Io {
        path: dir.display().to_string(),
        message: error.to_string(),
    })?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| CliError::Io {
            path: dir.display().to_string(),
            message: error.to_string(),
        })?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "gmdl") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

/// Load and run one model, turning any failure into an outcome instead of an
/// abort.
fn sweep(path: &Path, config: SimConfig) -> ModelOutcome {
    let name = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());
    let shown = path.display().to_string();
    let outcome = model::load(path).and_then(|loaded| {
        let result = Simulation::new(&loaded.program, config)
            .run()
            .map_err(|error| CliError::model(path, error))?;
        Ok((result.end_time, result.end_reason))
    });
    match outcome {
        Ok((end_time, end_reason)) => {
            ModelOutcome::ok(name, shown, end_time, end_reason.as_str())
        }
        Err(error) => ModelOutcome::failed(name, shown, error.detail()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failure_does_not_hide_the_other_models() {
        let report = RunAllReport {
            dir: "fixtures/models".to_string(),
            seed: 1,
            until: None,
            models_total: 2,
            models_ok: 1,
            models_failed: 1,
            models: vec![
                ModelOutcome::ok("a".to_string(), "a.gmdl".to_string(), 3600.0, "terminated"),
                ModelOutcome::failed(
                    "b".to_string(),
                    "b.gmdl".to_string(),
                    "no TERMINATE block".to_string(),
                ),
            ],
        };
        assert_eq!(report.exit_code(), ExitCode::Failure);
        let text = report.render();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("OK    a"), "{text}");
        assert!(lines[1].starts_with("FAIL  b"), "{text}");
        assert_eq!(lines[2], "2 models: 1 ok, 1 failed");
    }

    #[test]
    fn a_clean_sweep_succeeds() {
        let report = RunAllReport {
            dir: "models".to_string(),
            seed: 1,
            until: None,
            models_total: 0,
            models_ok: 0,
            models_failed: 0,
            models: Vec::new(),
        };
        assert_eq!(report.exit_code(), ExitCode::Success);
    }

    #[test]
    fn scanning_a_missing_directory_is_a_usage_error() {
        let error = scan(Path::new("fixtures/models/does-not-exist")).expect_err("must fail");
        assert_eq!(error.exit_code(), ExitCode::Usage);
    }
}
