//! Parsing, validation and writing of GenMSI model files (`.gmdl`) and block
//! class definitions (`gClasses.xml`).
//!
//! Design rules (see `docs/DECISIONS.md`):
//!
//! - Semantic model and layout are stored separately: node/link semantics must
//!   never depend on canvas positions.
//! - Every value read from XML is validated on load; the parser rejects
//!   structurally invalid models with precise errors instead of "fixing" them
//!   silently (the original tool accumulated structural bugs this way).
//! - This crate is the only place that knows the `.gmdl` on-disk format.

/// Error type for all parsing and validation failures.
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)] // scaffold: variants gain use in Phase 1
pub enum FormatError {
    /// The XML could not be parsed.
    #[error("invalid XML at {location}: {message}")]
    Xml { location: String, message: String },
    /// The XML parsed but does not satisfy the model schema.
    #[error("structural error in model: {0}")]
    Structural(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_error_display_is_actionable() {
        let err = FormatError::Xml {
            location: "line 3, column 10".to_string(),
            message: "unexpected closing tag".to_string(),
        };
        let text = err.to_string();
        assert!(text.contains("line 3"));
        assert!(text.contains("unexpected closing tag"));
    }
}
