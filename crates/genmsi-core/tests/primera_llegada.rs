//! The first arrival of a generator: with a spread, the original draws it.
//!
//! The manual of the original (`GENERATE`, "Acción"): "El operando C puede ser
//! usado para especificar un incremento de tiempo positivo para la primera
//! transacción. En otro caso, el primer tiempo de incremento es calculado
//! desde los operandos A y B." So with a spread in B the first increment is a
//! draw over `[A-B, A+B]`, the same one the arrivals that follow draw.
//!
//! The measurement of the original is in `odd/tasks/notas-reloj.md` §2: sixty
//! first arrivals read with a `MARK` and collected in a `TABLA` give a mean of
//! 57.133 against the 60 the model asks for, and the Kolmogorov-Smirnov test
//! against `U(30,90)` does not reject the uniform (D=0.1167, p=0.3666). Ours
//! gave the exact mean in the six hundred samples of ten seeds, and the same
//! test rejected them (D=0.5).
//!
//! The clock of the run is whole (`odd/tasks/notas-reloj.md` §1), so the draw
//! is a whole number over the inclusive range `[A-B, A+B]`, and the range of
//! the model holds seven of them.
//!
//! The three cases the rule has:
//!
//! - with a spread, the first arrival is drawn, and not the mean;
//! - without a spread there is nothing to draw, and the first arrival is the
//!   mean, exactly;
//! - with an offset, the offset is the increment of the first transaction of
//!   the original, and nothing is drawn.

mod support;

use genmsi_core::{Program, SimConfig, Simulation};
use support::{linear, module};

/// A generator with the given operands, feeding a terminate, with a clock
/// that ends the run.
fn generator(media: &'static str, spread: &'static str, offset: &'static str) -> Program {
    linear(
        &[
            module(
                "Generate",
                "Gen",
                &[
                    ("Media", media),
                    ("Modificador", spread),
                    ("OffSet", offset),
                ],
            ),
            module("Terminate", "T", &[("Unidades", "0")]),
        ],
        3600.0,
    )
}

/// The instants at which the generator produced a transaction, for one seed.
fn arrivals(program: &Program, seed: u64) -> Vec<f64> {
    let config = SimConfig {
        seed,
        stop_time: Some(200.0),
        trace: true,
        max_events: 5_000_000,
        ..SimConfig::default()
    };
    let result = Simulation::new(program, config).run().expect("the run works");
    let generator = support::block(program, "Generate");
    result
        .trace
        .iter()
        .filter(|record| record.event == "generate" && record.block == generator)
        .map(|record| record.time)
        .collect()
}

/// The first arrival of the generator, for one seed.
fn first_arrival(program: &Program, seed: u64) -> f64 {
    let times = arrivals(program, seed);
    assert!(!times.is_empty(), "the generator produced no transaction");
    times[0]
}

#[test]
fn con_dispersion_la_primera_llegada_se_sortea() {
    let program = generator("5", "3", "");
    let firsts: Vec<f64> = (1..=200).map(|seed| first_arrival(&program, seed)).collect();

    // The first arrival is a draw over [media - dispersion, media + dispersion].
    for (seed, time) in firsts.iter().enumerate() {
        assert!(
            (2.0..=8.0).contains(time),
            "seed {}: the first arrival {time} is outside [2, 8]",
            seed + 1
        );
    }

    // It is not the mean: the draw moves it.
    assert!(
        firsts.iter().any(|time| *time != 5.0),
        "every first arrival was the mean: nothing was drawn"
    );
    // The clock of the run is whole, so the draw is a whole number too: the
    // range of the model holds seven of them, and the draw reaches both of
    // its ends (the manual of the original: "inclusivamente").
    let distinct: std::collections::BTreeSet<u64> = firsts
        .iter()
        .map(|time| time.to_bits())
        .collect();
    assert!(
        distinct.len() > 1 && distinct.len() <= 7,
        "the draw gave {} distinct first arrivals in 200 seeds: the draw is not whole",
        distinct.len()
    );
    assert!(
        firsts.contains(&2.0),
        "the low end of the range was never drawn: {firsts:?}"
    );
    assert!(
        firsts.contains(&8.0),
        "the high end of the range was never drawn: {firsts:?}"
    );

    // And it is centered on the mean: the draw is the symmetric spread.
    let mean = firsts.iter().sum::<f64>() / firsts.len() as f64;
    assert!(
        (mean - 5.0).abs() < 0.5,
        "the mean of the first arrivals is {mean}, not the 5 of the model"
    );
}

#[test]
fn sin_dispersion_la_primera_llegada_es_la_media() {
    let program = generator("5", "", "");
    for seed in 1..=20 {
        assert_eq!(
            first_arrival(&program, seed),
            5.0,
            "seed {seed}: a generator without a spread starts at its mean"
        );
    }
}

#[test]
fn con_dispersion_cero_la_primera_llegada_es_la_media() {
    // A spread of zero is not a spread: there is nothing to draw.
    let program = generator("5", "0", "");
    for seed in 1..=20 {
        assert_eq!(
            first_arrival(&program, seed),
            5.0,
            "seed {seed}: a spread of zero starts at the mean"
        );
    }
}

#[test]
fn el_offset_manda_sobre_la_primera_llegada() {
    // The manual of the original: the offset is the increment of the first
    // transaction, and nothing is drawn for it.
    let program = generator("5", "3", "100");
    for seed in 1..=20 {
        assert_eq!(
            first_arrival(&program, seed),
            100.0,
            "seed {seed}: the offset is the increment of the first transaction"
        );
    }
}
