//! `MATCH`: transactions pair with the conjugate block.

mod support;

use genmsi_core::Program;
use support::{node, run, wired};

/// Every block of one class, in program order (`MATCH` blocks carry their
/// `RotuloPropio` as label, not the incoming connection label).
fn of_class(program: &Program, class: &str) -> Vec<usize> {
    program
        .blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| block.class == class)
        .map(|(index, _)| index)
        .collect()
}

/// Two flows paired through `MATCH` blocks, with the given transaction counts.
fn model(first: &'static str, second: &'static str) -> Program {
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A1"))],
            &[("Media", "1"), ("Modificador", "0"), ("Cantidad", first)],
        ),
        node(
            2,
            "Match",
            Some("A1"),
            &[("Salida1", Some("A2")), ("Salida2", None)],
            &[("RotuloPropio", "PA"), ("RotuloMatch", "PB")],
        ),
        node(3, "Terminate", Some("A2"), &[], &[("Unidades", "0")]),
        node(
            4,
            "Generate",
            None,
            &[("Salida1", Some("B1"))],
            &[("Media", "1"), ("Modificador", "0"), ("Cantidad", second)],
        ),
        node(
            5,
            "Match",
            Some("B1"),
            &[("Salida1", Some("B2")), ("Salida2", None)],
            &[("RotuloPropio", "PB"), ("RotuloMatch", "PA")],
        ),
        node(6, "Terminate", Some("B2"), &[], &[("Unidades", "0")]),
    ];
    let links = [(1, 0, 2, 0), (2, 0, 3, 0), (4, 0, 5, 0), (5, 0, 6, 0)];
    wired(&nodes, &links, 100.0)
}

/// Every arriving transaction must be paired and released, however many are
/// waiting at the conjugate block.
///
/// A single remembered waiter per block strands the rest: `TP5__burgerShop`
/// pairs its orders with `MATCH PIENSA` / `MATCH ESPERA` and the baseline
/// releases all 40 pairs (both blocks end with CURRENT 0 and 3) while the
/// engine left 26 and 15 transactions waiting forever.
#[test]
fn every_transaction_is_paired() {
    let program = model("4", "4");
    let result = run(&program, 100.0, 7);
    let matches = of_class(&program, "Match");
    assert_eq!(matches.len(), 2);
    for index in &matches {
        assert_eq!(result.blocks[*index].current, 0, "no stranded waiter");
    }
    for label in ["A2", "B2"] {
        assert_eq!(
            result.blocks[support::block(&program, label)].entries,
            4,
            "each branch released"
        );
    }
}

/// An unpaired transaction must still be waiting at the end of the run: the
/// original reports `CURRENT 3` at `PIENSA` for exactly this reason.
#[test]
fn an_unpaired_transaction_stays_waiting() {
    let program = model("4", "1");
    let result = run(&program, 100.0, 7);
    let waiting: u64 = of_class(&program, "Match")
        .iter()
        .map(|index| result.blocks[*index].current as u64)
        .sum();
    assert!(waiting >= 1, "a lone transaction waits for its conjugate");
}
