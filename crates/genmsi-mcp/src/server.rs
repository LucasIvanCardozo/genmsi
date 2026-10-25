//! The rmcp handler and the tools the server offers.
//!
//! The tools are thin adapters: each one loads a model through
//! [`crate::engine`], asks the engine for its document and hands that
//! document back unchanged. The documents are the ones the CLI publishes
//! (`genmsi.lint/1`, `genmsi.report/1`, `genmsi.trace/1`), so an agent and a
//! script read the same thing (decision D13).

use std::collections::BTreeMap;
use std::path::PathBuf;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, ServerCapabilities, ServerConfig};
use rmcp::schemars;
use rmcp::schemars::JsonSchema;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};
use serde::{Deserialize, Serialize};

use crate::engine::{self, ModelSource};
use crate::errors::ToolError;
use crate::glossary;

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
        source_of(&self.path, &self.model)
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
    /// The `START` of the run: how many termination units end it.
    ///
    /// The original has the same field in the panel of its simulator ("Start:",
    /// one by default) and the model does not declare it: it is a parameter of
    /// the run, like the horizon. It has to be greater than zero.
    #[serde(default = "default_start")]
    pub start: u64,
}

impl RunInput {
    /// The model the agent asked for.
    pub fn source(&self) -> Result<ModelSource, ToolError> {
        source_of(&self.path, &self.model)
    }
}

/// The model, the knobs of a run and the bound of the trace.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TraceInput {
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
    /// The `START` of the run: how many termination units end it.
    ///
    /// The original has the same field in the panel of its simulator ("Start:",
    /// one by default) and the model does not declare it: it is a parameter of
    /// the run, like the horizon. It has to be greater than zero.
    #[serde(default = "default_start")]
    pub start: u64,
    /// Maximum number of trace records to keep.
    #[serde(default = "default_limit")]
    pub limit: usize,
}

impl TraceInput {
    /// The model the agent asked for.
    pub fn source(&self) -> Result<ModelSource, ToolError> {
        source_of(&self.path, &self.model)
    }
}

/// Where a model goes and what goes there.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SaveInput {
    /// Path of the `.gmdl` file to write.
    pub path: String,
    /// The whole `.gmdl` document as text.
    ///
    /// The tool writes the whole document, not a patch: the agent reads the
    /// model, changes it and hands the complete text back.
    pub model: String,
}

/// The class the agent asks about.
#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub struct CatalogueInput {
    /// Name of the class to describe (`Generate`, `Queue`, ...).
    ///
    /// Without it the tool returns the list of the classes and the skeleton
    /// of a model.
    #[serde(default)]
    pub class: Option<String>,
    /// Names of several classes to describe, in one call.
    ///
    /// An agent that needs eight classes asks for the eight: one call
    /// instead of eight. Give `class` or `classes`, not both.
    #[serde(default)]
    pub classes: Option<Vec<String>>,
}

/// The model the agent asked for: a file on disk or the document itself.
fn source_of(path: &Option<String>, model: &Option<String>) -> Result<ModelSource, ToolError> {
    match (path, model) {
        (Some(path), None) => Ok(ModelSource::File(PathBuf::from(path))),
        (None, Some(text)) => Ok(ModelSource::Inline(text.clone())),
        (Some(_), Some(_)) => Err(ToolError::arguments(
            "give `path` or `model`, not both: one names the file, the other carries the document",
        )),
        (None, None) => Err(ToolError::arguments(
            "give `path` of a model file or `model` with the `.gmdl` document",
        )),
    }
}

/// The default seed of a run: one drawn at random, as the CLI does.
///
/// The run says which seed it used, so the caller can repeat it.
fn default_seed() -> u64 {
    genmsi_core::random_seed()
}

/// The default `START` of a run: one, as the original's panel shows.
fn default_start() -> u64 {
    genmsi_core::DEFAULT_START
}

