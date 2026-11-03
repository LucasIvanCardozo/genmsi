//! The proposals an agent leaves for the person.
//!
//! The desktop app and the MCP already talk in one direction: the app
//! publishes what it is showing ([`crate::live`]) and the verb refuses to
//! write over a model the reader has open with changes that are not saved.
//! The other direction was thin: the only way for an agent to show its work
//! to the person was to **overwrite the file** — a road that closes exactly
//! when the person is working.
//!
//! A proposal is the other road. The agent writes a model in the staging
//! directory of the app and describes it in the `genmsi.proposal/1`
//! document; the app picks it up, loads it **in memory** (dirty and
//! undoable, so the person sees it on the canvas without anything being
//! written), and the person decides: accept it or discard it. The verdict
//! travels back to the agent in the live document.
//!
//! The format lives here, in the engine, for the same reason the format of
//! the open model and of the live state live here: two clients — the MCP and
//! the app — must not have two dialects of the same file.
//!
//! - the MCP **creates** a proposal (state `pending`);
//! - the app **resolves** it (`accepted` or `rejected`).
//!
//! Neither rewrites the whole document on its own: each one writes the
//! fields it owns, through the functions of this module.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::documents::to_pretty_json;
use crate::live::same_file;
use crate::app_data::{AppDataError, app_data_dir};

/// The schema of the proposal document.
pub const PROPOSAL_SCHEMA: &str = "genmsi.proposal/1";

/// The name of the directory the proposals live in, inside the data
/// directory of the app.
pub const STAGING_DIR: &str = "staging";

/// The extension of the model a proposal carries.
pub const MODEL_EXTENSION: &str = "gmdl";

/// The extension of the document that describes a proposal.
pub const DOCUMENT_EXTENSION: &str = "json";

/// What happened to a proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProposalState {
    /// The proposal is waiting: the person has not decided yet.
    #[default]
    Pending,
    /// The person accepted it: the model reached the disk.
    Accepted,
    /// The person discarded it: the model did not reach the disk.
    Rejected,
}

impl ProposalState {
    /// The state as the document spells it.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProposalState::Pending => "pending",
            ProposalState::Accepted => "accepted",
            ProposalState::Rejected => "rejected",
        }
    }

    /// Whether the proposal is still waiting for the person.
    pub fn is_pending(&self) -> bool {
        matches!(self, ProposalState::Pending)
    }
}

/// A proposal: a model an agent offers to the person.
///
/// This is the body of the `genmsi.proposal/1` document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalDocument {
    /// The identity of the proposal: it names its own files.
    pub id: String,
    /// The absolute path of the model the proposal carries.
    pub model: String,
    /// The model the proposal proposes to change: the one the app has open.
    #[serde(default)]
    pub target: Option<String>,
    /// What the agent says it did, in one line.
    #[serde(default)]
    pub summary: Option<String>,
    /// What happened to the proposal.
    #[serde(default)]
    pub state: ProposalState,
    /// What the person answered when they discarded the proposal.
    #[serde(default)]
    pub note: Option<String>,
}

impl ProposalDocument {
    /// A proposal that is waiting for the person.
    pub fn pending(id: &str, model: &Path, target: Option<&Path>) -> Self {
        Self {
            id: id.to_string(),
            model: model.display().to_string(),
            target: target.map(|path| path.display().to_string()),
            summary: None,
            state: ProposalState::Pending,
            note: None,
        }
    }

    /// The proposal with a summary.
    pub fn with_summary(mut self, summary: Option<&str>) -> Self {
        self.summary = summary.map(|text| text.to_string());
        self
    }

    /// The proposal resolved, with the note the person left.
    pub fn resolved(&self, state: ProposalState, note: Option<&str>) -> Self {
        let mut resolved = self.clone();
        resolved.state = state;
        resolved.note = note.map(|text| text.to_string());
        resolved
    }
}

/// The proposal as the live document carries it: what the person has to
/// decide, or what they decided.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalBody {
    /// The identity of the proposal.
    pub id: String,
    /// What happened to it.
    pub state: ProposalState,
    /// What the agent said it did.
    #[serde(default)]
    pub summary: Option<String>,
    /// What the person answered.
    #[serde(default)]
    pub note: Option<String>,
}

impl From<&ProposalDocument> for ProposalBody {
    fn from(document: &ProposalDocument) -> Self {
        Self {
            id: document.id.clone(),
            state: document.state,
            summary: document.summary.clone(),
            note: document.note.clone(),
        }
    }
}

/// The document as the text that travels.
pub fn proposal_document(document: &ProposalDocument) -> String {
    to_pretty_json(PROPOSAL_SCHEMA, document)
}

/// The directory the proposals live in, in the data directory of the app.
pub fn staging_dir() -> Option<PathBuf> {
    app_data_dir().map(|directory| directory.join(STAGING_DIR))
}

