//! `genmsi gpss <model>`: print the GPSS program the model compiles to.
//!
//! The original editor shows this text in its "Código GPSS" tab. It is the
//! original's own statement of what each block does, and the harness in
//! `scripts/wine-oracle/` compares it with the original's for the same model.

use crate::cli::GpssArgs;
use crate::error::{CliError, ExitCode};
use crate::model;
use crate::output::{self, GPSS_SCHEMA};

/// Run `genmsi gpss`.
pub fn run(args: GpssArgs) -> Result<ExitCode, CliError> {
    let loaded = model::load(&args.model.path)?;
    let gpss = genmsi_core::gpss::render(&loaded.program);
    if args.output.json {
        let document = serde_json::json!({ "gpss": gpss });
        output::print_json(GPSS_SCHEMA, &document)?;
    } else {
        output::print(gpss.trim_end_matches('\n'))?;
    }
    Ok(ExitCode::Success)
}
