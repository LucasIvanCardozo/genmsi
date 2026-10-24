//! What each GPSS statement of the engine does.
//!
//! The glossary is the vocabulary an agent needs to read a model and to write
//! one: every class the engine implements has one line that says what the
//! statement does and which operands it takes. The text follows the
//! semantics of `genmsi-core`, not the wording of the original editor.
//!
//! The glossary lives here, next to the catalogue of the classes, because it
//! describes the same classes: the catalogue says what a block of a class
//! looks like and the glossary says what that class does.

use std::collections::BTreeMap;

use genmsi_format::GmdlModel;

/// The description of every class the engine implements.
const GLOSSARY: &[(&str, &str)] = &[
    (
        "Advance",
        "Delays the transaction by Media units of simulated time, plus Modificador for the deviation.",
    ),
    (
        "Assign",
        "Sets the parameter Parametro to Valor, applying Operador (+, -, * or /).",
    ),
    (
        "Assemble",
        "Waits until Cantidad transactions of the same group have arrived, then releases them together.",
    ),
    (
        "Buffer",
        "Buffers the transactions that arrive and releases them one at a time.",
    ),
    (
        "Clock",
        "Control clock: creates a transaction every Frecuencia time units and ends the run when the counter reaches one.",
    ),
    (
        "Counter",
        "Counts the arrivals of the group Param and stores the count in the savevalue SaveValue.",
    ),
    (
        "Depart",
        "Removes Cantidad transactions from the queue Queue, without using a facility.",
    ),
    (
        "Enter",
        "Enters Cantidad units of the storage Storage.",
    ),
    (
        "ExamineNumber",
        "Waits until the group Grupo holds Numero transactions, then lets the transaction continue.",
    ),
    (
        "FacilityInt",
        "Preempts the facility Facility after Media units of time and waits in the queue Queue until it is free again.",
    ),
    (
        "FacilityUse",
        "Uses the facility Facility for Media units of time; the transaction waits in the queue Queue while it is busy.",
    ),
    (
        "Function",
        "Defines a function for the expressions (FN:name).",
    ),
    (
        "FunctionDist",
        "Defines a function with a distribution (FN:name).",
    ),
    (
        "FunctionExpon",
        "Defines an exponential function (FN:name).",
    ),
    (
        "Gather",
        "Waits until Cantidad transactions of the same group have arrived (like Assemble, with the group's own label).",
    ),
    (
        "Generate",
        "Creates transactions: Media is the mean time between them, Cantidad how many to create (leave it empty to create them forever; a number limits the total, and zero creates none), with the optional Modificador, OffSet and Prioridad.",
    ),
    (
        "GateF",
        "Gate: lets the transaction continue only when the facility Facility is free (op = libre) or busy.",
    ),
    (
        "GateL",
        "Gate: lets the transaction continue only when the logic switch LogicSwich is set (op = set) or reset.",
    ),
    (
        "GateS",
        "Gate: lets the transaction continue only when the storage Storage is empty (op = vacio) or not empty.",
    ),
    (
        "InitialLS",
        "Sets the initial state of a logic switch.",
    ),
    (
        "InitiaSV",
        "Sets the initial value of a savevalue.",
    ),
    (
        "JoinMe",
        "Joins the group Grupo.",
    ),
    (
        "JoinNumber",
        "Joins the group Grupo and waits until it holds Numero transactions.",
    ),
    (
        "Leave",
        "Releases Cantidad units of the storage Storage.",
    ),
    (
        "Link",
        "Puts the transaction at the end of the user chain Cadena (Modo LIFO puts it at the front).",
    ),
    (
        "Logic",
        "Sets (Op) or resets the logic switch LogicSwich.",
    ),
    (
        "Loop",
        "Sends the transaction back to the beginning of its loop (loop parameter Parametro).",
    ),
    (
        "Match",
        "Pairs the transaction with the one that carries the conjugate label RotuloMatch; the pair continues together.",
    ),
    (
        "Multifurcacion",
        "Branches the transaction: the properties Prob1..ProbN hold the share that goes to each output.",
    ),
    (
        "Note",
        "A comment; it produces no statement.",
    ),
    (
        "Priority",
        "Sets the priority of the transaction to Prioridad.",
    ),
    (
        "Queue",
        "The transaction joins the queue Queue; Cantidad units leave it.",
    ),
    (
        "Release",
        "Releases the facility Facility.",
    ),
    (
        "RemoveTrans",
        "Removes Cantidad transactions from the group Grupo (Modo Todos removes all), optionally the ones whose CompParam equals CompValue.",
    ),
    (
        "SaveValue",
        "Stores Valor in the savevalue SaveValue, applying Op.",
    ),
    (
        "Scan",
        "Scans the group Grupo for the transactions whose CompParam equals CompValue and stores their parameter range (DeParam..EnParam).",
    ),
    (
        "Seize",
        "Seizes the facility Facility; the transaction waits until it is free.",
    ),
    (
        "Select",
        "Selects an entity of the class ClaseSNA between minimo and maximo and stores its name in Param. The class is one of the 38 of the original's list (Q queue, S storage, R storage left, F facility, LS logic switch...).",
    ),
    (
        "SelectF",
        "Selects the first free facility between minimo and maximo.",
    ),
    (
        "Split",
        "Creates Cantidad copies of the transaction.",
    ),
    (
        "StorageDef",
        "Declares the capacity of a storage.",
    ),
    (
        "StorageUse",
        "Takes Cantidad units of the storage Storage for Media units of time; the transaction waits in the queue Queue while the storage is full.",
    ),
    (
        "Table",
        "Declares a table for the TABULATE statement.",
    ),
    (
        "QTable",
        "Declares a queue table.",
    ),
    (
        "Tabulate",
        "Records the value in the table NombreTabla, with the optional weight Peso.",
    ),
    (
        "Terminate",
        "Ends the run when Unidades transactions reach the statement (zero means the statement never fires).",
    ),
    (
        "Test",
        "Compares the operands A and B with the operator 2_OP; the transaction continues when the comparison holds.",
    ),
    (
        "Transfer",
        "Sends the transaction to the next block with the probability Prob; the share goes to the first output.",
    ),
    (
        "Unlink",
        "Removes Cantidad transactions from the user chain Cadena (Modo Todos removes all), optionally the ones whose CompParam equals CompValue; the unlinked transactions leave through the labelled output.",
    ),
    (
        "Variable",
        "Defines an arithmetic variable (V:name).",
    ),
    (
        "VariableA",
        "Defines a random variable (V:name).",
    ),
];

