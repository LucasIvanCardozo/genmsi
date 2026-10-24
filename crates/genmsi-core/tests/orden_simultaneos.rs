//! The order of the events of the same instant: the rule of the original.
//!
//! The original (GenMSI 1.2020.07.28 under Wine) keeps its time in whole
//! units, so many events share an instant and the order inside the instant
//! decides who takes a facility and who is preempted.  The rule was
//! measured against the original and is written down in
//! `odd/tasks/notas-orden-simultaneos.md`:
//!
//! - the events of an instant are processed by priority, from the highest to
//!   the lowest;
//! - the ones of the same priority are processed the other way round: the
//!   last one scheduled is the first one processed;
//! - the pending arrival of a generator carries the priority of the
//!   transaction it will create, so a pending arrival of priority 0 does not
//!   get ahead of the events of the transactions of a higher priority.
//!
//! The trace says in which order the transactions are processed, and that is
//! what these tests pin.

mod support;

use genmsi_core::{Program, RunResult, SimConfig, Simulation};

use support::{node, wired};

/// Run a model with the trace on, so the order of the events is visible.
fn run_with_trace(program: &Program) -> RunResult {
    Simulation::new(
        program,
        SimConfig {
            seed: 1,
            stop_time: Some(100.0),
            trace: true,
            trace_limit: 100_000,
            max_events: 5_000_000,
            start: 1,
            stop: None,
        },
    )
    .run()
    .expect("runs")
}

/// The labels of the blocks the transactions entered, in the order they
/// entered them.
fn entered(result: &RunResult) -> Vec<String> {
    result
        .trace
        .iter()
        .filter(|event| event.event == "enter")
        .map(|event| event.label.clone().unwrap_or_default())
        .collect()
}

/// The labels of the blocks the transactions entered in the given instant, in
/// the order they entered them.
fn entered_at(result: &RunResult, time: f64) -> Vec<String> {
    result
        .trace
        .iter()
        .filter(|event| event.event == "enter" && event.time == time)
        .map(|event| event.label.clone().unwrap_or_default())
        .collect()
}

/// The label of the first block of the given class that a transaction
/// entered.
fn first_entered_of(result: &RunResult, class: &str) -> String {
    result
        .trace
        .iter()
        .find(|event| event.event == "enter" && event.class == class)
        .map(|event| event.label.clone().unwrap_or_default())
        .unwrap_or_default()
}

/// Two generators that are born in the same instant, each one with its own
/// chain, so the trace says in which order they are processed.
///
/// The first generator is the one with the label `A` and the second the one
/// with the label `C`.
fn two_generators(priority: &'static str) -> Program {
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A"))],
            &[("Media", "10"), ("Modificador", "0"), ("Prioridad", priority)],
        ),
        node(
            2,
            "Assign",
            Some("A"),
            &[("Salida1", Some("B"))],
            &[("Parametro", "P1"), ("Valor", "1")],
        ),
        node(
            3,
            "Generate",
            None,
            &[("Salida1", Some("C"))],
            &[("Media", "10"), ("Modificador", "0"), ("Prioridad", priority)],
        ),
        node(
            4,
            "Assign",
            Some("C"),
            &[("Salida1", Some("D"))],
            &[("Parametro", "P2"), ("Valor", "2")],
        ),
    ];
    let links = [(1, 0, 2, 0), (3, 0, 4, 0)];
    wired(&nodes, &links, 3600.0)
}

#[test]
fn the_last_scheduled_event_of_an_instant_is_the_first_one_processed() {
    // The two transactions are born in the same instant (the two generators
    // have the same mean) and they have the same priority, so the tie-break
    // of the original decides: the last one scheduled is the first one
    // processed.  The generator that comes last in the program is the one
    // scheduled last, so its transaction is born first.
    let program = two_generators("0");
    let result = run_with_trace(&program);
    let order = entered(&result);
    assert_eq!(
        order.first().map(String::as_str),
        Some("C"),
        "the transaction of the generator that comes last is processed first: {order:?}"
    );
}

#[test]
fn the_pending_arrival_of_a_generator_carries_the_priority_of_its_transaction() {
    // The pending arrival of a generator competes in the queue of events
    // with the events of the transactions that are already running, so it
    // carries the priority of the transaction it will create.
    //
    // The model has a transaction of priority 5 that resumes from an advance
    // in the instant 10, and a generator of priority 10 whose pending arrival
    // is also in the instant 10: the pending arrival is processed first,
    // because its priority is the priority of the transaction it will
    // create.
    //
    // Without that priority the pending arrival would be an event of
    // priority 0, and the resume of the transaction of priority 5 would go
    // before it.
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A"))],
            &[("Media", "10"), ("Modificador", "0"), ("Prioridad", "10")],
        ),
        node(
            2,
            "Assign",
            Some("A"),
            &[("Salida1", Some("B"))],
            &[("Parametro", "P1"), ("Valor", "1")],
        ),
        node(
            3,
            "Generate",
            None,
            &[("Salida1", Some("C"))],
            &[("Media", "5"), ("Modificador", "0"), ("Prioridad", "0")],
        ),
        node(
            4,
            "Priority",
            Some("C"),
            &[("Salida1", Some("D"))],
            &[("Prioridad", "5")],
        ),
        node(
            5,
            "Advance",
            Some("D"),
            &[("Salida1", Some("E"))],
            &[("Media", "5"), ("Modificador", "0")],
        ),
        node(
            6,
            "Assign",
            Some("E"),
            &[("Salida1", Some("F"))],
            &[("Parametro", "P2"), ("Valor", "2")],
        ),
    ];
    let links = [(1, 0, 2, 0), (3, 0, 4, 0), (4, 0, 5, 0), (5, 0, 6, 0)];
    let program = wired(&nodes, &links, 3600.0);
    let result = run_with_trace(&program);
    let order = entered_at(&result, 10.0);
    assert_eq!(
        order.first().map(String::as_str),
        Some("A"),
        "the pending arrival of priority 10 is processed before the event of the transaction of priority 5: {order:?}"
    );
}

#[test]
fn the_transactions_of_the_same_instant_and_priority_take_the_facility_in_the_reverse_order() {
    // The rule decides who takes a facility that is free: the two
    // transactions are born in the same instant, they have the same priority
    // and they ask for the same facility, so the one that is processed first
    // takes it and the other one waits.
    //
    // This is the probe `dosnuevos` of the note: the original gives the
    // facility to the transaction of the generator that comes last in the
    // model.
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
            &[("Facility", "Recurso")],
        ),
        node(
            3,
            "Generate",
            None,
            &[("Salida1", Some("C"))],
            &[("Media", "10"), ("Modificador", "0"), ("Prioridad", "0")],
        ),
        node(
            4,
            "Seize",
            Some("C"),
            &[("Salida1", Some("D"))],
            &[("Facility", "Recurso")],
        ),
    ];
    let links = [(1, 0, 2, 0), (3, 0, 4, 0)];
    let program = wired(&nodes, &links, 3600.0);
    let result = run_with_trace(&program);
    assert_eq!(
        first_entered_of(&result, "Seize"),
        "C",
        "the transaction that is processed first takes the facility"
    );
}
