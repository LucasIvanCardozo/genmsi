//! The rmcp handler and the tools the server offers.
//!
//! The tools are thin adapters: each one loads a model through
//! [`crate::engine`], asks the engine for its document and hands that
//! document back unchanged. The documents are the ones the CLI publishes
//! (`genmsi.lint/1`, `genmsi.report/1`, `genmsi.trace/1`), so an agent and a
//! script read the same thing (decision D13).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, ServerCapabilities, ServerConfig};
use rmcp::schemars;
use rmcp::schemars::JsonSchema;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};
use serde::{Deserialize, Serialize};

use genmsi_format::EditCommand;

use crate::engine::{self, Destination, Layout, ModelSource};
use crate::errors::{Failure, ToolError};
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
    pub fn source(&self) -> Result<ModelSource, Failure> {
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
    pub fn source(&self) -> Result<ModelSource, Failure> {
        source_of(&self.path, &self.model)
    }
}

/// The model, the knobs of a run, the bound of the trace and its filters.
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
    ///
    /// The limit counts the events that pass the filter, and not the events
    /// of the run.
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Keep only the events of these classes of block.
    ///
    /// The names are the classes of the catalogue (`"Queue"`, `"Terminate"`,
    /// ...). Empty (the default) keeps the events of every class.
    #[serde(default)]
    pub classes: Vec<String>,
    /// Keep only the events of these blocks of the document.
    ///
    /// The numbers are the `moduleid` of the blocks —the `node` the events
    /// carry and the `node` the map of the explain publishes—. Empty (the
    /// default) keeps the events of every block.
    #[serde(default)]
    pub nodes: Vec<u32>,
    /// Keep only the events of this transaction.
    ///
    /// Without it the events of every transaction are kept.
    #[serde(default)]
    pub xact: Option<u64>,
    /// Keep only the events from this simulated time on.
    #[serde(default)]
    pub from: Option<f64>,
    /// Keep only the events until this simulated time.
    #[serde(default)]
    pub to: Option<f64>,
    /// Answer with the summary of the run instead of its events.
    ///
    /// The summary says how many times the flow went through each block, and
    /// the first and the last time it went through it.
    #[serde(default)]
    pub summary: bool,
}

impl TraceInput {
    /// The model the agent asked for.
    pub fn source(&self) -> Result<ModelSource, Failure> {
        source_of(&self.path, &self.model)
    }

    /// The filter the agent asked for.
    ///
    /// The filter goes to the engine, which applies it when the event is
    /// recorded: an empty filter keeps every event, exactly as a run without
    /// a filter does.
    pub fn filter(&self) -> genmsi_core::TraceFilter {
        genmsi_core::TraceFilter {
            classes: self.classes.clone(),
            nodes: self.nodes.clone(),
            xact: self.xact,
            from: self.from,
            to: self.to,
        }
    }
}

/// Where a model goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    /// The model goes to the file it was named by.
    #[default]
    File,
    /// The model goes to the staging area of the app: it does not touch the
    /// disk, and the person decides.
    Proposal,
}

/// Where a model goes and what goes there.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SaveInput {
    /// Path of the `.gmdl` file to write.
    ///
    /// Without it the model the app has open is the one that is written.
    #[serde(default)]
    pub path: Option<String>,
    /// The whole `.gmdl` document as text.
    ///
    /// The tool writes the whole document, not a patch: the agent reads the
    /// model, changes it and hands the complete text back.
    #[serde(default)]
    pub model: Option<String>,
    /// Where the model goes.
    ///
    /// `file` (the default) writes the model to the file it was named by;
    /// `proposal` leaves it in the staging area of the app, without touching
    /// the disk, and the person decides.
    #[serde(default)]
    pub target: Target,
    /// What the agent did, in one line.
    ///
    /// It travels with the proposal, so the person reads it before deciding.
    #[serde(default)]
    pub summary: Option<String>,
}

