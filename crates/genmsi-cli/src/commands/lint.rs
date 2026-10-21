//! `genmsi lint <model>`: can this model load, validate, compile and run?
//!
//! Lint stops before the simulation. It reports what the model compiles to and
//! any non-fatal compiler observation; a model that cannot be compiled is a
//! failure (exit 1), and a model that compiles with diagnostics exits 3 so a
//! script can tell "broken" from "usable, with remarks".

use serde::Serialize;

use crate::cli::LintArgs;
use crate::error::{CliError, ExitCode};
use crate::model;
use crate::output::{self, LINT_SCHEMA};

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

/// Outcome of one `lint` invocation.
#[derive(Debug, Serialize)]
pub struct LintReport<'a> {
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

impl LintReport<'_> {
    /// Exit code this report implies.
    pub fn exit_code(&self) -> ExitCode {
        if self.diagnostics.is_empty() {
            ExitCode::Success
        } else {
            ExitCode::Diagnostics
        }
    }

    /// Human-readable form.
    pub fn render(&self) -> String {
        let definitions = &self.definitions;
        let mut out = String::new();
        out.push_str(&format!("model: {}\n", self.path));
        out.push_str(&format!("title: {}\n", self.title));
        out.push_str(&format!("status: {}\n", self.status));
        out.push_str(&format!("blocks: {}\n", self.blocks));
        out.push_str(&format!(
            "definitions: functions {}, variables {}, random_variables {}, storages {}, \
             tables {}, qtables {}\n",
            definitions.functions,
            definitions.variables,
            definitions.random_variables,
            definitions.storages,
            definitions.tables,
            definitions.qtables
        ));
        if self.diagnostics.is_empty() {
            out.push_str("diagnostics: none\n");
        } else {
            out.push_str("diagnostics:\n");
            for message in self.diagnostics {
                out.push_str(&format!("  - {message}\n"));
            }
        }
        out
    }
}

/// Run `genmsi lint`.
pub fn run(args: LintArgs) -> Result<ExitCode, CliError> {
    let loaded = model::load(&args.model.path)?;
    let program = &loaded.program;
    let definitions = &program.definitions;
    let report = LintReport {
        path: args.model.path.display().to_string(),
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
    };
    if args.output.json {
        output::print_json(LINT_SCHEMA, &report)?;
    } else {
        output::print(&report.render())?;
    }
    Ok(report.exit_code())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(diagnostics: &[String]) -> LintReport<'_> {
        LintReport {
            path: "model.gmdl".to_string(),
            status: if diagnostics.is_empty() {
                "ok"
            } else {
                "diagnostics"
            },
            title: "model",
            blocks: 3,
            definitions: DefinitionCounts {
                functions: 0,
                variables: 0,
                random_variables: 0,
                storages: 0,
                tables: 0,
                qtables: 0,
            },
            diagnostics,
        }
    }

    #[test]
    fn a_clean_model_succeeds() {
        let report = report(&[]);
        assert_eq!(report.exit_code(), ExitCode::Success);
        let text = report.render();
        assert!(text.contains("status: ok"), "{text}");
        assert!(text.contains("diagnostics: none"), "{text}");
    }

    #[test]
    fn diagnostics_on_a_runnable_model_exit_with_three() {
        let diagnostics = vec!["TABULATE tiempos has no TABLE definition".to_string()];
        let report = report(&diagnostics);
        assert_eq!(report.exit_code(), ExitCode::Diagnostics);
        let text = report.render();
        assert!(text.contains("status: diagnostics"), "{text}");
        assert!(text.contains("  - TABULATE tiempos"), "{text}");
    }
}
