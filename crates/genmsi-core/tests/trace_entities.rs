//! The trace must publish what the report counts.
//!
//! The `enter` event publishes the entity a block touches: a queue named by a
//! parameter (`*caja`) or by an expression is only known at run time, so the
//! trace has to carry the resolved name. Blocks that touch no entity keep an
//! empty detail, and the `wait` and `leave` events keep the detail they had.
//!
//! The report also publishes the samples of a table, the state of a logic
//! switch, the value of a savevalue and the members that leave a chain. The
//! trace says all of it, so a reader can rebuild the report's figures from
//! the trace alone:
//!
//! - `table`: the table's name and the value that was tabulated,
//! - `logic`: the switch's name and its state (`1` set, `0` cleared),
//! - `value`: the savevalue's name and its new value,
//! - `unlink`: the transactions that left the chain, in the order they left.
//!
//! The name and the value are separated by the *last* space: a table's name
//! may contain spaces, a number never does. A savevalue's value is written
//! the way the report writes it (a comma is the decimal separator); a
//! tabulated sample keeps every digit, so the histogram can be rebuilt
//! exactly.

use std::path::{Path, PathBuf};

use genmsi_core::program::{BlockKind, ExecBlock};
use genmsi_core::{Program, RunResult, SimConfig, Simulation};

mod support;

use support::{block, linear, module};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Compile and run the reference model with the trace on.
///
/// `TP2__problema1` names its queue `centro` literally.
fn run_model() -> (Program, RunResult) {
    let path = root().join("fixtures/models/TP2__problema1.gmdl");
    let model = genmsi_format::load(&path).expect("model loads");
    let program = genmsi_core::compile_model(&model).expect("model compiles");
    let result = Simulation::new(
        &program,
        SimConfig {
            trace: true,
            ..SimConfig::default()
        },
    )
    .run()
    .expect("model runs");
    (program, result)
}

