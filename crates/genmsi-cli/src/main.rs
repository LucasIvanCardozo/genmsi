//! GenMSI command-line interface.
//!
//! A thin layer over `genmsi-core` (decision D2: the CLI carries no simulation
//! logic). Layout:
//!
//! - [`cli`]: the argument surface, one struct per subcommand.
//! - [`commands`]: one module per subcommand; each renders one document.
//! - [`model`]: the only place that calls the engine crates.
//! - [`output`]: the versioned JSON envelope and stdout helpers.
//! - [`error`]: exit codes and the failure type reported on stderr.
//!
//! Every subcommand prints a deterministic document, which is what makes the
//! output snapshot-testable (ROADMAP Phase 2) and usable from CI.

mod cli;
mod commands;
mod error;
mod model;
mod output;

use std::io::{self, Write as _};

use clap::Parser as _;

use crate::cli::Cli;

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    match commands::run(cli.command) {
        Ok(code) => code.into(),
        Err(error) => {
            let _ = writeln!(io::stderr(), "error: {error}");
            error.exit_code().into()
        }
    }
}