/// The directory the proposals live in, in a directory of the app.
pub fn staging_dir_in(directory: &Path) -> PathBuf {
    directory.join(STAGING_DIR)
}

/// The file that describes a proposal.
pub fn document_file_in(directory: &Path, id: &str) -> PathBuf {
    staging_dir_in(directory).join(format!("{id}.{DOCUMENT_EXTENSION}"))
}

/// The file that carries the model of a proposal.
pub fn model_file_in(directory: &Path, id: &str) -> PathBuf {
    staging_dir_in(directory).join(format!("{id}.{MODEL_EXTENSION}"))
}

/// An identity for a proposal.
///
/// The identity carries the time it was made, so the proposals of a
/// directory come out ordered by age: the person sees the oldest one first.
pub fn new_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_millis())
        .unwrap_or_default();
    format!("{millis}-{:x}", crate::random_seed())
}

/// Write the model of a proposal.
///
/// The write is atomic: the text goes to a temporary file next to the target
/// and that file is renamed over it, so the app never reads half of a model.
pub fn write_proposal_model(
    directory: &Path,
    id: &str,
    text: &str,
) -> Result<PathBuf, AppDataError> {
    let file = model_file_in(directory, id);
    write_atomically(&file, text)?;
    Ok(file)
}

/// Write the document of a proposal.
///
/// The write is atomic, for the same reason the write of the model is.
pub fn write_proposal(
    directory: &Path,
    document: &ProposalDocument,
) -> Result<PathBuf, AppDataError> {
    let file = document_file_in(directory, &document.id);
    write_atomically(&file, &proposal_document(document))?;
    Ok(file)
}

/// Read the document of a proposal.
///
/// Tolerant: a proposal that cannot be read is no proposal, not a failure.
pub fn read_proposal_in(directory: &Path, id: &str) -> Option<ProposalDocument> {
    let text = std::fs::read_to_string(document_file_in(directory, id)).ok()?;
    serde_json::from_str(&text).ok()
}

/// Read every proposal of a directory, oldest first.
///
/// The order is the order of the identities, which carry the time they were
/// made: the person sees the oldest proposal first.
pub fn read_proposals_in(directory: &Path) -> Vec<ProposalDocument> {
    let Ok(entries) = std::fs::read_dir(staging_dir_in(directory)) else {
        return Vec::new();
    };
    let mut documents: Vec<ProposalDocument> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == DOCUMENT_EXTENSION)
        })
        .filter_map(|entry| {
            let text = std::fs::read_to_string(entry.path()).ok()?;
            serde_json::from_str(&text).ok()
        })
        .collect();
    documents.sort_by(|left, right| left.id.cmp(&right.id));
    documents
}

/// The oldest proposal that is still waiting for the person.
pub fn oldest_pending_in(directory: &Path) -> Option<ProposalDocument> {
    read_proposals_in(directory)
        .into_iter()
        .find(|document| document.state.is_pending())
}

/// The oldest proposal that waits **for a model**.
///
/// A proposal names the model it is about, and it waits for that one: a
/// proposal about another file is not the business of the app that has this
/// one open, and it does not hold the queue in front of the proposals that
/// are about it.
///
/// A proposal that names no model waits for whatever the app has open, and
/// a proposal that names one waits for nothing when the app has no model
/// open: there is no model to propose a change to.
pub fn oldest_pending_for_in(directory: &Path, open: Option<&Path>) -> Option<ProposalDocument> {
    read_proposals_in(directory)
        .into_iter()
        .find(|document| document.state.is_pending() && is_about(document, open))
}

/// Whether a proposal is about a model.
fn is_about(document: &ProposalDocument, open: Option<&Path>) -> bool {
    let Some(target) = document.target.as_deref() else {
        // The proposal names no model: it is about whatever the app has.
        return true;
    };
    let Some(open) = open else {
        // It names a model and the app has none open: it waits for one.
        return false;
    };
    same_file(Path::new(target), open)
}

/// The document of a proposal, resolved: the person accepted it or discarded
/// it, and the note they left.
pub fn resolve_proposal_in(
    directory: &Path,
    id: &str,
    state: ProposalState,
    note: Option<&str>,
) -> Result<ProposalDocument, AppDataError> {
    let Some(document) = read_proposal_in(directory, id) else {
        return Err(AppDataError::Write {
            path: document_file_in(directory, id),
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "there is no proposal with that identity",
            ),
        });
    };
    let resolved = document.resolved(state, note);
    write_proposal(directory, &resolved)?;
    Ok(resolved)
}

