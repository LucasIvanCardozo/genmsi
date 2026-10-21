//! Command-line surface: argument definitions only, no behaviour.
//!
//! One struct per subcommand, plus two shared groups so a new subcommand only
//! declares the options it actually needs:
//!
//! - [`RunArgs`]: the knobs of a simulation (`--seed`, `--until`).
//! - [`OutputArgs`]: `--json`, which every subcommand honours.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// GenMSI: discrete-event simulation toolkit, headless engine with CLI, MCP and
/// GUI clients.
///
/// Every subcommand prints one deterministic document (report, trace, lint
/// result or run summary), so the output is comparable as a snapshot and
/// parseable by scripts. Add `--json` to get the versioned JSON document
/// instead of the human-readable form; the schema is documented in
/// `docs/cli.md`.
#[derive(Debug, Parser)]
#[command(name = "genmsi", version, about, long_about = None)]
pub struct Cli {
    /// Subcommand to run.
    #[command(subcommand)]
    pub command: Command,
}

/// Top-level subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Validate a model and report structural, modelling or compiler problems.
    Lint(LintArgs),
    /// Simulate a model and print the GPSS report.
    Simulate(SimulateArgs),
    /// Simulate a model and print its event trace.
    Trace(TraceArgs),
    /// Simulate every model of a directory and summarise the outcome.
    RunAll(RunAllArgs),
}

/// Options shared by the subcommands that read one model.
#[derive(Debug, Args)]
pub struct ModelArgs {
    /// Path to the `.gmdl` model file.
    pub path: PathBuf,
}

/// Options shared by the subcommands that run a simulation.
#[derive(Debug, Args)]
pub struct RunArgs {
    /// Seed of every random stream; the same seed reproduces the same run.
    #[arg(long, value_name = "SEED", default_value_t = 1)]
    pub seed: u64,
    /// Stop once the simulated clock passes this time, instead of running to
    /// the model's own termination.
    #[arg(long, value_name = "TIME")]
    pub until: Option<f64>,
}

/// Output form of every subcommand.
#[derive(Debug, Args)]
pub struct OutputArgs {
    /// Emit the versioned JSON document instead of human-readable text.
    #[arg(long)]
    pub json: bool,
}

/// `genmsi lint <model>`.
#[derive(Debug, Args)]
pub struct LintArgs {
    /// Options of the model to lint.
    #[command(flatten)]
    pub model: ModelArgs,
    /// Output form.
    #[command(flatten)]
    pub output: OutputArgs,
}

/// `genmsi simulate <model>`.
#[derive(Debug, Args)]
pub struct SimulateArgs {
    /// Options of the model to simulate.
    #[command(flatten)]
    pub model: ModelArgs,
    /// Simulation knobs.
    #[command(flatten)]
    pub run: RunArgs,
    /// Output form.
    #[command(flatten)]
    pub output: OutputArgs,
}

/// `genmsi trace <model>`.
#[derive(Debug, Args)]
pub struct TraceArgs {
    /// Options of the model to trace.
    #[command(flatten)]
    pub model: ModelArgs,
    /// Simulation knobs.
    #[command(flatten)]
    pub run: RunArgs,
    /// Output form.
    #[command(flatten)]
    pub output: OutputArgs,
    /// Maximum number of trace records to keep.
    #[arg(long, value_name = "N", default_value_t = 100)]
    pub limit: usize,
}

/// `genmsi run-all <directory>`.
#[derive(Debug, Args)]
pub struct RunAllArgs {
    /// Directory holding the `.gmdl` models (one level, not recursive).
    pub dir: PathBuf,
    /// Simulation knobs applied to every model.
    #[command(flatten)]
    pub run: RunArgs,
    /// Output form.
    #[command(flatten)]
    pub output: OutputArgs,
}