/// The batch of editing commands an agent applies to a model.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct EditInput {
    /// Path of the `.gmdl` model file.
    #[serde(default)]
    pub path: Option<String>,
    /// The `.gmdl` document as text, for a model that is not on disk yet.
    #[serde(default)]
    pub model: Option<String>,
    /// The commands to apply, in order.
    ///
    /// The commands are the engine's own editing commands: the same ones the
    /// desktop application speaks. The schema the tool publishes is the one of
    /// those types, so the vocabulary is the same everywhere.
    pub commands: Vec<EditCommand>,
    /// Where the blocks the commands add go.
    ///
    /// `auto` (the default) puts every new block where there is room in its
    /// container, like a drop in the application; `explicit` uses the `x` and
    /// the `y` of the command.
    #[serde(default)]
    pub layout: Layout,
    /// Where the result goes.
    ///
    /// `file` (the default) writes the model to the file the batch edited;
    /// `proposal` leaves it in the staging area of the app, without touching
    /// the disk, and the person decides.
    #[serde(default)]
    pub target: Target,
    /// What the agent did, in one line.
    ///
    /// It travels with the proposal, so the person reads it before deciding.
    #[serde(default)]
    pub summary: Option<String>,
    /// Try the batch without writing it.
    ///
    /// The commands are applied and the model is compiled, and **nothing is
    /// written**: the answer says what would have happened, and carries the
    /// model it would have written.
    #[serde(default)]
    pub dry_run: bool,
}

