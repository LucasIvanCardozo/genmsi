//! The documents every client of the engine publishes.
//!
//! The CLI and the MCP server must show the same numbers for the same model,
//! so the documents live here, next to the engine that produces them, instead
//! of in each client. Every document is a versioned envelope
//! (`{"schema": "genmsi.<document>/<major>", ...}`); within one major version
//! the body only grows (decision D13).

use serde::Serialize;

use crate::program::Program;
use crate::sim::{RunResult, TraceEvent};
use genmsi_format::{GmdlModel, percent_decode};

/// Schema id of the `lint` document.
pub const LINT_SCHEMA: &str = "genmsi.lint/1";
/// Schema id of the `simulate` document.
pub const REPORT_SCHEMA: &str = "genmsi.report/1";
/// Schema id of the `trace` document.
pub const TRACE_SCHEMA: &str = "genmsi.trace/1";
/// Schema id of the `explain` document.
pub const EXPLAIN_SCHEMA: &str = "genmsi.explain/1";
/// Schema id of the model document.
pub const MODEL_SCHEMA: &str = "genmsi.model/1";
/// Schema id of the catalogue document.
pub const CATALOGUE_SCHEMA: &str = "genmsi.catalogue/1";
/// Schema id of the `gpss` document.
pub const GPSS_SCHEMA: &str = "genmsi.gpss/1";

/// A versioned document: the schema id followed by the body fields.
#[derive(Debug, Serialize)]
pub struct Envelope<'a, T: Serialize> {
    /// Schema id of this document, `<name>/<major>`.
    pub schema: &'a str,
    /// Document fields, serialized at the top level.
    #[serde(flatten)]
    pub body: T,
}

/// Render one document as pretty JSON.
///
/// The rendering is the one the CLI prints, so a client and the CLI produce the
/// same bytes for the same model.
pub fn to_pretty_json<T: Serialize + ?Sized>(schema: &str, body: &T) -> String {
    let document = Envelope { schema, body };
    serde_json::to_string_pretty(&document).expect("the engine's documents always serialize")
}

/// Counts of the definitions a model declares.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct DefinitionCounts {
    /// `Function` / `FunctionExpon` / `FunctionDist` definitions.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub functions: usize,
    /// `Variable` definitions.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub variables: usize,
    /// `VariableA` definitions.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub random_variables: usize,
    /// `StorageDef` capacities.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub storages: usize,
    /// `Table` definitions.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub tables: usize,
    /// `QTable` definitions.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub qtables: usize,
}

/// Body of the `genmsi.lint/1` document.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct LintDocument<'a> {
    /// Model path, as the user wrote it.
    pub path: String,
    /// `ok` when the compiler was silent, `diagnostics` otherwise.
    pub status: &'static str,
    /// Program title (the model root's name).
    pub title: &'a str,
    /// Executable statements the model compiles to.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub blocks: usize,
    /// Definitions the model declares.
    pub definitions: DefinitionCounts,
    /// Non-fatal findings: the compiler's observations and the format's
    /// warnings.
    pub diagnostics: Vec<String>,
}

/// Build the `genmsi.lint/1` body for a compiled model.
///
/// `path` is the model as the caller named it; a model that never touched the
/// disk is named `<memory>`.
pub fn lint_document<'a>(program: &'a Program, model: &GmdlModel, path: &str) -> LintDocument<'a> {
    let definitions = &program.definitions;
    let diagnostics = lint_diagnostics(program, model);
    LintDocument {
        path: path.to_string(),
        status: if diagnostics.is_empty() {
            "ok"
        } else {
            "diagnostics"
        },
        title: &program.title,
        blocks: program.blocks.len(),
        definitions: DefinitionCounts {
            functions: definitions.functions.len(),
            variables: definitions.variables.len(),
            random_variables: definitions.random_variables.len(),
            storages: definitions.storages.len(),
            tables: definitions.tables.len(),
            qtables: definitions.qtables.len(),
        },
        diagnostics,
    }
}

