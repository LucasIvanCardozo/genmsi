//! The trace of a transaction that waits in a block that does not open.
//!
//! The original counts a transaction that waits at a shut gate in the block
//! it came from, and its report says so: the queue of the reference model
//! reads `9 9 0` and the gate `0 0 0`. The trace has to say the same thing,
//! or the replay of the app, which rebuilds the occupation of every block
//! with the `enter` and `leave` events, counts the transaction in the gate
//! while the report counts it in the queue.
//!
//! The trace says it with a new event, `backup`: it is recorded in the block
//! that waits, and its detail names the block the transaction backs up to
//! (`-> 13`), the convention the `leave` event already follows.
//!
//! The transaction leaves the block it is counted in, and not the one it
//! waits in: the `leave` event of the trace is recorded in the queue, and
//! the report's count does not move a single number.

mod support;

use std::path::{Path, PathBuf};

use genmsi_core::Program;
use genmsi_core::{RunResult, SimConfig, Simulation};

use support::{block, linear, module};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The reference model of the app's slice, run with the trace on.
fn reference() -> (Program, RunResult) {
    let path = root().join("fixtures/models/TP6__Clase 6_1_PuertaBanco.gmdl");
    let model = genmsi_format::load(&path).expect("model loads");
    let program = genmsi_core::compile_model(&model).expect("model compiles");
    let result = Simulation::new(
        &program,
        SimConfig {
            trace: true,
            trace_limit: 1_000_000,
            ..SimConfig::default()
        },
    )
    .run()
    .expect("model runs");
    (program, result)
}

/// The events of one transaction: the block, the event and its detail.
fn events_of(result: &RunResult, xact: u64) -> Vec<(usize, String, String)> {
    result
        .trace
        .iter()
        .filter(|event| event.xact == xact)
        .map(|event| (event.block, event.event.to_string(), event.detail.clone()))
        .collect()
}

#[test]
fn the_transaction_that_waits_at_the_gate_says_where_it_is_counted() {
    let (program, result) = reference();
    let queue = block(&program, "A_Queue");
    let gate = block(&program, "GateL");
    let generator = block(&program, "Generate");

    // The first transaction of the run waits at the gate: the gate of the
    // model never opens.
    let first = result
        .trace
        .iter()
        .find(|event| event.event == "enter" && event.block == queue)
        .expect("the queue is entered");
    let events = events_of(&result, first.xact);
    assert_eq!(
        events,
        vec![
            (generator, "generate".to_string(), String::new()),
            (generator, "leave".to_string(), format!("-> {queue}")),
            (queue, "enter".to_string(), "qAfuera".to_string()),
            (queue, "leave".to_string(), format!("-> {gate}")),
            (gate, "enter".to_string(), String::new()),
            (gate, "wait".to_string(), "gate".to_string()),
            // The new event: the transaction backs up to the block it is
            // counted in.
            (gate, "backup".to_string(), format!("-> {queue}")),
        ],
        "the trace says where the transaction that waits is counted",
    );
}

#[test]
fn the_report_of_the_reference_model_does_not_move() {
    let (program, result) = reference();
    let queue = block(&program, "A_Queue");
    let gate = block(&program, "GateL");

    // The queue holds the transactions that wait at the gate: the original
    // counts them there (`A_Queue  QUEUE  qAfuera  13 13`).
    assert_eq!(result.blocks[queue].entries, 9, "the queue's entries");
    assert_eq!(result.blocks[queue].current, 9, "the queue's current");
    // And the gate counts nothing: the attempt is counted on the switch.
    assert_eq!(result.blocks[gate].entries, 0, "the gate's entries");
    assert_eq!(result.blocks[gate].current, 0, "the gate's current");
    assert_eq!(result.blocks[gate].retries, 0, "the gate's retries");
}

#[test]
fn the_transaction_that_waits_leaves_the_block_it_is_counted_in() {
    // A facility that is busy: the second transaction waits at the seize,
    // and it is counted in the queue it came from. When the facility is
    // released, the transaction leaves the queue, and not the seize.
    let program = linear(
        &[
            module(
                "Generate",
                "Gen",
                &[("Media", "1"), ("Modificador", "0"), ("Cantidad", "2")],
            ),
            module("Queue", "Q", &[("Queue", "q")]),
            module("Seize", "S", &[("Facility", "F")]),
            module("Advance", "A", &[("Media", "100"), ("Modificador", "0")]),
            module("Release", "R", &[("Facility", "F")]),
            module("Terminate", "T", &[("Unidades", "0")]),
        ],
        1000.0,
    );
    let result = Simulation::new(
        &program,
        SimConfig {
            trace: true,
            trace_limit: 1_000_000,
            ..SimConfig::default()
        },
    )
    .run()
    .expect("model runs");
    let queue = block(&program, "Q");
    let seize = block(&program, "S");
    let advance = block(&program, "A");
    let release = block(&program, "R");
    let terminate = block(&program, "T");
    let generator = block(&program, "Generate");

    // The transaction that waits at the seize.
    let waiting = result
        .trace
        .iter()
        .find(|event| event.event == "wait" && event.block == seize)
        .expect("a transaction waits at the seize");
    let events = events_of(&result, waiting.xact);
    assert_eq!(
        events,
        vec![
            (generator, "generate".to_string(), String::new()),
            (generator, "leave".to_string(), format!("-> {queue}")),
            (queue, "enter".to_string(), "q".to_string()),
            (queue, "leave".to_string(), format!("-> {seize}")),
            (seize, "enter".to_string(), "F".to_string()),
            (seize, "wait".to_string(), "F".to_string()),
            (seize, "backup".to_string(), format!("-> {queue}")),
            // The transaction leaves the block it is counted in.
            (queue, "leave".to_string(), format!("-> {advance}")),
            (advance, "enter".to_string(), String::new()),
            (advance, "leave".to_string(), format!("-> {release}")),
            (release, "enter".to_string(), "F".to_string()),
            (release, "leave".to_string(), format!("-> {terminate}")),
            (terminate, "enter".to_string(), String::new()),
            (terminate, "terminate".to_string(), String::new()),
        ],
        "the trace says where the transaction that waits is counted",
    );

    // And the report counts the same thing: the queue holds nobody at the
    // end of the run, and the seize counted the two seizures it made.
    assert_eq!(result.blocks[queue].current, 0, "the queue holds nobody");
    assert_eq!(result.blocks[seize].entries, 2, "the seize's entries");
    assert_eq!(result.blocks[seize].current, 0, "the seize holds nobody");
}
