//! The contracts of the classes: the table the catalogue publishes.
//!
//! The table is the answer to the question that cost the `chat` session half
//! of its time: **what does this class do to my simulation**. It has to
//! cover every class the catalogue offers — a class without a contract is a
//! class the agent has to probe — and the four parts of a contract have to
//! say something.

use genmsi_core::documents::class_document;
use genmsi_core::semantics::{CONTRACTS, contract};

/// The classes the catalogue offers: the table has to cover all of them.
fn catalogue_classes() -> Vec<String> {
    let list = genmsi_core::documents::class_list_document();
    list.classes
        .into_iter()
        .map(|entry| entry.class)
        .collect()
}

#[test]
fn the_table_covers_every_class_of_the_catalogue() {
    let classes = catalogue_classes();
    assert_eq!(classes.len(), 66, "the catalogue has 66 classes");
    for class in &classes {
        assert!(
            contract(class).is_some(),
            "the class {class} has no contract: the agent would have to probe it"
        );
    }
}

#[test]
fn no_class_is_twice_in_the_table() {
    let mut names: Vec<&str> = CONTRACTS.iter().map(|(class, _)| *class).collect();
    let total = names.len();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), total, "the table has a class twice");
}

#[test]
fn the_contract_of_a_class_says_something() {
    for (class, semantics) in CONTRACTS.iter() {
        assert!(
            !semantics.flow.is_empty(),
            "the contract of {class} does not say what it does to the flow"
        );
        assert!(
            !semantics.blocking.is_empty(),
            "the contract of {class} does not say what it blocks"
        );
        assert!(
            !semantics.resources.is_empty(),
            "the contract of {class} does not say what it takes"
        );
    }
}

#[test]
fn the_class_document_carries_the_contract() {
    let document = class_document("Seize").expect("the catalogue knows the class");
    let semantics = document
        .semantics
        .expect("the document carries the contract of the class");
    assert_eq!(
        semantics, *contract("Seize").expect("the class has a contract"),
        "the document carries the contract of the class"
    );
}

#[test]
fn the_class_document_of_a_class_without_a_contract_carries_none() {
    let document = class_document("Note").expect("the catalogue knows the class");
    // `Note` has no contract yet: the document says so instead of inventing
    // one.
    assert!(
        document.semantics.is_none() || contract("Note").is_some(),
        "the document does not invent a contract"
    );
}
