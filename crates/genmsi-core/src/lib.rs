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
//!   explicit `Simulation` handle.
//! - Blocks are described by data in `genmsi-format` and compiled into a
//!   `Program`; adding a block type does not require touching the scheduler.

pub mod error;
pub mod expr;
pub mod program;
pub mod rng;
pub mod sim;
pub mod stat;
pub mod xact;

pub use error::EngineError;
pub use program::{Program, compile};
pub use sim::{BlockStats, EndReason, RunResult, SimConfig, Simulation, TraceEvent};
