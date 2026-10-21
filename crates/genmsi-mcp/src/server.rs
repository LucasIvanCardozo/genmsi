//! The rmcp handler and the tools the server offers.
//!
//! The tools are thin adapters: each one loads a model through
//! [`crate::engine`], asks the engine for its document and hands that
//! document back unchanged. The documents are the ones the CLI publishes
//! (`genmsi.lint/1`, `genmsi.report/1`, `genmsi.trace/1`), so an agent and a
//! script read the same thing (decision D13).

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::{ServerCapabilities, ServerConfig};
use rmcp::schemars;
use rmcp::schemars::JsonSchema;
use rmcp::{tool_handler, tool_router, ServerHandler};
use serde::Deserialize;

use crate::engine::{self, ModelSource};
use crate::errors::ToolError;

/// The GenMSI MCP server: the engine served over stdio.
#[derive(Debug, Clone)]
pub struct GenmsiServer {
    tool_router: ToolRouter<GenmsiServer>,
}

impl GenmsiServer {
    /// A server with its tools.
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

impl Default for GenmsiServer {
    fn default() -> Self {
        Self::new()
    }
}

/// The model a tool works on.
///
/// A model that already exists on disk is named by `path`; a model an agent
/// is building from scratch is passed inline as `model`, the `.gmdl` document
/// as text.
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub struct ModelInput {
    /// Path of the `.gmdl` model file.
    #[serde(default)]
    pub path: Option<String>,
    /// The `.gmdl` document as text, for a model that is not on disk yet.
    #[serde(default)]
    pub model: Option<String>,
}

impl ModelInput {
    /// The model the agent asked for.
    pub fn source(&self) -> Result<ModelSource, ToolError> {
        match (&self.path, &self.model) {
            (Some(path), None) => Ok(ModelSource::File(engine::file_path(path))),
            (None, Some(text)) => Ok(ModelSource::Inline(text.clone())),
            (Some(_), Some(_)) => Err(ToolError::arguments(
                "give `path` or `model`, not both: one names the file, the other carries the document",
            )),
            (None, None) => Err(ToolError::arguments(
                "give `path` of a model file or `model` with the `.gmdl` document",
            )),
        }
    }
}

/// The model and the knobs of a run.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RunInput {
    /// Path of the `.gmdl` model file.
    #[serde(default)]
    pub path: Option<String>,
    /// The `.gmdl` document as text, for a model that is not on disk yet.
    #[serde(default)]
    pub model: Option<String>,
    /// Seed of every random stream; the same seed reproduces the same run.
    #[serde(default = "default_seed")]
    pub seed: u64,
    /// Stop once the simulated clock passes this time.
    #[serde(default)]
    pub until: Option<f64>,
    /// Maximum number of trace records to keep.
    #[serde(default = "default_limit")]
    pub limit: usize,
}

impl RunInput {
    /// The model the agent asked for.
    pub fn source(&self) -> Result<ModelSource, ToolError> {
        ModelInput {
            path: self.path.clone(),
            model: self.model.clone(),
        }
        .source()
    }
}

/// The default seed of a run, the same default the CLI uses.
fn default_seed() -> u64 {
    1
}

/// The default number of trace records, the same default the CLI uses.
fn default_limit() -> usize {
    100
}

#[tool_router(router = tool_router, allow_empty)]
impl GenmsiServer {}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for GenmsiServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "GenMSI: a discrete-event simulation engine for GPSS-style models. \
             Use `lint` to validate a model, `simulate` to run it, `get_trace` to read \
             its events and `explain_model` to describe it. Give the model either as \
             `path` (a .gmdl file) or as `model` (the .gmdl document as text).",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_names_a_file() {
        let input = ModelInput {
            path: Some("model.gmdl".to_string()),
            model: None,
        };
        assert!(matches!(input.source(), Ok(ModelSource::File(_))));
    }

    #[test]
    fn a_document_is_the_model_itself() {
        let input = ModelInput {
            path: None,
            model: Some("<gmodel/>".to_string()),
        };
        assert!(matches!(input.source(), Ok(ModelSource::Inline(_))));
    }

    #[test]
    fn two_sources_are_ambiguous() {
        let input = ModelInput {
            path: Some("model.gmdl".to_string()),
            model: Some("<gmodel/>".to_string()),
        };
        let error = input.source().expect_err("must fail");
        assert_eq!(error.error, "invalid_arguments");
    }

    #[test]
    fn no_source_is_an_argument_error() {
        let error = ModelInput::default().source().expect_err("must fail");
        assert_eq!(error.error, "invalid_arguments");
    }
}
