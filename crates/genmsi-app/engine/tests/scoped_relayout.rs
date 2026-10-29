//! The recomposition of one scope: the request carries it and the session
//! composes it.
//!
//! The reader sees one level of the model at a time —the root, or the
//! content of a container— so the request says which level it is talking
//! about and the session composes that level and nothing else. Four rules
//! are pinned here:
//!
//! - the request carries the scope, and a request without one does not
//!   parse;
//! - the composition of a scope leaves every block of any other scope where
//!   it was;
//! - the composition of a scope is **one** entry of the undo history;
//! - a scope with nothing to compose changes nothing at all.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use genmsi_app_engine::protocol::SessionRequest;
use genmsi_app_engine::{Session, answer};
use genmsi_format::{GmdlModel, Layout};

/// The reference models of the repository.
fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/models")
}

/// The model with the containers, used by the scope tests.
///
/// Its container 1 ("Generación de pescadores") holds nine blocks, and the
/// root holds seven containers and four activity blocks: a scope that is
/// composed says what happens to the blocks of the other scopes.
fn reference() -> PathBuf {
    fixtures().join("TP10__tp10.gmdl")
}

/// The container the scope tests compose.
const SCOPE: u32 = 1;

/// The geometry of every block of a model, by id.
fn geometry(model: &GmdlModel) -> HashMap<u32, (i32, i32, i32, i32)> {
    model
        .blocks
        .iter()
        .map(|block| {
            let layout = block.layout.unwrap_or(Layout {
                width: 0,
                height: 0,
                x: 0,
                y: 0,
            });
            (block.id, (layout.x, layout.y, layout.width, layout.height))
        })
        .collect()
}

/// A short mark of a text, so a failure does not print the whole model.
fn fingerprint(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// A session with the reference model open.
fn session() -> Session {
    let path = reference();
    Session::open(&path, &path.display().to_string()).expect("the model opens")
}

#[test]
fn the_relayout_request_carries_the_scope() {
    let request: SessionRequest =
        serde_json::from_str(r#"{"op":"relayout","scope":7}"#).expect("the request parses");
    match request {
        SessionRequest::Relayout { scope } => assert_eq!(scope, 7, "the scope did not travel"),
        other => panic!("the request is not a relayout: {other:?}"),
    }

    // The scope is not optional: a request that does not say which scope it
    // is talking about is refused before it reaches the session.
    assert!(
        serde_json::from_str::<SessionRequest>(r#"{"op":"relayout"}"#).is_err(),
        "a relayout without a scope was accepted"
    );
}

#[test]
fn the_request_reaches_the_session_with_its_scope() {
    let path = reference();
    let mut session: Option<Session> = None;
    let opened = answer(
        &mut session,
        SessionRequest::Open {
            path: path.display().to_string(),
        },
    );
    assert!(opened.ok, "the model did not open: {:?}", opened.error);
    let before = geometry(session.as_ref().expect("the session is open").model());

    let response = answer(&mut session, SessionRequest::Relayout { scope: SCOPE });
    assert!(response.ok, "the scope did not compose: {:?}", response.error);
    let after = geometry(session.as_ref().expect("the session is open").model());

    // The blocks of the other scopes did not move: the request composed the
    // scope it named and nothing else.
    for block in session.as_ref().expect("the session is open").model().blocks.iter() {
        if block.owner == SCOPE || block.id == SCOPE {
            continue;
        }
        assert_eq!(
            after.get(&block.id),
            before.get(&block.id),
            "the block {} of another scope moved",
            block.id
        );
    }
}

#[test]
fn composing_a_scope_leaves_the_blocks_outside_it_where_they_were() {
    let mut session = session();
    let before = geometry(session.model());
    session.relayout(SCOPE).expect("the scope composes");
    let after = geometry(session.model());

    // Every block of another scope kept its place: the composition of a
    // scope never reaches outside of it.
    for block in session.model().blocks.iter() {
        if block.owner == SCOPE || block.id == SCOPE {
            continue;
        }
        assert_eq!(
            after.get(&block.id),
            before.get(&block.id),
            "the block {} of another scope moved",
            block.id
        );
    }

    // The composition of the scope is deterministic: the same model and the
    // same scope give the same picture, every time. The model is the one
    // the composition starts from, so it is the one the undo brings back.
    let composed = fingerprint(&genmsi_format::to_string(session.model()));
    session.undo().expect("undoes");
    session.relayout(SCOPE).expect("the scope composes again");
    assert_eq!(
        fingerprint(&genmsi_format::to_string(session.model())),
        composed,
        "the composition of the scope is not deterministic"
    );
}

#[test]
fn composing_a_scope_is_one_undo_entry() {
    let mut session = session();
    let before = fingerprint(&genmsi_format::to_string(session.model()));
    assert!(
        !session.state().can_undo,
        "the session starts with something to undo"
    );

    session.relayout(SCOPE).expect("the scope composes");
    assert!(
        session.state().can_undo,
        "the composition of a scope is not an undo entry"
    );

    // One undo brings the whole model back: the composition of a scope is
    // one operation, like every other edit.
    session.undo().expect("undoes");
    assert_eq!(
        fingerprint(&genmsi_format::to_string(session.model())),
        before,
        "one undo did not bring the model back"
    );
    assert!(
        !session.state().can_undo,
        "the composition of a scope took more than one undo entry"
    );
}

#[test]
fn a_scope_with_nothing_to_compose_changes_nothing() {
    // A model with nothing in it: the root scope has no blocks of its own.
    let mut session = Session::new_model();
    let before = fingerprint(&genmsi_format::to_string(session.model()));

    session.relayout(0).expect("the root scope composes");

    assert_eq!(
        fingerprint(&genmsi_format::to_string(session.model())),
        before,
        "the composition of an empty scope changed the model"
    );
    assert!(
        !session.state().can_undo,
        "the composition of an empty scope is an undo entry"
    );
}
