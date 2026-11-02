//! The loop of the proposal: what an agent proposes and what the person
//! decides.
//!
//! The agent writes a model in the staging directory of the app and
//! describes it in the `genmsi.proposal/1` document. The app adopts that
//! model **in memory** — dirty and undoable, so the person sees it on the
//! canvas without anything being written — and the person decides: accept
//! it or discard it.
//!
//! The format lives in `genmsi-core`: this module is the flow, not the
//! format. The core's writer is the only one that writes the document of a
//! proposal, so the app and the MCP server cannot disagree about it (S5:
//! one owner of the format).
//!
//! The directory the proposals live in is given by the caller: the desktop
//! shell resolves it with the resolver of the platform, and the tests
//! resolve it with a directory of their own.

use std::path::Path;

use genmsi_core::proposal::{
    ProposalBody, ProposalDocument, ProposalState, oldest_pending_for_in, oldest_pending_in,
    proposal_document, resolve_proposal_in,
};

use crate::{AppError, SaveOutcome, Session, SessionState};

/// The proposal that waits for the person, if there is one.
///
/// The oldest one that is **about the model the app has open** waits: a
/// proposal about another file is not the business of this app, and it does
/// not hold the queue in front of the proposals that are about it.
pub fn pending_for(directory: &Path, open: Option<&Path>) -> Option<ProposalDocument> {
    oldest_pending_for_in(directory, open)
}

/// The document of the proposal that waits, as the text that travels.
///
/// `None` when there is no proposal waiting: there is nothing to decide.
pub fn pending_document_for(directory: &Path, open: Option<&Path>) -> Option<String> {
    pending_for(directory, open).map(|document| proposal_document(&document))
}

/// The proposal the session holds, when it is on the canvas.
///
/// A proposal that was undone is not on the canvas any more: there is
/// nothing to accept or to discard, and the person has to adopt it again.
fn the_proposal_on_the_canvas(session: &Session) -> Result<ProposalBody, AppError> {
    let Some(proposal) = session.proposal() else {
        return Err(AppError::Usage(
            "there is no proposal on the table".to_string(),
        ));
    };
    if !session.adopted() {
        return Err(AppError::Usage(
            "the proposal is not on the canvas: it has to be adopted before it is decided".to_string(),
        ));
    }
    Ok(proposal.clone())
}

/// Adopt the proposal that waits into the session.
///
/// The model of the proposal enters the session dirty and undoable, and the
/// file the app has open does not move: adopting is not writing.
pub fn adopt_in(session: &mut Session, directory: &Path) -> Result<SessionState, AppError> {
    match pending_for(directory, session.path()) {
        Some(document) => session.adopt_proposal(&document),
        None => Err(AppError::Usage(
            // The answer says why there is nothing to adopt: a proposal
            // about another model is waiting, and it is not this app's
            // business. It does not hold the queue in front of the
            // proposals that are about this model.
            if oldest_pending_in(directory).is_some() {
                "the proposal is about another model: it stays pending".to_string()
            } else {
                "there is no proposal waiting for the person".to_string()
            },
        )),
    }
}

/// Accept the proposal the session holds.
///
/// Accepting is the normal save of the app: the model in memory goes to the
/// file the app has open. The proposal becomes `accepted`.
pub fn accept_in(
    session: &mut Session,
    directory: &Path,
) -> Result<(SessionState, SaveOutcome), AppError> {
    let proposal = the_proposal_on_the_canvas(session)?;
    // The model reaches the disk first: the file is the thing the person
    // decided about, and a failure there must not leave the proposal
    // resolved without the model.
    let outcome = session.save(None)?;
    // The verdict is written by the core: one owner of the format.
    resolve_proposal_in(directory, &proposal.id, ProposalState::Accepted, None)
        .map_err(|error| AppError::Usage(error.to_string()))?;
    session.resolve_proposal(ProposalState::Accepted, None);
    Ok((session.state(), outcome))
}

/// Discard the proposal the session holds.
///
/// The model that was there comes back to the canvas, and the proposal
/// becomes `rejected`, with the note the person wrote.
pub fn discard_in(
    session: &mut Session,
    directory: &Path,
    note: Option<&str>,
) -> Result<SessionState, AppError> {
    let proposal = the_proposal_on_the_canvas(session)?;
    // The verdict goes to the disk before the canvas moves: if the write
    // fails, the person still sees the proposal and can try again.
    resolve_proposal_in(directory, &proposal.id, ProposalState::Rejected, note)
        .map_err(|error| AppError::Usage(error.to_string()))?;
    session.discard_proposal()?;
    session.resolve_proposal(ProposalState::Rejected, note);
    Ok(session.state())
}

#[cfg(test)]
mod tests {
    use super::*;
    use genmsi_core::proposal::{
        ProposalState, read_proposal_in, write_proposal, write_proposal_model,
    };
    use std::path::{Path, PathBuf};

