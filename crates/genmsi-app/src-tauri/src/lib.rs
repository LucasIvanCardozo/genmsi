//! The desktop shell.
//!
//! The shell is a window around the same frontend the Vite server serves: it
//! moves the engine's documents through the same facade the dev bridge's
//! binary runs, so both backends return the same bytes.
//!
//! The shell also carries the plugin of the dialogs: the app opens and
//! saves the models with the dialogs of the system, and the recents live in
//! the app's data directory.

mod commands;

/// The root of the repository, when the app is run from one.
///
/// While the app is developed it is the repository itself, so the model paths
/// the frontend uses are the same ones the dev bridge uses. The shell lives
/// in `crates/genmsi-app/src-tauri`, so the root is three levels up.
///
/// **An installed app has no repository**: the path is baked in at compile
/// time and it does not exist on the machine of the reader. That is not a
/// problem — the models are opened with the dialogs of the system and the
/// recents live in the data directory of the app — so the root is only used
/// when it is really there.
pub fn app_root() -> Option<std::path::PathBuf> {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    root.is_dir().then_some(root)
}

/// The variable that turns off WebKitGTK's DMABUF renderer.
///
/// On Wayland that renderer paints the layers of the app at a reduced
/// resolution and then scales them: the canvas of the graph comes out
/// blurry —the shapes and the letters alike— after any repaint of the
/// window, and it is back sharp only until the next one. The other
/// renderer paints it right, and the app loses nothing measurable with it.
///
/// The blur is WebKit's, not ours: the DOM of the canvas is clean (no
/// filter, no opacity, no `will-change`) and the same drawing in the
/// lateral panel is sharp. Upstream it is the `contentsScale` of the
/// layers (webkit.org bug 271714) and the report of Tauri
/// (tauri-apps/wry#1727), which says it only happens on Wayland.
///
/// The environment has the last word: with the variable set to `0` the
/// renderer stays on, and with it set to anything else it stays as the
/// user asked.
const DMABUF_RENDERER: &str = "WEBKIT_DISABLE_DMABUF_RENDERER";

/// What the environment needs for the renderer, given what it says now.
///
/// `None` means the environment already has an opinion and this program
/// does not meddle with it.
fn renderer_environment_change(
    current: Option<&std::ffi::OsStr>,
) -> Option<(&'static str, &'static str)> {
    match current {
        Some(_) => None,
        None => Some((DMABUF_RENDERER, "1")),
    }
}

/// Turn off the DMABUF renderer, unless the environment already says.
#[cfg(target_os = "linux")]
fn prepare_render_environment() {
    let Some((name, value)) =
        renderer_environment_change(std::env::var_os(DMABUF_RENDERER).as_deref())
    else {
        return;
    };
    // SAFETY: the webview has not started yet, so the WebKit process that
    // reads the variable is still to be spawned, and it inherits the
    // environment as it is right now.
    unsafe {
        std::env::set_var(name, value);
    }
}

/// Start the shell.
pub fn run() {
    // The renderer of WebKit is set before the webview exists: the WebKit
    // process reads the environment when it is spawned.
    #[cfg(target_os = "linux")]
    prepare_render_environment();
    // The shell has no console of its own: the line goes to the log the
    // smoke check reads, so the path of the desktop app is observable.
    eprintln!("[genmsi-app] el shell arranca");
    // The app works on a machine where the repository is not: it only moves
    // to it when it is there, and it never fails for it.
    if let Some(root) = app_root() {
        let _ = std::env::set_current_dir(&root);
    }
    tauri::Builder::default()
        // The dialogs of the system: the app opens and saves the models
        // with them, and the frontend asks the plugin's commands.
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::app_version,
            commands::list_models,
            commands::load_model,
            commands::lint_model,
            commands::model_fingerprint,
            commands::simulate_model,
            commands::trace_model,
            commands::session,
            commands::cancel_run,
            commands::recent_models,
            commands::remember_model,
        ])
        .build(tauri::generate_context!())
        .expect("the GenMSI application could not start")
        .run(|app, event| {
            // The app is closing: the file that says which model it has
            // open goes away with it, so nobody reads it and believes
            // there is a model open.
            if let tauri::RunEvent::Exit = event {
                commands::forget_open_model(app);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The renderer of WebKit is turned off when the environment does not
    /// say anything about it.
    #[test]
    fn the_dmabuf_renderer_is_turned_off_when_nothing_says_otherwise() {
        assert_eq!(
            renderer_environment_change(None),
            Some((DMABUF_RENDERER, "1"))
        );
    }

    /// The environment has the last word: if it already says something, the
    /// program does not meddle with it.
    #[test]
    fn the_environment_decides_when_it_already_says_something() {
        for valor in ["0", "1", "cualquier cosa"] {
            assert_eq!(
                renderer_environment_change(Some(std::ffi::OsStr::new(valor))),
                None
            );
        }
    }
}