/// The findings of a model: what the compiler said and what the format says.
///
/// The format reads the model too: a block that uses an entity the model does
/// not declare (a storage nobody declares, for instance) is a warning, and a
/// model with one is not silent about it any more.
///
/// The compiler reports some of those findings on its own, with its own
/// wording: a storage nobody declares has no capacity, so the run stops when
/// a transaction reaches it, and the compiler says so. A warning the compiler
/// already said is not said twice: its wording wins, because it carries what
/// the run does about the finding.
pub fn lint_diagnostics(program: &Program, model: &GmdlModel) -> Vec<String> {
    let mut diagnostics = program.diagnostics.clone();
    for warning in genmsi_format::declaration_warnings(model) {
        if diagnostics.iter().any(|said| said.contains(&warning)) {
            continue;
        }
        diagnostics.push(warning);
    }
    diagnostics
}

/// Canvas geometry of a block, in the model's own canvas units.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ModelRect {
    /// Horizontal position.
    pub x: i32,
    /// Vertical position.
    pub y: i32,
    /// Block width.
    pub width: i32,
    /// Block height.
    pub height: i32,
}

/// One property of a block, with its decoded value.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ModelProperty {
    /// Property name as the block schema declares it (`Media`, `Queue`, ...).
    pub name: String,
    /// Decoded value, empty when the model does not set it.
    pub value: String,
    /// Declared type (`TSNA`, `EntityUse`, ...).
    pub ty: String,
    /// Declared subtype (`SNA`, `Facility`, `Queue`, ...).
    pub subtype: String,
    /// Tooltip the editor shows, decoded.
    pub hint: String,
    /// Whether the property defines an entity.
    pub unique: bool,
    /// Whether the editor shows the property.
    pub visible: bool,
}

/// One port of a block: a connection point.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ModelPort {
    /// Port id, unique inside the block.
    pub ioid: u32,
    /// Port name (`Entrada1`, `Salida1`, ...).
    pub name: String,
    /// Port caption the editor shows, decoded.
    pub caption: String,
    /// Connection label shared by the two ends of a link.
    pub label: Option<String>,
}

/// One block of the model: an activity block or a container.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ModelBlock {
    /// `moduleid`, unique in the model.
    pub id: u32,
    /// `moduleclass` (`Generate`, `Queue`, ...); empty for plain containers.
    pub class: String,
    /// `kind`: `A` for an activity block, `C` for a container.
    pub kind: String,
    /// Block name, decoded.
    pub name: String,
    /// Block caption, decoded.
    pub caption: String,
    /// GPSS label, when the block declares one.
    pub label: Option<String>,
    /// Id of the container that owns the block.
    pub container: u32,
    /// Canvas geometry, absent when the model does not place the block.
    pub layout: Option<ModelRect>,
    /// Properties, in the order the model declares them.
    pub properties: Vec<ModelProperty>,
    /// Input ports, in file order.
    pub inputs: Vec<ModelPort>,
    /// Output ports, in file order.
    pub outputs: Vec<ModelPort>,
}

/// One link between two blocks.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ModelEdge {
    /// Stable id of the link (`<container>:<from>:<port>-><to>:<port>`).
    pub id: String,
    /// Id of the container that owns the link.
    pub container: u32,
    /// Source module id.
    pub from_module: u32,
    /// Source port id.
    pub from_ioid: u32,
    /// Source port name (`Salida1`, ...).
    pub from_port: String,
    /// Target module id.
    pub to_module: u32,
    /// Target port id.
    pub to_ioid: u32,
    /// Target port name (`Entrada1`, ...).
    pub to_port: String,
    /// Connection label shared by the two ends of the link.
    pub label: Option<String>,
}

/// The ports of a block, in file order, with their decoded captions.
fn ports_of(ports: &[genmsi_format::Port]) -> Vec<ModelPort> {
    ports
        .iter()
        .map(|port| ModelPort {
            ioid: port.ioid,
            name: port.name.clone(),
            caption: percent_decode(&port.caption),
            label: port.label().map(percent_decode),
        })
        .collect()
}

