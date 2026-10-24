//! The properties of a block and the class that declares them.
//!
//! A block carries the properties of its class, and the class is what says
//! which properties there are. A property the class does not declare is a
//! typo — `Tamano` for `Tamaño` — and the engine reads it as nothing: the
//! storage keeps no capacity, the table no frequencies, and the model runs
//! without saying why. A model that hangs with no explanation is the worst
//! answer an agent can get.
//!
//! The finding is a warning and not a refusal: the original opens a model
//! with a property it does not know (the `Note` blocks of the reference
//! models carry free text, and the `Note` class declares no property at
//! all), so the model keeps loading and is simply not silent about it any
//! more.

use crate::catalogue::class_spec;
use crate::model::GmdlModel;

/// One property a block carries that its class does not declare.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndeclaredProperty {
    /// Id of the block that carries the property.
    pub block: u32,
    /// Class of the block (`StorageDef`, ...).
    pub class: String,
    /// The name the property carries (`Tamano`).
    pub property: String,
    /// The property of the class it looks like, when there is one.
    pub looks_like: Option<String>,
    /// The properties the class declares, in schema order.
    pub declared: Vec<String>,
}

impl UndeclaredProperty {
    /// What the model has to hear about this property.
    ///
    /// The message says which block carries it, what the class declares and
    /// which name it probably meant: a warning that only says "unknown"
    /// leaves the reader guessing.
    pub fn message(&self) -> String {
        let declared = self.declared.join(", ");
        let meant = match &self.looks_like {
            Some(name) => format!(" (did you mean '{name}'?)"),
            None => String::new(),
        };
        format!(
            "the block {} ({}) declares the property '{}', which the class does not \
             declare{}. The class declares: {}.",
            self.block, self.class, self.property, meant, declared
        )
    }
}

/// The properties the model carries that their classes do not declare.
///
/// The list is empty when every property of every block belongs to its class.
pub fn undeclared_properties(model: &GmdlModel) -> Vec<UndeclaredProperty> {
    let mut found = Vec::new();
    for block in &model.blocks {
        let Some(spec) = class_spec(&block.class) else {
            continue;
        };
        // A class that declares no property (the `Note`, which is a comment
        // the editor fills with whatever the reader types) cannot be checked
        // against: every property of such a block is unknown, and the
        // original's own models carry plenty of them.
        if spec.properties.is_empty() {
            continue;
        }
        let declared: Vec<String> = spec
            .properties
            .iter()
            .map(|property| property.name.to_string())
            .collect();
        for property in &block.props {
            if declared.iter().any(|name| name == &property.name) {
                continue;
            }
            let looks_like = declared
                .iter()
                .find(|name| folds_to_the_same(name, &property.name))
                .cloned();
            found.push(UndeclaredProperty {
                block: block.id,
                class: block.class.clone(),
                property: property.name.clone(),
                looks_like,
                declared: declared.clone(),
            });
        }
    }
    found
}

/// The warnings the format found in the properties of the model.
pub fn property_warnings(model: &GmdlModel) -> Vec<String> {
    undeclared_properties(model)
        .iter()
        .map(UndeclaredProperty::message)
        .collect()
}

/// Whether two names are the same once the case and the accents are gone.
///
/// `Tamano` and `Tamaño` are the same name written twice: the accent is the
/// only difference, and the difference is exactly what a typo is.
fn folds_to_the_same(left: &str, right: &str) -> bool {
    fn fold(text: &str) -> String {
        text.chars()
            .flat_map(char::to_lowercase)
            .map(|character| match character {
                'á' | 'à' | 'ä' | 'â' | 'ã' => 'a',
                'é' | 'è' | 'ë' | 'ê' => 'e',
                'í' | 'ì' | 'ï' | 'î' => 'i',
                'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'o',
                'ú' | 'ù' | 'ü' | 'û' => 'u',
                'ñ' => 'n',
                'ç' => 'c',
                other => other,
            })
            .collect()
    }
    fold(left) == fold(right)
}
