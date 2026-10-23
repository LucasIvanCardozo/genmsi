//! The trace must publish what the report counts.
//!
//! The `enter` event publishes the entity a block touches: a queue named by a
//! parameter (`*caja`) or by an expression is only known at run time, so the
//! trace has to carry the resolved name. Blocks that touch no entity keep an
//! empty detail, and the `wait` and `leave` events keep the detail they had.
//!
//! A transaction born in a generator leaves it for the next block: the trace
//! says so, with the block it goes to, so a reader can follow the
//! transaction from the place it was born.
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

use genmsi_core::program::{BlockKind, ExecBlock, FunctionDef};
use genmsi_core::{Program, RunResult, SimConfig, Simulation};

mod support;

use support::{block, linear, module, node, run, wired};

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
fn the_generator_publishes_the_leave_of_the_transaction_it_born() {
    let (program, result) = run_model();
    let generators: Vec<usize> = program
        .blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| block.class == "Generate")
        .map(|(index, _)| index)
        .collect();
    assert!(!generators.is_empty(), "the model has a Generate block");
    let mut checked = 0;
    for (at, event) in result.trace.iter().enumerate() {
        if event.event != "generate" {
            continue;
        }
        let generator = event.block;
        assert!(
            generators.contains(&generator),
            "the transaction is born in a Generate block: {generator}"
        );
        // The transaction that was born is the one that leaves.
        let (leave_at, leave) = result
            .trace
            .iter()
            .enumerate()
            .find(|(later, other)| {
                *later > at
                    && other.event == "leave"
                    && other.block == generator
                    && other.xact == event.xact
            })
            .unwrap_or_else(|| {
                panic!(
                    "the transaction {} leaves the generator {generator}",
                    event.xact
                )
            });
        let target: usize = leave
            .detail
            .strip_prefix("-> ")
            .unwrap_or_else(|| panic!("unexpected leave detail {:?}", leave.detail))
            .parse()
            .expect("the target is a block index");
        assert!(target < program.blocks.len(), "the target block exists");
        // The transaction goes on to the block the trace names.
        let enters = result.trace.iter().enumerate().any(|(later, other)| {
            later > leave_at
                && other.event == "enter"
                && other.block == target
                && other.xact == event.xact
        });
        assert!(
            enters,
            "the transaction {} enters the block {target} it left the generator for",
            event.xact
        );
        checked += 1;
    }
    assert!(checked > 0, "the model generates transactions");
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

// The `Preempt` and `Return` classes: the facility they interrupt and the
// one they give back.
//
// The benches of the two classes (`scripts/wine-oracle/benches/Preempt.gmdl`
// and `Return.gmdl`) are the models the wine oracle runs against the
// original, so the report of the original says what the classes do.

/// Compile the bench of a class: the model the oracle runs.
fn bench(name: &str) -> Program {
    let path = root().join(format!("scripts/wine-oracle/benches/{name}.gmdl"));
    let model = genmsi_format::load(&path).expect("the bench loads");
    genmsi_core::compile_model(&model).expect("the bench compiles")
}

/// Run a bench for the length of the original's run (a clock of one hour).
fn run_bench(program: &Program) -> RunResult {
    Simulation::new(
        program,
        SimConfig {
            seed: 1,
            stop_time: Some(3600.0),
            trace: true,
            ..SimConfig::default()
        },
    )
    .run()
    .expect("the bench runs")
}

/// The number of transactions that entered the block with that label.
fn entries(program: &Program, result: &RunResult, label: &str) -> u64 {
    let index = program
        .blocks
        .iter()
        .position(|block| block.label.as_deref() == Some(label))
        .unwrap_or_else(|| panic!("no block labelled {label}"));
    result.blocks[index].entries
}

