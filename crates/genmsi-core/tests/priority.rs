//! Transaction priority: which waiter gets the resource first.

mod support;

use genmsi_core::{Program, SimConfig, Simulation};
use support::{node, wired};

/// One occupier plus two waiters with different priorities.
///
/// The occupier takes the facility at t=1 for 500 time units, so both waiters
/// (priority 0 at t=100, priority 1 at t=101) are queued before it frees.
fn model() -> Program {
    let nodes = [
        // Occupier.
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
        node(4, "Release", Some("O3"), &[("Salida1", None)], &[("Facility", "F")]),
        // Low priority waiter, arrives first.
        node(
            5,
            "Generate",
            None,
            &[("Salida1", Some("L1"))],
            &[("Media", "100"), ("Modificador", "0"), ("Cantidad", "1")],
        ),
        node(
            6,
            "Priority",
            Some("L1"),
            &[("Salida1", Some("L2"))],
            &[("Prioridad", "0")],
        ),
        node(7, "Seize", Some("L2"), &[("Salida1", Some("L3"))], &[("Facility", "F")]),
        node(
            8,
            "Advance",
            Some("L3"),
            &[("Salida1", Some("L4"))],
            &[("Media", "5"), ("Modificador", "0")],
        ),
        node(9, "Release", Some("L4"), &[("Salida1", Some("L5"))], &[("Facility", "F")]),
        node(10, "Terminate", Some("L5"), &[], &[("Unidades", "0")]),
        // High priority waiter, arrives one time unit later.
        node(
            11,
            "Generate",
            None,
            &[("Salida1", Some("H1"))],
            &[("Media", "101"), ("Modificador", "0"), ("Cantidad", "1")],
        ),
        node(
            12,
            "Priority",
            Some("H1"),
            &[("Salida1", Some("H2"))],
            &[("Prioridad", "1")],
        ),
        node(13, "Seize", Some("H2"), &[("Salida1", Some("H3"))], &[("Facility", "F")]),
        node(
            14,
            "Advance",
            Some("H3"),
            &[("Salida1", Some("H4"))],
            &[("Media", "5"), ("Modificador", "0")],
        ),
        node(15, "Release", Some("H4"), &[("Salida1", Some("H5"))], &[("Facility", "F")]),
        node(16, "Terminate", Some("H5"), &[], &[("Unidades", "0")]),
    ];
    let links = [
        (1, 0, 2, 0),
        (2, 0, 3, 0),
        (3, 0, 4, 0),
        (5, 0, 6, 0),
        (6, 0, 7, 0),
        (7, 0, 8, 0),
        (8, 0, 9, 0),
        (9, 0, 10, 0),
        (11, 0, 12, 0),
        (12, 0, 13, 0),
        (13, 0, 14, 0),
        (14, 0, 15, 0),
        (15, 0, 16, 0),
    ];
    wired(&nodes, &links, 1000.0)
}

/// A higher `PRIORITY` value is served first in this dialect.
///
/// Published GPSS orders transactions by ascending priority, but the original
/// GenMSI does the opposite and the oracle proves it: `TP3__aeropuerto` sets
/// `PRIORITY *pEquipaje`, which is 1 for the business class and 0 for everyone
/// else, and the baseline's baggage claim serves the 1 group first — that is the
/// only reading that reproduces both the 60/40 split at `Entrada13` and the
/// 1576-unit average wait of `QENTREGAEQUIPAJE` (the full population mix would
/// give 34% and 300).
#[test]
fn higher_priority_number_is_served_first() {
    let program = model();
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
    let low = support::block(&program, "L2");
    let high = support::block(&program, "H2");
    let first_leave = |block: usize| {
        result
            .trace
            .iter()
            .filter(|event| event.block == block && event.event == "leave")
            .map(|event| event.time)
            .next()
            .unwrap_or_else(|| panic!("no leave event for block {block}"))
    };
    let low_time = first_leave(low);
    let high_time = first_leave(high);
    assert!(
        high_time < low_time,
        "the priority-1 waiter must be admitted first (high {high_time}, low {low_time})"
    );
    assert_eq!(high_time, 501.0, "admitted as soon as the facility frees");
}
