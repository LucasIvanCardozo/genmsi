//! The order of the chain of delay: who takes the resource first.
//!
//! The chain of delay of a facility is served by priority, descending, and
//! among the waiters of the same priority by the order in which they
//! arrived (`odd/tasks/notas-orden-simultaneos.md` measures it against the
//! original: the `fifo` and `prioridadesCola` probes).
//!
//! The chain of a storage is served the other way round among the waiters
//! of the same priority: the last one that was programmed is the first one
//! served, because the original takes the events of an instant in the
//! reverse order of their programming and the chain of a storage is served
//! by the events the engine programs for it. That order was never measured
//! against the original (the note says so), so this test pins the order the
//! engine has, and a change to it has to be deliberate.

mod support;

use genmsi_core::{Program, RunResult, SimConfig, Simulation};
use support::{node, wired};

/// One occupier and two waiters.
///
/// The occupier takes the facility at t=1 and holds it until t=501, so both
/// waiters are in the chain of delay before it frees. The first waiter
/// arrives at t=100 and the second one at t=101; the priorities are the ones
/// the caller asks for.
fn facility_model(first: &'static str, second: &'static str) -> Program {
    let nodes = [
        // The occupier: it takes the facility and holds it.
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("O1"))],
            &[("Media", "1"), ("Modificador", "0"), ("Cantidad", "1")],
        ),
        node(2, "Seize", Some("O1"), &[("Salida1", Some("O2"))], &[("Facility", "F")]),
        node(
            3,
            "Advance",
            Some("O2"),
            &[("Salida1", Some("O3"))],
            &[("Media", "500"), ("Modificador", "0")],
        ),
        node(4, "Release", Some("O3"), &[("Salida1", Some("O4"))], &[("Facility", "F")]),
        node(5, "Terminate", Some("O4"), &[], &[("Unidades", "0")]),
        // The first waiter: it arrives at t=100.
        node(
            6,
            "Generate",
            None,
            &[("Salida1", Some("A1"))],
            &[("Media", "100"), ("Modificador", "0"), ("Cantidad", "1")],
        ),
        node(
            7,
            "Priority",
            Some("A1"),
            &[("Salida1", Some("A2"))],
            &[("Prioridad", first)],
        ),
        node(8, "Seize", Some("A2"), &[("Salida1", Some("A3"))], &[("Facility", "F")]),
        node(
            9,
            "Advance",
            Some("A3"),
            &[("Salida1", Some("A4"))],
            &[("Media", "5"), ("Modificador", "0")],
        ),
        node(10, "Release", Some("A4"), &[("Salida1", Some("A5"))], &[("Facility", "F")]),
        node(11, "Terminate", Some("A5"), &[], &[("Unidades", "0")]),
        // The second waiter: it arrives at t=101.
        node(
            12,
            "Generate",
            None,
            &[("Salida1", Some("B1"))],
            &[("Media", "101"), ("Modificador", "0"), ("Cantidad", "1")],
        ),
        node(
            13,
            "Priority",
            Some("B1"),
            &[("Salida1", Some("B2"))],
            &[("Prioridad", second)],
        ),
        node(14, "Seize", Some("B2"), &[("Salida1", Some("B3"))], &[("Facility", "F")]),
        node(
            15,
            "Advance",
            Some("B3"),
            &[("Salida1", Some("B4"))],
            &[("Media", "5"), ("Modificador", "0")],
        ),
        node(16, "Release", Some("B4"), &[("Salida1", Some("B5"))], &[("Facility", "F")]),
        node(17, "Terminate", Some("B5"), &[], &[("Unidades", "0")]),
    ];
    let links = [
        (1, 0, 2, 0),
        (2, 0, 3, 0),
        (3, 0, 4, 0),
        (4, 0, 5, 0),
        (6, 0, 7, 0),
        (7, 0, 8, 0),
        (8, 0, 9, 0),
        (9, 0, 10, 0),
        (10, 0, 11, 0),
        (12, 0, 13, 0),
        (13, 0, 14, 0),
        (14, 0, 15, 0),
        (15, 0, 16, 0),
        (16, 0, 17, 0),
    ];
    wired(&nodes, &links, 1000.0)
}

/// The instant a waiter takes the facility.
///
/// The waiter that was waiting leaves the block it is counted in and enters
/// the block that follows the seizure: the instant it enters that block is
/// the instant it took the facility.
fn admission(result: &RunResult, program: &Program, label: &str) -> f64 {
    let block = support::block(program, label);
    result
        .trace
        .iter()
        .find(|event| event.block == block && event.event == "enter")
        .map(|event| event.time)
        .unwrap_or_else(|| panic!("the waiter of the block {label} never took the facility"))
}

/// A waiter of a higher priority takes the facility before the one that was
/// waiting before it.
///
/// The original serves the chain of delay by priority: the `prioridadesCola`
/// probe of `odd/tasks/notas-orden-simultaneos.md` says that the waiter of
/// priority 10 took the facility before the one of priority 5, even though
/// the one of priority 5 had arrived first.
#[test]
fn the_waiter_of_the_higher_priority_takes_the_facility_first() {
    let program = facility_model("0", "1");
    let result = Simulation::new(
        &program,
        SimConfig {
            seed: 7,
            stop_time: Some(1000.0),
            trace: true,
            ..SimConfig::default()
        },
    )
    .run()
    .expect("runs");
    let first = admission(&result, &program, "A3");
    let second = admission(&result, &program, "B3");
    assert_eq!(first, 506.0, "the waiter of priority 0 waits for the other one");
    assert_eq!(second, 501.0, "the waiter of priority 1 takes the facility first");
}

