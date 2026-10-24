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

/// The root of the repository the shell reads its models from.
///
/// While the app is developed it is the repository itself, so the model paths
/// the frontend uses are the same ones the dev bridge uses. The shell lives
/// in `crates/genmsi-app/src-tauri`, so the root is three levels up.
pub fn app_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// Start the shell.
pub fn run() {
    // The shell has no console of its own: the line goes to the log the
    // smoke check reads, so the path of the desktop app is observable.
    eprintln!("[genmsi-app] el shell arranca");
    let root = app_root();
    if std::env::set_current_dir(&root).is_err() {
        eprintln!(
            "warning: the application could not move to {}",
            root.display()
        );
    }
    tauri::Builder::default()
        // The dialogs of the system: the app opens and saves the models
        // with them, and the frontend asks the plugin's commands.
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_models,
            commands::load_model,
            commands::lint_model,
            commands::simulate_model,
            commands::trace_model,
            commands::session,
            commands::recent_models,
            commands::remember_model,
        ])
        .run(tauri::generate_context!())
        .expect("the GenMSI application could not start");
}
