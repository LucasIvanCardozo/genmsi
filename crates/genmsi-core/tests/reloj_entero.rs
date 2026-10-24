//! The clock of the run is whole, like the clock of the original.
//!
//! The measured evidence is in `odd/tasks/notas-reloj.md`: the original's
//! simulated time is an integer, its pending-event table shows whole times
//! (`BDT 3610`), the busy time of a facility is a whole number in 50 of the
//! 50 facilities of its 91 reports, and the gaps between the arrivals of a
//! probe are whole numbers in 217 of the 217. The original cannot even open
//! a model with a fractional operand ("El modelo no puede abrirse porque el
//! archivo presenta errores estructurales").
//!
//! So the rule of this engine is:
//!
//! - a delay with a spread is a **whole number** drawn over the inclusive
//!   range `[A-B, A+B]` (the manual of the `ADVANCE`: "un número aleatorio
//!   entre A-B y A + B, inclusivamente");
//! - the product of a mean with the value of a modifier function is
//!   truncated to an integer (the same manual: "El producto es truncado a
//!   un entero");
//! - every instant a reader sees — in the report, in the trace, in the
//!   transactions and in the statistics — is a whole number of simulated
//!   time, so the times of a facility, its busy time included, are whole.

mod support;

use genmsi_core::{Program, SimConfig, Simulation};
use support::{block, linear, module, run};

/// A generator with a spread that feeds a terminate, with a clock that ends
/// the run.
fn generator(media: &'static str, spread: &'static str) -> Program {
    linear(
        &[
            module(
                "Generate",
                "Gen",
                &[("Media", media), ("Modificador", spread)],
            ),
            module("Terminate", "T", &[("Unidades", "0")]),
        ],
        3600.0,
    )
}

/// The instants at which the generator produced a transaction.
fn arrivals(program: &Program, seed: u64) -> Vec<f64> {
    let result = Simulation::new(
        program,
        SimConfig {
            seed,
            stop_time: Some(600.0),
            trace: true,
            max_events: 5_000_000,
            ..SimConfig::default()
        },
    )
    .run()
    .expect("the run works");
    let generator = block(program, "Generate");
    result
        .trace
        .iter()
        .filter(|record| record.event == "generate" && record.block == generator)
        .map(|record| record.time)
        .collect()
}

/// Whether a time is a whole number of simulated time.
fn is_whole(value: f64) -> bool {
    (value - value.trunc()).abs() < 1e-9
}

#[test]
fn el_intervalo_sorteado_es_un_numero_entero_dentro_del_rango() {
    // The manual of the original: the draw is a whole number over the
    // inclusive range [A-B, A+B]. Both ends are inside it.
    let program = generator("60", "30");
    let times = arrivals(&program, 7);
    assert!(times.len() > 10, "the generator made {} arrivals", times.len());

    for time in &times {
        assert!(
            is_whole(*time),
            "the arrival {time} is not a whole number of simulated time"
        );
    }

    // The first arrival is the drawn interval: a whole number over the
    // inclusive range [A-B, A+B].
    let first = times[0];
    assert!(
        (30.0..=90.0).contains(&first),
        "the first arrival {first} is outside [30, 90]"
    );

    // The gaps between the arrivals are the drawn intervals: whole numbers
    // inside the same range, and both ends of the range are reached (the
    // manual says "inclusivamente").
    let mut gaps: Vec<f64> = Vec::new();
    for seed in 1..=20 {
        let times = arrivals(&program, seed);
        gaps.extend(times.windows(2).map(|pair| pair[1] - pair[0]));
    }
    for gap in &gaps {
        assert!(
            is_whole(*gap),
            "the gap {gap} is not a whole number of simulated time"
        );
        assert!((30.0..=90.0).contains(gap), "the gap {gap} is outside [30, 90]");
    }
    assert!(
        gaps.contains(&30.0),
        "the low end of the range was never drawn: {gaps:?}"
    );
    assert!(
        gaps.contains(&90.0),
        "the high end of the range was never drawn: {gaps:?}"
    );
}

#[test]
fn el_tiempo_de_una_facilidad_con_media_fractionaria_es_un_entero() {
    // A facility whose mean is 30.5: the original cannot open a model with
    // a fractional operand, so ours truncates the delay and the time of the
    // facility is a whole number of simulated time.
    let horizon = 3600.0;
    let program = linear(
        &[
            module(
                "Generate",
                "Gen",
                &[("Media", "10"), ("Modificador", "0")],
            ),
            module("Seize", "S", &[("Facility", "servidor")]),
            module("Advance", "A", &[("Media", "30.5"), ("Modificador", "0")]),
            module("Release", "R", &[("Facility", "servidor")]),
            module("Terminate", "T", &[("Unidades", "0")]),
        ],
        horizon,
    );
    let result = run(&program, horizon, 3);
    let facility = result
        .entities
        .facilities
        .get("servidor")
        .expect("the facility of the model");
    assert!(facility.entries > 0, "the facility was never taken");
    assert!(
        is_whole(facility.busy_time),
        "the busy time of the facility is {}",
        facility.busy_time
    );
    // The delay is truncated: thirty units and not thirty and a half. The
    // average is not a whole number (the original's averages have decimals
    // too), but it is the average of the whole holds of thirty units.
    assert!(
        (29.0..=31.0).contains(&facility.average_time()),
        "the average time of the facility is {}",
        facility.average_time()
    );
}

#[test]
fn los_instantes_del_reloj_del_reporte_y_de_la_traza_son_enteros() {
    let horizon = 3600.0;
    let program = linear(
        &[
            module(
                "Generate",
                "Gen",
                &[("Media", "7"), ("Modificador", "3")],
            ),
            module("Queue", "Q", &[("Queue", "fila")]),
            module("Seize", "S", &[("Facility", "servidor")]),
            module("Advance", "A", &[("Media", "12"), ("Modificador", "4")]),
            module("Release", "R", &[("Facility", "servidor")]),
            module("Depart", "D", &[("Queue", "fila")]),
            module("Terminate", "T", &[("Unidades", "0")]),
        ],
        horizon,
    );
    let result = Simulation::new(
        &program,
        SimConfig {
            seed: 11,
            stop_time: Some(horizon),
            trace: true,
            max_events: 5_000_000,
            ..SimConfig::default()
        },
    )
    .run()
    .expect("the run works");

    assert!(
        is_whole(result.end_time),
        "the end time of the run is {}",
        result.end_time
    );
    for record in &result.trace {
        assert!(
            is_whole(record.time),
            "the trace holds the time {}",
            record.time
        );
    }
    for event in &result.pending {
        assert!(
            is_whole(event.time),
            "the pending event holds the time {}",
            event.time
        );
    }
    let facility = result
        .entities
        .facilities
        .get("servidor")
        .expect("the facility of the model");
    assert!(
        is_whole(facility.busy_time),
        "the busy time of the facility is {}",
        facility.busy_time
    );
    let queue = result
        .entities
        .queues
        .get("fila")
        .expect("the queue of the model");
    assert!(
        is_whole(queue.area),
        "the area of the queue is {}",
        queue.area
    );
    assert!(
        is_whole(queue.total_time),
        "the accumulated wait of the queue is {}",
        queue.total_time
    );
}