/// The description of one class, when the engine knows it.
pub fn describe(class: &str) -> Option<&'static str> {
    GLOSSARY
        .iter()
        .find(|(name, _)| *name == class)
        .map(|(_, description)| *description)
}

/// Every class the glossary describes, with its description.
///
/// The list is the glossary itself: a class the engine does not describe is
/// not in it.
pub fn entries() -> &'static [(&'static str, &'static str)] {
    GLOSSARY
}

/// The glossary of the classes a model uses.
///
/// The classes come from the model itself, so a definition block (a storage
/// capacity, a variable) is described as well as a runtime statement.
pub fn for_model(model: &GmdlModel) -> BTreeMap<String, String> {
    let mut glossary = BTreeMap::new();
    for block in model.activity_blocks() {
        if let Some(description) = describe(&block.class) {
            glossary
                .entry(block.class.clone())
                .or_insert_with(|| description.to_string());
        }
    }
    glossary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_runtime_statement_has_an_entry() {
        for class in [
            "Advance",
            "Assign",
            "Clock",
            "Generate",
            "Queue",
            "Terminate",
            "Transfer",
            "Unlink",
        ] {
            assert!(describe(class).is_some(), "{class} has no description");
        }
    }

    #[test]
    fn an_unknown_class_has_no_description() {
        assert!(describe("NotAStatement").is_none());
    }

    #[test]
    fn the_entries_are_the_descriptions_of_their_classes() {
        for (class, description) in entries() {
            assert_eq!(describe(class), Some(*description));
        }
    }

    #[test]
    fn the_glossary_of_a_model_covers_its_classes() {
        let xml = "<gmodel versioncode=\"1\"><gclasses/><modules>\
            <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"m\" name=\"m\" kind=\"C\"><links/><inputs/><outputs/></module>\
            <module moduleid=\"1\" moduleclass=\"Generate\" owner=\"0\" caption=\"g\" name=\"g\" kind=\"A\"><props/></module>\
            <module moduleid=\"2\" moduleclass=\"StorageDef\" owner=\"0\" caption=\"s\" name=\"s\" kind=\"A\"><props/></module>\
            </modules></gmodel>";
        let model = genmsi_format::parse_str(xml).expect("model parses");
        let glossary = for_model(&model);
        assert!(glossary.contains_key("Generate"));
        assert!(glossary.contains_key("StorageDef"));
        assert_eq!(glossary.len(), 2);
    }
}
