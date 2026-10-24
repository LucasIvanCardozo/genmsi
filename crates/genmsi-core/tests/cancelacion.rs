//! The engine stops when the reader raises the flag.
//!
//! The flag is the seam of the cancellation: the shell raises it and the
//! engine reads it, one event at a time. A run nobody stops comes out exactly
//! as it came out before the flag existed.

mod support;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use genmsi_core::{EndReason, Program, SimConfig, Simulation};

/// A model that keeps the engine busy for a while: a generator that makes a
/// transaction every hundredth of a second, and a clock of half an hour.
///
/// The model has no `Terminate` operand, so the run lasts until the clock
/// ends it: the engine has hundreds of thousands of events to work on, and
/// the flag has time to be raised in the middle of them.
fn busy() -> Program {
    support::linear(
        &[
            support::module("Generate", "G", &[("Media", "0.01")]),
            support::module("Terminate", "T", &[("Unidades", "0")]),
        ],
        1800.0,
    )
}

/// The flag of the cancellation, and the thread that raises it.
fn raiser(delay: Duration) -> (Arc<AtomicBool>, thread::JoinHandle<()>) {
    let flag = Arc::new(AtomicBool::new(false));
    let raised = flag.clone();
    let handle = thread::spawn(move || {
        thread::sleep(delay);
        raised.store(true, Ordering::Relaxed);
    });
    (flag, handle)
}

#[test]
fn a_run_that_is_cancelled_ends_before_the_run_would_end() {
    let program = busy();

    // The whole run: how long the engine takes when nobody stops it.
    let whole_start = Instant::now();
    let whole = Simulation::new(&program, SimConfig::default())
        .run()
        .expect("the run works");
    let whole_elapsed = whole_start.elapsed();

    // The same run, with the reader raising the flag in the middle of it.
    let (flag, handle) = raiser(Duration::from_millis(20));
    let cancelled_start = Instant::now();
    let cancelled = Simulation::new(
        &program,
        SimConfig {
            stop: Some(flag),
            ..SimConfig::default()
        },
    )
    .run()
    .expect("the run works");
    let cancelled_elapsed = cancelled_start.elapsed();
    handle.join().expect("the flag is raised");

    println!(
        "whole: {whole_elapsed:?} ({:?} sim), cancelled: {cancelled_elapsed:?} ({:?} sim)",
        whole.end_time, cancelled.end_time
    );

    assert_eq!(
        cancelled.end_reason,
        EndReason::Cancelled,
        "the run did not end for the reason of the cancellation"
    );
    assert!(
        whole_elapsed > Duration::from_millis(50),
        "the whole run is too short to be cancelled in the middle: {whole_elapsed:?}"
    );
    assert!(
        cancelled_elapsed * 2 < whole_elapsed,
        "the cancelled run did not end much earlier: {cancelled_elapsed:?} against {whole_elapsed:?}"
    );
    assert!(
        cancelled.end_time < whole.end_time,
        "the cancelled run reached the end of the horizon: {:?} against {:?}",
        cancelled.end_time,
        whole.end_time
    );
}

#[test]
fn a_run_nobody_stops_is_the_run_it_was() {
    let program = busy();

    // The run without a flag, and the run with a flag nobody raises: the two
    // are the same run, event by event.
    let plain = Simulation::new(&program, SimConfig::default())
        .run()
        .expect("the run works");
    let flag = Arc::new(AtomicBool::new(false));
    let with_flag = Simulation::new(
        &program,
        SimConfig {
            stop: Some(flag),
            ..SimConfig::default()
        },
    )
    .run()
    .expect("the run works");

    assert_eq!(plain.end_reason, EndReason::Terminated);
    assert_eq!(plain.end_reason, with_flag.end_reason);
    assert_eq!(plain.end_time, with_flag.end_time);
    assert_eq!(plain.events, with_flag.events);
    assert_eq!(plain.transactions, with_flag.transactions);
    // The report of the two runs is the same document, byte for byte: the
    // flag nobody raised changes nothing.
    let plain_report = serde_json::to_string(&genmsi_core::json_report(&plain, &program))
        .expect("the report serializes");
    let flag_report = serde_json::to_string(&genmsi_core::json_report(&with_flag, &program))
        .expect("the report serializes");
    assert_eq!(plain_report, flag_report, "the flag changed the run");
}
