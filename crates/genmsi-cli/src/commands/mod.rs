//! One module per subcommand.
//!
//! Each command follows the same shape: turn the arguments into one document
//! through the engine boundary ([`crate::model`]), print it, and report the
//! exit code that document implies.

pub mod gpss;
pub mod lint;
pub mod mcp;
pub mod run_all;
pub mod simulate;
pub mod trace;

use crate::cli::Command;
use crate::error::{CliError, ExitCode};

/// Dispatch one parsed subcommand.
pub fn run(command: Command) -> Result<ExitCode, CliError> {
    match command {
        Command::Lint(args) => lint::run(args),
        Command::Simulate(args) => simulate::run(args),
        Command::Trace(args) => trace::run(args),
        Command::Gpss(args) => gpss::run(args),
        Command::RunAll(args) => run_all::run(args),
        // The MCP server is dispatched before any document work: it serves
        // the protocol on stdout until the client disconnects and reports its
        // own failure, so its result is already the process exit code.
        Command::Mcp => Ok(mcp::run()),
    }
}
