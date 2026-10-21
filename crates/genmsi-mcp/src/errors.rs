//! The failure a tool returns to the agent.
//!
//! An agent has to fix the model, so the failure says what class it is, which
//! block caused it and where that block stands in the source file. The line
//! and the column point at the opening tag of the `<module>` element of the
//! `.gmdl` file, which is where an agent has to look.

use genmsi_core::EngineError;
use genmsi_format::{FormatError, GmdlModel};
use serde::Serialize;

/// A failure a tool returns to the agent.
#[derive(Debug, Clone, Serialize)]
pub struct ToolError {
    /// Stable class of the failure; an agent can branch on it.
    pub error: &'static str,
    /// What went wrong, in one sentence.
    pub message: String,
    /// The model the failure belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// 1-based line of the offending block in the `.gmdl` source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// 1-based column of the offending block in the `.gmdl` source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
    /// `moduleid` of the offending block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<u32>,
    /// GPSS class of the offending block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
}

impl ToolError {
    /// The arguments of the call do not make sense.
    pub fn arguments(message: impl Into<String>) -> Self {
        Self {
            error: "invalid_arguments",
            message: message.into(),
            path: None,
            line: None,
            column: None,
            node: None,
            class: None,
        }
    }

    /// The model file does not exist.
    pub fn not_found(path: impl Into<String>) -> Self {
        let path = path.into();
        Self {
            error: "model_not_found",
            message: format!("cannot read model: {path}"),
            path: Some(path),
            line: None,
            column: None,
            node: None,
            class: None,
        }
    }

    /// The model file could not be read or parsed.
    ///
    /// `model` is the parsed model when the failure is a rule the model
    /// breaks: the block the rule names then carries its position.
    pub fn from_format(
        error: &FormatError,
        path: Option<&str>,
        model: Option<&GmdlModel>,
    ) -> Self {
        let (class, message) = match error {
            FormatError::Xml { message, .. } => ("invalid_xml", format!("invalid XML: {message}")),
            FormatError::Structural { message, .. } => ("invalid_model", message.clone()),
        };
        let location = error.location();
        let failure = Self {
            error: class,
            message,
            path: path.map(str::to_string),
            line: location.line.map(|line| line as u32),
            column: None,
            node: location.node,
            class: None,
        };
        match model {
            Some(model) => failure.locate(model),
            None => failure,
        }
    }

    /// The engine refused the model.
    pub fn from_engine(error: &EngineError, model: &GmdlModel, path: &str) -> Self {
        let mut failure = match error {
            EngineError::Unsupported { node, class, message } => Self {
                error: "unsupported_block",
                message: message.clone(),
                path: Some(path.to_string()),
                line: None,
                column: None,
                node: Some(*node),
                class: Some(class.clone()),
            },
            EngineError::InvalidBlock { node, class, message } => Self {
                error: "invalid_block",
                message: message.clone(),
                path: Some(path.to_string()),
                line: None,
                column: None,
                node: Some(*node),
                class: Some(class.clone()),
            },
            EngineError::Expression { text, message } => Self {
                error: "invalid_expression",
                message: format!("expression {text:?}: {message}"),
                path: Some(path.to_string()),
                line: None,
                column: None,
                node: None,
                class: None,
            },
            EngineError::Format(format) => Self::from_format(format, Some(path), Some(model)),
            EngineError::Graph { message } => Self {
                error: "invalid_graph",
                message: message.clone(),
                path: Some(path.to_string()),
                line: None,
                column: None,
                node: None,
                class: None,
            },
            EngineError::Runtime(message) => Self {
                error: "runtime_error",
                message: message.clone(),
                path: Some(path.to_string()),
                line: None,
                column: None,
                node: None,
                class: None,
            },
        };
        failure = failure.locate(model);
        failure
    }

    /// Fill the position and the class of the offending block from the model.
    fn locate(mut self, model: &GmdlModel) -> Self {
        let Some(node) = self.node else {
            return self;
        };
        let Some(block) = model.block(node) else {
            return self;
        };
        if self.class.is_none() {
            self.class = Some(block.class.clone());
        }
        if let Some(source) = block.source {
            self.line = Some(source.line as u32);
            self.column = Some(source.column as u32);
        }
        self
    }

    /// The error as the JSON body a tool returns.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("the error body always serializes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use genmsi_format::parse_str;

    const MINI: &str = "<gmodel versioncode=\"1\"><gclasses/><modules>\
        <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"m\" name=\"m\" kind=\"C\"><links/><inputs/><outputs/></module>\
        <module moduleid=\"7\" moduleclass=\"Queue\" owner=\"0\" caption=\"q\" name=\"q\" kind=\"A\"><props/></module>\
        </modules></gmodel>";

    #[test]
    fn an_engine_failure_points_at_the_block() {
        let model = parse_str(MINI).expect("model parses");
        let error = EngineError::invalid_block(7, "Queue", "the queue name is empty");
        let failure = ToolError::from_engine(&error, &model, "model.gmdl");
        assert_eq!(failure.error, "invalid_block");
        assert_eq!(failure.message, "the queue name is empty");
        assert_eq!(failure.node, Some(7));
        assert_eq!(failure.class.as_deref(), Some("Queue"));
        assert_eq!(failure.line, Some(1));
        assert!(failure.column.is_some());
        assert_eq!(failure.path.as_deref(), Some("model.gmdl"));
    }

    #[test]
    fn the_body_omits_what_it_does_not_know() {
        let failure = ToolError::arguments("give `path` or `model`");
        let body = failure.to_json();
        assert!(body.contains("\"error\": \"invalid_arguments\""), "{body}");
        assert!(!body.contains("\"line\""), "{body}");
        assert!(!body.contains("\"node\""), "{body}");
    }

    #[test]
    fn a_missing_file_is_its_own_class() {
        let failure = ToolError::not_found("missing.gmdl");
        assert_eq!(failure.error, "model_not_found");
        assert_eq!(failure.path.as_deref(), Some("missing.gmdl"));
    }
}