impl EditInput {
    /// The model the agent asked for.
    pub fn source(&self) -> Result<ModelSource, Failure> {
        source_of(&self.path, &self.model)
    }
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
///
/// When the agent names neither, the model the app has open is the one it
/// means: the agent works on what the reader is looking at.
fn source_of(path: &Option<String>, model: &Option<String>) -> Result<ModelSource, Failure> {
    match (path, model) {
        (Some(path), None) => Ok(ModelSource::File(PathBuf::from(path))),
        (None, Some(text)) => Ok(ModelSource::Inline(text.clone())),
        (Some(_), Some(_)) => Err(Box::new(ToolError::arguments(
            "give `path` or `model`, not both: one names the file, the other carries the document",
        ))),
        (None, None) => Ok(ModelSource::File(open_model_path()?)),
    }
}

/// The path of the model the app has open.
///
/// The app publishes the model it has open, and the pid of the process that
/// has it: an app that is not running any more says nothing, so the model it
/// left behind is not the destination of anybody.
///
/// Without a model open there is no destination to guess: the failure is its
/// own class, and it says what to do.
fn open_model_path() -> Result<PathBuf, Failure> {
    match genmsi_core::live::open_model_from_live().and_then(|document| document.path) {
        Some(path) => Ok(PathBuf::from(path)),
        None => Err(Box::new(ToolError::no_open_model())),
    }
}

/// The file a model goes to: the one the agent named, or the one the app has
/// open.
fn destination_path(path: &Option<String>) -> Result<PathBuf, Failure> {
    match path {
        Some(path) => Ok(PathBuf::from(path)),
        None => open_model_path(),
    }
}

/// The text of a model, as the agent sent it.
fn model_text(input: &SaveInput) -> Result<&str, Failure> {
    match input.model.as_deref() {
        Some(text) => Ok(text),
        None => Err(Box::new(ToolError::arguments(
            "give `model` with the whole .gmdl document to write",
        ))),
    }
}

/// A proposal left in the staging area of the app.
struct Staged {
    /// The identity of the proposal.
    id: String,
    /// The model the proposal carries.
    model: PathBuf,
    /// The document that describes the proposal: where its verdict lives.
    document: PathBuf,
}

/// Leave a model in the staging area of the app.
///
/// The model does not touch the disk: it waits in the staging directory of
/// the app for the app to adopt it, and the person decides. The proposal
/// document describes it: what it is, what it proposes to change and what the
/// agent says it did.
fn stage(text: &str, target: &Path, summary: Option<&str>) -> Result<Staged, Failure> {
    let directory = genmsi_core::app_data_dir().ok_or_else(|| {
        Box::new(ToolError::cannot_stage(
            "the machine does not say where the data of the app lives",
        )) as Failure
    })?;
    let id = genmsi_core::new_id();
    let model = genmsi_core::write_proposal_model(&directory, &id, text)
        .map_err(ToolError::cannot_stage)?;
    let document = genmsi_core::ProposalDocument::pending(&id, &model, Some(target))
        .with_summary(summary);
    let written = genmsi_core::write_proposal(&directory, &document).map_err(ToolError::cannot_stage)?;
    Ok(Staged {
        id,
        model,
        document: written,
    })
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
fn checked_start(start: u64) -> Result<u64, Failure> {
    if start == 0 {
        return Err(Box::new(ToolError::arguments(
            "the start must be greater than zero",
        )));
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
    /// --json` prints. The trace can be filtered, and the run can be
    /// summarized.
    #[tool(
        name = "get_trace",
        description = "Simulate a GenMSI model and return its event trace, or —with `summary`— the summary of the run. Takes `path` or `model`, plus the optional `seed`, `until`, `start` and `limit`, and returns the genmsi.trace/1 document. The trace can be FILTERED: `classes` keeps only the events of those block classes (\"Queue\", \"Terminate\", ...), `nodes` keeps only the events of those blocks of the document (their `moduleid`, the `node` every event carries), `xact` keeps only the events of that transaction, and `from` and `to` keep only the events of that window of simulated time (the window is closed at both ends; `until` is something else: it is the horizon of the run). The filter is applied in the engine WHEN THE EVENT IS RECORDED, so `limit` counts the events that pass the filter and not the events of the run: asking for the events of one class gives those events, not the first ones of the run. Without a filter the answer is the whole trace, exactly as before. With `summary: true` the tool answers the genmsi.trace_summary/1 document instead: for every block, how many times the flow went through it and the first and the last time it went through it — the question the agent asks the ninety percent of the time (\"how many times did the flow go through here?\") — and not the events. The summary is the summary of the WHOLE run, and the filters do not cut it."
    )]
    pub fn get_trace(&self, Parameters(input): Parameters<TraceInput>) -> CallToolResult {
        finish(trace_document(&input))
    }

    /// Describe a model: its blocks, its operands and what each statement does.
    ///
    /// Returns the `genmsi.explain/1` document: the engine's own program
    /// description, the map between the statements and the blocks of the
    /// document, and the glossary of the classes the model uses.
    #[tool(
        name = "explain_model",
        description = "Describe a GenMSI model: its blocks, operands and ports, plus what each GPSS statement of the model does. Takes `path` (a .gmdl file) or `model` (the .gmdl document as text) and returns the genmsi.explain/1 document. The document has three parts: `program` is the compiled GPSS (one entry per statement, with the block of the document it came from and the line and column of that block in the .gmdl), `model` is the document itself, with every block, its properties by name and the links that wire the blocks, and `map` joins the two: one row per compiled statement, with the `node` of the block of the document it came from, the class of that statement, the name the agent gave the block (its `caption`) and the line and column of the block in the .gmdl. With the `node` that every event of the trace carries, the map answers \"what block is the statement 218?\" in one line."
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

    /// Say what the app is showing right now.
    ///
    /// Returns the `genmsi.live/1` document: the model the app has open,
    /// whether the reader has changes that are not saved on it, what the app
    /// found in it and the last run it made.
    #[tool(
        name = "app_state",
        description = "See what the GenMSI desktop app is showing right now: the model it has open (`path`), whether the reader has changes that are not saved on it (`dirty`), what the app found in that model (`diagnostics`) and the last run the app made (`run`). It returns the genmsi.live/1 document, the one the app publishes; when the app is not running the document has `path: null` —that is an answer, not a failure. Ask it BEFORE you write on the model the app has open: `apply_edit` and `save_model` refuse with `model_locked` while the app has unsaved changes on that model, so tell the reader to save it and send the same edit again."
    )]
    pub fn app_state(&self) -> CallToolResult {
        finish(Ok(app_state_document()))
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
        description = "Write a GPSS model to a .gmdl file. The model is checked before it is written: one that does not load, does not validate or does not compile is NOT written, and the error names the block that caused it. A model that carries no canvas geometry is composed before it is written, so the editor can draw it; a model that already carries its geometry keeps the disposition its author left. Takes `model` (the whole .gmdl document as text, not a patch) and `path` (the file to write): `path` may be left out, and then the model the app has open is the one that is written. `target` says where the model goes: `file` (the default) writes it to that file, and `proposal` leaves the model in the staging area of the app, without touching the disk (the `path` is not written to), where the person sees it and decides to accept it or to discard it — the verdict comes back in `app_state`. `summary` is one line about what the agent did; it travels with the proposal. Returns the genmsi.lint/1 document of what was written. Use it to create or to change a model: read it first with explain_model, change the text and hand the complete document back."
    )]
    pub fn save_model(&self, Parameters(input): Parameters<SaveInput>) -> CallToolResult {
        finish(save_document(&input))
    }

    /// Add, edit and connect the blocks of a model.
    ///
    /// The commands are the engine's own editing commands, applied in order.
    /// The batch is atomic: a command that fails leaves the model as it was.
    #[tool(
        name = "apply_edit",
        description = "Add, edit and connect the blocks of a GenMSI model. The commands are the engine's own editing commands, the same ones the desktop application speaks: `addBlock` {class, container, x, y} creates a block of a catalogue class (call `catalogue` first: it gives the class names, their ports (ioid) and their properties), `connect` {from, fromIoid, to, toIoid} wires an output port to an input port, `setProperty` {id, property, value} sets a declared property, `setCaption` {id, caption} names a block, `moveBlock` {id, x, y} moves one, `removeBlock` {id} takes one away and `removeBlocks` {ids} takes several away as ONE operation; the schema of the tool carries the whole vocabulary. The commands are applied in order and the batch is ATOMIC: if one fails, none is applied and the failure names the command that failed —its index, its `op`, the block it was about with the class and the name the agent gave it, and the property it was setting. `layout` says where the blocks the batch adds go: `auto` (the default) puts every block an `addBlock` adds where there is room in its container, like a drop in the application; `explicit` uses the `x` and the `y` of the command. `dry_run` tries the batch WITHOUT writing it: the commands are applied and the model is compiled, nothing is written, and the answer carries the model it would have written. The model is checked before it is written: one that does not compile is NOT written, and the error names the block that caused it. The blocks the batch creates take the next free ids, in the order of the commands. Takes `path` (a .gmdl file) or `model` (the .gmdl document as text): `path` may be left out, and then the model the app has open is the one the batch edits; a `path` that is not on the disk yet is a model the batch CREATES, and the file is written when the model compiles. `target` says where the result goes: `file` (the default) writes it to the file the batch edited, and `proposal` leaves the model in the staging area of the app, without touching the disk, where the person sees it and decides to accept it or to discard it — the verdict comes back in `app_state`. `summary` is one line about what the batch did; it travels with the proposal. Returns the genmsi.edit/1 document."
    )]
    pub fn apply_edit(&self, Parameters(input): Parameters<EditInput>) -> CallToolResult {
        finish(edit_document(&input))
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
    /// The map between the compiled statements and the blocks of the document.
    map: Vec<StatementMap>,
}

