//! A `DEPART` of a queue the transaction never entered.
//!
//! The departure is legal: a queue is created by use, and a model may
//! depart from a queue that nobody entered. The simulation counts it (the
//! departure leaves the queue with the wait it had, which is none).
//!
//! But the counters of the queue were left inconsistent: the departure
//! counted a zero wait, and the queue had **no entry** to subtract it from,
//! so the report divided by a number that went below zero — and the engine
//! panicked when it rendered the queue. A model that a person can write
//! must not take the engine down.

mod support;

use genmsi_core::report::render;
use support::{linear, module, run};

/// The model: a generator that departs from a queue it never entered.
fn model() -> genmsi_core::Program {
    linear(
        &[
            module("Generate", "GENERATE", &[("Cantidad", "1"), ("Media", "10")]),
            module("Depart", "DEPART", &[("Queue", "Q"), ("Cantidad", "1")]),
        ],
        100.0,
    )
}

#[test]
fn the_report_of_a_depart_of_a_queue_nobody_entered_is_written() {
    let program = model();
    let result = run(&program, 100.0, 1);
    // The report renders the queue: the render must not panic.
    let report = render(&result, &program);
    assert!(report.contains("Q"), "the queue is in the report: {report}");
}

#[test]
fn the_average_of_a_queue_nobody_entered_is_zero() {
    let program = model();
    let result = run(&program, 100.0, 1);
    let queue = result
        .entities
        .queues
        .iter()
        .find(|(name, _)| name.as_str() == "Q")
        .map(|(_, queue)| queue)
        .expect("the queue is in the report");
    assert_eq!(
        queue.average_time_nonzero(),
        0.0,
        "a queue nobody entered has no average waiting time"
    );
}
