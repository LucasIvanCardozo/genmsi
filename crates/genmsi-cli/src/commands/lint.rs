//! `genmsi lint <model>`: can this model load, validate, compile and run?
//!
//! Lint stops before the simulation. It reports what the model compiles to and
//! any non-fatal finding: a compiler observation, or a warning of the format
//! (a block that uses an entity the model does not declare). A model that
//! cannot be compiled is a failure (exit 1), and a model that compiles with
//! findings exits 3 so a script can tell "broken" from "usable, with remarks".

use genmsi_core::documents::{LintDocument, lint_document};

use crate::cli::LintArgs;
use crate::error::{CliError, ExitCode};
use crate::model;
use crate::output::{self, LINT_SCHEMA};

/// Exit code a lint document implies.
fn exit_code(document: &LintDocument<'_>) -> ExitCode {
    if document.diagnostics.is_empty() {
        ExitCode::Success
    } else {
        ExitCode::Diagnostics
    }
}

/// Human-readable form of a lint document.
fn render(document: &LintDocument<'_>) -> String {
    let definitions = &document.definitions;
    let mut out = String::new();
    out.push_str(&format!("model: {}\n", document.path));
    out.push_str(&format!("title: {}\n", document.title));
    out.push_str(&format!("status: {}\n", document.status));
    out.push_str(&format!("blocks: {}\n", document.blocks));
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
    if document.diagnostics.is_empty() {
        out.push_str("diagnostics: none\n");
    } else {
        out.push_str("diagnostics:\n");
        for message in &document.diagnostics {
            out.push_str(&format!("  - {message}\n"));
        }
    }
    out
}

/// Run `genmsi lint`.
pub fn run(args: LintArgs) -> Result<ExitCode, CliError> {
    let loaded = model::load(&args.model.path)?;
    let document = lint_document(
        &loaded.program,
        &loaded.model,
        &args.model.path.display().to_string(),
    );
    if args.output.json {
        output::print_json(LINT_SCHEMA, &document)?;
    } else {
        output::print(&render(&document))?;
    }
    Ok(exit_code(&document))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(diagnostics: &[String]) -> LintDocument<'_> {
        LintDocument {
            path: "model.gmdl".to_string(),
            status: if diagnostics.is_empty() {
                "ok"
            } else {
                "diagnostics"
            },
            title: "model",
            blocks: 3,
            definitions: genmsi_core::documents::DefinitionCounts {
                functions: 0,
                variables: 0,
                random_variables: 0,
                storages: 0,
                tables: 0,
                qtables: 0,
            },
            diagnostics: diagnostics.to_vec(),
        }
    }

    #[test]
    fn a_clean_model_succeeds() {
        let document = report(&[]);
        assert_eq!(exit_code(&document), ExitCode::Success);
        let text = render(&document);
        assert!(text.contains("status: ok"), "{text}");
        assert!(text.contains("diagnostics: none"), "{text}");
    }

    #[test]
    fn diagnostics_on_a_runnable_model_exit_with_three() {
        let diagnostics = vec!["TABULATE tiempos has no TABLE definition".to_string()];
        let document = report(&diagnostics);
        assert_eq!(exit_code(&document), ExitCode::Diagnostics);
        let text = render(&document);
        assert!(text.contains("status: diagnostics"), "{text}");
        assert!(text.contains("  - TABULATE tiempos"), "{text}");
    }
}