/// The `START` of a run, checked.
///
/// The original refuses a `Start` of zero ("El Start debe ser mayor a
/// cero."), and so does the server: a run whose counter of terminations
/// starts at zero would end before it begins.
fn checked_start(start: u64) -> Result<u64, ToolError> {
    if start == 0 {
        return Err(ToolError::arguments("the start must be greater than zero"));
    }
    Ok(start)
}

/// The default number of trace records, the same default the CLI uses.
fn default_limit() -> usize {
    100
}

#[tool_router(router = tool_router)]
impl GenmsiServer {
    /// Validate a model: it loads, it validates and it compiles.
    ///
    /// Returns the `genmsi.lint/1` document. A model that does not compile is
    /// a tool error that names the block and its position in the source.
    #[tool(
        name = "lint",
        description = "Validate a GenMSI model: it loads, validates and compiles, and reports what it found (the compiler's observations and the warnings of the format, such as an entity the model uses without declaring it). Takes `path` (a .gmdl file) or `model` (the .gmdl document as text) and returns the genmsi.lint/1 document."
    )]
    pub fn lint(&self, Parameters(input): Parameters<ModelInput>) -> CallToolResult {
        finish(lint_document(&input))
    }

    /// Run a model and return its report.
    ///
    /// Returns the `genmsi.report/1` document, the same one `genmsi simulate
    /// --json` prints.
    #[tool(
        name = "simulate",
        description = "Simulate a GenMSI model and return its report. Takes `path` or `model`, plus the optional `seed`, `until` and `start`, and returns the genmsi.report/1 document."
    )]
    pub fn simulate(&self, Parameters(input): Parameters<RunInput>) -> CallToolResult {
        finish(report_document(&input))
    }

    /// Run a model and return its event trace.
    ///
    /// Returns the `genmsi.trace/1` document, the same one `genmsi trace
    /// --json` prints.
    #[tool(
        name = "get_trace",
        description = "Simulate a GenMSI model and return its event trace. Takes `path` or `model`, plus the optional `seed`, `until`, `start` and `limit`, and returns the genmsi.trace/1 document."
    )]
    pub fn get_trace(&self, Parameters(input): Parameters<TraceInput>) -> CallToolResult {
        finish(trace_document(&input))
    }

    /// Describe a model: its blocks, its operands and what each statement does.
    ///
    /// Returns the `genmsi.explain/1` document: the engine's own program
    /// description plus the glossary of the classes the model uses.
    #[tool(
        name = "explain_model",
        description = "Describe a GenMSI model: its blocks, operands and ports, plus what each GPSS statement of the model does. Takes `path` (a .gmdl file) or `model` (the .gmdl document as text) and returns the genmsi.explain/1 document."
    )]
    pub fn explain_model(&self, Parameters(input): Parameters<ModelInput>) -> CallToolResult {
        finish(explain_document(&input))
    }

    /// Describe the block classes a model is built from.
    ///
    /// Returns the `genmsi.classlist/1` document (the classes there are and
    /// the skeleton of a model) or, when the agent names a class, the
    /// `genmsi.class/1` document of that class: what it does, its ports, its
    /// properties and the XML of a block of it.
    #[tool(
        name = "catalogue",
        description = "Describe the block classes a GenMSI model is built from. Call it BEFORE writing a model: it tells you which classes exist, what each one does (its description), which ports (ioid) and properties (name, hint, ty, subtype) each class declares, which entities the class declares and which it uses, and the exact XML of a block of that class —with the default value of each property— so you never have to guess an element name. Without arguments it returns the genmsi.classlist/1 document (every class plus the skeleton of a model). With `class` (for example \"Generate\") it returns the genmsi.class/1 document of that class; with `classes` (a list of names) it returns the genmsi.classes/1 document with every class asked for, so an agent that needs several classes asks for them all in ONE call."
    )]
    pub fn catalogue(&self, Parameters(input): Parameters<CatalogueInput>) -> CallToolResult {
        finish(catalogue_document(input.class.as_deref(), input.classes.as_deref()))
    }

    /// Say which model the app has open.
    ///
    /// Returns the `genmsi.openmodel/1` document.
    #[tool(
        name = "open_model",
        description = "Say which model the GenMSI desktop app has open, so you do not have to guess where to write it. It returns the genmsi.openmodel/1 document: `path` is the absolute path of the file the app has open and `pid` is the process that has it open; both are null when the app is not running. Write the model to that path (with save_model) and the app, which watches the file it has open, reloads it and runs it again, so the reader sees the change."
    )]
    pub fn open_model(&self) -> CallToolResult {
        finish(Ok(genmsi_core::open_model_document()))
    }

    /// Write a model to a file.
    ///
    /// The model is checked first: a model that does not load, does not
    /// validate or does not compile is **not** written, and the error names
    /// the block that caused it.
    ///
    /// A model that carries no canvas geometry is composed before it is
    /// written, so the editor can draw it; a model that already carries its
    /// geometry keeps the disposition its author left.
    #[tool(
        name = "save_model",
        description = "Write a GPSS model to a .gmdl file. The model is checked before it is written: one that does not load, does not validate or does not compile is NOT written, and the error names the block that caused it. A model that carries no canvas geometry is composed before it is written, so the editor can draw it; a model that already carries its geometry keeps the disposition its author left. Takes `path` (the file to write) and `model` (the whole .gmdl document as text, not a patch) and returns the genmsi.lint/1 document of what was written. Use it to create or to change a model: read it first with explain_model, change the text and hand the complete document back."
    )]
    pub fn save_model(&self, Parameters(input): Parameters<SaveInput>) -> CallToolResult {
        finish(save_document(&input))
    }
}

