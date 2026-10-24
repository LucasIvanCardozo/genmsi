//! The file that says which model the app has open.
//!
//! The desktop app publishes the model it has open in a file of its data
//! directory: `open-model.json`, beside `recent-models.json`. The file
//! carries the `genmsi.openmodel/1` document: the **absolute path** of the
//! model and the **pid** of the process that has it open.
//!
//! That is what closes the cycle with an agent: the agent asks the MCP
//! server which model the app has open, writes the model there and the app —
//! which watches the file it has open — reloads it and runs it again. Nobody
//! has to guess where the app writes, and nobody has to ask the reader.
//!
//! **The file belongs to the app; the engine only knows how to read and to
//! write it.** Where the data of the app lives is the platform's business:
//! [`app_data_dir`] is the one place that knows it, and the shell of the app
//! resolves the same directory with Tauri.

use std::env;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::documents::{OPEN_MODEL_SCHEMA, to_pretty_json};

/// The name of the file the app publishes, inside its data directory.
pub const OPEN_MODEL_FILE: &str = "open-model.json";

/// The identifier of the app: the directory of its data is named after it.
///
/// The desktop shell declares it in `tauri.conf.json`; a test pins the two
/// together, so a change of the identifier does not leave the MCP server
/// looking in a directory nobody writes to.
pub const APP_IDENTIFIER: &str = "org.gensmi.desktop";

/// The model the app has open.
///
/// This is the body of the `genmsi.openmodel/1` document: the file the app
/// has open, and the process that has it open. The two are empty when the
/// app has no model open.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenModelDocument {
    /// The absolute path of the model the app has open.
    #[serde(default)]
    pub path: Option<String>,
    /// The pid of the process that has the model open.
    #[serde(default)]
    pub pid: Option<u32>,
}

impl OpenModelDocument {
    /// The document of an app with no model open.
    pub fn empty() -> Self {
        Self {
            path: None,
            pid: None,
        }
    }

    /// The document of the model a process has open.
    ///
    /// The path is the absolute one: the agent that reads the document has
    /// to be able to write the model without knowing where the app was
    /// started from.
    pub fn of(path: &Path, pid: u32) -> Self {
        Self {
            path: Some(absolute(path).display().to_string()),
            pid: Some(pid),
        }
    }
}

/// The failure of publishing the file of the open model.
#[derive(Debug, thiserror::Error)]
pub enum OpenModelError {
    /// The file could not be written.
    #[error("cannot write {path}: {source}")]
    Write {
        /// The file that could not be written.
        path: PathBuf,
        /// What went wrong.
        #[source]
        source: io::Error,
    },
}

/// The directory the app's data lives in: the base directory of the platform
/// and the identifier of the app.
///
/// - Linux: `$XDG_DATA_HOME` (or `$HOME/.local/share`) and the identifier.
/// - macOS: `$HOME/Library/Application Support` and the identifier.
/// - Windows: `%APPDATA%` and the identifier.
///
/// `None` when the machine does not say where its data lives.
pub fn app_data_dir() -> Option<PathBuf> {
    let home = env_path("HOME");
    let xdg_data_home = env_path("XDG_DATA_HOME");
    let appdata = env_path("APPDATA");
    base_data_dir(
        home.as_deref(),
        xdg_data_home.as_deref(),
        appdata.as_deref(),
    )
    .map(|base| base.join(APP_IDENTIFIER))
}

/// The base directory of the app's data, per platform.
///
/// The rules are the platform's: Linux takes `$XDG_DATA_HOME` when it is an
/// absolute path and falls back to `$HOME/.local/share`; macOS takes
/// `$HOME/Library/Application Support`; Windows takes `%APPDATA%`.
#[cfg(target_os = "linux")]
fn base_data_dir(
    home: Option<&Path>,
    xdg_data_home: Option<&Path>,
    _appdata: Option<&Path>,
) -> Option<PathBuf> {
    xdg_data_home
        .filter(|directory| directory.is_absolute())
        .map(Path::to_path_buf)
        .or_else(|| home.map(|home| home.join(".local").join("share")))
}

