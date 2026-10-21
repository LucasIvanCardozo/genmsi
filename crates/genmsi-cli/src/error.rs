//! Exit codes and the single failure type the CLI reports.
//!
//! The codes are part of the CLI contract (see `docs/cli.md`):
//!
//! | Code | Meaning |
//! |---|---|
//! | `0` | Success. |
//! | `1` | The model does not load, does not validate or does not run. |
//! | `2` | Usage error: reserved for `clap`'s argument errors and for an argument that cannot be used (for example a `run-all` target that is not a directory). |
//! | `3` | `lint` found compiler diagnostics on a model that is still runnable. |

use std::fmt;
use std::path::Path;

/// Exit code of a finished invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// The command did what it was asked to do.
    Success = 0,
    /// The model could not be loaded, validated, compiled or run.
    Failure = 1,
    /// The command line was not usable.
    Usage = 2,
    /// `lint` reported diagnostics but the model is runnable.
    Diagnostics = 3,
}

impl From<ExitCode> for std::process::ExitCode {
    fn from(code: ExitCode) -> Self {
        Self::from(code as u8)
    }
}

/// Something that stopped the command before it produced its document.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// The model did not load, validate, compile or run.
    ///
    /// The path comes first so a log line says which model failed; the engine
    /// message keeps the source line and the node it carries.
    #[error("{path}: {message}")]
    Model {
        /// Model path, as the user wrote it.
        path: String,
        /// Engine message.
        message: String,
    },
    /// An argument is well-formed but cannot be used; exits with code 2.
    #[error("{0}")]
    Usage(String),
    /// Reading a directory failed.
    #[error("{path}: {message}")]
    Io {
        /// Path involved.
        path: String,
        /// Operating system message.
        message: String,
    },
    /// A document could not be serialized.
    #[error("cannot serialize output: {0}")]
    Serialize(String),
}

impl CliError {
    /// A model failure, with the path the user asked for.
    pub fn model(path: &Path, error: impl fmt::Display) -> Self {
        Self::Model {
            path: path.display().to_string(),
            message: error.to_string(),
        }
    }

    /// The message without the path prefix.
    ///
    /// Used by documents that already carry the path in their own field
    /// (`run-all` names the model on its own line).
    pub fn detail(&self) -> String {
        match self {
            Self::Model { message, .. } => message.clone(),
            other => other.to_string(),
        }
    }

    /// Exit code this failure produces.
    pub fn exit_code(&self) -> ExitCode {
        match self {
            Self::Usage(_) => ExitCode::Usage,
            Self::Model { .. } | Self::Io { .. } | Self::Serialize(_) => ExitCode::Failure,
        }
    }
}

/// A failure reported while writing the document.
impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        Self::Serialize(error.to_string())
    }
}

impl fmt::Display for ExitCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_failures_exit_with_one() {
        let error = CliError::model(Path::new("model.gmdl"), "node 4 (Advance): Media is empty");
        assert_eq!(error.exit_code(), ExitCode::Failure);
        assert_eq!(
            error.to_string(),
            "model.gmdl: node 4 (Advance): Media is empty"
        );
    }

    #[test]
    fn the_detail_drops_the_path_but_keeps_the_engine_message() {
        let error = CliError::model(Path::new("m.gmdl"), "no TERMINATE block");
        assert_eq!(error.detail(), "no TERMINATE block");
        assert_eq!(
            CliError::Usage("not a directory".to_string()).detail(),
            "not a directory"
        );
    }

    #[test]
    fn unusable_arguments_exit_with_two() {
        let error = CliError::Usage("not a directory".to_string());
        assert_eq!(error.exit_code(), ExitCode::Usage);
        assert_eq!(ExitCode::Usage.to_string(), "2");
    }
}