/// Body of the `genmsi.explain/1` document.
#[derive(Debug, Serialize)]
struct ExplainBody {
    /// The engine's description of the compiled program.
    program: serde_json::Value,
    /// The model as the document writes it: every block with its properties
    /// by name, and the links that wire the blocks.
    model: genmsi_core::ModelDocument,
    /// What each statement of the model does.
    glossary: BTreeMap<String, String>,
}

/// The explain document of a model.
fn explain_document(input: &ModelInput) -> Result<String, ToolError> {
    let loaded = engine::load(&input.source()?)?;
    let body = ExplainBody {
        program: genmsi_core::program_json(&loaded.program),
        model: genmsi_core::model_document(&loaded.model, &loaded.name),
        glossary: glossary::for_model(&loaded.model),
    };
    Ok(genmsi_core::to_pretty_json(
        genmsi_core::EXPLAIN_SCHEMA,
        &body,
    ))
}

/// The catalogue document: the list of the classes, the detail of one class,
/// or the detail of several classes in one call.
fn catalogue_document(
    class: Option<&str>,
    classes: Option<&[String]>,
) -> Result<String, ToolError> {
    match (class, classes) {
        (Some(_), Some(_)) => Err(ToolError::arguments(
            "give `class` or `classes`, not both: `class` names one class and `classes` names several",
        )),
        (None, None) => Ok(genmsi_core::to_pretty_json(
            genmsi_core::CLASSLIST_SCHEMA,
            &genmsi_core::class_list_document(),
        )),
        (Some(name), None) => {
            let document = genmsi_core::class_document(name)
                .ok_or_else(|| ToolError::class_not_found(name))?;
            Ok(genmsi_core::to_pretty_json(
                genmsi_core::CLASS_SCHEMA,
                &document,
            ))
        }
        (None, Some(names)) => {
            let mut classes = Vec::with_capacity(names.len());
            for name in names {
                classes.push(
                    genmsi_core::class_document(name)
                        .ok_or_else(|| ToolError::class_not_found(name))?,
                );
            }
            Ok(genmsi_core::to_pretty_json(
                genmsi_core::CLASSES_SCHEMA,
                &genmsi_core::ClassesDocument { classes },
            ))
        }
    }
}