#[test]
fn the_preempt_class_takes_the_facility_and_gives_it_back() {
    // The report of the original for this bench:
    //
    //     1  60  GENERATE 60,30
    //     2  59  A_Preempt   PREEMPT Recurso,PR,,Remanente
    //     3  59  A_Return    RELEASE Recurso
    //     4  59  A_Terminate TERMINATE
    //     FACILITY RECURSO: 59 entries, 0,000 of utilization
    //
    // The generator draws its arrivals at random, so the number of
    // transactions the run creates differs by one between the original and
    // us; everything the class does must agree.
    let program = bench("Preempt");
    let result = run_bench(&program);

    let preempted = entries(&program, &result, "A_Preempt");
    assert!(preempted > 0, "the bench interrupts a facility");
    assert_eq!(
        entries(&program, &result, "A_Return"),
        preempted,
        "every transaction that interrupts the facility gives it back"
    );
    assert_eq!(
        entries(&program, &result, "A_Terminate"),
        preempted,
        "every transaction that interrupts the facility goes on"
    );

    // The facility is taken and given back: one entry per transaction and no
    // time busy, as the original's report says.
    let facility = &result.entities.facilities["Recurso"];
    assert_eq!(facility.entries, preempted, "the facility is taken once each");
    assert_eq!(facility.busy_time, 0.0, "the facility is given back at once");
    assert!(facility.available, "the facility is free at the end");

    // The trace says which facility the block touches.
    let detail = result
        .trace
        .iter()
        .find(|event| event.event == "enter" && event.detail == "Recurso")
        .expect("the trace publishes the facility of the block");
    assert!(detail.detail.contains("Recurso"));
}

#[test]
fn the_return_class_is_the_release_of_the_facility() {
    // The report of the original for this bench:
    //
    //     2  60  A_Seize     SEIZE Recurso
    //     3  60  A_Return    RELEASE Recurso
    //     4  60  A_Terminate TERMINATE
    //     FACILITY RECURSO: 60 entries, 0,000 of utilization
    //
    // and its GPSS: `A_Return RELEASE Recurso`. The macro code of the class
    // is a release, not the GPSS `RETURN` of the `FacilityInt` class.
    let program = bench("Return");
    let release = program
        .blocks
        .iter()
        .find(|block| block.class == "Release")
        .expect("the Return class compiles to a release");
    assert!(
        matches!(release.kind, BlockKind::Release(_)),
        "the Return class is a release"
    );

    let result = run_bench(&program);
    let seized = entries(&program, &result, "A_Seize");
    assert!(seized > 0, "the bench takes a facility");
    assert_eq!(
        entries(&program, &result, "A_Return"),
        seized,
        "every transaction that takes the facility gives it back"
    );
    assert_eq!(
        entries(&program, &result, "A_Terminate"),
        seized,
        "every transaction that takes the facility goes on"
    );
    let facility = &result.entities.facilities["Recurso"];
    assert_eq!(facility.entries, seized, "the facility is taken once each");
    assert_eq!(facility.busy_time, 0.0, "the facility is given back at once");
    assert!(facility.available, "the facility is free at the end");
}

/// A model where one transaction takes a facility and another tries to
/// interrupt it: the priority of the newcomer says whether it may.
fn preemption_model(newcomer_priority: &'static str) -> Program {
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A"))],
            &[("Media", "1"), ("Modificador", "0"), ("Prioridad", "0")],
        ),
        node(
            2,
            "Seize",
            Some("A"),
            &[("Salida1", Some("B"))],
            &[("Facility", "Maquina")],
        ),
        node(
            3,
            "Advance",
            Some("B"),
            &[("Salida1", Some("C"))],
            &[("Media", "10"), ("Modificador", "0")],
        ),
        node(
            4,
            "Release",
            Some("C"),
            &[("Salida1", Some("D"))],
            &[("Facility", "Maquina")],
        ),
        node(
            5,
            "Generate",
            None,
            &[("Salida1", Some("E"))],
            &[("Media", "1"), ("Modificador", "0"), ("Prioridad", "0")],
        ),
        node(
            6,
            "Priority",
            Some("E"),
            &[("Salida1", Some("F"))],
            &[("Prioridad", newcomer_priority)],
        ),
        node(
            7,
            "Preempt",
            Some("F"),
            &[("Salida1", Some("G"))],
            &[("Facility", "Maquina"), ("Remanente", "Remanente")],
        ),
        node(8, "Terminate", Some("G"), &[], &[("Unidades", "0")]),
    ];
    let links = [
        (1, 0, 2, 0),
        (2, 0, 3, 0),
        (3, 0, 4, 0),
        (5, 0, 6, 0),
        (6, 0, 7, 0),
        (7, 0, 8, 0),
    ];
    wired(&nodes, &links, 3600.0)
}

