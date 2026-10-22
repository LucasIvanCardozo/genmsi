//! The desktop shell.
//!
//! The shell is a window around the same frontend the Vite server serves: it
//! moves the engine's documents through the same facade the dev bridge's
//! binary runs, so both backends return the same bytes.

mod commands;

/// The root of the repository the shell reads its models from.
///
/// While the app is developed it is the repository itself, so the model paths
/// the frontend uses are the same ones the dev bridge uses.
pub fn app_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Start the shell.
pub fn run() {
    let root = app_root();
    if std::env::set_current_dir(&root).is_err() {
        eprintln!(
            "warning: the application could not move to {}",
            root.display()
        );
    }
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::list_models,
            commands::load_model,
            commands::lint_model,
            commands::simulate_model,
            commands::session,
        ])
        .run(tauri::generate_context!())
        .expect("the GenMSI application could not start");
}
