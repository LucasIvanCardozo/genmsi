//! The live model: the session reloads the model of the disk when the file
//! changes, and the reload does not take the reader's work away.
//!
//! The three rules are pinned here: the reload brings the model of the
//! disk, the undo history survives it —the reload is an operation of the
//! session, like an edit— and a model that does not compile leaves the
//! session exactly as it was.

use std::path::{Path, PathBuf};

use genmsi_app_engine::Session;
use genmsi_format::edit::EditCommand;

/// The reference models of the repository.
fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/models")
}

/// The model the tests open.
fn reference() -> PathBuf {
    fixtures().join("TP6__Clase 6_1_PuertaBanco.gmdl")
}

/// Another model of the repository: the one that arrives in the file.
fn other() -> PathBuf {
    fixtures().join("TP6__Clase 6_2_UltimoCliente.gmdl")
}

/// A directory of its own for the files the tests write.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("genmsi-app-live-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the scratch directory is created");
    dir
}

#[test]
fn the_reload_brings_the_model_of_the_disk() {
    let dir = scratch("reload");
    let target = dir.join("live.gmdl");
    std::fs::copy(reference(), &target).expect("the model is copied");

    let mut session = Session::open(&target, &target.display().to_string()).expect("opens");
    let before = genmsi_format::to_string(session.model());

    // Somebody else writes the file: this is what an agent that is
    // building the model does.
    std::fs::copy(other(), &target).expect("the other model is copied");

    let state = session.reload(&target).expect("reloads");
    assert_eq!(
        genmsi_format::to_string(session.model()),
        std::fs::read_to_string(other()).expect("the other model is there"),
        "the session did not take the model of the disk"
    );
    assert!(
        !state.dirty,
        "the model of the disk is not saved, and it is not dirty"
    );
    assert_eq!(state.path.as_deref(), Some(target.to_string_lossy().as_ref()));
    assert_ne!(before, genmsi_format::to_string(session.model()));

    std::fs::remove_dir_all(&dir).expect("the scratch directory is removed");
}

#[test]
fn the_reload_does_not_take_the_undo_history_away() {
    let dir = scratch("undo");
    let target = dir.join("live.gmdl");
    std::fs::copy(reference(), &target).expect("the model is copied");

    let mut session = Session::open(&target, &target.display().to_string()).expect("opens");
    let before = genmsi_format::to_string(session.model());

    // The reader changes the model: the session has an undo entry.
    session
        .apply(EditCommand::SetProperty {
            id: 6,
            property: "Media".to_string(),
            value: "45".to_string(),
        })
        .expect("sets");
    let edited = genmsi_format::to_string(session.model());
    assert!(session.state().can_undo, "the change cannot be undone");

    // The model of the disk arrives: another model.
    std::fs::copy(other(), &target).expect("the other model is copied");
    let state = session.reload(&target).expect("reloads");
    assert!(
        state.can_undo,
        "the reload cannot be undone: the reader's work is lost"
    );

    // The first undo is the reload: the model the reader had comes back,
    // with the undo stack it had.
    session.undo().expect("undoes");
    assert_eq!(
        genmsi_format::to_string(session.model()),
        edited,
        "the reload did not come undone"
    );
    assert!(
        session.state().can_undo,
        "the change of the reader cannot be undone any more"
    );

    // The second undo is the change of the reader.
    session.undo().expect("undoes");
    assert_eq!(
        genmsi_format::to_string(session.model()),
        before,
        "the change of the reader did not come undone"
    );

    std::fs::remove_dir_all(&dir).expect("the scratch directory is removed");
}

#[test]
fn a_model_that_does_not_compile_leaves_the_session_as_it_was() {
    let dir = scratch("broken");
    let target = dir.join("live.gmdl");
    std::fs::copy(reference(), &target).expect("the model is copied");

    let mut session = Session::open(&target, &target.display().to_string()).expect("opens");
    let before = genmsi_format::to_string(session.model());

    // The agent breaks the file: the compiler refuses it.
    std::fs::copy(fixtures().join("TP9__tp9.gmdl"), &target).expect("the broken model is copied");

    let error = session.reload(&target).expect_err("the compiler refuses it");
    assert!(
        error.to_string().contains("operand B"),
        "the error does not say what the compiler said: {error}"
    );

    // The session is exactly as it was: the model, its undo stack and its
    // clean state.
    assert_eq!(
        genmsi_format::to_string(session.model()),
        before,
        "the session lost the model"
    );
    assert!(!session.state().dirty, "the session is dirty after a refused reload");

    std::fs::remove_dir_all(&dir).expect("the scratch directory is removed");
}
