//! The entities a model uses and the ones it declares.
//!
//! A GPSS entity has to exist before a block uses it: a storage needs a
//! capacity, a table needs its frequencies, a function needs its
//! distribution. The original editor says so in the description of the
//! classes that use an entity (the `StorageUse` class says "El Storage debe
//! estar declarado (esta clase no lo declara)") and its entity picker only
//! offers the entities the model declares.
//!
//! This module is the format's answer to that rule: it reads the model and
//! says which entity a block uses that nothing declares. The finding is a
//! warning, not a load error: the original opens a model that uses an
//! undeclared entity (one of the reference models does, with a typo in the
//! name of a storage), so the reader keeps opening it; the model is simply
//! not silent about it any more.
//!
//! The check reads the model only: a property of type `EntityDef` declares
//! the entity it names and a property of type `EntityUse` uses one. Nothing
//! here knows about the engine, and nothing here changes how the engine
//! behaves.

use std::collections::{BTreeMap, BTreeSet};

use crate::error::FormatError;
use crate::model::GmdlModel;

/// When an entity kind is checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Policy {
    /// The kind must be declared before it is used: an undeclared storage has
    /// no capacity and an undeclared table has no frequencies, so the model
    /// cannot mean what it says.
    Required,
    /// The kind is created by the block that uses it (GPSS creates a save
    /// value, a logic switch or a queue on demand), so the check only fires
    /// when the model declares this kind somewhere else and this name is not
    /// among the declared ones.
    WhenDeclared,
}

/// One kind of entity the check knows.
struct EntityKind {
    /// The kind as the file names it (`Storage`, `SaveValue`, ...).
    name: &'static str,
    /// When the kind is checked.
    policy: Policy,
    /// The classes that declare this kind.
    declarers: &'static [&'static str],
}

/// The kinds the check knows about.
///
/// The kinds left out are the ones GPSS creates by use: a queue, a facility,
/// a chain, a transaction group, a numeric group, a parameter and a random
/// seed. The original's own models rely on that (23 of the 25 reference
/// models use a queue that nothing declares), so flagging them would bury
/// the real finding. A kind is one line away from being checked, should the
/// project decide that a queue must be declared too.
const KINDS: &[EntityKind] = &[
    EntityKind {
        name: "Storage",
        policy: Policy::Required,
        declarers: &["StorageDef"],
    },
    EntityKind {
        name: "Table",
        policy: Policy::Required,
        declarers: &["Table"],
    },
    EntityKind {
        name: "Function",
        policy: Policy::Required,
        declarers: &[
            "Function",
            "FunctionDist",
            "FunctionExpon",
            "Multifurcacion",
            "MultifurcacionP",
        ],
    },
    EntityKind {
        name: "SaveValue",
        policy: Policy::WhenDeclared,
        declarers: &["InitiaSV"],
    },
    EntityKind {
        name: "LogicSwich",
        policy: Policy::WhenDeclared,
        declarers: &["InitialLS"],
    },
    EntityKind {
        name: "Label",
        policy: Policy::WhenDeclared,
        declarers: &["Match"],
    },
    EntityKind {
        name: "QTable",
        policy: Policy::WhenDeclared,
        declarers: &["QTable"],
    },
    EntityKind {
        name: "Variable",
        policy: Policy::WhenDeclared,
        declarers: &["Variable", "VariableA", "VariableN"],
    },
];

/// One entity a block uses that no block of the model declares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndeclaredEntity {
    /// Id of the block that uses the entity.
    pub block: u32,
    /// Class of the block that uses the entity (`StorageUse`, ...).
    pub class: String,
    /// Property that carries the name (`Storage`, ...).
    pub property: String,
    /// Kind of the entity (`Storage`, ...).
    pub kind: String,
    /// The name the block uses.
    pub name: String,
    /// The classes that declare this kind of entity.
    pub declarers: &'static [&'static str],
    /// Names of this kind the model declares, in order.
    pub declared: Vec<String>,
}