/// The base directory of the app's data, per platform.
#[cfg(target_os = "macos")]
fn base_data_dir(
    home: Option<&Path>,
    _xdg_data_home: Option<&Path>,
    _appdata: Option<&Path>,
) -> Option<PathBuf> {
    home.map(|home| home.join("Library").join("Application Support"))
}

/// The base directory of the app's data, per platform.
#[cfg(target_os = "windows")]
fn base_data_dir(
    _home: Option<&Path>,
    _xdg_data_home: Option<&Path>,
    appdata: Option<&Path>,
) -> Option<PathBuf> {
    appdata.map(Path::to_path_buf)
}

/// The base directory of the app's data, per platform.
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn base_data_dir(
    home: Option<&Path>,
    xdg_data_home: Option<&Path>,
    _appdata: Option<&Path>,
) -> Option<PathBuf> {
    xdg_data_home
        .filter(|directory| directory.is_absolute())
        .map(Path::to_path_buf)
        .or_else(|| home.map(|home| home.join(".local").join("share")))
}

/// The file the app publishes, in the app's data directory.
pub fn open_model_file() -> Option<PathBuf> {
    app_data_dir().map(|directory| directory.join(OPEN_MODEL_FILE))
}

/// Write the file that says which model the app has open.
///
/// The path is the model the app has open; `None` means the app has no model
/// open, and the file is **removed**: a file that says nothing is not a lie,
/// but a file that says a model is open when there is none is.
///
/// The write is atomic: the text goes to a temporary file next to the target
/// and that file is renamed over it, so a reader never sees half of one.
pub fn write_open_model(
    directory: &Path,
    path: Option<&Path>,
    pid: u32,
) -> Result<(), OpenModelError> {
    let file = directory.join(OPEN_MODEL_FILE);
    let Some(path) = path else {
        return remove_file(&file);
    };
    let document = OpenModelDocument::of(path, pid);
    let text = to_pretty_json(OPEN_MODEL_SCHEMA, &document);
    std::fs::create_dir_all(directory).map_err(|source| OpenModelError::Write {
        path: directory.to_path_buf(),
        source,
    })?;
    let temporary = directory.join(format!("{OPEN_MODEL_FILE}.{pid}.tmp"));
    std::fs::write(&temporary, text).map_err(|source| OpenModelError::Write {
        path: temporary.clone(),
        source,
    })?;
    std::fs::rename(&temporary, &file).map_err(|source| {
        // The temporary file is not left behind: the directory of the app
        // does not fill with the remains of a failed write.
        let _ = std::fs::remove_file(&temporary);
        OpenModelError::Write {
            path: file.clone(),
            source,
        }
    })
}

/// Remove the file that says which model the app has open.
///
/// A file that is not there is not a failure: there is nothing to remove.
fn remove_file(file: &Path) -> Result<(), OpenModelError> {
    match std::fs::remove_file(file) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(OpenModelError::Write {
            path: file.to_path_buf(),
            source,
        }),
    }
}

/// Read the file the app published, in a directory.
///
/// `None` when the file is not there or when it cannot be read: a file that
/// cannot be read says nothing about the model the app has open, and a
/// broken file is not a failure of the reader.
pub fn read_open_model_in(directory: &Path) -> Option<OpenModelDocument> {
    let text = std::fs::read_to_string(directory.join(OPEN_MODEL_FILE)).ok()?;
    serde_json::from_str(&text).ok()
}

/// Read the file the app published, in the app's data directory.
pub fn read_open_model() -> Option<OpenModelDocument> {
    read_open_model_in(&app_data_dir()?)
}

/// The model the app has open, when the app is still running.
///
/// A file left behind by an app that is not running any more says nothing:
/// the pid of the document says whether the process is alive.
pub fn current_open_model_in(directory: &Path) -> Option<OpenModelDocument> {
    let document = read_open_model_in(directory)?;
    match document.pid {
        Some(pid) if !process_is_running(pid) => None,
        _ => Some(document),
    }
}