#[test]
fn the_preempt_class_gives_the_facility_to_the_transaction_that_outranks_the_owner() {
    // The `PR` operand of the original: the preempting transaction's own
    // priority decides whether it may take the facility.
    let program = preemption_model("1");
    let result = run(&program, 100.0, 1);
    let facility = &result.entities.facilities["Maquina"];
    assert!(
        facility.inter > 0,
        "the transaction with the higher priority interrupts the owner"
    );
    assert!(
        facility.entries > 1,
        "the facility is taken again by every transaction that interrupts it"
    );
}

#[test]
fn the_preempt_class_leaves_the_facility_to_the_owner_that_outranks_the_newcomer() {
    // The owner keeps the facility when it has the priority: the newcomer
    // waits for it, as a `SEIZE` that finds the facility busy does.
    let program = preemption_model("0");
    let result = run(&program, 100.0, 1);
    let facility = &result.entities.facilities["Maquina"];
    assert_eq!(
        facility.inter, 0,
        "the transaction with the same priority does not interrupt the owner"
    );
    let index = program
        .blocks
        .iter()
        .position(|block| block.class == "Preempt")
        .expect("the preempt block is in the program");
    assert!(
        result.blocks[index].retries > 0,
        "the newcomer waits for the facility"
    );
}

/// A model where a transaction takes a facility and another interrupts it:
/// the owner goes on to the interrupt output of the `Preempt` block, unless
/// the class says the owner is removed.
fn interrupted_owner_model(remove_owner: bool) -> Program {
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A"))],
            &[("Media", "10"), ("Modificador", "0"), ("Prioridad", "0")],
        ),
        node(
            2,
            "Seize",
            Some("A"),
            &[("Salida1", Some("B"))],
            &[("Facility", "Maquina")],
        ),
        node(
            3,
            "Advance",
            Some("B"),
            &[("Salida1", Some("C"))],
            &[("Media", "10"), ("Modificador", "0")],
        ),
        node(
            4,
            "Release",
            Some("C"),
            &[("Salida1", Some("D"))],
            &[("Facility", "Maquina")],
        ),
        node(
            5,
            "Generate",
            None,
            &[("Salida1", Some("E"))],
            &[("Media", "10"), ("Modificador", "0"), ("Prioridad", "0")],
        ),
        node(
            6,
            "Priority",
            Some("E"),
            &[("Salida1", Some("F"))],
            &[("Prioridad", "1")],
        ),
        node(
            7,
            "Preempt",
            Some("F"),
            &[("Salida1", Some("G")), ("SalidaAlt", Some("ALT"))],
            &[
                ("Facility", "Maquina"),
                ("Remanente", "Remanente"),
                ("ModoArrebato", if remove_owner { "Remover" } else { "NoRemover" }),
            ],
        ),
        node(
            8,
            "Release",
            Some("G"),
            &[("Salida1", Some("H"))],
            &[("Facility", "Maquina")],
        ),
        node(9, "Terminate", Some("H"), &[], &[("Unidades", "0")]),
        node(10, "Terminate", Some("ALT"), &[], &[("Unidades", "0")]),
    ];
    let links = [
        (1, 0, 2, 0),
        (2, 0, 3, 0),
        (3, 0, 4, 0),
        (5, 0, 6, 0),
        (6, 0, 7, 0),
        (7, 0, 8, 0),
        (7, 1, 10, 0),
    ];
    wired(&nodes, &links, 3600.0)
}

#[test]
fn the_preempt_class_sends_the_owner_it_interrupts_to_its_own_output() {
    // The macro code of the original routes the owner of the interrupted
    // facility to the `SalidaAlt` output (`Propietaria`).
    let program = interrupted_owner_model(false);
    let result = run(&program, 100.0, 1);
    let facility = &result.entities.facilities["Maquina"];
    assert!(facility.inter > 0, "the owner is interrupted");
    assert_eq!(
        entries(&program, &result, "ALT"),
        facility.inter,
        "every interrupted owner goes on to the interrupt output"
    );
    assert_eq!(
        entries(&program, &result, "C"),
        0,
        "the interrupted owner does not carry on with its own work"
    );
}

