//! `GATE` on a storage: the six operators of the original and what each one
//! asks of the storage.
//!
//! The truth table below is measured against the original `genmsi.exe` under
//! Wine with the probes in `scripts/wine-oracle/probes/gates/`
//! (`gateS_vacio`, `gateS_uno`, `gateS_lleno` and `gateS_op_*`): the report
//! of the original says, operator by operator, whether the gate opened with
//! the storage empty, with one unit of ten in it and with it full.
//!
//! | Storage                | `SF` | `SNF` | `SE` | `SNE` | `SV` | `SNV` |
//! |------------------------|------|-------|------|-------|------|------|
//! | empty (0 of 10)        | shut | open  | open | shut  | open | shut |
//! | one unit of ten        | shut | open  | shut | open  | open | shut |
//! | full (1 of 1)          | open | shut  | shut | open  | open | shut |
//!
//! The class `GateS` of the original has `NoLleno` as its default: its gate
//! opens when the storage still has room, not when it is empty.

mod support;

use genmsi_core::Program;
use genmsi_core::report::render;
use support::{block, node, run, wired};

/// The state of the storage when the gate is evaluated.
#[derive(Clone, Copy)]
enum State {
    /// The storage is declared and nobody entered it.
    Empty,
    /// The storage is declared with ten units and one of them is in use.
    One,
    /// The storage is declared with one unit and it is in use.
    Full,
}

/// The model of one state: a generator, an optional `ENTER` that takes a
/// unit of the storage, the gate and a terminate.
///
/// The generator produces one transaction (the first one arrives at the mean
/// and the run stops before the second), so the storage holds exactly what
/// the `ENTER` put in it.
fn model(op: &'static str, state: State) -> Program {
    let (size, enters) = match state {
        State::Empty => ("10", false),
        State::One => ("10", true),
        State::Full => ("1", true),
    };
    let mut nodes = vec![
        node(
            1,
            "StorageDef",
            None,
            &[],
            &[("Storage", "Almacen"), ("Tamaño", size)],
        ),
        node(
            2,
            "Generate",
            None,
            &[("Salida1", Some("A"))],
            &[("Media", "60"), ("Modificador", "0")],
        ),
    ];
    let mut links = vec![(2u32, 0usize, 3u32, 0usize)];
    let mut next = 3u32;
    let mut gate_input = "A";
    if enters {
        nodes.push(node(
            next,
            "Enter",
            Some("A"),
            &[("Salida1", Some("E"))],
            &[("Storage", "Almacen"), ("Cantidad", "1")],
        ));
        links.push((next, 0, next + 1, 0));
        next += 1;
        gate_input = "E";
    }
    nodes.push(node(
        next,
        "GateS",
        Some(gate_input),
        &[("Salida1", Some("B")), ("Salida2", None)],
        &[("Storage", "Almacen"), ("op", op)],
    ));
    links.push((next, 0, next + 1, 0));
    next += 1;
    nodes.push(node(next, "Terminate", Some("B"), &[], &[]));
    wired(&nodes, &links, 3600.0)
}

/// Whether the gate let the transaction through.
fn opened(program: &Program) -> bool {
    let result = run(program, 100.0, 1);
    let terminate = block(program, "B");
    result.blocks[terminate].entries > 0
}

/// The letter the gate writes in the program listing.
fn letter(program: &Program) -> String {
    let text = render(&run(program, 100.0, 1), program);
    let fields: Vec<&str> = text
        .lines()
        .find(|line| line.contains("GATE"))
        .unwrap_or_default()
        .split_whitespace()
        .collect();
    let position = fields.iter().position(|field| *field == "GATE");
    position
        .and_then(|index| fields.get(index + 1))
        .unwrap_or(&"")
        .to_string()
}

#[test]
fn the_gate_on_a_storage_asks_the_state_the_original_asks() {
    // (operator, letter, opens with an empty storage, opens with one unit of
    // ten, opens with the storage full).
    let table = [
        ("Lleno", "SF", false, false, true),
        ("NoLleno", "SNF", true, true, false),
        ("Vacio", "SE", true, false, false),
        ("NoVacio", "SNE", false, true, true),
        ("Disponible", "SV", true, true, true),
        ("NoDisponible", "SNV", false, false, false),
    ];
    for (op, expected_letter, empty, one, full) in table {
        let program = model(op, State::Empty);
        assert_eq!(letter(&program), expected_letter, "the letter of {op}");
        assert_eq!(opened(&program), empty, "{op} with an empty storage");
        assert_eq!(
            opened(&model(op, State::One)),
            one,
            "{op} with one unit of ten"
        );
        assert_eq!(opened(&model(op, State::Full)), full, "{op} with a full storage");
    }
}

#[test]
fn the_gate_of_the_class_opens_when_the_storage_has_room() {
    // The default of the class is `NoLleno`: the gate opens while the storage
    // is not full and closes when it is.
    let program = model("NoLleno", State::Empty);
    assert!(opened(&program));
    assert!(!opened(&model("NoLleno", State::Full)));
}
