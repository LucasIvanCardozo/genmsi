//! The contracts of the classes: what each one does to the simulation.
//!
//! The families are split by what the classes have in common, so each one
//! can be read next to the others: the flow, the resources, the groups and
//! the chains, the control, the selection and the data, and the functions
//! and the variables.
//!
//! The module joins them into one table, and the table is what the
//! catalogue publishes: `genmsi.class/1` carries the contract of the class
//! it describes.

pub mod contract;
pub mod f1_flow;
pub mod f2_resources;
pub mod f3_groups;
pub mod f4_control;
pub mod f5_data;
pub mod f6_functions;

pub use contract::Semantics;

/// Every contract, in the order of the catalogue.
///
/// The families are joined here: the table is the one the catalogue reads,
/// and it is built from the six families so that each of them can be
/// written (and read) on its own.
pub static CONTRACTS: std::sync::LazyLock<Vec<(&'static str, Semantics)>> =
    std::sync::LazyLock::new(|| {
        [
            // The flow.
            f1_flow::CONTRACTS,
            // The resources.
            f2_resources::CONTRACTS,
            // The groups and the chains.
            f3_groups::CONTRACTS,
            // The control.
            f4_control::CONTRACTS,
            // The selection and the data.
            f5_data::CONTRACTS,
            // The functions and the variables.
            f6_functions::CONTRACTS,
        ]
        .concat()
    });

/// The contract of a class, when it has one.
pub fn contract(class: &str) -> Option<&'static Semantics> {
    CONTRACTS
        .iter()
        .find(|(name, _)| *name == class)
        .map(|(_, semantics)| semantics)
}