#[test]
fn the_preempt_class_removes_the_owner_it_interrupts_when_the_class_says_so() {
    // The `ModoArrebato` of the original: `Remover` takes the owner out of
    // the model instead of routing it.
    let program = interrupted_owner_model(true);
    let result = run(&program, 100.0, 1);
    let facility = &result.entities.facilities["Maquina"];
    assert!(facility.inter > 0, "the owner is interrupted");
    assert_eq!(
        entries(&program, &result, "ALT"),
        0,
        "the removed owner goes nowhere"
    );
    assert_eq!(
        entries(&program, &result, "C"),
        0,
        "the removed owner does not carry on with its own work"
    );
}

// The classes of the original that work with the entities of the model: the
// `Mark` class (the hour goes to a parameter), the `RemoveMe` class (the
// transaction leaves its group) and the three selections (a logic switch, a
// storage and an entity of a class).

#[test]
fn the_mark_class_writes_the_hour_in_the_parameter() {
    // The macro code of the original: `MARK [Parametro:'Parametro donde se
    // guarda la hora']`. The class writes the time of the model in the
    // parameter and lets the transaction go on.
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("T1"))],
            &[("Media", "10"), ("Modificador", "0")],
        ),
        node(
            2,
            "Mark",
            Some("T1"),
            &[("Salida1", Some("T2"))],
            &[("Parametro", "Param")],
        ),
        node(
            3,
            "SaveValue",
            Some("T2"),
            &[("Salida1", Some("T3"))],
            &[("SaveValue", "V"), ("Op", "="), ("Valor", "P:Param")],
        ),
        node(4, "Terminate", Some("T3"), &[], &[("Unidades", "0")]),
    ];
    let links = [(1, 0, 2, 0), (2, 0, 3, 0), (3, 0, 4, 0)];
    let program = wired(&nodes, &links, 400.0);
    let result = Simulation::new(
        &program,
        SimConfig {
            seed: 1,
            stop_time: Some(400.0),
            trace: true,
            ..SimConfig::default()
        },
    )
    .run()
    .expect("runs");

    // The value published is the hour of the transaction: the time the trace
    // records and the value the parameter holds are the same.
    let values: Vec<(f64, f64)> = result
        .trace
        .iter()
        .filter(|event| event.event == "value")
        .map(|event| {
            let value = event
                .detail
                .rsplit_once(' ')
                .map(|(_, value)| value)
                .expect("the detail carries the value")
                .parse::<f64>()
                .expect("the value is a number");
            (event.time, value)
        })
        .collect();
    assert!(!values.is_empty(), "the trace publishes the value");
    for (time, value) in values {
        assert!(
            (time - value).abs() < 1e-9,
            "the parameter holds the hour of the transaction: {time} and {value}"
        );
    }
}

#[test]
fn the_remove_me_class_takes_the_transaction_out_of_its_group() {
    // The class of the original removes the transaction from the group it
    // names; the examination that follows finds it in no group.
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("T1"))],
            &[("Media", "1"), ("Modificador", "0")],
        ),
        node(
            2,
            "JoinMe",
            Some("T1"),
            &[("Salida1", Some("T2"))],
            &[("Grupo", "UnGrupo")],
        ),
        node(
            3,
            "RemoveMe",
            Some("T2"),
            &[("Salida1", Some("T3")), ("Salida2", None)],
            &[("Grupo", "UnGrupo")],
        ),
        node(
            4,
            "Examine",
            Some("T3"),
            &[("Salida1", Some("YES")), ("Salida2", Some("NO"))],
            &[("Grupo", "UnGrupo")],
        ),
        node(5, "Terminate", Some("YES"), &[], &[("Unidades", "0")]),
        node(6, "Terminate", Some("NO"), &[], &[("Unidades", "0")]),
    ];
    let links = [
        (1, 0, 2, 0),
        (2, 0, 3, 0),
        (3, 0, 4, 0),
        (4, 0, 5, 0),
        (4, 1, 6, 0),
    ];
    let program = wired(&nodes, &links, 400.0);
    let result = run(&program, 400.0, 1);
    let in_group = entries(&program, &result, "YES");
    let out_group = entries(&program, &result, "NO");
    assert!(out_group > 100, "the bench creates transactions: {out_group}");
    assert_eq!(
        in_group, 0,
        "the transaction left the group before the examination"
    );
}

