//! What a class does to the simulation.
//!
//! The catalogue says what a class **is**: its name, its ports, its
//! properties, the entity it declares or uses, and the XML of a block of
//! that class. That is enough to build a model, and an agent that has it
//! never guesses an element name.
//!
//! It is not enough to build a model that **behaves**: the catalogue does
//! not say whether the transaction waits, what it takes, what it gives
//! back, or where the class has a trap. An agent that does not know those
//! rules has to find them by experiment, one probe at a time, and the
//! probes are the expensive part of building a model.
//!
//! This module is the answer: the contract of every class, in four parts.
//!
//! - **flow**: what the class does to the flow of the transaction;
//! - **blocking**: what it blocks, and what it does not;
//! - **resources**: the rules of taking and releasing;
//! - **gotchas**: the traps, and the divergence from the original when
//!   there is one.
//!
//! The contract describes **what the engine does**, and the engine is
//! measured against the original: the oracle test compares its output with
//! the original's recorded answers, and the tests of the divergences pin
//! what is left out (`crates/genmsi-core/tests/diferencias_*.rs`). A
//! divergence is not a mistake to hide: it is a fact the reader has to
//! know, and it goes in the gotchas.

/// What a class does to the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Semantics {
    /// What the class does to the flow of the transaction: which port the
    /// transaction takes, which one it loses, which one it waits at.
    pub flow: &'static str,
    /// What the class blocks, and what it does not: whether the
    /// transaction waits, where it waits, and what lets it go.
    pub blocking: &'static str,
    /// The rules of taking and releasing: what the class takes, what it
    /// gives back, and what happens when the transaction ends.
    pub resources: &'static str,
    /// The traps, and the divergence from the original when there is one.
    ///
    /// Empty when the class has no trap worth the name.
    pub gotchas: &'static str,
}

impl Semantics {
    /// The four parts of a contract, in the order they are read.
    pub fn parts(&self) -> [&'static str; 4] {
        [self.flow, self.blocking, self.resources, self.gotchas]
    }
}