/// The map between the compiled statements and the blocks of the document.
///
/// The trace numbers the **statements** of the compiled program, and the
/// model numbers the **blocks** of the document: this map joins the two, so
/// the agent does not have to invert the order of the compilation by hand.
#[derive(Debug, Clone, Serialize)]
pub struct StatementMap {
    /// The statement: its index in the program.
    pub statement: usize,
    /// The block of the document it came from (its `moduleid`).
    pub node: u32,
    /// The class of the block.
    pub class: String,
    /// The name the agent gave the block (its `caption`).
    pub caption: String,
    /// The line of the block in the `.gmdl`, when it has a position.
    pub line: Option<u32>,
    /// The column of the block in the `.gmdl`, when it has a position.
    pub column: Option<u32>,
}

/// The map between the compiled statements and the blocks of the document.
///
/// One row per statement, in the order of the program: the program knows the
/// block of the document the statement came from, the class it compiled to
/// and the place of that block in the `.gmdl`, and the model knows the name
/// the agent gave the block.
fn statement_map(loaded: &engine::Loaded) -> Vec<StatementMap> {
    loaded
        .program
        .blocks
        .iter()
        .enumerate()
        .map(|(statement, block)| StatementMap {
            statement,
            node: block.module_id,
            class: block.class.clone(),
            caption: loaded
                .model
                .block(block.module_id)
                .map(|module| module.caption.clone())
                .unwrap_or_default(),
            line: block.source.map(|position| position.line as u32),
            column: block.source.map(|position| position.column as u32),
        })
        .collect()
}

