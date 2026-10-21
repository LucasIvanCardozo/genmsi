//! One module per subcommand.
//!
//! Each command follows the same shape: turn the arguments into one document
//! through the engine boundary ([`crate::model`]), print it, and report the
//! exit code that document implies.

pub mod lint;
pub mod simulate;

use crate::cli::Command;
use crate::error::{CliError, ExitCode};

/// Dispatch one parsed subcommand.
pub fn run(command: Command) -> Result<ExitCode, CliError> {
    match command {
        Command::Lint(args) => lint::run(args),
        Command::Simulate(args) => simulate::run(args),
    }
}