/// Among the waiters of the same priority, the one that arrived first is
/// served first.
///
/// The `fifo` probe of `odd/tasks/notas-orden-simultaneos.md` says it: three
/// waiters of the same priority took the facility in the order in which they
/// arrived.
#[test]
fn among_the_waiters_of_the_same_priority_the_one_that_arrived_first_is_served_first() {
    let program = facility_model("0", "0");
    let result = Simulation::new(
        &program,
        SimConfig {
            seed: 7,
            stop_time: Some(1000.0),
            trace: true,
            ..SimConfig::default()
        },
    )
    .run()
    .expect("runs");
    let first = admission(&result, &program, "A3");
    let second = admission(&result, &program, "B3");
    assert_eq!(first, 501.0, "the waiter that arrived first takes the facility first");
    assert_eq!(second, 506.0, "the waiter that arrived second waits for the first one");
}

/// One occupier and two waiters of a storage of one unit.
///
/// The occupier takes the storage at t=1 and holds it until t=501, so both
/// waiters are in the chain of delay before it frees.
fn storage_model() -> Program {
    let nodes = [
        // The occupier: it takes the storage and holds it.
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("O1"))],
            &[("Media", "1"), ("Modificador", "0"), ("Cantidad", "1")],
        ),
        node(
            2,
            "Enter",
            Some("O1"),
            &[("Salida1", Some("O2"))],
            &[("Storage", "S"), ("Cantidad", "1")],
        ),
        node(
            3,
            "Advance",
            Some("O2"),
            &[("Salida1", Some("O3"))],
            &[("Media", "500"), ("Modificador", "0")],
        ),
        node(
            4,
            "Leave",
            Some("O3"),
            &[("Salida1", Some("O4"))],
            &[("Storage", "S"), ("Cantidad", "1")],
        ),
        node(5, "Terminate", Some("O4"), &[], &[("Unidades", "0")]),
        // The first waiter: it arrives at t=100.
        node(
            6,
            "Generate",
            None,
            &[("Salida1", Some("A1"))],
            &[("Media", "100"), ("Modificador", "0"), ("Cantidad", "1")],
        ),
        node(
            7,
            "Enter",
            Some("A1"),
            &[("Salida1", Some("A2"))],
            &[("Storage", "S"), ("Cantidad", "1")],
        ),
        node(
            8,
            "Advance",
            Some("A2"),
            &[("Salida1", Some("A3"))],
            &[("Media", "5"), ("Modificador", "0")],
        ),
        node(
            9,
            "Leave",
            Some("A3"),
            &[("Salida1", Some("A4"))],
            &[("Storage", "S"), ("Cantidad", "1")],
        ),
        node(10, "Terminate", Some("A4"), &[], &[("Unidades", "0")]),
        // The second waiter: it arrives at t=101.
        node(
            11,
            "Generate",
            None,
            &[("Salida1", Some("B1"))],
            &[("Media", "101"), ("Modificador", "0"), ("Cantidad", "1")],
        ),
        node(
            12,
            "Enter",
            Some("B1"),
            &[("Salida1", Some("B2"))],
            &[("Storage", "S"), ("Cantidad", "1")],
        ),
        node(
            13,
            "Advance",
            Some("B2"),
            &[("Salida1", Some("B3"))],
            &[("Media", "5"), ("Modificador", "0")],
        ),
        node(
            14,
            "Leave",
            Some("B3"),
            &[("Salida1", Some("B4"))],
            &[("Storage", "S"), ("Cantidad", "1")],
        ),
        node(15, "Terminate", Some("B4"), &[], &[("Unidades", "0")]),
        // The storage: it holds one unit.
        node(16, "StorageDef", None, &[], &[("Storage", "S"), ("Tamaño", "1")]),
    ];
    let links = [
        (1, 0, 2, 0),
        (2, 0, 3, 0),
        (3, 0, 4, 0),
        (4, 0, 5, 0),
        (6, 0, 7, 0),
        (7, 0, 8, 0),
        (8, 0, 9, 0),
        (9, 0, 10, 0),
        (11, 0, 12, 0),
        (12, 0, 13, 0),
        (13, 0, 14, 0),
        (14, 0, 15, 0),
    ];
    wired(&nodes, &links, 1000.0)
}

/// The chain of delay of a storage serves the last waiter that was programmed
/// first.
///
/// The chain of a storage is served by the events the engine programs for
/// it, and the original takes the events of an instant in the reverse order
/// of their programming: among the waiters of the same priority the last one
/// that was programmed is the first one served. The note of the order of the
/// simultaneous events says that the chain of a storage was never measured
/// against the original, so this test pins the order the engine has.
#[test]
fn the_chain_of_a_storage_serves_the_last_programmed_waiter_first() {
    let program = storage_model();
    let result = Simulation::new(
        &program,
        SimConfig {
            seed: 7,
            stop_time: Some(1000.0),
            trace: true,
            ..SimConfig::default()
        },
    )
    .run()
    .expect("runs");
    let first = admission(&result, &program, "A2");
    let second = admission(&result, &program, "B2");
    assert_eq!(second, 501.0, "the waiter that was programmed last gets in first");
    assert_eq!(first, 506.0, "the waiter that was programmed first waits for it");
}
