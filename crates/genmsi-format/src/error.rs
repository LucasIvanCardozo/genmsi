//! Error types for reading, validating and writing `.gmdl` models.
//!
//! Every failure carries a [`Location`] with the file, the 1-based source
//! line and column, and the model node (`moduleid`) involved, so an agent or
//! a human can go straight to the offending block.

use std::fmt;

/// Where a format problem happened.
///
/// Fields are optional because not every error has all of them: a file that
/// cannot be read has a name but no position, an error found while scanning
/// the text has a line and a column but often no node.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Location {
    /// Source file name (or `<memory>` when parsing a string).
    pub file: Option<String>,
    /// 1-based line in the source file.
    pub line: Option<usize>,
    /// 1-based column in the source file, when the exact offset is known.
    pub column: Option<usize>,
    /// `moduleid` of the node involved, when the error is node-scoped.
    pub node: Option<u32>,
}

impl Location {
    /// A location with only a file name.
    pub fn file(file: impl Into<String>) -> Self {
        Self {
            file: Some(file.into()),
            line: None,
            column: None,
            node: None,
        }
    }

    /// A location pointing at a node.
    pub fn node(node: u32) -> Self {
        Self {
            file: None,
            line: None,
            column: None,
            node: Some(node),
        }
    }

    /// Set the file name.
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Set the source line.
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Set the source column.
    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    /// Add a node id when one is not set yet.
    pub fn with_node(mut self, node: u32) -> Self {
        if self.node.is_none() {
            self.node = Some(node);
        }
        self
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.file, self.line, self.node) {
            (None, None, None) => f.write_str("<unknown location>"),
            _ => {
                let mut first = true;
                if let Some(file) = &self.file {
                    write!(f, "{file}")?;
                    first = false;
                }
                if let Some(line) = self.line {
                    if !first {
                        f.write_str(":")?;
                    }
                    write!(f, "{line}")?;
                    first = false;
                }
                if let Some(node) = self.node {
                    if !first {
                        f.write_str(" ")?;
                    }
                    write!(f, "(node {node})")?;
                }
                Ok(())
            }
        }
    }
}

/// Failure while reading, validating or writing a model file.
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    /// The document is not well-formed XML.
    #[error("{location}: invalid XML: {message}")]
    Xml {
        /// Where the parser stopped.
        location: Location,
        /// Parser message.
        message: String,
    },
    /// The document is well-formed XML but violates the `.gmdl` schema or a
    /// documented structural rule.
    #[error("{location}: {message}")]
    Structural {
        /// Where the problem is.
        location: Location,
        /// Human-readable description of the violated rule.
        message: String,
    },
    /// The model file could not be read: it is missing, the operating system
    /// refuses it or it is not UTF-8. The parser reads the file itself, so a
    /// client that loads through it gets the same `cannot read model` text
    /// the CLI prints.
    #[error("{location}: cannot read model: {message}")]
    Read {
        /// The file that could not be read.
        location: Location,
        /// The reader's message.
        message: String,
        /// The kind of I/O failure, to tell a missing file from the rest.
        kind: std::io::ErrorKind,
    },
}

impl FormatError {
    /// A structural error attached to a node.
    pub fn structural(node: u32, message: impl Into<String>) -> Self {
        Self::Structural {
            location: Location::node(node),
            message: message.into(),
        }
    }

    /// A structural error attached to a source line.
    pub fn structural_at_line(line: usize, message: impl Into<String>) -> Self {
        Self::Structural {
            location: Location::default().with_line(line),
            message: message.into(),
        }
    }

    /// Where the failure happened.
    pub fn location(&self) -> &Location {
        match self {
            Self::Xml { location, .. }
            | Self::Structural { location, .. }
            | Self::Read { location, .. } => location,
        }
    }

    /// Whether the failure is the model file itself not existing.
    pub fn is_missing_file(&self) -> bool {
        matches!(
            self,
            Self::Read {
                kind: std::io::ErrorKind::NotFound,
                ..
            }
        )
    }

    /// Add a file name if the error does not carry one yet.
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        let file = file.into();
        match &mut self {
            Self::Xml { location, .. }
            | Self::Structural { location, .. }
            | Self::Read { location, .. } => {
                if location.file.is_none() {
                    location.file = Some(file);
                }
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_display_joins_available_parts() {
        let loc = Location::default()
            .with_file("model.gmdl")
            .with_line(9)
            .with_node(109);
        assert_eq!(loc.to_string(), "model.gmdl:9 (node 109)");
        assert_eq!(Location::node(7).to_string(), "(node 7)");
        assert_eq!(Location::default().to_string(), "<unknown location>");
    }

    #[test]
    fn xml_error_display_is_actionable() {
        let err = FormatError::Xml {
            location: Location::default().with_file("a.gmdl").with_line(3),
            message: "unexpected closing tag".to_string(),
        };
        let text = err.to_string();
        assert!(text.contains("a.gmdl:3"), "{text}");
        assert!(text.contains("unexpected closing tag"), "{text}");
    }

    #[test]
    fn structural_error_keeps_node_context_through_with_file() {
        let err = FormatError::structural(42, "missing property").with_file("m.gmdl");
        assert_eq!(err.location().node, Some(42));
        assert_eq!(err.location().file.as_deref(), Some("m.gmdl"));
    }
}
