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
//! - Blocks are defined by data (block definitions live in `genmsi-format`),
//!   so adding a block type must not require recompiling the engine.

/// Error type for engine failures (bad expressions, invalid model, runtime).
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)] // scaffold: variants gain use in Phase 1
pub enum EngineError {
    /// An expression in the model could not be evaluated.
    #[error("expression error: {0}")]
    Expression(String),
    /// The model is valid XML but not runnable as-is.
    #[error("invalid simulation setup: {0}")]
    InvalidSetup(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_error_display_is_actionable() {
        let err = EngineError::Expression("unknown function FN:MediaClientes".to_string());
        let text = err.to_string();
        assert!(text.contains("FN:MediaClientes"));
    }
}
