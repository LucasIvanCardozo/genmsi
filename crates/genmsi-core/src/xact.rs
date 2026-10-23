//! Transactions (GPSS "XACTs") and their state machine.
//!
//! A transaction is always in exactly one state; the engine only moves it
//! through legal transitions, which is what the `proptest` invariants check.

use std::collections::BTreeMap;

/// Identifier of a transaction inside one run.
pub type XactId = u64;

/// Where a transaction currently is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XactState {
    /// Sitting on the current events chain, waiting to execute.
    Ready,
    /// Scheduled on the future events chain (an ADVANCE, a generator tick).
    Scheduled,
    /// Blocked in an entity wait list (facility, storage, chain, gate, ...).
    Waiting,
    /// Removed from the model by TERMINATE.
    Terminated,
}

impl XactState {
    /// Human-readable name used in traces.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Scheduled => "scheduled",
            Self::Waiting => "waiting",
            Self::Terminated => "terminated",
        }
    }
}

/// One simulated transaction.
#[derive(Debug, Clone)]
pub struct Transaction {
    /// Identity, used in reports and traces.
    pub id: XactId,
    /// Current priority; lower runs first.
    pub priority: i64,
    /// Creation time.
    pub created: f64,
    /// Time the transaction was marked (MARK block or creation).
    pub mark: f64,
    /// Named parameters (`P$...`), created on first assignment.
    pub parameters: BTreeMap<String, f64>,
    /// Index of the block the transaction is executing or waiting in.
    pub current_block: Option<usize>,
    /// Index of the block the transaction came from.
    ///
    /// The original counts a transaction that waits for a storage in the
    /// block it came from, not in the `ENTER` that could not admit it: the
    /// `ENTER` row of its report counts the entries that got in and its
    /// `CURRENT` column stays empty.
    pub previous_block: Option<usize>,
    /// Assembly set this transaction belongs to, when it was split.
    pub assembly: Option<u64>,
    /// Chain the transaction is linked into, when waiting in a user chain.
    pub chain: Option<String>,
    /// Current state.
    pub state: XactState,
    /// Arrival time at the current block (used for waiting statistics).
    pub entered_at: f64,
    /// Time of the pending resume event, when the transaction is delayed.
    pub scheduled_at: Option<f64>,
}

impl Transaction {
    /// Create a transaction ready to run.
    pub fn new(id: XactId, priority: i64, now: f64) -> Self {
        Self {
            id,
            priority,
            created: now,
            mark: now,
            parameters: BTreeMap::new(),
            current_block: None,
            previous_block: None,
            assembly: None,
            chain: None,
            state: XactState::Ready,
            entered_at: now,
            scheduled_at: None,
        }
    }

    /// Read a parameter (missing parameters are zero, as in GPSS).
    pub fn parameter(&self, name: &str) -> f64 {
        *self.parameters.get(name).unwrap_or(&0.0)
    }

    /// Write a parameter.
    pub fn set_parameter(&mut self, name: &str, value: f64) {
        self.parameters.insert(name.to_string(), value);
    }

    /// Move to a new state.
    pub fn transition(&mut self, next: XactState) -> bool {
        let legal = matches!(
            (self.state, next),
            (XactState::Ready, XactState::Scheduled)
                | (XactState::Ready, XactState::Waiting)
                | (XactState::Ready, XactState::Terminated)
                | (XactState::Scheduled, XactState::Ready)
                | (XactState::Scheduled, XactState::Waiting)
                | (XactState::Scheduled, XactState::Terminated)
                | (XactState::Waiting, XactState::Ready)
                | (XactState::Waiting, XactState::Terminated)
                | (XactState::Ready, XactState::Ready)
                | (XactState::Scheduled, XactState::Scheduled)
                | (XactState::Waiting, XactState::Waiting)
                | (XactState::Terminated, XactState::Terminated)
        );
        if legal {
            self.state = next;
        }
        legal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameters_default_to_zero() {
        let mut xact = Transaction::new(1, 0, 0.0);
        assert_eq!(xact.parameter("Media"), 0.0);
        xact.set_parameter("Media", 12.5);
        assert_eq!(xact.parameter("Media"), 12.5);
    }

    #[test]
    fn illegal_transitions_are_rejected() {
        let mut xact = Transaction::new(1, 0, 0.0);
        assert!(xact.transition(XactState::Terminated));
        assert!(!xact.transition(XactState::Ready));
        assert_eq!(xact.state, XactState::Terminated);
    }

    #[test]
    fn scheduled_can_only_resume_or_wait_or_terminate() {
        let mut xact = Transaction::new(2, 0, 0.0);
        assert!(xact.transition(XactState::Scheduled));
        assert!(xact.transition(XactState::Ready));
        assert!(xact.transition(XactState::Waiting));
        assert!(xact.transition(XactState::Ready));
    }
}