/// Write a file atomically: a temporary file next to the target, renamed
/// over it.
fn write_atomically(file: &Path, text: &str) -> Result<(), AppDataError> {
    let directory = file.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(directory).map_err(|source| AppDataError::Write {
        path: directory.to_path_buf(),
        source,
    })?;
    let temporary = directory.join(format!(
        "{}.{}.tmp",
        file.file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string()),
        std::process::id()
    ));
    std::fs::write(&temporary, text).map_err(|source| AppDataError::Write {
        path: temporary.clone(),
        source,
    })?;
    std::fs::rename(&temporary, file).map_err(|source| {
        // The temporary file is not left behind: the staging directory does
        // not fill with the remains of a failed write.
        let _ = std::fs::remove_file(&temporary);
        AppDataError::Write {
            path: file.to_path_buf(),
            source,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A directory that cleans itself up.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "genmsi-proposal-{tag}-{}-{:?}",
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

    fn scratch() -> TempDir {
        TempDir::new("scratch")
    }

    #[test]
    fn a_proposal_round_trips_through_its_document() {
        let directory = scratch();
        let document = ProposalDocument::pending(
            "1700000000000-ab",
            Path::new("/tmp/proposed.gmdl"),
            Some(Path::new("/tmp/open.gmdl")),
        )
        .with_summary(Some("the bus loop"));
        let file = write_proposal(directory.path(), &document).expect("the proposal is written");
        assert!(file.ends_with("1700000000000-ab.json"), "{file:?}");
        let read = read_proposal_in(directory.path(), "1700000000000-ab").expect("the proposal");
        assert_eq!(read, document);
    }

    #[test]
    fn the_model_of_a_proposal_is_written_next_to_its_document() {
        let directory = scratch();
        let file = write_proposal_model(directory.path(), "abc", "<gmodel/>").expect("the model");
        assert!(file.ends_with("abc.gmdl"), "{file:?}");
        assert_eq!(
            std::fs::read_to_string(&file).expect("the model is there"),
            "<gmodel/>"
        );
    }

    #[test]
    fn the_proposals_come_out_ordered_by_age() {
        let directory = scratch();
        for id in ["200", "100", "300"] {
            let document = ProposalDocument::pending(id, Path::new("/tmp/a.gmdl"), None);
            write_proposal(directory.path(), &document).expect("the proposal is written");
        }
        let ids: Vec<String> = read_proposals_in(directory.path())
            .into_iter()
            .map(|document| document.id)
            .collect();
        assert_eq!(ids, ["100", "200", "300"]);
    }

    #[test]
    fn the_oldest_pending_is_the_one_that_waits() {
        let directory = scratch();
        for id in ["200", "100", "300"] {
            let document = ProposalDocument::pending(id, Path::new("/tmp/a.gmdl"), None);
            write_proposal(directory.path(), &document).expect("the proposal is written");
        }
        resolve_proposal_in(directory.path(), "100", ProposalState::Rejected, Some("no"))
            .expect("the proposal is resolved");
        let pending = oldest_pending_in(directory.path()).expect("a proposal waits");
        assert_eq!(pending.id, "200");
    }

    #[test]
    fn a_resolved_proposal_carries_the_verdict_and_the_note() {
        let directory = scratch();
        let document = ProposalDocument::pending("7", Path::new("/tmp/a.gmdl"), None);
        write_proposal(directory.path(), &document).expect("the proposal is written");
        let resolved = resolve_proposal_in(
            directory.path(),
            "7",
            ProposalState::Rejected,
            Some("el bus no hace dos vueltas"),
        )
        .expect("the proposal is resolved");
        assert_eq!(resolved.state, ProposalState::Rejected);
        assert_eq!(
            resolved.note.as_deref(),
            Some("el bus no hace dos vueltas")
        );
        let read = read_proposal_in(directory.path(), "7").expect("the proposal");
        assert_eq!(read, resolved);
    }

    #[test]
    fn a_proposal_that_is_not_there_is_no_proposal() {
        let directory = scratch();
        assert!(read_proposal_in(directory.path(), "nope").is_none());
        assert!(read_proposals_in(directory.path()).is_empty());
        assert!(oldest_pending_in(directory.path()).is_none());
    }

    #[test]
    fn the_document_of_a_proposal_is_versioned() {
        let document = ProposalDocument::pending("1", Path::new("/tmp/a.gmdl"), None);
        let text = proposal_document(&document);
        assert!(text.contains("genmsi.proposal/1"), "{text}");
        assert!(text.contains("\"state\": \"pending\""), "{text}");
    }

    #[test]
    fn an_identity_carries_the_time_it_was_made() {
        let first = new_id();
        let second = new_id();
        assert_ne!(first, second, "two proposals are two proposals");
        let (millis, suffix) = first.split_once('-').expect("the identity has two parts");
        assert!(millis.parse::<u128>().is_ok(), "{millis} is the time");
        assert!(!suffix.is_empty(), "{first} has a suffix");
        // Two identities made in the same millisecond differ: the suffix
        // tells them apart.
        let (other_millis, _) = second.split_once('-').expect("the identity has two parts");
        if other_millis == millis {
            assert_ne!(first, second, "{first} and {second} are different");
        }
    }
}