/// The lint document of a model.
fn lint_document(input: &ModelInput) -> Result<String, ToolError> {
    let loaded = engine::load(&input.source()?)?;
    let document = genmsi_core::lint_document(&loaded.program, &loaded.model, &loaded.name);
    Ok(genmsi_core::to_pretty_json(
        genmsi_core::LINT_SCHEMA,
        &document,
    ))
}

/// The lint document of a model that was written.
fn save_document(input: &SaveInput) -> Result<String, ToolError> {
    let path = PathBuf::from(&input.path);
    let saved = engine::save_model(&path, &input.model)?;
    let mut document = genmsi_core::lint_document(
        &saved.program,
        &saved.model,
        &saved.path.display().to_string(),
    );
    // The composition is not a finding of the compiler: it is what the
    // engine did to the model before writing it, and the agent has to know
    // it, because the file it wrote is not the file it sent.
    if let Some(blocks) = saved.composed {
        document.diagnostics.push(composition_note(blocks));
        document.status = "diagnostics";
    }
    Ok(genmsi_core::to_pretty_json(
        genmsi_core::LINT_SCHEMA,
        &document,
    ))
}

/// What the lint document says when the engine composed the model.
///
/// The number is the blocks the composition placed: the model root is not
/// drawn, so it is not one of them.
fn composition_note(blocks: usize) -> String {
    format!(
        "the model carried no canvas geometry: the engine composed {blocks} blocks so the editor can draw it"
    )
}

/// The report document of a run.
fn report_document(input: &RunInput) -> Result<String, ToolError> {
    let loaded = engine::load(&input.source()?)?;
    let start = checked_start(input.start)?;
    let config = engine::config(input.seed, input.until, start, false, 0);
    let result = engine::run(&loaded, config)?;
    let document = genmsi_core::json_report(&result, &loaded.program);
    Ok(genmsi_core::to_pretty_json(
        genmsi_core::REPORT_SCHEMA,
        &document,
    ))
}

/// The trace document of a run.
fn trace_document(input: &TraceInput) -> Result<String, ToolError> {
    let loaded = engine::load(&input.source()?)?;
    let start = checked_start(input.start)?;
    let config = engine::config(input.seed, input.until, start, true, input.limit);
    let result = engine::run(&loaded, config)?;
    let document = genmsi_core::trace_document(&result, input.limit);
    Ok(genmsi_core::to_pretty_json(
        genmsi_core::TRACE_SCHEMA,
        &document,
    ))
}

