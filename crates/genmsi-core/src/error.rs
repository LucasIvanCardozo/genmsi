//! Engine failures with enough context to fix the model.
//!
//! Errors name the block (`node`) and the offending source text, because the
//! consumers are CLIs, agents over MCP, and humans reading a report.

use std::fmt;

/// A failure while compiling or running a model.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    /// The model parsed as XML but uses a block the engine does not implement.
    #[error("node {node} ({class}): {message}")]
    Unsupported {
        /// `moduleid` of the block.
        node: u32,
        /// Block class.
        class: String,
        /// What is missing.
        message: String,
    },
    /// A property or expression could not be interpreted.
    #[error("node {node} ({class}): {message}")]
    InvalidBlock {
        /// `moduleid` of the block.
        node: u32,
        /// Block class.
        class: String,
        /// What is wrong.
        message: String,
    },
    /// An expression failed to parse or evaluate.
    #[error("expression {text:?}: {message}")]
    Expression {
        /// Original expression text.
        text: String,
        /// Parser or evaluator message.
        message: String,
    },
    /// The model file is structurally invalid.
    #[error("{0}")]
    Format(#[from] genmsi_format::FormatError),
    /// The graph cannot be traversed (dangling link, missing port).
    #[error("{message}")]
    Graph {
        /// Human-readable description.
        message: String,
    },
    /// A run exceeded its safety limits (for example an infinite zero-time loop).
    #[error("{0}")]
    Runtime(String),
}

impl EngineError {
    /// An unsupported block.
    pub fn unsupported(node: u32, class: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Unsupported {
            node,
            class: class.into(),
            message: message.into(),
        }
    }

    /// An invalid block definition.
    pub fn invalid_block(node: u32, class: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidBlock {
            node,
            class: class.into(),
            message: message.into(),
        }
    }
}

/// Format a numeric entity name the way GPSS numbers entities.
pub fn numeric_name(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

/// Round a floating point value for display: trims `1.0` to `1`.
#[derive(Debug, Clone, Copy)]
pub struct Number(pub f64);

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.0;
        if value.fract() == 0.0 && value.abs() < 1e15 {
            write!(f, "{}", value as i64)
        } else {
            write!(f, "{value}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_carry_node_and_class() {
        let err = EngineError::invalid_block(42, "Advance", "Media is empty");
        assert_eq!(
            err.to_string(),
            "node 42 (Advance): Media is empty"
        );
    }

    #[test]
    fn numeric_names_are_integers_when_exact() {
        assert_eq!(numeric_name(2.0), "2");
        assert_eq!(numeric_name(2.5), "2.5");
        assert_eq!(Number(10.0).to_string(), "10");
    }
}
