//! `genmsi mcp`: serve the MCP protocol over stdio.
//!
//! This is not a document command: standard output belongs to the protocol and
//! nothing else may be printed there, and there is nothing to parse beyond
//! what clap needs (the subcommand takes no options). It hosts the same
//! server as the standalone `genmsi-mcp` binary, through
//! [`genmsi_mcp::serve_stdio_blocking`].

use std::io::{self, Write as _};

use crate::error::ExitCode;

/// Run `genmsi mcp`.
///
/// On failure the message goes to stderr — never stdout — and the command
/// reports the CLI's exit code for "could not run" (1).
pub fn run() -> ExitCode {
    match genmsi_mcp::serve_stdio_blocking() {
        Ok(()) => ExitCode::Success,
        Err(error) => {
            let _ = writeln!(io::stderr(), "error: {error}");
            ExitCode::Failure
        }
    }
}