/// The model the app has open, when the app is still running.
pub fn current_open_model() -> Option<OpenModelDocument> {
    current_open_model_in(&app_data_dir()?)
}

/// The document that says which model the app has open, from a directory.
///
/// The document has no path and no pid when the app is not running: the
/// question has an answer either way, and an answer is never a failure.
pub fn open_model_document_in(directory: &Path) -> String {
    let document = current_open_model_in(directory).unwrap_or_else(OpenModelDocument::empty);
    to_pretty_json(OPEN_MODEL_SCHEMA, &document)
}

/// The document that says which model the app has open.
pub fn open_model_document() -> String {
    match app_data_dir() {
        Some(directory) => open_model_document_in(&directory),
        None => to_pretty_json(OPEN_MODEL_SCHEMA, &OpenModelDocument::empty()),
    }
}

/// Whether a process is running.
///
/// The check is the one the platform offers: on Linux the directory of the
/// process is there while it lives; on the other systems the `kill` of the
/// shell asks the kernel. A platform that cannot answer says the process is
/// running: a file that says nothing is better than a lie.
#[cfg(target_os = "linux")]
pub fn process_is_running(pid: u32) -> bool {
    // The processes of the machine are in `/proc`; a machine without `/proc`
    // cannot answer the question, and the answer is "yes".
    if !Path::new("/proc/self").exists() {
        return true;
    }
    Path::new(&format!("/proc/{pid}")).exists()
}

/// Whether a process is running.
#[cfg(all(unix, not(target_os = "linux")))]
pub fn process_is_running(pid: u32) -> bool {
    match std::process::Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                return true;
            }
            // The kernel says which failure it was: a process that is not
            // there is dead, a process of another user is alive.
            let message = String::from_utf8_lossy(&output.stderr);
            !message.contains("No such process")
        }
        // The question could not be asked: the answer is "yes".
        Err(_) => true,
    }
}

/// Whether a process is running.
#[cfg(not(unix))]
pub fn process_is_running(_pid: u32) -> bool {
    true
}

/// The absolute path of a file, as the machine sees it.
///
/// The path is not resolved through the links of the file system: the agent
/// that writes the model has to write it where the app is looking for it,
/// and the app looks for the path it was given.
fn absolute(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}

/// The value of an environment variable, as a path.
///
/// An empty variable is not a path: the machine does not say anything.
fn env_path(name: &str) -> Option<PathBuf> {
    let value = env::var_os(name)?;
    if value.is_empty() {
        return None;
    }
    Some(PathBuf::from(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn linux_takes_the_xdg_directory_and_falls_back_to_the_home() {
        assert_eq!(
            base_data_dir(
                Some(Path::new("/home/reader")),
                Some(Path::new("/data")),
                None
            ),
            Some(PathBuf::from("/data"))
        );
        assert_eq!(
            base_data_dir(Some(Path::new("/home/reader")), None, None),
            Some(PathBuf::from("/home/reader/.local/share"))
        );
        // A relative directory is not one: the specification of XDG says
        // that a relative path has to be ignored.
        assert_eq!(
            base_data_dir(
                Some(Path::new("/home/reader")),
                Some(Path::new("data")),
                None
            ),
            Some(PathBuf::from("/home/reader/.local/share"))
        );
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn macos_takes_the_application_support_of_the_home() {
        assert_eq!(
            base_data_dir(Some(Path::new("/Users/reader")), None, None),
            Some(PathBuf::from("/Users/reader/Library/Application Support"))
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn windows_takes_the_roaming_directory() {
        assert_eq!(
            base_data_dir(None, None, Some(Path::new("C:/Users/reader/AppData/Roaming"))),
            Some(PathBuf::from("C:/Users/reader/AppData/Roaming"))
        );
    }

    #[test]
    fn the_path_of_the_model_is_absolute() {
        let document = OpenModelDocument::of(Path::new("modelo.gmdl"), 42);
        let path = document.path.expect("the document names the model");
        assert!(
            Path::new(&path).is_absolute(),
            "the path is not absolute: {path}"
        );
    }
}
