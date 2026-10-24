//! `genmsi-app-engine`: the engine's documents on stdout, for the dev bridge.
//!
//! A thin binary over the facade: it spawns once per request from the Vite dev
//! bridge, prints one document to stdout and nothing else. On failure it
//! prints one line `error: <message>` to stderr and exits with code 1, the
//! same contract the `genmsi` CLI follows.
//!
//! The `session` subcommand is the exception: it keeps one model in memory
//! and speaks the editing protocol, one JSON request per line and one JSON
//! answer per line. It is what the dev bridge holds open while the user
//! edits.

use std::io::{self, BufRead, Write as _};
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use genmsi_app_engine::{self as engine, AppError, SessionRequest};

#[derive(Parser)]
#[command(
    name = "genmsi-app-engine",
    about = "Print the GenMSI engine's documents for the app's backends"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// The `genmsi.model/1` document of a model: its blocks and its links.
    Model {
        /// Model file.
        path: PathBuf,
    },
    /// The `genmsi.lint/1` document of a model: what it compiles to.
    Lint {
        /// Model file.
        path: PathBuf,
    },
    /// The `genmsi.report/1` document of a finished run.
    Simulate {
        /// Model file.
        path: PathBuf,
        /// Seed of the run.
        #[arg(long, default_value_t = 1)]
        seed: u64,
        /// Stop the run at this simulation time.
        #[arg(long)]
        until: Option<f64>,
    },
    /// The `genmsi.trace/1` document of a finished run.
    Trace {
        /// Model file.
        path: PathBuf,
        /// Seed of the run.
        #[arg(long, default_value_t = 1)]
        seed: u64,
        /// Stop the run at this simulation time.
        #[arg(long)]
        until: Option<f64>,
        /// Maximum number of trace records kept.
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    /// The app's model list: the `.gmdl` files of one directory.
    Models {
        /// Directory to scan (default: the current directory).
        dir: Option<PathBuf>,
    },
    /// The `genmsi.catalogue/1` document: the block classes the editor offers.
    Catalogue,
    /// Keep one model in memory and speak the editing protocol.
    ///
    /// One JSON request per line on stdin, one JSON answer per line on
    /// stdout. The dev bridge holds this process open while the user edits.
    Session,
}

/// Print text to stdout; a closed pipe is success, not an error.
fn print(text: &str) {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let result = out.write_all(text.as_bytes()).and_then(|()| out.flush());
    if let Err(error) = result
        && error.kind() != io::ErrorKind::BrokenPipe
    {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

/// Run the editing protocol until the input ends.
///
/// Every answer is one line of compact JSON, flushed as soon as it is written:
/// the dev bridge reads it before it sends the next request.
fn run_session() -> Result<(), AppError> {
    let stdin = io::stdin();
    let mut session: Option<engine::Session> = None;
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for line in stdin.lock().lines() {
        let line = line
            .map_err(|error| AppError::Usage(format!("cannot read the request: {error}")))?;
        if line.trim().is_empty() {
            continue;
        }
        let request: SessionRequest = serde_json::from_str(&line).map_err(|error| {
            AppError::Usage(format!("cannot read the request: {error}"))
        })?;
        let response = engine::answer(&mut session, request);
        let text =
            serde_json::to_string(&response).expect("the answers of the facade always serialize");
        writeln!(out, "{text}").map_err(|error| {
            AppError::Usage(format!("cannot write the answer: {error}"))
        })?;
        out.flush()
            .map_err(|error| AppError::Usage(format!("cannot write the answer: {error}")))?;
    }
    Ok(())
}

fn run(command: Command) -> Result<String, AppError> {
    match command {
        Command::Model { path } => engine::model_document_bytes(&path),
        Command::Lint { path } => engine::lint_document_bytes(&path),
        Command::Simulate {
            path,
            seed,
            until,
        } => engine::simulate_document_bytes(&path, seed, until, None),
        Command::Trace {
            path,
            seed,
            until,
            limit,
        } => engine::trace_document_bytes(&path, seed, until, limit, None),
        Command::Models { dir } => {
            let dir = dir.unwrap_or_else(|| PathBuf::from("."));
            engine::models_document_bytes(&[dir])
        }
        Command::Catalogue => Ok(engine::catalogue_document_bytes()),
        Command::Session => {
            run_session()?;
            Ok(String::new())
        }
    }
}

fn main() {
    let cli = Cli::parse();
    match run(cli.command) {
        Ok(document) => print(&document),
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    }
}
