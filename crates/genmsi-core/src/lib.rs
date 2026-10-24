//! Headless discrete-event simulation engine for GenMSI.
//!
//! This crate is independent of any user interface. Clients (CLI, MCP server,
//! GUI) are thin layers over the API defined here.
//!
//! Design rules (see `docs/DECISIONS.md`):
//!
//! - Every run is seeded: a simulation with the same seed is bit-for-bit
//!   reproducible, which is the basis for replications, confidence intervals
//!   and regression testing.
//! - No opaque global state; the engine owns all mutable state behind an
//!   explicit [`Simulation`] handle.
//! - Blocks are described by data in `genmsi-format` and compiled into a
//!   [`Program`]; adding a block type does not require touching the scheduler.
//!
//! # Example
//!
//! ```no_run
//! use genmsi_core::{simulate, SimConfig};
//! use genmsi_format::load;
//!
//! let model = load("fixtures/models/TP2__problema1.gmdl").unwrap();
//! let (program, result) = simulate(&model, SimConfig::default()).unwrap();
//! println!("{}", genmsi_core::render(&result, &program));
//! ```

pub mod documents;
pub mod error;
pub mod expr;
pub mod glossary;
pub mod oracle;
pub mod gpss;
pub mod program;
pub mod report;
pub mod replication;
pub mod rng;
pub mod sim;
pub mod stat;
pub mod xact;

pub use documents::{
    CATALOGUE_SCHEMA, CLASSLIST_SCHEMA, CLASS_SCHEMA, CatalogueClass, CatalogueDocument,
    CataloguePort, CatalogueProperty, ClassDocument, ClassListDocument, ClassListEntry,
    DefinitionCounts, EXPLAIN_SCHEMA, Envelope, LINT_SCHEMA, LintDocument, MODEL_SCHEMA,
    ModelBlock, ModelDocument, ModelEdge, ModelPort, ModelProperty, ModelRect, REPORT_SCHEMA,
    TRACE_SCHEMA, TraceDocument, catalogue_document, class_document, class_list_document,
    lint_diagnostics, lint_document, model_document, to_pretty_json, trace_document,
};
pub use error::EngineError;
pub use program::{Program, compile, known_classes};
pub use report::{json_report, program_json, render, to_csv, to_json};
pub use replication::{
    MetricSummary, ReplicationConfig, ReplicationResult, render_summary, replicate,
};
pub use sim::{
    BlockStats, DEFAULT_MAX_EVENTS, DEFAULT_START, EndReason, RunResult, SimConfig, Simulation,
    TraceEvent,
};

use genmsi_format::GmdlModel;

/// Compile and run a model once.
pub fn simulate(
    model: &GmdlModel,
    config: SimConfig,
) -> Result<(Program, RunResult), EngineError> {
    let title = model
        .root()
        .map(|root| root.name.clone())
        .unwrap_or_else(|| "model".to_string());
    let program = compile(model, title)?;
    let result = Simulation::new(&program, config).run()?;
    Ok((program, result))
}

/// Compile a model without running it.
pub fn compile_model(model: &GmdlModel) -> Result<Program, EngineError> {
    let title = model
        .root()
        .map(|root| root.name.clone())
        .unwrap_or_else(|| "model".to_string());
    compile(model, title)
}

/// A random seed for a run.
///
/// A run is entirely determined by its seed, so a run that does not take one
/// says which it used: the reader can repeat it by passing the same number.
pub fn random_seed() -> u64 {
    rand::random()
}
