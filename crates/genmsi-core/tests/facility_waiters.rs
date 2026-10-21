//! Facility contention: no waiting transaction may be stranded.

mod support;

use support::{block, linear, module, run};

/// A RELEASE must retry the head of the pending chain and keep the rest queued.
/// Resuming one waiter and dropping the others strands every transaction behind
/// it in the SEIZE block forever.
///
/// The original retries the whole pending chain on each release (`RETRY` in the
/// block table) and only the winner keeps the facility, so every generated
/// transaction eventually passes. `Parcial 6-11-23` hit exactly this: 331 of 369
/// transactions were lost at one facility, which starved the GATHER that opens
/// the museum and left the whole inner topology unreachable.
#[test]
fn release_retries_the_pending_chain_without_losing_waiters() {
    let horizon = 10_000.0;
    let program = linear(
        &[
            module(
                "Generate",
                "Gen",
                &[("Media", "1"), ("Modificador", "0"), ("Cantidad", "100")],
            ),
            module("Seize", "S", &[("Facility", "servidor")]),
            module("Advance", "A", &[("Media", "50"), ("Modificador", "0")]),
            module("Release", "R", &[("Facility", "servidor")]),
            module("Terminate", "T", &[("Unidades", "0")]),
        ],
        horizon,
    );
    // Each holder keeps the facility for 50 time units, so the queue drains far
    // later than the arrivals.
    let result = run(&program, horizon, 5);
    let seize = block(&program, "S");
    let release = block(&program, "R");
    let terminate = block(&program, "T");
    assert_eq!(result.blocks[block(&program, "Generate")].entries, 100);
    assert_eq!(
        result.blocks[seize].entries, 100,
        "every generated transaction must seize the facility"
    );
    assert_eq!(result.blocks[release].entries, 100, "released");
    assert_eq!(result.blocks[terminate].entries, 100, "terminated");
    assert_eq!(result.blocks[seize].current, 0, "no stranded waiter");
}
