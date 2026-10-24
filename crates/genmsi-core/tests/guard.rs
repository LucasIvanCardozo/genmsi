//! The guard of the number of events cuts a run that does not end.
//!
//! A model with a zero delay keeps making events without advancing the
//! clock: the horizon never arrives, the counter of terminations never
//! empties and the run would go on forever, taking the memory of the
//! machine with it (fifty million events are six gigabytes).
//!
//! The guard stops the run, and the report says the truth: the run was cut
//! (`event_limit`), and it did not end by itself. The guard is far above the
//! biggest legitimate run of the repository, so it never cuts a run that
//! means to end.

mod support;

use std::path::{Path, PathBuf};

use genmsi_core::{DEFAULT_MAX_EVENTS, Program, RunResult, SimConfig, Simulation};
use support::{linear, module};

/// The root of the repository.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A model whose generator has no delay.
///
/// Every transaction arrives at the same instant and the model has no way to
/// end: the clock never advances and the counter of terminations never
/// empties.
fn zero_delay() -> Program {
    linear(
        &[
            module(
                "Generate",
                "Gen",
                &[("Media", "0"), ("Modificador", "0")],
            ),
            module("Terminate", "T", &[("Unidades", "0")]),
        ],
        3600.0,
    )
}

/// Run a model with a guard of `max_events` events.
fn run(program: &Program, max_events: u64) -> RunResult {
    Simulation::new(
        program,
        SimConfig {
            seed: 1,
            max_events,
            ..SimConfig::default()
        },
    )
    .run()
    .expect("the guard cuts the run, it does not refuse the model")
}

#[test]
fn un_modelo_con_retardo_cero_termina_con_el_guard() {
    let program = zero_delay();
    let result = run(&program, 5_000);
    assert_eq!(result.end_reason.as_str(), "event_limit");
    assert!(
        result.events <= 5_001,
        "the run went past the guard: {} events",
        result.events
    );
}

#[test]
fn el_informe_dice_que_la_corrida_fue_cortada() {
    let program = zero_delay();
    let result = run(&program, 5_000);
    let detail = result
        .diagnostics
        .iter()
        .find(|message| message.contains("the guard cut the run"))
        .expect("the report says why the run was cut");
    assert!(detail.contains("keeps making events without ending"), "{detail}");
}

#[test]
fn el_guard_por_defecto_no_corta_una_corrida_legitima() {
    // The biggest legitimate run of the repository (the model of the parcial
    // of 2023, which lasts 36000 units of simulated time) uses less than
    // fifty thousand events: the guard of two million is far above it.
    const { assert!(DEFAULT_MAX_EVENTS > 50_000) };

    let path = root().join("fixtures/models/Parcial 6-11-23__Veron - Parcial.gmdl");
    let model = genmsi_format::load(&path).expect("the model loads");
    let program = genmsi_core::compile_model(&model).expect("the model compiles");
    let result = Simulation::new(&program, SimConfig::default())
        .run()
        .expect("the model runs");
    assert_eq!(result.end_reason.as_str(), "terminated");
    assert!(
        result.events < DEFAULT_MAX_EVENTS,
        "the legitimate run is near the guard: {} events",
        result.events
    );
}
