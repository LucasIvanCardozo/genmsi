//! The `enter` trace event must publish the entity a block touches.
//!
//! A queue named by a parameter (`*caja`) or by an expression is only known
//! at run time, so the trace has to carry the resolved name. Blocks that
//! touch no entity keep an empty detail, and the `wait` and `leave` events
//! keep the detail they had.

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