/// Body of the `genmsi.model/1` document: the model's blocks and its links.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct ModelDocument {
    /// Model path, as the caller named it.
    pub path: String,
    /// Model title: the root block's name, decoded.
    pub title: String,
    /// Blocks, in file order.
    pub blocks: Vec<ModelBlock>,
    /// Links, in file order.
    pub edges: Vec<ModelEdge>,
}

/// Build the `genmsi.model/1` body for a model.
///
/// The document is built from the file's semantic model, not from the compiled
/// program, so a model that does not compile still has a graph: a client can
/// show the blocks and the error at the same time. Names, captions and property
/// values are decoded, because this document exists to be read.
pub fn model_document(model: &GmdlModel, path: &str) -> ModelDocument {
    let blocks = model
        .blocks
        .iter()
        .map(|block| ModelBlock {
            id: block.id,
            class: block.class.clone(),
            kind: block.kind.to_string(),
            name: percent_decode(&block.name),
            caption: percent_decode(&block.caption),
            label: block.label().map(percent_decode),
            container: block.owner,
            layout: block.layout.map(|layout| ModelRect {
                x: layout.x,
                y: layout.y,
                width: layout.width,
                height: layout.height,
            }),
            properties: block
                .props
                .iter()
                .map(|property| ModelProperty {
                    name: property.name.clone(),
                    value: property.value(),
                    ty: property.ty.clone(),
                    subtype: property.subtype.clone(),
                    hint: percent_decode(&property.hint),
                    unique: property.unique,
                    visible: property.visible,
                })
                .collect(),
            inputs: ports_of(&block.inputs),
            outputs: ports_of(&block.outputs),
        })
        .collect();

    let mut edges = Vec::new();
    for container in model.blocks.iter().filter(|block| block.is_container()) {
        for link in &container.links {
            let source = model.block(link.from_module);
            let target = model.block(link.to_module);
            let from_port = source.and_then(|block| block.output(link.from_ioid));
            let to_port = target.and_then(|block| block.input(link.to_ioid));
            let label = from_port
                .and_then(|port| port.label())
                .or_else(|| to_port.and_then(|port| port.label()))
                .map(percent_decode);
            edges.push(ModelEdge {
                id: format!(
                    "{}:{}:{}->{}:{}",
                    container.id, link.from_module, link.from_ioid, link.to_module, link.to_ioid
                ),
                container: container.id,
                from_module: link.from_module,
                from_ioid: link.from_ioid,
                from_port: from_port
                    .map(|port| port.name.clone())
                    .unwrap_or_default(),
                to_module: link.to_module,
                to_ioid: link.to_ioid,
                to_port: to_port.map(|port| port.name.clone()).unwrap_or_default(),
                label,
            });
        }
    }

    ModelDocument {
        path: path.to_string(),
        title: model
            .root()
            .map(|root| percent_decode(&root.name))
            .unwrap_or_default(),
        blocks,
        edges,
    }
}

/// One port of a block class, as the catalogue declares it.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct CataloguePort {
    /// Port id, unique inside the block.
    pub ioid: u32,
    /// Port name (`Entrada1`, `Salida1`, ...).
    pub name: String,
    /// Port caption the editor shows.
    pub caption: String,
}

/// One property of a block class, as the catalogue declares it.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct CatalogueProperty {
    /// Property name (`Media`, `Queue`, ...).
    pub name: String,
    /// Tooltip the editor shows.
    pub hint: String,
    /// Declared type (`TSNA`, `EntityUse`, `Listed`, ...).
    pub ty: String,
    /// Declared subtype (`SNA`, `Queue`, ...).
    pub subtype: String,
    /// Whether the property defines an entity.
    pub unique: bool,
    /// The options a `Listed` property offers; empty for every other type.
    pub options: Vec<String>,
}