#[test]
fn the_select_l_class_picks_the_logic_switch_that_is_set() {
    // The class of the original selects a logic switch: `SELECT LS
    // param,1,10`. The number of the switch goes to the parameter.
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("T1"))],
            &[("Media", "1"), ("Modificador", "0")],
        ),
        node(
            2,
            "Logic",
            Some("T1"),
            &[("Salida1", Some("T2"))],
            &[("LogicSwich", "3"), ("Op", "Set")],
        ),
        node(
            3,
            "SelectL",
            Some("T2"),
            &[("Salida1", Some("T3")), ("Salida2", None)],
            &[
                ("Param", "param"),
                ("minimo", "1"),
                ("maximo", "10"),
                ("op", "Set"),
            ],
        ),
        node(
            4,
            "Test",
            Some("T3"),
            &[("Salida1", Some("YES")), ("Salida2", Some("NO"))],
            &[("1_Valor", "P:param"), ("2_OP", "="), ("3_Valor", "3")],
        ),
        node(5, "Terminate", Some("YES"), &[], &[("Unidades", "0")]),
        node(6, "Terminate", Some("NO"), &[], &[("Unidades", "0")]),
    ];
    let links = [
        (1, 0, 2, 0),
        (2, 0, 3, 0),
        (3, 0, 4, 0),
        (4, 0, 5, 0),
        (4, 1, 6, 0),
    ];
    let program = wired(&nodes, &links, 400.0);
    let result = run(&program, 400.0, 1);
    assert!(
        entries(&program, &result, "YES") > 100,
        "the switch that is set is the one the class picks"
    );
    assert_eq!(
        entries(&program, &result, "NO"),
        0,
        "every transaction picks the same switch"
    );
}

#[test]
fn the_select_s_class_picks_the_storage_that_is_not_full() {
    // `SELECT SNF param,1,10`: the class picks a storage that has room.
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("T1"))],
            &[("Media", "1"), ("Modificador", "0")],
        ),
        node(
            2,
            "SelectS",
            Some("T1"),
            &[("Salida1", Some("T2")), ("Salida2", None)],
            &[
                ("Param", "param"),
                ("minimo", "1"),
                ("maximo", "10"),
                ("op", "NoLleno"),
            ],
        ),
        node(
            3,
            "Test",
            Some("T2"),
            &[("Salida1", Some("YES")), ("Salida2", Some("NO"))],
            &[("1_Valor", "P:param"), ("2_OP", ">="), ("3_Valor", "1")],
        ),
        node(4, "Terminate", Some("YES"), &[], &[("Unidades", "0")]),
        node(5, "Terminate", Some("NO"), &[], &[("Unidades", "0")]),
    ];
    let links = [(1, 0, 2, 0), (2, 0, 3, 0), (3, 0, 4, 0), (3, 1, 5, 0)];
    let program = wired(&nodes, &links, 400.0);
    let result = run(&program, 400.0, 1);
    assert!(
        entries(&program, &result, "YES") > 100,
        "the class picks a storage"
    );
    assert_eq!(
        entries(&program, &result, "NO"),
        0,
        "every transaction picks a storage"
    );
}

#[test]
fn the_select_sna_class_picks_the_entity_whose_value_is_the_compared_one() {
    // `SELECT E param,1,10,*1,Q,`: the class picks the entity of the class
    // whose value is the one that is compared against.
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("T1"))],
            &[("Media", "1"), ("Modificador", "0")],
        ),
        node(
            2,
            "SelectSNA",
            Some("T1"),
            &[("Salida1", Some("T2")), ("Salida2", None)],
            &[
                ("ClaseSNA", "Q"),
                ("Param", "param"),
                ("minimo", "1"),
                ("maximo", "10"),
                ("op", "="),
                ("valorComp", "0"),
            ],
        ),
        node(
            3,
            "Test",
            Some("T2"),
            &[("Salida1", Some("YES")), ("Salida2", Some("NO"))],
            &[("1_Valor", "P:param"), ("2_OP", "="), ("3_Valor", "1")],
        ),
        node(4, "Terminate", Some("YES"), &[], &[("Unidades", "0")]),
        node(5, "Terminate", Some("NO"), &[], &[("Unidades", "0")]),
    ];
    let links = [(1, 0, 2, 0), (2, 0, 3, 0), (3, 0, 4, 0), (3, 1, 5, 0)];
    let program = wired(&nodes, &links, 400.0);
    let result = run(&program, 400.0, 1);
    assert!(
        entries(&program, &result, "YES") > 100,
        "the class picks the entity"
    );
    assert_eq!(
        entries(&program, &result, "NO"),
        0,
        "every transaction picks the same entity"
    );
}

