//! Shared rendering: the versioned JSON envelope and the stdout helpers.
//!
//! Every `--json` document is `{"schema": "<name>/<major>", ...}`. The schema
//! id is the contract `docs/cli.md` publishes: within one major version the
//! body only grows (new fields may appear; existing names, types and meanings
//! stay), and a breaking change bumps the number.

use std::io::{self, Write as _};

use serde::Serialize;

use crate::error::CliError;

// The schema ids and the envelope are the engine's documents, published from
// one place (`genmsi_core::documents`) so every client agrees on them. The
// CLI adds `run-all`, which only it produces.
pub use genmsi_core::documents::{GPSS_SCHEMA, LINT_SCHEMA, REPORT_SCHEMA};
/// Schema id of the `run-all` document.
pub const RUN_ALL_SCHEMA: &str = "genmsi.run-all/1";

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
    let document = genmsi_core::documents::Envelope { schema, body };
    let text = serde_json::to_string_pretty(&document)
        .map_err(|error| CliError::Serialize(error.to_string()))?;
    print(&format!("{text}\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use genmsi_core::documents::TRACE_SCHEMA;

    #[derive(Serialize)]
    struct Body {
        answer: u32,
    }

    #[test]
    fn the_envelope_starts_with_the_schema() {
        let document = genmsi_core::documents::Envelope {
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