    /// The reference model of the app's slice.
    fn reference() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../fixtures/models/TP6__Clase 6_1_PuertaBanco.gmdl")
    }

    /// A directory that cleans itself up.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "genmsi-app-proposal-{tag}-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("the temporary directory is created");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// The caption of the model that is on the canvas.
    fn caption(session: &Session) -> String {
        session.model().blocks[0].caption.clone()
    }

    /// The model the app has open: a copy of the reference, in a directory
    /// of its own, so the tests never write in the repository.
    fn open_model(directory: &Path) -> PathBuf {
        let text = std::fs::read_to_string(reference()).expect("the reference model is there");
        let path = directory.join("open.gmdl");
        std::fs::write(&path, text).expect("the model is written");
        path
    }

    /// A proposal of an agent: a model that is not the one on the disk.
    fn proposal(directory: &Path, open: &Path, id: &str) -> ProposalDocument {
        proposal_about(directory, open, id)
    }

    /// A proposal of an agent about the file named by `target`.
    ///
    /// The model it carries is the version of that file: the app adopts it
    /// only when it has that file open.
    fn proposal_about(directory: &Path, target: &Path, id: &str) -> ProposalDocument {
        let mut model = genmsi_format::load(reference()).expect("the reference model loads");
        model.blocks[0].caption = "Modelo propuesto".to_string();
        let text = genmsi_format::to_string(&model);
        let file = write_proposal_model(directory, id, &text).expect("the model of the proposal");
        let document = ProposalDocument::pending(id, &file, Some(target))
            .with_summary(Some("the bus loop"));
        write_proposal(directory, &document).expect("the proposal is written");
        document
    }

    #[test]
    fn the_live_body_carries_the_proposal_that_waits() {
        let directory = TempDir::new("live-pending");
        let open = open_model(directory.path());
        proposal(directory.path(), &open, "1700000000000-ab");
        let session = Session::open(&open, &open.display().to_string()).expect("the model opens");

        // The proposal has not been adopted yet: it waits for the person.
        // The live body says so, so an agent that left it finds out that it
        // is waiting instead of hearing nothing.
        let body = session.live_body_in(Some(directory.path()));
        let published = body.proposal.expect("the proposal travels");
        assert_eq!(published.id, "1700000000000-ab");
        assert_eq!(published.state, ProposalState::Pending);
        assert_eq!(published.summary.as_deref(), Some("the bus loop"));

        // A proposal about another model is not this app's business: it is
        // not published either.
        let other = directory.path().join("other.gmdl");
        std::fs::write(&other, std::fs::read_to_string(reference()).expect("the model is there"))
            .expect("the model is written");
        proposal_about(directory.path(), &other, "200");
        let body = session.live_body_in(Some(directory.path()));
        let published = body.proposal.expect("the proposal travels");
        assert_eq!(
            published.id, "1700000000000-ab",
            "the proposal about the open model is the one that travels"
        );
    }

    #[test]
    fn accepting_writes_the_file_and_resolves_the_document() {
        let directory = TempDir::new("accept");
        let open = open_model(directory.path());
        let document = proposal(directory.path(), &open, "1700000000000-ab");
        let mut session = Session::open(&open, &open.display().to_string()).expect("the model opens");
        adopt_in(&mut session, directory.path()).expect("the proposal is adopted");

        let (state, outcome) = accept_in(&mut session, directory.path()).expect("the proposal is accepted");
        assert_eq!(state.path.as_deref(), Some(open.display().to_string().as_str()));
        assert_eq!(outcome.path, open.display().to_string());
        // The file the app has open holds the model of the proposal.
        let written = std::fs::read_to_string(&open).expect("the file is there");
        assert!(
            written.contains("Modelo propuesto"),
            "the file does not hold the model of the proposal: {written}"
        );
        // The document of the proposal is accepted.
        let resolved = read_proposal_in(directory.path(), "1700000000000-ab").expect("the proposal");
        assert_eq!(resolved.state, ProposalState::Accepted);
        assert_eq!(resolved.id, document.id);
        // The session is clean: the model reached the disk.
        assert!(!state.dirty, "the session is still dirty: {state:?}");
    }

    #[test]
    fn discarding_restores_the_model_and_resolves_the_document_with_the_note() {
        let directory = TempDir::new("discard");
        let open = open_model(directory.path());
        proposal(directory.path(), &open, "1700000000000-ab");
        let mut session = Session::open(&open, &open.display().to_string()).expect("the model opens");
        adopt_in(&mut session, directory.path()).expect("the proposal is adopted");
        assert_eq!(caption(&session), "Modelo propuesto");

        let state = discard_in(&mut session, directory.path(), Some("the bus is not right"))
            .expect("the proposal is discarded");
        // The model that was there is on the canvas again.
        assert_eq!(caption(&session), "Modelo");
        assert!(!state.dirty, "the session is still dirty: {state:?}");
        // The document of the proposal is rejected, with the note.
        let resolved = read_proposal_in(directory.path(), "1700000000000-ab").expect("the proposal");
        assert_eq!(resolved.state, ProposalState::Rejected);
        assert_eq!(resolved.note.as_deref(), Some("the bus is not right"));
    }

    #[test]
    fn the_oldest_pending_proposal_is_the_one_that_waits() {
        let directory = TempDir::new("pending");
        let open = open_model(directory.path());
        proposal(directory.path(), &open, "200");
        proposal(directory.path(), &open, "100");

        let document = pending_for(directory.path(), Some(&open)).expect("a proposal waits");
        assert_eq!(document.id, "100", "the newest proposal waits");
        let text = pending_document_for(directory.path(), Some(&open)).expect("the document travels");
        let parsed: serde_json::Value = serde_json::from_str(&text).expect("the document is JSON");
        assert_eq!(parsed["schema"], "genmsi.proposal/1");
        assert_eq!(parsed["id"], "100");
        assert_eq!(parsed["state"], "pending");

        // A resolved proposal does not wait any more.
        resolve_proposal_in(directory.path(), "100", ProposalState::Accepted, None)
            .expect("the proposal is resolved");
        let document = pending_for(directory.path(), Some(&open)).expect("a proposal waits");
        assert_eq!(document.id, "200");
    }

    #[test]
    fn a_proposal_about_another_model_stays_pending() {
        let directory = TempDir::new("other-target");
        let open = open_model(directory.path());
        // The proposal is about another file: the app does not have it
        // open.
        let other = directory.path().join("other.gmdl");
        std::fs::write(&other, std::fs::read_to_string(reference()).expect("the model is there"))
            .expect("the model is written");
        proposal_about(directory.path(), &other, "1700000000000-ab");
        let mut session = Session::open(&open, &open.display().to_string()).expect("the model opens");

        let error = adopt_in(&mut session, directory.path()).expect_err("the adoption is refused");
        assert!(
            error.to_string().contains("about another model"),
            "the message does not say why: {error}"
        );
        // The proposal stays pending: nobody touched it.
        let read = read_proposal_in(directory.path(), "1700000000000-ab").expect("the proposal");
        assert_eq!(read.state, ProposalState::Pending);
        // The canvas did not move.
        assert_eq!(caption(&session), "Modelo");
    }

    #[test]
    fn a_proposal_about_another_model_does_not_hold_the_queue() {
        let directory = TempDir::new("queue");
        let open = open_model(directory.path());
        // The oldest proposal is about another file: it is not this app's
        // business, and it must not stand in front of the one that is.
        let other = directory.path().join("other.gmdl");
        std::fs::write(&other, std::fs::read_to_string(reference()).expect("the model is there"))
            .expect("the model is written");
        proposal_about(directory.path(), &other, "100");
        proposal(directory.path(), &open, "200");

        let mut session = Session::open(&open, &open.display().to_string()).expect("the model opens");
        adopt_in(&mut session, directory.path()).expect("the proposal is adopted");

        // The proposal about the open model is the one that was adopted.
        let proposal = session.proposal().expect("the session holds a proposal");
        assert_eq!(proposal.id, "200", "the queue is not held");
        // The other one still waits, for the app that has that file open.
        let read = read_proposal_in(directory.path(), "100").expect("the proposal");
        assert_eq!(read.state, ProposalState::Pending);
    }

    #[test]
    fn there_is_no_proposal_to_adopt_when_the_directory_is_empty() {
        let directory = TempDir::new("empty");
        let open = open_model(directory.path());
        let mut session = Session::open(&open, &open.display().to_string()).expect("the model opens");
        let error = adopt_in(&mut session, directory.path()).expect_err("there is no proposal");
        assert!(
            error.to_string().contains("no proposal waiting"),
            "the message does not say it: {error}"
        );
        assert!(pending_document_for(directory.path(), Some(&open)).is_none());
    }

    #[test]
    fn accepting_or_discarding_without_a_proposal_is_refused() {
        let directory = TempDir::new("no-proposal");
        let open = open_model(directory.path());
        let mut session = Session::open(&open, &open.display().to_string()).expect("the model opens");
        let error = accept_in(&mut session, directory.path()).expect_err("there is no proposal");
        assert!(
            error.to_string().contains("no proposal on the table"),
            "the message does not say it: {error}"
        );
        let error = discard_in(&mut session, directory.path(), None).expect_err("there is no proposal");
        assert!(
            error.to_string().contains("no proposal on the table"),
            "the message does not say it: {error}"
        );
    }
}
