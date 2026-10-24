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

/// Start the shell.
pub fn run() {
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