/// The explain document of a model.
fn explain_document(input: &ModelInput) -> Result<String, Failure> {
    let loaded = engine::load(&input.source()?)?;
    let body = ExplainBody {
        program: genmsi_core::program_json(&loaded.program),
        model: genmsi_core::model_document(&loaded.model, &loaded.name),
        glossary: glossary::for_model(&loaded.model),
        map: statement_map(&loaded),
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
) -> Result<String, Failure> {
    match (class, classes) {
        (Some(_), Some(_)) => Err(Box::new(ToolError::arguments(
            "give `class` or `classes`, not both: `class` names one class and `classes` names several",
        ))),
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
fn lint_document(input: &ModelInput) -> Result<String, Failure> {
    let loaded = engine::load(&input.source()?)?;
    let document = genmsi_core::lint_document(&loaded.program, &loaded.model, &loaded.name);
    Ok(genmsi_core::to_pretty_json(
        genmsi_core::LINT_SCHEMA,
        &document,
    ))
}

/// The live state of the app, as the core publishes it.
///
/// The core owns the shape of the document (D19): the app writes it in its
/// data directory and the MCP reads it there, so neither of them invents
/// the state of the other. Without an app the answer is the empty document,
/// with no path: an answer is never a failure (the same rule `open_model`
/// follows).
fn app_state_document() -> String {
    match genmsi_core::current_live() {
        Some(document) => serde_json::to_string_pretty(&document)
            .expect("the live document of the app always serializes"),
        None => genmsi_core::live_document(&genmsi_core::LiveBody::empty()),
    }
}

/// The document of an edit.
fn edit_document(input: &EditInput) -> Result<String, Failure> {
    let source = input.source()?;
    // The trial comes first: a dry run writes nothing, whatever the target
    // says. The engine already compiles the model before it writes it, so
    // the trial is the same road without the writing.
    let destination = if input.dry_run {
        Destination::Trial
    } else {
        match input.target {
            Target::File => Destination::File,
            Target::Proposal => Destination::Staging,
        }
    };
    // The proposal is about the model the agent worked on: the file it
    // named, or the model the app has open when it named none. It is
    // resolved before the batch runs, so a proposal that has nowhere to go
    // fails without touching anything.
    let target = match destination {
        Destination::File | Destination::Trial => None,
        Destination::Staging => Some(destination_path(&input.path)?),
    };
    let edited = engine::apply_edits(&source, &input.commands, input.layout, destination)?;
    let lint = genmsi_core::lint_document(&edited.program, &edited.model, &edited.name);
    let proposal = match target {
        // The model went to the disk: there is no proposal.
        None => None,
        Some(target) => {
            let text = edited.text.as_deref().ok_or_else(|| {
                Box::new(ToolError::arguments("the batch produced no model to stage")) as Failure
            })?;
            let staged = stage(text, &target, input.summary.as_deref())?;
            Some(genmsi_core::documents::EditProposal {
                id: staged.id,
                model: staged.model.display().to_string(),
                document: staged.document.display().to_string(),
            })
        }
    };
    let document = genmsi_core::edit_document(
        &lint,
        input.commands.len(),
        edited.written,
        edited.created,
        edited.source,
        proposal,
    );
    Ok(genmsi_core::to_pretty_json(
        genmsi_core::EDIT_SCHEMA,
        &document,
    ))
}

/// The lint document of a model that was written.
fn save_document(input: &SaveInput) -> Result<String, Failure> {
    match input.target {
        Target::File => {
            // The destination comes first: without a path the model the app
            // has open is the one that is written, and there is none when
            // the app has no model open.
            let path = destination_path(&input.path)?;
            let text = model_text(input)?;
            let saved = engine::save_model(&path, text)?;
            // The composition is not a finding of the compiler: it is what the engine
            // did to the model before writing it, and it does not belong in the list
            // of findings (a model with no finding is `ok`, and a model the engine
            // composed has none). The tool's own description says the composition
            // happens, and `genmsi layout` says what it composed.
            let document = genmsi_core::lint_document(
                &saved.program,
                &saved.model,
                &saved.path.display().to_string(),
            );
            Ok(genmsi_core::to_pretty_json(
                genmsi_core::LINT_SCHEMA,
                &document,
            ))
        }
        Target::Proposal => {
            // The proposal is about the model the agent worked on: the file
            // it named, or the model the app has open when it named none.
            let target = destination_path(&input.path)?;
            let text = model_text(input)?;
            // The model is prepared exactly like a model that goes to the
            // disk: it is checked and its canvas is composed, so the person
            // sees the same model in both cases.
            let prepared = engine::prepare_save(text, "<proposal>")?;
            let staged = stage(
                &genmsi_format::to_string(&prepared.model),
                &target,
                input.summary.as_deref(),
            )?;
            let document = genmsi_core::lint_document(
                &prepared.program,
                &prepared.model,
                &staged.model.display().to_string(),
            );
            Ok(genmsi_core::to_pretty_json(
                genmsi_core::LINT_SCHEMA,
                &document,
            ))
        }
    }
}

/// What the lint document says when the engine composed the model.
///
/// The number is the blocks the composition placed: the model root is not
/// drawn, so it is not one of them.
/// The report document of a run.
fn report_document(input: &RunInput) -> Result<String, Failure> {
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
///
/// With `summary` the answer is the summary of the run —the `genmsi.trace_summary/1`
/// document— and not the events. The summary is the summary of the **whole**
/// run: the filters cut the events, and not the summary.
fn trace_document(input: &TraceInput) -> Result<String, Failure> {
    let loaded = engine::load(&input.source()?)?;
    let start = checked_start(input.start)?;
    let config = engine::trace_config(
        input.seed,
        input.until,
        start,
        input.limit,
        input.filter(),
        input.summary,
    );
    let result = engine::run(&loaded, config)?;
    if input.summary {
        let summary = genmsi_core::trace_summary(&result);
        return Ok(genmsi_core::to_pretty_json(
            genmsi_core::TRACE_SUMMARY_SCHEMA,
            &summary,
        ));
    }
    let document = genmsi_core::trace_document(&result, input.limit);
    Ok(genmsi_core::to_pretty_json(
        genmsi_core::TRACE_SCHEMA,
        &document,
    ))
}

/// The outcome of a tool: the document it produced, or the failure it found.
fn finish(outcome: Result<String, Failure>) -> CallToolResult {
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
             Use `apply_edit` to build or to change a model: it takes a batch of \
             the engine's editing commands (`addBlock`, `connect`, `setProperty`, \
             `moveBlock`, `removeBlock`, ...) and writes the model; the batch is \
             atomic, so a command that fails leaves the model as it was. \
             Use `lint` to validate a model, `simulate` to run it, `get_trace` to read \
             its events and `explain_model` to describe it: its `program` is the \
             compiled GPSS and its `model` is the document itself, with every block, \
             its properties by name and the links that wire the blocks. \
             `get_trace` can be FILTERED (`classes`, `nodes`, `xact`, `from`, `to`) \
             and can answer with the summary of the run (`summary`), which says how \
             many times the flow went through each block; `explain_model` publishes \
             the `map` that joins the statements of the compiled program with the \
             blocks of the document, and every event of the trace carries the `node` \
             of the block of the document it came from. \
             Give the model \
             either as `path` (a .gmdl file) or as `model` (the .gmdl document as text); \
             when you give neither, the model the desktop app has open is the one the tool \
             works on: the same model the reader is looking at. \
             `apply_edit` and `save_model` take a `target`: `file` (the default) writes \
             the model to the file, and `proposal` leaves it in the staging area of the \
             app, where the reader sees it and decides; the verdict comes back in \
             `app_state`. \
             Two blocks are connected by a link inside the `<links>` of their container: \
             `<link frommoduleid=\"1\" fromioid=\"1\" tomoduleid=\"2\" toioid=\"1\"/>`, \
             where the ioids are the ports the catalogue publishes for the class. \
             Call `app_state` to see what the reader is looking at right now (the \
             model the app has open, whether it has changes that are not saved, \
             what it found and the last run it made): ask it before you write on \
             the model the app has open, because `apply_edit` and `save_model` \
             refuse with `model_locked` while the reader has unsaved changes on \
             that model. \
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
            classes: Vec::new(),
            nodes: Vec::new(),
            xact: None,
            from: None,
            to: None,
            summary: false,
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
    fn the_trace_is_filtered_by_class() {
        let mut input = trace_input();
        input.classes = vec!["Terminate".to_string()];
        let result = GenmsiServer::new().get_trace(Parameters(input));
        assert_eq!(result.is_error, Some(false));
        let text = text_of(&result);
        assert!(text.contains("\"schema\": \"genmsi.trace/1\""), "{text}");
        assert!(text.contains("\"class\": \"Terminate\""), "{text}");
        assert!(
            !text.contains("\"class\": \"Generate\""),
            "the filter keeps only the events of the class: {text}"
        );
        // The document says what was filtered.
        assert!(text.contains("\"classes\": ["), "{text}");
        assert!(text.contains("\"Terminate\""), "{text}");
    }

    #[test]
    fn the_summary_answers_the_summary_document() {
        let mut input = trace_input();
        input.summary = true;
        let result = GenmsiServer::new().get_trace(Parameters(input));
        assert_eq!(result.is_error, Some(false));
        let text = text_of(&result);
        assert!(
            text.contains("\"schema\": \"genmsi.trace_summary/1\""),
            "{text}"
        );
        assert!(text.contains("\"blocks\""), "{text}");
        assert!(text.contains("\"entries\""), "{text}");
        assert!(text.contains("\"first\""), "{text}");
        assert!(text.contains("\"last\""), "{text}");
    }

    #[test]
    fn the_explain_document_publishes_the_map_of_the_statements() {
        let result = GenmsiServer::new().explain_model(Parameters(model_input()));
        assert_eq!(result.is_error, Some(false));
        let text = text_of(&result);
        assert!(text.contains("\"map\""), "{text}");
        assert!(text.contains("\"statement\": 0"), "{text}");
        assert!(text.contains("\"node\": 1"), "{text}");
        assert!(text.contains("\"caption\": \"g\""), "{text}");
        assert!(text.contains("\"line\""), "{text}");
        assert!(text.contains("\"column\""), "{text}");
    }

    #[test]
    fn a_model_without_a_path_is_the_model_the_app_has_open() {
        // The machine the test runs on may have the app open with a model:
        // then the tool lints that model. Otherwise there is no destination
        // to guess, and the failure is its own class.
        let result = GenmsiServer::new().lint(Parameters(ModelInput::default()));
        let text = text_of(&result);
        if result.is_error == Some(true) {
            assert!(text.contains("\"error\": \"no_open_model\""), "{text}");
        } else {
            assert!(text.contains("\"schema\": \"genmsi.lint/1\""), "{text}");
        }
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
    fn no_source_is_the_model_the_app_has_open() {
        // The machine the test runs on may have the app open with a model:
        // then that model is the source. Otherwise there is no destination
        // to guess, and the failure is its own class and says what to do.
        match ModelInput::default().source() {
            Ok(ModelSource::File(path)) => {
                let open = genmsi_core::live::open_model_from_live()
                    .and_then(|document| document.path)
                    .expect("the app has a model open");
                assert_eq!(path, PathBuf::from(open));
            }
            Ok(ModelSource::Inline(_)) => {
                panic!("a model with no source is the model the app has open")
            }
            Err(error) => {
                assert_eq!(error.error, "no_open_model");
                let message = error.message;
                assert!(
                    message.contains("give `path`") && message.contains("open a model in the app"),
                    "the message says what to do: {message}"
                );
            }
        }
    }

    #[test]
    fn the_target_is_spell_in_lower_case() {
        let input: SaveInput = serde_json::from_str(
            r#"{"model":"<gmodel/>","target":"proposal","summary":"the bus loop"}"#,
        )
        .expect("the input parses");
        assert_eq!(input.target, Target::Proposal);
        assert!(input.path.is_none(), "the path may be left out");
        assert_eq!(input.summary.as_deref(), Some("the bus loop"));

        let default: SaveInput =
            serde_json::from_str(r#"{"model":"<gmodel/>","path":"model.gmdl"}"#)
                .expect("the input parses");
        assert_eq!(default.target, Target::File, "the file is the default");
    }
}