/// One block class the editor offers to create.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct CatalogueClass {
    /// `moduleclass` the blocks of this class carry.
    pub class: String,
    /// What the class does, in the words of the interface.
    pub title: String,
    /// Input ports, in file order.
    pub inputs: Vec<CataloguePort>,
    /// Output ports, in file order.
    pub outputs: Vec<CataloguePort>,
    /// Properties, in schema order.
    pub properties: Vec<CatalogueProperty>,
}

/// Body of the `genmsi.catalogue/1` document.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct CatalogueDocument {
    /// The classes the editor offers, in alphabetical order.
    pub classes: Vec<CatalogueClass>,
}

/// Build the `genmsi.catalogue/1` body: the block classes an editor needs to
/// create a block.
///
/// The catalogue lives in `genmsi-format` (it says what a block looks like in
/// the file); this document only carries it to the clients. The classes the
/// compiler understands live in [`crate::program::known_classes`], and a test
/// pins the catalogue inside that list.
pub fn catalogue_document() -> CatalogueDocument {
    let classes = genmsi_format::catalogue::CLASSES
        .iter()
        .map(|class| CatalogueClass {
            class: class.class.to_string(),
            title: class.title.to_string(),
            inputs: class
                .inputs
                .iter()
                .map(|port| CataloguePort {
                    ioid: port.ioid,
                    name: port.name.to_string(),
                    caption: port.caption.to_string(),
                })
                .collect(),
            outputs: class
                .outputs
                .iter()
                .map(|port| CataloguePort {
                    ioid: port.ioid,
                    name: port.name.to_string(),
                    caption: port.caption.to_string(),
                })
                .collect(),
            properties: class
                .properties
                .iter()
                .map(|property| CatalogueProperty {
                    name: property.name.to_string(),
                    hint: property.hint.to_string(),
                    ty: property.ty.to_string(),
                    subtype: property.subtype.to_string(),
                    unique: property.unique,
                    options: genmsi_format::catalogue::listed_options(property.subtype)
                        .iter()
                        .map(|option| option.to_string())
                        .collect(),
                })
                .collect(),
        })
        .collect();
    CatalogueDocument { classes }
}

/// Body of the `genmsi.trace/1` document.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct TraceDocument<'a> {
    /// Program title.
    pub title: &'a str,
    /// Seed used.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub seed: u64,
    /// Start time.
    pub start_time: f64,
    /// End time.
    pub end_time: f64,
    /// Why the run ended (`terminated`, `no_events` or `time_limit`).
    pub end_reason: &'a str,
    /// Record limit this run used.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub limit: usize,
    /// Records actually kept.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub records: usize,
    /// Whether the limit may have cut the trace.
    pub truncated: bool,
    /// Trace records, in simulation order.
    pub events: &'a [TraceEvent],
}

