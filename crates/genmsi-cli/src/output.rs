//! Shared rendering: the versioned JSON envelope and the stdout helpers.
//!
//! Every `--json` document is `{"schema": "<name>/<major>", ...}`. The schema
//! id is the contract `docs/cli.md` publishes: within one major version the
//! body only grows (new fields may appear; existing names, types and meanings
//! stay), and a breaking change bumps the number.

use std::io::{self, Write as _};

use serde::Serialize;

use crate::error::CliError;

/// Schema id of the `lint` document.
pub const LINT_SCHEMA: &str = "genmsi.lint/1";
/// Schema id of the `simulate` document.
pub const REPORT_SCHEMA: &str = "genmsi.report/1";
/// Schema id of the `trace` document.
pub const TRACE_SCHEMA: &str = "genmsi.trace/1";
/// Schema id of the `run-all` document.
pub const RUN_ALL_SCHEMA: &str = "genmsi.run-all/1";

/// A versioned document: the schema id followed by the body fields.
#[derive(Debug, Serialize)]
pub struct Envelope<'a, T: Serialize> {
    /// Schema id of this document, `<name>/<major>`.
    pub schema: &'a str,
    /// Document fields, serialized at the top level.
    #[serde(flatten)]
    pub body: T,
}

/// Write text to stdout.
///
/// A closed pipe (`genmsi simulate model.gmdl | head`) is success, not an
/// error: the consumer stopped reading on purpose.
pub fn print(text: &str) -> Result<(), CliError> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    match out.write_all(text.as_bytes()).and_then(|()| out.flush()) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(error) => Err(CliError::from(error)),
    }
}

/// Write one pretty-printed JSON document.
pub fn print_json<T: Serialize + ?Sized>(schema: &str, body: &T) -> Result<(), CliError> {
    let document = Envelope { schema, body };
    let text = serde_json::to_string_pretty(&document)
        .map_err(|error| CliError::Serialize(error.to_string()))?;
    print(&format!("{text}\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct Body {
        answer: u32,
    }

    #[test]
    fn the_envelope_starts_with_the_schema() {
        let document = Envelope {
            schema: LINT_SCHEMA,
            body: Body { answer: 42 },
        };
        let text = serde_json::to_string(&document).expect("serializes");
        assert_eq!(text, r#"{"schema":"genmsi.lint/1","answer":42}"#);
    }

    #[test]
    fn schema_ids_are_versioned() {
        for (schema, name) in [
            (LINT_SCHEMA, "genmsi.lint"),
            (REPORT_SCHEMA, "genmsi.report"),
            (TRACE_SCHEMA, "genmsi.trace"),
            (RUN_ALL_SCHEMA, "genmsi.run-all"),
        ] {
            assert_eq!(schema, format!("{name}/1"));
        }
    }
}
