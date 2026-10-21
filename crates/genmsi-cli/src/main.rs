use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// GenMSI: discrete-event simulation toolkit (GPSS-like), headless engine with
/// CLI, MCP and GUI clients.
///
/// The simulator is under active development; subcommands report
/// "not implemented yet" until their phase lands (see docs/ROADMAP.md).
#[derive(Parser)]
#[command(name = "genmsi", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a .gmdl model file and report structural problems.
    Lint {
        /// Path to the model file.
        path: PathBuf,
    },
    /// Run a simulation and print the report (add --json for machine output).
    Simulate {
        /// Path to the model file.
        path: PathBuf,
        /// Emit the report as typed JSON instead of human-readable text.
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Lint { .. } => unimplemented_stub("lint", "Phase 1"),
        Commands::Simulate { .. } => unimplemented_stub("simulate", "Phase 2"),
    }
}

fn unimplemented_stub(command: &str, phase: &str) {
    eprintln!("error: `{command}` is not implemented yet (planned for {phase})");
    eprintln!("see docs/ROADMAP.md for the phase plan");
    std::process::exit(2);
}