impl UndeclaredEntity {
    /// What the model has to hear about this entity.
    ///
    /// The message says what the model uses, what it declares and how to
    /// fix the gap: a warning that only says "missing" leaves the reader
    /// guessing.
    pub fn message(&self) -> String {
        let declarers = list_of(self.declarers);
        let declared = if self.declared.is_empty() {
            format!("the model declares no {} at all", self.kind)
        } else {
            format!("the model declares: {}", self.declared.join(", "))
        };
        format!(
            "the model uses the {} '{}' in the {} block {}, and no {} block declares '{}' \
             ({}). Declare '{}' with a {} block, or use one of the declared names.",
            self.kind,
            self.name,
            self.class,
            self.block,
            declarers,
            self.name,
            declared,
            self.name,
            declarers
        )
    }
}

/// The entities the model uses that it does not declare.
///
/// The list is empty when every entity a block uses is declared somewhere in
/// the model.
pub fn undeclared_entities(model: &GmdlModel) -> Vec<UndeclaredEntity> {
    let declared = declarations(model);
    let mut found = Vec::new();
    for block in &model.blocks {
        for prop in &block.props {
            if !prop.ty.eq_ignore_ascii_case("EntityUse") {
                continue;
            }
            let Some(kind) = kind_of(&prop.subtype) else {
                continue;
            };
            let value = prop.value();
            let value = value.trim();
            // A value that is not a plain name is an expression (`*Cuadra`,
            // `P:Param`) or a placeholder (`[storage]`): the model computes
            // the name at run time, and no declaration can be checked against
            // it.
            if !is_literal_name(value) {
                continue;
            }
            let names = declared.get(kind.name);
            let missing = match kind.policy {
                Policy::Required => !names.is_some_and(|set| set.contains(value)),
                Policy::WhenDeclared => names.is_some_and(|set| !set.contains(value)),
            };
            if !missing {
                continue;
            }
            found.push(UndeclaredEntity {
                block: block.id,
                class: block.class.clone(),
                property: prop.name.clone(),
                kind: kind.name.to_string(),
                name: value.to_string(),
                declarers: kind.declarers,
                declared: names.map(|set| set.iter().cloned().collect()).unwrap_or_default(),
            });
        }
    }
    found
}

/// The strict form of the check: the first entity the model uses without
/// declaring it is an error.
///
/// The reader does not call it: a model that uses an undeclared entity is a
/// model the original opens, so it is a warning there. A client that wants
/// to refuse such a model (a validator, a batch tool) can.
pub fn validate_declarations(model: &GmdlModel) -> Result<(), FormatError> {
    match undeclared_entities(model).into_iter().next() {
        Some(entity) => Err(FormatError::structural(entity.block, entity.message())),
        None => Ok(()),
    }
}

/// The messages of the entities the model uses without declaring.
pub fn declaration_warnings(model: &GmdlModel) -> Vec<String> {
    undeclared_entities(model)
        .iter()
        .map(UndeclaredEntity::message)
        .collect()
}

/// The entities the model declares, by kind.
///
/// Only the kinds the check knows are collected, and only the declarations
/// that carry a plain name: a placeholder (`[storage]`) is not the name of
/// an entity, and an expression is computed at run time.
fn declarations(model: &GmdlModel) -> BTreeMap<&'static str, BTreeSet<String>> {
    let mut declared: BTreeMap<&'static str, BTreeSet<String>> = BTreeMap::new();
    for block in &model.blocks {
        for prop in &block.props {
            if !prop.ty.eq_ignore_ascii_case("EntityDef") {
                continue;
            }
            let Some(kind) = kind_of(&prop.subtype) else {
                continue;
            };
            let value = prop.value();
            let name = value.trim();
            if !is_literal_name(name) {
                continue;
            }
            declared
                .entry(kind.name)
                .or_default()
                .insert(name.to_string());
        }
    }
    declared
}

/// The kind the check knows, if it knows it.
fn kind_of(subtype: &str) -> Option<&'static EntityKind> {
    KINDS.iter().find(|kind| kind.name.eq_ignore_ascii_case(subtype))
}

/// Whether a value is a plain entity name.
///
/// The names a model carries are letters, digits and the three characters
/// the original allows inside one (`_`, `.` and `-`). Everything else is an
/// expression, a placeholder or a value that is computed at run time.
fn is_literal_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-'))
}

/// A list of names in prose: `A`, `A or B`, `A, B or C`.
fn list_of(names: &[&str]) -> String {
    match names {
        [] => String::new(),
        [one] => (*one).to_string(),
        [rest @ .., last] => format!("{rest} or {last}", rest = rest.join(", ")),
    }
}