/// The outcome of a tool: the document it produced, or the failure it found.
fn finish(outcome: Result<String, ToolError>) -> CallToolResult {
    match outcome {
        Ok(document) => CallToolResult::success(vec![ContentBlock::text(document)]),
        Err(error) => CallToolResult::error(vec![ContentBlock::text(error.to_json())]),
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for GenmsiServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "GenMSI: a discrete-event simulation engine for GPSS-style models. \
             Call `catalogue` before you write a model: it says which block classes \
             exist and what the XML of a block of each class looks like; ask for \
             every class you need in ONE call, with `classes`. \
             Use `lint` to validate a model, `simulate` to run it, `get_trace` to read \
             its events and `explain_model` to describe it: its `program` is the \
             compiled GPSS and its `model` is the document itself, with every block, \
             its properties by name and the links that wire the blocks. Give the model \
             either as `path` (a .gmdl file) or as `model` (the .gmdl document as text). \
             Two blocks are connected by a link inside the `<links>` of their container: \
             `<link frommoduleid=\"1\" fromioid=\"1\" tomoduleid=\"2\" toioid=\"1\"/>`, \
             where the ioids are the ports the catalogue publishes for the class. \
             Call `open_model` to learn which file the desktop app has open: write \
             the model there and the app reloads it and runs it again. \
             The `genmsi-modeling` skill carries the rules of the format and the \
             procedure to follow.",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINI: &str = "<gmodel versioncode=\"1\"><gclasses/><modules>\
        <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"m\" name=\"m\" kind=\"C\"><links>\
        <link frommoduleid=\"1\" fromioid=\"1\" tomoduleid=\"2\" toioid=\"1\"/>\
        </links><inputs/><outputs/></module>\
        <module moduleid=\"1\" moduleclass=\"Generate\" owner=\"0\" caption=\"g\" name=\"g\" kind=\"A\"><outputs><output ioid=\"1\" name=\"Salida1\" caption=\"\" mylabel=\"L1\"/></outputs>\
        <props><property name=\"Media\" hint=\"\" value=\"10\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/></props></module>\
        <module moduleid=\"2\" moduleclass=\"Terminate\" owner=\"0\" caption=\"t\" name=\"t\" kind=\"A\"><inputs><input ioid=\"1\" name=\"i\" caption=\"\" mylabel=\"L1\"/></inputs>\
        <props><property name=\"Unidades\" hint=\"\" value=\"1\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/></props></module>\
        </modules></gmodel>";

    fn model_input() -> ModelInput {
        ModelInput {
            path: None,
            model: Some(MINI.to_string()),
        }
    }

    fn run_input() -> RunInput {
        RunInput {
            path: None,
            model: Some(MINI.to_string()),
            seed: 1,
            until: None,
            start: 1,
        }
    }

    fn trace_input() -> TraceInput {
        TraceInput {
            path: None,
            model: Some(MINI.to_string()),
            seed: 1,
            until: None,
            start: 1,
            limit: 100,
        }
    }

    fn text_of(result: &CallToolResult) -> String {
        let block = &result.content[0];
        match block {
            ContentBlock::Text(content) => content.text.clone(),
            other => panic!("expected text content, got {other:?}"),
        }
    }

    #[test]
    fn lint_returns_the_lint_document() {
        let result = GenmsiServer::new().lint(Parameters(model_input()));
        assert_eq!(result.is_error, Some(false));
        let text = text_of(&result);
        assert!(text.contains("\"schema\": \"genmsi.lint/1\""), "{text}");
        assert!(text.contains("\"status\": \"ok\""), "{text}");
    }

    #[test]
    fn simulate_returns_the_report_document() {
        let result = GenmsiServer::new().simulate(Parameters(run_input()));
        assert_eq!(result.is_error, Some(false));
        let text = text_of(&result);
        assert!(text.contains("\"schema\": \"genmsi.report/1\""), "{text}");
        assert!(text.contains("\"end_reason\": \"terminated\""), "{text}");
    }

    #[test]
    fn get_trace_returns_the_trace_document() {
        let result = GenmsiServer::new().get_trace(Parameters(trace_input()));
        assert_eq!(result.is_error, Some(false));
        let text = text_of(&result);
        assert!(text.contains("\"schema\": \"genmsi.trace/1\""), "{text}");
        assert!(text.contains("\"event\": \"generate\""), "{text}");
    }

    #[test]
    fn a_model_without_a_path_is_an_argument_error() {
        let result = GenmsiServer::new().lint(Parameters(ModelInput::default()));
        assert_eq!(result.is_error, Some(true));
        let text = text_of(&result);
        assert!(text.contains("\"error\": \"invalid_arguments\""), "{text}");
    }

    #[test]
    fn explain_model_describes_the_model_and_its_statements() {
        let result = GenmsiServer::new().explain_model(Parameters(model_input()));
        assert_eq!(result.is_error, Some(false));
        let text = text_of(&result);
        assert!(text.contains("\"schema\": \"genmsi.explain/1\""), "{text}");
        assert!(text.contains("\"program\""), "{text}");
        assert!(text.contains("\"Generate\""), "{text}");
        assert!(text.contains("\"glossary\""), "{text}");
    }

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
