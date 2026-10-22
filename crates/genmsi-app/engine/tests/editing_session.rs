//! The editing session, proved on the reference models.
//!
//! The three rules the phase asks for are pinned here: a save without
//! changes is byte-identical, a paste never reuses an id and never leaves a
//! link dangling, and a model the engine refuses can still be saved and the
//! file is refused with the same class as the diagnostic the app shows.

use std::path::{Path, PathBuf};

use genmsi_app_engine::Session;
use genmsi_app_engine::session::diagnostics_of;

/// The reference models of the repository.
fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/models")
}

/// A directory of its own for the files the tests write.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("genmsi-app-engine-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the scratch directory is created");
    dir
}

/// Every reference model.
fn models() -> Vec<PathBuf> {
    let mut models: Vec<PathBuf> = std::fs::read_dir(fixtures())
        .expect("the reference models are there")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "gmdl"))
        .collect();
    models.sort();
    models
}

#[test]
fn saving_a_model_without_changes_is_byte_identical() {
    let dir = scratch("identical");
    let mut refused = 0;
    for model in models() {
        let name = model
            .file_name()
            .expect("the model has a name")
            .to_string_lossy()
            .into_owned();
        let target = dir.join(&name);
        // The writer always round-trips a model it can read: the file comes
        // back byte for byte.
        let text = std::fs::read_to_string(&model).expect("the model is there");
        let parsed = genmsi_format::parse_str(&text).expect("the model parses");
        assert_eq!(
            genmsi_format::to_string(&parsed),
            text,
            "{name} changed without an edit"
        );

        // The app opens the models the reader accepts and saves them back
        // byte for byte.
        let mut session = match Session::open(&model, &model.display().to_string()) {
            Ok(session) => session,
            Err(error) => {
                // One reference model is refused by the reader, by design:
                // the app shows the error and the file stays untouched.
                refused += 1;
                assert!(error.to_string().contains("invalid block"), "{error}");
                continue;
            }
        };
        let outcome = session.save(Some(&target)).expect("saves");
        assert_eq!(
            std::fs::read(&target).expect("the file is there"),
            std::fs::read(&model).expect("the original is there"),
            "{name} changed without an edit"
        );
        assert_eq!(outcome.bytes, std::fs::metadata(&model).unwrap().len() as usize);
        assert!(!session.state().dirty, "{name} is dirty after the save");
    }
    // The five reference models the engine refuses are refused in two
    // classes: four by the compiler (the app opens them and shows the
    // error) and one by the reader (the app cannot open it at all).
    assert_eq!(refused, 1, "the number of models the reader refuses moved");
}

#[test]
fn pasting_twice_leaves_every_id_unique_and_every_link_resolvable() {
    let model = fixtures().join("TP6__Clase 6_1_PuertaBanco.gmdl");
    let dir = scratch("paste");
    let target = dir.join("pasted.gmdl");
    let mut session = Session::open(&model, &model.display().to_string()).expect("opens");

    // A block with a link, copied twice.
    let source = 6;
    let first = session
        .apply(genmsi_format::edit::EditCommand::Paste {
            ids: vec![source],
            dx: 180,
            dy: 90,
        })
        .expect("pastes");
    let second = session
        .apply(genmsi_format::edit::EditCommand::Paste {
            ids: vec![source],
            dx: 180,
            dy: 90,
        })
        .expect("pastes again");

    let copies: Vec<u32> = first
        .model
        .blocks
        .iter()
        .chain(second.model.blocks.iter())
        .map(|block| block.id)
        .collect();
    // Every id of the model is unique: the copies never reuse one.
    let mut ids: Vec<u32> = session.model().blocks.iter().map(|block| block.id).collect();
    let total = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), total, "the model has a repeated id");
    assert!(copies.len() >= 2, "the paste did not produce copies");

    // The saved file loads, validates and compiles: every link resolves.
    session.save(Some(&target)).expect("saves");
    let saved = genmsi_format::load(&target).expect("the saved file loads");
    genmsi_format::validate(&saved).expect("the saved model is valid");
    genmsi_core::compile_model(&saved).expect("the saved model compiles");
}

#[test]
fn a_model_the_engine_refuses_is_saved_and_stays_refused() {
    let model = fixtures().join("TP9__tp9.gmdl");
    let dir = scratch("refused");
    let target = dir.join("refused.gmdl");
    let mut session = Session::open(&model, &model.display().to_string()).expect("opens");

    // The diagnostic the app shows.
    let shown = session.state().diagnostics;
    assert_eq!(shown.status, "error", "the app does not show the refusal");
    let message = shown.message.expect("the engine's message");
    assert!(message.contains("operand B"), "{message}");

    // Saving is not blocked.
    session.save(Some(&target)).expect("saves");
    let saved = genmsi_format::load(&target).expect("the saved file loads");
    // And the file is refused by the engine with the same message the app
    // showed: the same class of failure.
    let error = genmsi_core::compile_model(&saved).expect_err("the engine refuses it");
    assert_eq!(error.to_string(), message);
    assert_eq!(diagnostics_of(&saved).message, Some(message));
}

#[test]
fn one_operation_is_one_undo_entry() {
    let model = fixtures().join("TP6__Clase 6_1_PuertaBanco.gmdl");
    let mut session = Session::open(&model, &model.display().to_string()).expect("opens");
    let before = genmsi_format::to_string(session.model());

    session
        .apply(genmsi_format::edit::EditCommand::SetProperty {
            id: 6,
            property: "Media".to_string(),
            value: "45".to_string(),
        })
        .expect("sets");
    assert!(session.state().can_undo);
    assert_eq!(
        session.model().block(6).expect("the block").prop_value("Media"),
        Some("45".to_string())
    );

    session.undo().expect("undoes");
    assert_eq!(genmsi_format::to_string(session.model()), before);
    assert!(session.state().can_redo);

    session.redo().expect("redoes");
    assert_eq!(
        session.model().block(6).expect("the block").prop_value("Media"),
        Some("45".to_string())
    );
}

#[test]
fn a_new_model_is_the_root_container_and_nothing_else() {
    let session = Session::new_model();
    assert_eq!(session.model().blocks.len(), 1);
    let root = session.model().root().expect("the model has a root");
    assert_eq!(root.kind, 'C');
    assert_eq!(root.class, "main");
    assert!(session.model().activity_blocks().next().is_none());
    // The new model is written as the original editor wrote a new file.
    let text = genmsi_format::to_string(session.model());
    assert!(text.starts_with("<gmodel versioncode="), "{text}");
    assert!(text.contains("moduleclass=\"main\""), "{text}");
}
