//! Where the data of the app lives, and whether a process is still running.
//!
//! The desktop app publishes what it is showing in its own data directory
//! (`live.json`, beside `recent-models.json`), and the MCP server reads it
//! from there. This module is the one that knows **where** that directory
//! is: the rules are the platform's, and the identifier of the app names
//! the directory inside it.
//!
//! It also knows whether a process is alive, which is how a document left
//! behind by an app that is gone is read as "no app": the pid of the
//! document says so.
//!
//! The file that used to say which model the app has open
//! (`genmsi.openmodel/1`) is **gone**: the live document carries the model
//! and the pid, and two files could disagree.

use std::env;
use std::io;
use std::path::{Path, PathBuf};

/// The identifier of the app: the directory of its data is named after it.
///
/// The desktop shell declares it in `tauri.conf.json`; a test pins the two
/// together, so a change of the identifier does not leave the MCP server
/// looking in a directory nobody writes to.
pub const APP_IDENTIFIER: &str = "org.gensmi.desktop";

/// The failure of publishing a file in the data directory of the app.
#[derive(Debug, thiserror::Error)]
pub enum AppDataError {
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
}
