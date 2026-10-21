//! `genmsi simulate <model>`: run the model and print its GPSS report.

use genmsi_core::Simulation;

use crate::cli::SimulateArgs;
use crate::error::{CliError, ExitCode};
use crate::model;
use crate::output::{self, REPORT_SCHEMA};

/// Run `genmsi simulate`.
pub fn run(args: SimulateArgs) -> Result<ExitCode, CliError> {
    let loaded = model::load(&args.model.path)?;
    let config = model::config(args.run.seed, args.run.until);
    let result = Simulation::new(&loaded.program, config)
        .run()
        .map_err(|error| CliError::model(&args.model.path, error))?;
    if args.output.json {
        let document = genmsi_core::json_report(&result, &loaded.program);
        output::print_json(REPORT_SCHEMA, &document)?;
    } else {
        output::print(&genmsi_core::render(&result, &loaded.program))?;
    }
    Ok(ExitCode::Success)
}