/// Build the `genmsi.trace/1` body for a finished run.
///
/// `limit` is the record bound the run used; a run that kept exactly `limit`
/// records may have been cut short, which the document says.
pub fn trace_document<'a>(result: &'a RunResult, limit: usize) -> TraceDocument<'a> {
    let records = result.trace.len();
    TraceDocument {
        title: &result.title,
        seed: result.seed,
        start_time: result.start_time,
        end_time: result.end_time,
        end_reason: result.end_reason.as_str(),
        limit,
        records,
        truncated: records >= limit,
        events: &result.trace,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::program::compile;
    use crate::sim::{SimConfig, Simulation};
    use genmsi_format::parse_str;
    use std::path::Path;

    const MINI: &str = "<gmodel versioncode=\"1\"><gclasses/><modules>\
        <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"m\" name=\"m\" kind=\"C\"><links>\
        <link frommoduleid=\"1\" fromioid=\"1\" tomoduleid=\"2\" toioid=\"1\"/>\
        </links><inputs/><outputs/></module>\
        <module moduleid=\"1\" moduleclass=\"Generate\" owner=\"0\" caption=\"g\" name=\"g\" kind=\"A\"><outputs><output ioid=\"1\" name=\"Salida1\" caption=\"\" mylabel=\"L1\"/></outputs>\
        <props><property name=\"Media\" hint=\"\" value=\"10\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/></props></module>\
        <module moduleid=\"2\" moduleclass=\"Terminate\" owner=\"0\" caption=\"t\" name=\"t\" kind=\"A\"><inputs><input ioid=\"1\" name=\"i\" caption=\"\" mylabel=\"L1\"/></inputs>\
        <props><property name=\"Unidades\" hint=\"\" value=\"1\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/></props></module>\
        </modules></gmodel>";

    fn program() -> Program {
        compile(&parse_str(MINI).unwrap(), "mini").expect("compiles")
    }

    /// The reference model of the app's slice.
    fn reference_model() -> GmdlModel {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/models/TP6__Clase 6_1_PuertaBanco.gmdl");
        genmsi_format::load(path).expect("the reference model loads")
    }

    #[test]
    fn the_model_document_carries_the_blocks_and_the_links() {
        let model = reference_model();
        let document = model_document(&model, "PuertaBanco.gmdl");
        assert_eq!(document.path, "PuertaBanco.gmdl");
        assert_eq!(document.title, "Puerta Banco");
        // The root container, the statement container and the activity blocks.
        assert!(document.blocks.len() > 10, "{}", document.blocks.len());
        let generate = document
            .blocks
            .iter()
            .find(|block| block.class == "Generate")
            .expect("a Generate block");
        assert_eq!(generate.kind, "A");
        assert_eq!(generate.caption, "Clientes");
        let media = generate
            .properties
            .iter()
            .find(|property| property.name == "Media")
            .expect("the Media property");
        assert_eq!(media.value, "90");
        // The geometry comes from the file, not from a layout engine.
        let placed = document
            .blocks
            .iter()
            .filter(|block| block.layout.is_some())
            .count();
        assert!(placed > 5, "{placed}");
        // Every link of every container is an edge.
        assert!(!document.edges.is_empty());
        let edge = document
            .edges
            .iter()
            .find(|edge| edge.from_module == 6 && edge.to_module == 16)
            .expect("the link from the Generate block to the Queue block");
        assert_eq!(edge.from_port, "Salida1");
        assert_eq!(edge.to_port, "Entrada1");
        assert_eq!(edge.label.as_deref(), Some("A_Queue"));
    }

    #[test]
    fn a_model_that_does_not_compile_still_has_a_graph() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/models/TP9__tp9.gmdl");
        let model = genmsi_format::load(path).expect("the model is structurally valid");
        assert!(compile(&model, "x").is_err(), "the engine refuses this model");
        let document = model_document(&model, "tp9.gmdl");
        assert!(!document.blocks.is_empty());
        assert!(!document.edges.is_empty());
    }

    #[test]
    fn the_model_document_names_its_schema() {
        assert_eq!(MODEL_SCHEMA, "genmsi.model/1");
        let text = to_pretty_json(MODEL_SCHEMA, &model_document(&reference_model(), "m.gmdl"));
        assert!(text.starts_with("{\n  \"schema\": \"genmsi.model/1\",\n"), "{text}");
        assert!(text.contains("\"path\": \"m.gmdl\""), "{text}");
    }

    /// Run the model with the trace enabled, as the `trace` client does.
    fn traced(program: &Program, limit: usize) -> RunResult {
        let config = SimConfig {
            trace: true,
            trace_limit: limit,
            ..SimConfig::default()
        };
        Simulation::new(program, config).run().expect("runs")
    }

    #[test]
    fn the_lint_document_names_the_schema_and_the_model() {
        let program = program();
        let model = parse_str(MINI).expect("the model parses");
        let text = to_pretty_json(LINT_SCHEMA, &lint_document(&program, &model, "model.gmdl"));
        assert!(text.starts_with("{\n  \"schema\": \"genmsi.lint/1\",\n"), "{text}");
        assert!(text.contains("\"path\": \"model.gmdl\""), "{text}");
        assert!(text.contains("\"status\": \"ok\""), "{text}");
        assert!(text.contains("\"blocks\": 2"), "{text}");
    }

    /// A model that uses a storage no block declares.
    ///
    /// The `StorageUse` compiles to an `ENTER` of a storage the model does not
    /// declare, so the compiler reports it too: the finding must be said once.
    fn model_with_an_undeclared_storage() -> GmdlModel {
        let xml = "<gmodel versioncode=\"1\"><gclasses/><modules>\
            <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"m\" name=\"m\" kind=\"C\">\
            <links/><inputs/><outputs/></module>\
            <module moduleid=\"1\" moduleclass=\"StorageUse\" owner=\"0\" caption=\"c\" name=\"n\" kind=\"A\">\
            <props>\
            <property name=\"Media\" hint=\"\" value=\"10\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/>\
            <property name=\"Storage\" hint=\"\" value=\"Almacen\" type=\"EntityUse\" subType=\"Storage\" visible=\"1\" unique=\"0\"/>\
            </props></module>\
            <module moduleid=\"2\" moduleclass=\"Terminate\" owner=\"0\" caption=\"c\" name=\"n\" kind=\"A\">\
            <props>\
            <property name=\"Unidades\" hint=\"\" value=\"1\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/>\
            </props></module>\
            </modules></gmodel>";
        parse_str(xml).expect("the model parses")
    }

    #[test]
    fn the_lint_document_says_the_warning_of_the_format() {
        let model = model_with_an_undeclared_storage();
        let program = compile(&model, "m").expect("the model compiles");
        let document = lint_document(&program, &model, "model.gmdl");
        assert_eq!(document.status, "diagnostics");
        let warning = document
            .diagnostics
            .iter()
            .find(|message| message.contains("Almacen"))
            .expect("the warning names the storage");
        assert!(warning.contains("StorageDef"), "{warning}");
    }

    #[test]
    fn the_lint_document_says_the_warning_once() {
        let model = model_with_an_undeclared_storage();
        let program = compile(&model, "m").expect("the model compiles");
        let document = lint_document(&program, &model, "model.gmdl");
        // The compiler reports the storage on its own, with the format's
        // wording and what the run does about it: the format's warning is
        // not said a second time.
        assert_eq!(
            document.diagnostics.len(),
            1,
            "the finding is said twice: {:#?}",
            document.diagnostics
        );
        assert!(
            document.diagnostics[0].contains("no capacity"),
            "the compiler's wording is the one that stays: {:#?}",
            document.diagnostics
        );
    }

    #[test]
    fn a_clean_model_has_no_findings() {
        let model = parse_str(MINI).expect("the model parses");
        let program = compile(&model, "m").expect("the model compiles");
        let diagnostics = lint_diagnostics(&program, &model);
        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }

    #[test]
    fn the_trace_document_reports_the_records_it_kept() {
        let program = program();
        let result = traced(&program, 100);
        let text = to_pretty_json(TRACE_SCHEMA, &trace_document(&result, 100));
        assert!(text.contains("\"schema\": \"genmsi.trace/1\""), "{text}");
        assert!(text.contains("\"truncated\": false"), "{text}");
        assert!(text.contains("\"event\": \"generate\""), "{text}");
    }

    #[test]
    fn a_trace_that_hit_the_limit_says_so() {
        let program = program();
        let result = traced(&program, 1);
        let document = trace_document(&result, 1);
        assert_eq!(document.records, 1);
        assert!(document.truncated);
    }

    #[test]
    fn the_schema_ids_are_versioned() {
        assert_eq!(LINT_SCHEMA, "genmsi.lint/1");
        assert_eq!(REPORT_SCHEMA, "genmsi.report/1");
        assert_eq!(TRACE_SCHEMA, "genmsi.trace/1");
        assert_eq!(EXPLAIN_SCHEMA, "genmsi.explain/1");
    }
}
