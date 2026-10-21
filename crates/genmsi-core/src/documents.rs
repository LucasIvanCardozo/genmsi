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

/// Schema id of the `lint` document.
pub const LINT_SCHEMA: &str = "genmsi.lint/1";
/// Schema id of the `simulate` document.
pub const REPORT_SCHEMA: &str = "genmsi.report/1";
/// Schema id of the `trace` document.
pub const TRACE_SCHEMA: &str = "genmsi.trace/1";
/// Schema id of the `explain` document.
pub const EXPLAIN_SCHEMA: &str = "genmsi.explain/1";

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
#[derive(Debug, Serialize)]
pub struct DefinitionCounts {
    /// `Function` / `FunctionExpon` / `FunctionDist` definitions.
    pub functions: usize,
    /// `Variable` definitions.
    pub variables: usize,
    /// `VariableA` definitions.
    pub random_variables: usize,
    /// `StorageDef` capacities.
    pub storages: usize,
    /// `Table` definitions.
    pub tables: usize,
    /// `QTable` definitions.
    pub qtables: usize,
}

/// Body of the `genmsi.lint/1` document.
#[derive(Debug, Serialize)]
pub struct LintDocument<'a> {
    /// Model path, as the user wrote it.
    pub path: String,
    /// `ok` when the compiler was silent, `diagnostics` otherwise.
    pub status: &'static str,
    /// Program title (the model root's name).
    pub title: &'a str,
    /// Executable statements the model compiles to.
    pub blocks: usize,
    /// Definitions the model declares.
    pub definitions: DefinitionCounts,
    /// Non-fatal observations made while compiling.
    pub diagnostics: &'a [String],
}

/// Build the `genmsi.lint/1` body for a compiled model.
///
/// `path` is the model as the caller named it; a model that never touched the
/// disk is named `<memory>`.
pub fn lint_document<'a>(program: &'a Program, path: &str) -> LintDocument<'a> {
    let definitions = &program.definitions;
    LintDocument {
        path: path.to_string(),
        status: if program.diagnostics.is_empty() {
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
        diagnostics: &program.diagnostics,
    }
}

/// Body of the `genmsi.trace/1` document.
#[derive(Debug, Serialize)]
pub struct TraceDocument<'a> {
    /// Program title.
    pub title: &'a str,
    /// Seed used.
    pub seed: u64,
    /// Start time.
    pub start_time: f64,
    /// End time.
    pub end_time: f64,
    /// Why the run ended (`terminated`, `no_events` or `time_limit`).
    pub end_reason: &'a str,
    /// Record limit this run used.
    pub limit: usize,
    /// Records actually kept.
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
        let text = to_pretty_json(LINT_SCHEMA, &lint_document(&program, "model.gmdl"));
        assert!(text.starts_with("{\n  \"schema\": \"genmsi.lint/1\",\n"), "{text}");
        assert!(text.contains("\"path\": \"model.gmdl\""), "{text}");
        assert!(text.contains("\"status\": \"ok\""), "{text}");
        assert!(text.contains("\"blocks\": 2"), "{text}");
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
