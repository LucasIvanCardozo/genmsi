//! One run, two documents.
//!
//! The app's «Ejecutar» used to make **two** runs of the engine: one for the
//! report and one for the trace. Now it makes one, with the trace on, and
//! the two documents come from it.
//!
//! The test proves both halves of the rule: the documents are the ones the
//! two separate runs gave, and the engine ran **once**.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

use genmsi_app_engine::{
    Session, SessionRequest, answer, run_documents_bytes, runs_made, simulate_document_bytes,
    trace_document_bytes,
};

/// The reference models of the repository.
fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/models")
}

/// The reference model of the app's slice.
fn reference() -> PathBuf {
    fixtures().join("TP6__Clase 6_1_PuertaBanco.gmdl")
}

/// The counter of the runs belongs to the whole process, so the tests that
/// read it take this lock: they never read each other's runs.
fn lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[test]
fn the_two_documents_of_one_run_are_the_two_runs_documents() {
    let _guard = lock();
    let path = reference();
    let seed = 7;
    let limit = 500;

    // One run, not two.
    let before = runs_made();
    let documents = run_documents_bytes(&path, seed, None, 1, limit, None).expect("runs");
    let after = runs_made();
    assert_eq!(
        after - before,
        1,
        "the engine ran {} times, not once",
        after - before
    );

    // The same documents the two separate runs used to give.
    let report = simulate_document_bytes(&path, seed, None, 1, None).expect("runs");
    let trace = trace_document_bytes(&path, seed, None, 1, limit, None).expect("runs");
    assert_eq!(documents.report, report, "the report changed");
    assert_eq!(documents.trace, trace, "the trace changed");

    // The two separate runs are two runs: that is what the app used to do.
    let before_two = runs_made();
    let _ = simulate_document_bytes(&path, seed, None, 1, None).expect("runs");
    let _ = trace_document_bytes(&path, seed, None, 1, limit, None).expect("runs");
    assert_eq!(
        runs_made() - before_two,
        2,
        "the two separate runs are not two runs"
    );

    // The trace keeps the bound it was asked for.
    let document: serde_json::Value = serde_json::from_str(&documents.trace).expect("parses");
    assert_eq!(document["schema"], "genmsi.trace/1");
    assert_eq!(document["limit"], limit);
}

#[test]
fn the_session_protocol_carries_the_two_documents_of_one_run() {
    let _guard = lock();
    let path = reference();
    let mut session = Some(Session::open(&path, &path.display().to_string()).expect("opens"));

    let before = runs_made();
    let answer = answer(
        &mut session,
        SessionRequest::RunCurrent {
            seed: Some(3),
            until: None,
            limit: Some(200),
            start: None,
        },
    );
    let after = runs_made();
    assert!(answer.ok, "the request failed: {:?}", answer.error);
    assert_eq!(
        after - before,
        1,
        "the engine ran {} times, not once",
        after - before
    );

    let documents = answer.documents.expect("the answer carries the documents");
    assert!(answer.document.is_none(), "the answer carries a single document");

    let report: serde_json::Value = serde_json::from_str(&documents.report).expect("parses");
    let trace: serde_json::Value = serde_json::from_str(&documents.trace).expect("parses");
    assert_eq!(report["schema"], "genmsi.report/1");
    assert_eq!(report["seed"], 3);
    assert_eq!(trace["schema"], "genmsi.trace/1");
    assert_eq!(trace["seed"], 3);
    assert_eq!(trace["limit"], 200);
}