/// Compile and run a fixture model with the trace on.
fn run_fixture(name: &str) -> (Program, RunResult) {
    let path = root().join(format!("fixtures/models/{name}.gmdl"));
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

/// Compile and run a fixture model without the trace.
fn run_fixture_untraced(name: &str) -> (Program, RunResult) {
    let path = root().join(format!("fixtures/models/{name}.gmdl"));
    let model = genmsi_format::load(&path).expect("model loads");
    let program = genmsi_core::compile_model(&model).expect("model compiles");
    let result = Simulation::new(
        &program,
        SimConfig {
            trace: false,
            ..SimConfig::default()
        },
    )
    .run()
    .expect("model runs");
    (program, result)
}

/// Split a detail into the name and the value it carries.
///
/// The two are separated by the *last* space: a name may contain spaces, a
/// value never does.
fn split_detail(detail: &str) -> (&str, &str) {
    detail
        .rsplit_once(' ')
        .unwrap_or_else(|| panic!("the detail {detail:?} carries a name and a value"))
}

/// Parse a number the way the report writes it: the decimal separator is a
/// comma.
fn parse_number(text: &str) -> f64 {
    text.replace(',', ".")
        .parse()
        .unwrap_or_else(|_| panic!("{text:?} is a number"))
}

/// Name of the entity a block touches, as the model spells it.
fn entity_name(block: &ExecBlock) -> Option<String> {
    match &block.kind {
        BlockKind::Queue(spec) => Some(spec.queue.to_string()),
        BlockKind::Depart(spec) => Some(spec.queue.to_string()),
        BlockKind::Seize(spec) => Some(spec.facility.to_string()),
        BlockKind::Release(spec) => Some(spec.facility.to_string()),
        BlockKind::Preempt(spec) => Some(spec.facility.to_string()),
        BlockKind::Return(spec) => Some(spec.facility.to_string()),
        BlockKind::Enter(spec) => Some(spec.storage.to_string()),
        BlockKind::Leave(spec) => Some(spec.storage.to_string()),
        BlockKind::Link(spec) => Some(spec.chain.to_string()),
        BlockKind::Unlink(spec) => Some(spec.chain.to_string()),
        _ => None,
    }
}

/// Index of the first block that touches the entity named `name`.
fn block_of(program: &Program, name: &str) -> Option<usize> {
    program
        .blocks
        .iter()
        .position(|block| entity_name(block).as_deref() == Some(name))
}

#[test]
fn enter_trace_carries_the_queue_name() {
    let (program, result) = run_model();
    let queue = block_of(&program, "centro").expect("the centro queue is in the model");
    let enters: Vec<_> = result
        .trace
        .iter()
        .filter(|event| event.event == "enter" && event.block == queue)
        .collect();
    assert!(!enters.is_empty(), "the queue must be entered");
    for event in enters {
        assert_eq!(event.detail, "centro", "the enter event names the queue");
    }
}

#[test]
fn enter_trace_carries_the_name_resolved_from_a_parameter() {
    // A queue named by a parameter (`*caja`) is only known at run time.
    let program = linear(
        &[
            module(
                "Generate",
                "Gen",
                &[("Media", "1"), ("Modificador", "0"), ("Cantidad", "5")],
            ),
            module("Assign", "As", &[("Parametro", "caja"), ("Valor", "2")]),
            module("Queue", "Q", &[("Queue", "*caja")]),
            module("Terminate", "T", &[("Unidades", "0")]),
        ],
        100.0,
    );
    let result = Simulation::new(
        &program,
        SimConfig {
            trace: true,
            ..SimConfig::default()
        },
    )
    .run()
    .expect("model runs");
    let queue = block(&program, "Q");
    let enters: Vec<_> = result
        .trace
        .iter()
        .filter(|event| event.event == "enter" && event.block == queue)
        .collect();
    assert!(!enters.is_empty(), "the queue must be entered");
    for event in enters {
        assert_eq!(event.detail, "2", "the parameter resolves the queue name");
    }
}

#[test]
fn enter_trace_is_empty_for_blocks_without_an_entity() {
    let (program, result) = run_model();
    let mut covered = 0;
    for event in result.trace.iter().filter(|event| event.event == "enter") {
        let block = &program.blocks[event.block];
        if entity_name(block).is_none() {
            covered += 1;
            assert_eq!(event.detail, "", "block {} touches no entity", event.block);
        }
    }
    assert!(covered > 0, "the model must enter blocks without an entity");

    // The generator and the transfers are such blocks.
    for class in ["Generate", "Transfer"] {
        let index = program
            .blocks
            .iter()
            .position(|block| block.class == class)
            .unwrap_or_else(|| panic!("the model has a {class} block"));
        assert!(entity_name(&program.blocks[index]).is_none());
        let events = result.trace.iter().filter(|event| event.block == index);
        for event in events {
            if matches!(event.event, "enter" | "generate") {
                assert_eq!(event.detail, "", "a {class} block touches no entity");
            }
        }
    }
}

#[test]
fn wait_trace_still_carries_the_entity_name() {
    let (program, result) = run_model();
    let waits: Vec<_> = result
        .trace
        .iter()
        .filter(|event| event.event == "wait")
        .collect();
    assert!(!waits.is_empty(), "the model must block somewhere");
    for event in waits {
        let block = &program.blocks[event.block];
        let expected = match &block.kind {
            BlockKind::Seize(spec) => spec.facility.to_string(),
            BlockKind::Enter(spec) => spec.storage.to_string(),
            other => panic!("unexpected waiting block {other:?}"),
        };
        assert_eq!(event.detail, expected, "the wait event names the entity");
    }
}

#[test]
fn leave_trace_still_carries_the_target() {
    let (program, result) = run_model();
    let leaves: Vec<_> = result
        .trace
        .iter()
        .filter(|event| event.event == "leave")
        .collect();
    assert!(!leaves.is_empty(), "the model must leave blocks");
    for event in leaves {
        let target = event
            .detail
            .strip_prefix("-> ")
            .unwrap_or_else(|| panic!("unexpected leave detail {:?}", event.detail));
        let target: usize = target.parse().expect("the target is a block index");
        assert!(target < program.blocks.len(), "the target block exists");
        let enters = result
            .trace
            .iter()
            .any(|next| next.event == "enter" && next.block == target && next.xact == event.xact);
        assert!(enters, "the transaction enters the target block");
    }
}

#[test]
fn table_trace_carries_the_name_and_the_value() {
    let (_program, result) = run_fixture("TP4__supermercado");
    let samples: Vec<_> = result
        .trace
        .iter()
        .filter(|event| event.event == "table")
        .collect();
    assert!(!samples.is_empty(), "the model tabulates values");
    for event in &samples {
        let (name, value) = split_detail(&event.detail);
        let value = parse_number(value);
        assert!(
            result.entities.tables.contains_key(name) || result.entities.qtables.contains_key(name),
            "the detail names a table of the model: {name}"
        );
        assert!(value.is_finite(), "the tabulated value is a number");
    }
    // The two tables of the model are tabulated, and so is the queue table.
    let mut named: Vec<&str> = samples
        .iter()
        .map(|event| split_detail(&event.detail).0)
        .collect();
    named.sort_unstable();
    named.dedup();
    assert!(
        named.contains(&"TGondolas") && named.contains(&"TPatio"),
        "both tables of the model are tabulated: {named:?}"
    );
    assert!(
        named.contains(&"QPersonasSuper"),
        "the queue table is tabulated too: {named:?}"
    );
}

#[test]
fn table_trace_rebuilds_the_histogram() {
    let (_program, result) = run_fixture("TP4__supermercado");
    for (name, table) in result
        .entities
        .tables
        .iter()
        .chain(result.entities.qtables.iter())
    {
        let values: Vec<f64> = result
            .trace
            .iter()
            .filter(|event| event.event == "table")
            .filter_map(|event| {
                let (table_name, value) = split_detail(&event.detail);
                (table_name == name).then(|| parse_number(value))
            })
            .collect();
        assert_eq!(
            values.len() as u64,
            table.samples,
            "the trace carries one sample per observation of {name}"
        );
        if values.is_empty() {
            continue;
        }
        assert_eq!(
            values.iter().copied().fold(f64::MIN, f64::max),
            table.max_value,
            "the samples carry the maximum the report publishes"
        );
        // Rebuild the histogram the way the engine builds it.
        let mut frequencies = vec![0_u64; table.intervals];
        for value in &values {
            let index = if *value <= table.first {
                0
            } else {
                (((value - table.first) / table.width).ceil() as usize).min(table.intervals - 1)
            };
            frequencies[index] += 1;
        }
        assert_eq!(
            frequencies, table.frequencies,
            "the samples rebuild the histogram of {name}"
        );
    }
}

#[test]
fn logic_trace_carries_the_switch_and_its_state() {
    let (_program, result) = run_fixture("Problema 1__semaforo-1");
    let switches: Vec<_> = result
        .trace
        .iter()
        .filter(|event| event.event == "logic")
        .collect();
    assert!(!switches.is_empty(), "the model sets a switch");
    for event in &switches {
        let (name, state) = split_detail(&event.detail);
        assert!(
            state == "0" || state == "1",
            "the state of a switch is 1 when it is set and 0 when it is cleared"
        );
        assert!(
            result.entities.logic.contains_key(name),
            "the detail names a switch of the model: {name}"
        );
    }
    // The model sets the switch and clears it again, so both states are in
    // the trace.
    assert!(
        switches
            .iter()
            .any(|event| split_detail(&event.detail).1 == "1"),
        "the trace publishes the switch being set"
    );
    assert!(
        switches
            .iter()
            .any(|event| split_detail(&event.detail).1 == "0"),
        "the trace publishes the switch being cleared"
    );
    // The last event of a switch carries the state the report publishes.
    for (name, state) in &result.entities.logic {
        let last = switches
            .iter()
            .rev()
            .find(|event| split_detail(&event.detail).0 == name);
        let Some(last) = last else {
            continue; // a switch only read by a gate is never set
        };
        assert_eq!(
            split_detail(&last.detail).1,
            if *state { "1" } else { "0" },
            "the last event of {name} carries the state of the report"
        );
    }
}

#[test]
fn value_trace_carries_the_name_and_the_value() {
    let (_program, result) = run_fixture("TP4__supermercado");
    let values: Vec<_> = result
        .trace
        .iter()
        .filter(|event| event.event == "value")
        .collect();
    assert!(!values.is_empty(), "the model assigns a savevalue");
    for event in &values {
        let (name, value) = split_detail(&event.detail);
        assert!(
            result.entities.savevalues.contains_key(name),
            "the detail names a savevalue of the model: {name}"
        );
        parse_number(value);
    }
    // The last event of a savevalue carries the value the report publishes.
    for (name, value) in &result.entities.savevalues {
        let last = values
            .iter()
            .rev()
            .find(|event| split_detail(&event.detail).0 == name);
        let Some(last) = last else {
            continue;
        };
        assert_eq!(
            split_detail(&last.detail).1,
            genmsi_core::report::number(*value),
            "the last event of {name} carries the value of the report"
        );
    }
}

#[test]
fn unlink_trace_carries_the_members_that_left_the_chain() {
    let (program, result) = run_fixture("TP7__Clase 7_1_Link_Unlink");
    let unlinks: Vec<_> = result
        .trace
        .iter()
        .filter(|event| event.event == "unlink")
        .collect();
    assert!(!unlinks.is_empty(), "the model unlinks members");
    let mut removed: Vec<u64> = Vec::new();
    for event in &unlinks {
        assert!(
            event.detail.chars().all(|c| c.is_ascii_digit() || c == ','),
            "the detail lists transaction ids: {:?}",
            event.detail
        );
        for member in event.detail.split(',') {
            removed.push(member.parse().expect("a transaction id"));
        }
    }
    let chain = result
        .entities
        .chains
        .get("Cadena")
        .expect("the chain is in the report");
    assert_eq!(
        removed.len() as u64,
        chain.entries,
        "every transaction that entered the chain left it"
    );
    // The chain is a FIFO one, so the members leave in the order they came.
    let mut sorted = removed.clone();
    sorted.sort_unstable();
    assert_eq!(removed, sorted, "the members leave in the order they came");
    // The transaction that unlinked is not one of the members that left.
    for event in &unlinks {
        assert!(
            !removed.contains(&event.xact),
            "the transaction that unlinked is not a member of the chain"
        );
    }
    // The block unlinks many times and removes nothing most of the time:
    // an empty removal publishes no event at all.
    let unlink_block = program
        .blocks
        .iter()
        .position(|block| block.class == "Unlink")
        .expect("the model has an Unlink block");
    let entries = result
        .trace
        .iter()
        .filter(|event| event.event == "enter" && event.block == unlink_block)
        .count();
    assert!(
        entries > unlinks.len(),
        "the block unlinks {entries} times and removes a member {} times",
        unlinks.len()
    );
}

#[test]
fn a_run_without_the_trace_publishes_no_events() {
    let (_program, result) = run_fixture_untraced("TP4__supermercado");
    assert!(
        result.trace.is_empty(),
        "a run with the trace off publishes no events"
    );
}

#[test]
fn the_trace_does_not_change_the_run() {
    let (_program, traced) = run_fixture("TP4__supermercado");
    let (program, untraced) = run_fixture_untraced("TP4__supermercado");
    assert!(!traced.trace.is_empty(), "the traced run publishes events");
    assert_eq!(
        traced.end_time, untraced.end_time,
        "the run ends at the same time"
    );
    assert_eq!(
        traced.transactions, untraced.transactions,
        "the run creates the same transactions"
    );
    assert_eq!(
        traced.events, untraced.events,
        "the run processes the same events"
    );
    assert_eq!(
        genmsi_core::render(&traced, &program),
        genmsi_core::render(&untraced, &program),
        "the report is the same with and without the trace"
    );
}
