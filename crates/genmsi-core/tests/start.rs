//! The `START` is a parameter of the run, like the one of the original.
//!
//! The measurement is in `odd/tasks/notas-start.md`: the `START` of the
//! original is the **counter of terminations** of the run and whoever
//! launches the run sets it (the "Start:" field of the Simulador panel, one
//! by default). The model does not declare it anywhere: there is no `START`
//! class among the sixty-six, the root module has no properties and the
//! GPSS listing has no `START` sentence, so the value cannot live in the
//! model.
//!
//! What the measurement also says:
//!
//! - the `Terminate` **with** operand A takes that many units from the
//!   counter for every transaction that arrives, and the `Clock` takes one
//!   per tick; a `Terminate` without operand A takes nothing;
//! - the run ends when the counter reaches zero **or goes below it** (the
//!   original ends at −1);
//! - with the `Start` of the original at one (its default) the five
//!   compared reports come out byte for byte identical to the original's.
//!
//! So the engine keeps the rule it already had and moves the initial value
//! from the compiled program to the configuration of the run.

mod support;

use genmsi_core::{DEFAULT_START, Program, RunResult, SimConfig, Simulation};
use support::{block, linear, module};

/// A generator of one transaction every hundred units, and a terminate that
/// takes `units` from the `START` of the run.
///
/// The clock of the helper is far away (3600), so it does not take any unit
/// from the counter before the run ends: the only thing that counts here is
/// the terminate.
fn program(units: &'static str) -> Program {
    linear(
        &[
            module(
                "Generate",
                "Gen",
                &[("Media", "100"), ("Modificador", "0")],
            ),
            module("Terminate", "T", &[("Unidades", units)]),
        ],
        3600.0,
    )
}

/// Run a model with a `START` of `start`.
fn run(program: &Program, start: u64) -> RunResult {
    Simulation::new(
        program,
        SimConfig {
            seed: 1,
            start,
            max_events: 1_000_000,
            ..SimConfig::default()
        },
    )
    .run()
    .expect("the run works")
}

/// The transactions that reached the terminate of the model.
fn terminations(program: &Program, result: &RunResult) -> u64 {
    result.blocks[block(program, "T")].entries
}

#[test]
fn el_start_por_defecto_es_el_uno_del_original() {
    assert_eq!(DEFAULT_START, 1);
    assert_eq!(SimConfig::default().start, 1);
}

#[test]
fn con_un_start_de_uno_la_corrida_termina_en_la_primera_transaccion() {
    // The behaviour of today: with the default of the original, the run ends
    // with the first transaction that reaches the terminate.
    let program = program("1");
    let result = run(&program, 1);
    assert_eq!(result.end_reason.as_str(), "terminated");
    assert_eq!(result.end_time, 100.0);
    assert_eq!(terminations(&program, &result), 1);
}

#[test]
fn con_un_start_de_cinco_la_corrida_termina_en_la_quinta_transaccion() {
    // The counter goes 5, 4, 3, 2, 1 and the fifth transaction takes it to
    // zero: the run ends there, and not with the first one.
    let program = program("1");
    let result = run(&program, 5);
    assert_eq!(result.end_reason.as_str(), "terminated");
    assert_eq!(result.end_time, 500.0);
    assert_eq!(terminations(&program, &result), 5);
}

#[test]
fn un_terminate_de_tres_unidades_pasa_el_contador_de_tres_a_cero() {
    // The counter goes 3 to 0 with the first transaction: zero ends the run.
    let program = program("3");
    let result = run(&program, 3);
    assert_eq!(result.end_reason.as_str(), "terminated");
    assert_eq!(result.end_time, 100.0);
    assert_eq!(terminations(&program, &result), 1);
}

#[test]
fn un_terminate_de_tres_unidades_pasa_el_contador_de_cinco_a_menos_uno() {
    // The counter goes 5 to 2 and then to −1: the run ends there, which is
    // what the original does (its `pTermina3` with a Start of five ends at
    // the second transaction).
    let program = program("3");
    let result = run(&program, 5);
    assert_eq!(result.end_reason.as_str(), "terminated");
    assert_eq!(result.end_time, 200.0);
    assert_eq!(terminations(&program, &result), 2);
}

#[test]
fn el_reloj_tambien_baja_el_contador_del_start() {
    // The `Clock` of the original compiles to a generate and a `TERMINATE 1`,
    // so it takes one unit from the counter per tick: with a `START` of
    // three and a clock of a hundred, the run ends at the third tick (the
    // original's `pReloj`).
    let program = linear(&[], 100.0);
    let result = run(&program, 3);
    assert_eq!(result.end_reason.as_str(), "terminated");
    assert_eq!(result.end_time, 300.0);
    let clock = block(&program, "Terminate");
    assert_eq!(result.blocks[clock].entries, 3);
}
