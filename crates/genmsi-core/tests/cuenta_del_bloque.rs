//! The row of the report that counts the transactions of a block.
//!
//! A block of the model is one or several statements of the listing: a
//! facility use is five (a queue, a seize, a depart, an advance and a
//! release), and all of them count the transactions that went through the
//! block. The report says which row counts it, so that a client asks the
//! engine instead of adding the rows of the block up.

mod support;

use genmsi_core::json_report;
use support::{linear, module, run};

#[test]
fn la_primera_fila_de_un_bloque_es_la_que_lo_cuenta() {
    let horizon = 100.0;
    let program = linear(
        &[
            module(
                "Generate",
                "Gen",
                &[("Media", "1"), ("Modificador", "0"), ("Cantidad", "10")],
            ),
            module(
                "FacilityUse",
                "F",
                &[("Facility", "recurso"), ("Media", "1")],
            ),
            module("Terminate", "T", &[("Unidades", "0")]),
        ],
        horizon,
    );
    let result = run(&program, horizon, 1);
    let report = json_report(&result, &program);

    // The facility use is the second block of the model, and it is several
    // statements of the listing.
    let rows: Vec<_> = report.blocks.iter().filter(|row| row.node == 2).collect();
    assert!(rows.len() > 1, "a facility use is more than one statement");
    assert!(rows[0].counts_block, "the first row counts the block");
    for row in &rows[1..] {
        assert!(!row.counts_block, "only one row counts the block");
    }

    // The figure of the block is the one of that row: the ten transactions
    // that went through it, and not the sum of the rows, which counts every
    // transaction once per statement.
    assert_eq!(rows[0].entries, 10);
    let sum: u64 = rows.iter().map(|row| row.entries).sum();
    assert!(
        sum > rows[0].entries,
        "the sum counts every transaction once per statement"
    );
}

#[test]
fn cada_bloque_del_modelo_tiene_su_primera_fila() {
    let horizon = 100.0;
    let program = linear(
        &[
            module(
                "Generate",
                "Gen",
                &[("Media", "1"), ("Modificador", "0"), ("Cantidad", "3")],
            ),
            module("Advance", "A", &[("Media", "10"), ("Modificador", "0")]),
            module("Terminate", "T", &[("Unidades", "0")]),
        ],
        horizon,
    );
    let result = run(&program, horizon, 1);
    let report = json_report(&result, &program);

    // Every block of the model is counted by exactly one row.
    let mut counting: Vec<u32> = report
        .blocks
        .iter()
        .filter(|row| row.counts_block)
        .map(|row| row.node)
        .collect();
    counting.sort_unstable();
    counting.dedup();
    // The clock of the model is the fourth block.
    assert_eq!(counting, vec![1, 2, 3, 4], "every block counts itself once");
}
