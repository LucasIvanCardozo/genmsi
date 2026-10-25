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
/// result, layout result or run summary), so the output is comparable as a
/// snapshot and parseable by scripts. Add `--json` to get the versioned JSON
/// document instead of the human-readable form; the schema is documented in
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
    /// Print the GPSS program a model compiles to.
    Gpss(GpssArgs),
    /// Compose the canvas of a model and say what it composed.
    Layout(LayoutArgs),
    /// Simulate every model of a directory and summarise the outcome.
    RunAll(RunAllArgs),
    /// Serve the MCP protocol over stdio (the same server as `genmsi-mcp`).
    Mcp,
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
    /// Seed of every random stream; a random one when it is not given, and
    /// the run says which it used. The same seed reproduces the same run.
    #[arg(long, value_name = "SEED")]
    pub seed: Option<u64>,
    /// Stop once the simulated clock passes this time, instead of running to
    /// the model's own termination.
    #[arg(long, value_name = "TIME")]
    pub until: Option<f64>,
    /// The `START` of the run: how many termination units end it.
    ///
    /// The original has the same field in the panel of its simulator ("Start:",
    /// one by default) and the model does not declare it: it is a parameter of
    /// the run, like the horizon. The `Terminate` with its operand A takes
    /// that many units from the counter for every transaction that arrives,
    /// the `Clock` takes one per tick, and the run ends when the counter
    /// reaches zero or goes below it.
    #[arg(long, value_name = "N", value_parser = parse_start)]
    pub start: Option<u64>,
}

/// The value of the `--start` option.
///
/// The original refuses a `Start` of zero ("El Start debe ser mayor a
/// cero."), and so does this: a run that starts with a counter of zero would
/// end before it begins.
fn parse_start(value: &str) -> Result<u64, String> {
    let start: u64 = value
        .parse()
        .map_err(|_| format!("the start must be a whole number: {value}"))?;
    if start == 0 {
        return Err("the start must be greater than zero".to_string());
    }
    Ok(start)
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

/// `genmsi gpss <model>`.
#[derive(Debug, Args)]
pub struct GpssArgs {
    /// Options of the model to compile.
    #[command(flatten)]
    pub model: ModelArgs,
    /// Output form.
    #[command(flatten)]
    pub output: OutputArgs,
}

/// `genmsi layout <model>`.
#[derive(Debug, Args)]
pub struct LayoutArgs {
    /// Options of the model to compose.
    #[command(flatten)]
    pub model: ModelArgs,
    /// Write the composed model back to the file.
    ///
    /// Without it the command only says what it composed: writing is
    /// explicit, so the reader sees the composition before the file changes.
    #[arg(long)]
    pub write: bool,
    /// Output form.
    #[command(flatten)]
    pub output: OutputArgs,
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
