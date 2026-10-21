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

    /// The model node the failure belongs to, when it is node-scoped.
    ///
    /// A client uses it to look the block up in the model and report the place
    /// the agent has to fix.
    pub fn node(&self) -> Option<u32> {
        match self {
            Self::Unsupported { node, .. } | Self::InvalidBlock { node, .. } => Some(*node),
            Self::Format(error) => error.location().node,
            Self::Expression { .. } | Self::Graph { .. } | Self::Runtime(_) => None,
        }
    }

    /// The block class the failure belongs to, when the failure names one.
    pub fn class(&self) -> Option<&str> {
        match self {
            Self::Unsupported { class, .. } | Self::InvalidBlock { class, .. } => Some(class),
            Self::Format(_) | Self::Expression { .. } | Self::Graph { .. } | Self::Runtime(_) => {
                None
            }
        }
    }

    /// The source location of the failure, when it has one.
    ///
    /// Only format failures carry a location: an XML problem knows its line,
    /// a structural problem knows its node.
    pub fn location(&self) -> Option<&genmsi_format::Location> {
        match self {
            Self::Format(error) => Some(error.location()),
            _ => None,
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
        assert_eq!(err.node(), Some(42));
        assert_eq!(err.class(), Some("Advance"));
    }

    #[test]
    fn a_format_failure_keeps_its_location() {
        let error = genmsi_format::FormatError::structural(7, "no TERMINATE block")
            .with_file("model.gmdl");
        let err = EngineError::from(error);
        assert_eq!(err.node(), Some(7));
        assert_eq!(err.class(), None);
        let location = err.location().expect("a format failure has a location");
        assert_eq!(location.file.as_deref(), Some("model.gmdl"));
    }

    #[test]
    fn a_runtime_failure_names_no_block() {
        let err = EngineError::Runtime("zero-time loop".to_string());
        assert_eq!(err.node(), None);
        assert_eq!(err.class(), None);
        assert!(err.location().is_none());
    }

    #[test]
    fn numeric_names_are_integers_when_exact() {
        assert_eq!(numeric_name(2.0), "2");
        assert_eq!(numeric_name(2.5), "2.5");
        assert_eq!(Number(10.0).to_string(), "10");
    }
}