#[test]
fn the_select_classes_go_on_when_there_is_nothing_to_pick() {
    // The report of the original for the benches of the three classes shows
    // every transaction going on by the first output, although there is
    // nothing to select: no logic switch is set, no storage exists and the
    // queue the class looks at is empty.
    for (class, props) in [
        (
            "SelectL",
            vec![
                ("Param", "param"),
                ("minimo", "1"),
                ("maximo", "10"),
                ("op", "Set"),
            ],
        ),
        (
            "SelectS",
            vec![
                ("Param", "param"),
                ("minimo", "1"),
                ("maximo", "10"),
                ("op", "NoLleno"),
            ],
        ),
        (
            "SelectSNA",
            vec![
                ("ClaseSNA", "Q"),
                ("Param", "param"),
                ("minimo", "1"),
                ("maximo", "10"),
                ("op", "="),
                ("valorComp", "1"),
            ],
        ),
    ] {
        let nodes = [
            node(
                1,
                "Generate",
                None,
                &[("Salida1", Some("T1"))],
                &[("Media", "1"), ("Modificador", "0")],
            ),
            node(
                2,
                class,
                Some("T1"),
                &[("Salida1", Some("T2")), ("Salida2", Some("T3"))],
                &props,
            ),
            node(3, "Terminate", Some("T2"), &[], &[("Unidades", "0")]),
            node(4, "Terminate", Some("T3"), &[], &[("Unidades", "0")]),
        ];
        let links = [(1, 0, 2, 0), (2, 0, 3, 0), (2, 1, 4, 0)];
        let program = wired(&nodes, &links, 400.0);
        let result = run(&program, 400.0, 1);
        assert!(
            entries(&program, &result, "T2") > 100,
            "{class}: every transaction goes on by the first output"
        );
        assert_eq!(
            entries(&program, &result, "T3"),
            0,
            "{class}: the second output is the one of a selection that failed"
        );
    }
}

#[test]
fn the_variable_n_class_defines_a_normal_variable() {
    // The class creates the standard normal distribution with the seed it
    // names and the variable that reads it: `60+(1#FN$NORMAL)/10`. The
    // variable is a normal one around its mean, not a constant.
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("T1"))],
            &[("Media", "1"), ("Modificador", "0")],
        ),
        node(
            5,
            "VariableN",
            None,
            &[],
            &[
                ("Media", "60"),
                ("Desvio", "1"),
                ("NombreFunc", "NORMAL"),
                ("NombreVar", "Normal"),
                ("Seed", "90"),
            ],
        ),
        node(
            2,
            "Test",
            Some("T1"),
            &[("Salida1", Some("YES")), ("Salida2", Some("NO"))],
            &[("1_Valor", "V:Normal"), ("2_OP", ">"), ("3_Valor", "60")],
        ),
        node(3, "Terminate", Some("YES"), &[], &[("Unidades", "0")]),
        node(4, "Terminate", Some("NO"), &[], &[("Unidades", "0")]),
    ];
    let links = [(1, 0, 2, 0), (2, 0, 3, 0), (2, 1, 4, 0)];
    let program = wired(&nodes, &links, 4000.0);

    // The class defines the function it reads and the variable itself.
    let function = program
        .definitions
        .functions
        .get("NORMAL")
        .expect("the class creates its function");
    assert!(
        matches!(function, FunctionDef::Continuous { .. }),
        "the function is the continuous distribution of the normal variable"
    );
    assert!(
        program.definitions.variables.contains_key("Normal"),
        "the class creates its variable"
    );

    let result = run(&program, 4000.0, 1);
    let above = entries(&program, &result, "YES");
    let below = entries(&program, &result, "NO");
    assert!(above > 100, "the bench creates transactions: {above}");
    // The table of the original is the one of the standard normal
    // distribution, ten times the quantile: the variable is above its mean
    // about six times out of ten (the table of the original has its pair
    // `0.44,2` where the quantile is negative).
    let share = above as f64 / (above + below) as f64;
    assert!(
        (0.45..0.75).contains(&share),
        "the variable is a normal one around its mean, measured {share:.3}"
    );
}
