//! The live state of the app: what the reader is seeing.
//!
//! Two rules are pinned here:
//!
//! - the session keeps the last run and the parameters it ran with, so the
//!   publisher can say "this is what the reader is seeing";
//! - the model in memory travels **only** when the reader has changes that
//!   are not saved: when it does not, the file *is* the model and the agent
//!   reads it from there.

use std::path::{Path, PathBuf};

use genmsi_app_engine::{Session, SessionRequest, answer};
use genmsi_format::edit::EditCommand;

/// The reference model of the app's slice.
fn reference() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../fixtures/models/TP6__Clase 6_1_PuertaBanco.gmdl")
}

/// A directory of its own for the files the tests write.
///
/// The directory is outside the repository and the test removes it when it
/// is done.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("genmsi-app-live-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the scratch directory is created");
    dir
}

#[test]
fn the_session_keeps_the_last_run_and_the_parameters_it_ran_with() {
    let path = reference();
    let mut session = Some(
        Session::open(&path, &path.display().to_string()).expect("the model opens"),
    );
    let answered = answer(
        &mut session,
        SessionRequest::RunCurrent {
            seed: Some(7),
            until: None,
            limit: Some(50),
            start: Some(3),
        },
    );
    assert!(answered.ok, "the request failed: {:?}", answered.error);

    let session = session.as_ref().expect("the session is open");
    let run = session
        .last_run()
        .expect("the session kept the last run");
    assert_eq!(run.seed, 7, "the run did not keep its seed");
    assert_eq!(run.until, None, "the run did not keep its horizon");
    assert_eq!(run.start, 3, "the run did not keep its START");

    let report: serde_json::Value =
        serde_json::from_str(run.report.as_deref().expect("the run has a report"))
            .expect("the report parses");
    assert_eq!(report["schema"], "genmsi.report/1");
    assert_eq!(report["seed"], 7);
    let trace: serde_json::Value =
        serde_json::from_str(run.trace.as_deref().expect("the run has a trace"))
            .expect("the trace parses");
    assert_eq!(trace["schema"], "genmsi.trace/1");
}

#[test]
fn the_live_state_carries_the_last_run() {
    let path = reference();
    let mut session = Some(
        Session::open(&path, &path.display().to_string()).expect("the model opens"),
    );
    let answered = answer(
        &mut session,
        SessionRequest::RunCurrent {
            seed: Some(11),
            until: None,
            limit: Some(50),
            start: None,
        },
    );
    assert!(answered.ok, "the request failed: {:?}", answered.error);

    let session = session.as_ref().expect("the session is open");
    let live = session
        .live_body()
        .run
        .expect("the live state carries the last run");
    assert_eq!(live.seed, 11);
    assert_eq!(live.until, None);
    assert_eq!(live.start, 1, "the START of the original is one");
    let report = live.report.expect("the report of the run travels");
    assert_eq!(report["schema"], "genmsi.report/1");
    assert_eq!(report["seed"], 11);
}

#[test]
fn a_session_that_did_not_run_has_no_run_to_publish() {
    let path = reference();
    let session = Session::open(&path, &path.display().to_string()).expect("the model opens");
    assert!(session.last_run().is_none(), "the session kept a run it never made");
    assert!(
        session.live_body().run.is_none(),
        "the live state carries a run that never happened"
    );
}

#[test]
fn the_live_state_carries_the_model_in_memory_only_when_there_are_unsaved_changes() {
    let path = reference();
    let dir = scratch("model");
    let mut session = Session::open(&path, &path.display().to_string()).expect("the model opens");

    // Nothing changed: the file *is* the model, and the live state does
    // not carry it.
    let body = session.live_body();
    assert!(!body.dirty, "a model that was just opened has changes");
    assert!(
        body.model.is_none(),
        "the model in memory travels without changes: {body:?}"
    );
    assert_eq!(
        body.path.as_deref(),
        Some(path.display().to_string().as_str()),
        "the live state does not name the model that is open"
    );
    assert_eq!(body.pid, Some(std::process::id()));

    // An edit: the changes are not saved, and the model travels.
    session
        .apply(EditCommand::SetProperty {
            id: 6,
            property: "Media".to_string(),
            value: "45".to_string(),
        })
        .expect("the edit applies");
    let body = session.live_body();
    assert!(body.dirty, "the model has changes that are not saved");
    let model = body
        .model
        .clone()
        .expect("the model in memory travels with the changes");
    assert_eq!(model["schema"], "genmsi.model/1");
    assert!(
        model["blocks"]
            .as_array()
            .is_some_and(|blocks| !blocks.is_empty()),
        "the model travels empty: {model}"
    );

    // After the save, the file is the model again.
    let target = dir.join("guardado.gmdl");
    session.save(Some(&target)).expect("the model saves");
    let body = session.live_body();
    assert!(!body.dirty, "the model was saved");
    assert!(
        body.model.is_none(),
        "the model in memory travels after the save: {body:?}"
    );

    std::fs::remove_dir_all(&dir).expect("the scratch directory is removed");
}

#[test]
fn the_live_state_carries_what_the_app_found_in_the_model() {
    let path = reference();
    let session = Session::open(&path, &path.display().to_string()).expect("the model opens");
    let state = session.state();
    let body = session.live_body();
    assert_eq!(
        body.diagnostics, state.diagnostics.diagnostics,
        "the live state does not carry the findings of the app"
    );
}

#[test]
fn an_app_with_no_model_open_says_so() {
    let session = Session::new_model();
    let body = session.live_body();
    assert!(body.path.is_none(), "a new model has no path yet: {body:?}");
    assert!(body.run.is_none(), "a new model has no run: {body:?}");
    assert!(!body.dirty, "a new model has no changes: {body:?}");
}
