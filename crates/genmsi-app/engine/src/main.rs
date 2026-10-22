//! `genmsi-app-engine`: the engine's documents on stdout, for the dev bridge.
//!
//! A thin binary over the facade: it spawns once per request from the Vite dev
//! bridge, prints one document to stdout and nothing else. On failure it
//! prints one line `error: <message>` to stderr and exits with code 1, the
//! same contract the `genmsi` CLI follows.

use std::io::{self, Write as _};
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use genmsi_app_engine::{self as engine, AppError};

/// Schema id of the app's own model list; an application document, not an
/// engine document.
const MODELS_SCHEMA: &str = "genmsi.models/1";

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
    /// The app's model list: the `.gmdl` files of one directory.
    Models {
        /// Directory to scan (default: the current directory).
        dir: Option<PathBuf>,
    },
}

/// Body of the `genmsi.models/1` list document.
#[derive(serde::Serialize)]
struct ModelsBody<'a> {
    /// Directory as the caller named it.
    dir: &'a str,
    /// Models, sorted by file name.
    models: &'a [engine::ModelFile],
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

fn run(command: Command) -> Result<String, AppError> {
    match command {
        Command::Model { path } => engine::model_document_bytes(&path),
        Command::Lint { path } => engine::lint_document_bytes(&path),
        Command::Simulate {
            path,
            seed,
            until,
        } => engine::simulate_document_bytes(&path, seed, until),
        Command::Models { dir } => {
            let dir = dir.unwrap_or_else(|| PathBuf::from("."));
            let models = engine::list_models(&dir)?;
            let body = ModelsBody {
                dir: &dir.display().to_string(),
                models: &models,
            };
            Ok(format!(
                "{}\n",
                genmsi_core::documents::to_pretty_json(MODELS_SCHEMA, &body)
            ))
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
