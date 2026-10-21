//! `TRANSFER` probability: which output takes the `Prob` share.
//!
//! The editor's `Prob` hint says "probabilidad de continuar por la Salida1" and
//! the app's own GPSS export writes `TRANSFER P,<Salida2>,<Salida1>`: GPSS sends
//! the `P` fraction to its *second* named operand, so the generated text puts
//! `Salida2` first because `Salida1` receives the fraction. Every baseline row
//! that isolates one branch agrees with `Prob` -> `Salida1` (18 of 18 across the
//! 8 models with a free-standing `Transfer`). The single exception,
//! `ejercicio_4__ejercicio_4`'s `Entrada1`, sits at -1.4 sigma for n = 59 (27
//! observed against 32.45 expected, two-sided p ~ 0.20): sampling noise of a
//! baseline that is one sample, not a different rule.
//!
//! These tests pin the rule so a later attempt at that one metric cannot invert
//! it, and pin the bare-number form as thousandths (`550` == `55%`), which is
//! the form the original prints for the models that store it that way.

mod support;

use genmsi_core::{SimConfig, Simulation, compile_model};
use support::{node, run, wired};

/// `TRANSFER` with the given `Prob` value: `Salida1` ends at `SI`, `Salida2` at
/// `NO`, so the shares are readable from the block entry counts.
fn branch_model(prob: &'static str) -> genmsi_core::Program {
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("T1"))],
            &[("Media", "1"), ("Modificador", "0"), ("Cantidad", "2000")],
        ),
        node(
            2,
            "Transfer",
            Some("T1"),
            &[("Salida1", Some("SI")), ("Salida2", Some("NO"))],
            &[("Prob", prob)],
        ),
        node(3, "Terminate", Some("SI"), &[], &[("Unidades", "0")]),
        node(4, "Terminate", Some("NO"), &[], &[("Unidades", "0")]),
    ];
    let links = [(1, 0, 2, 0), (2, 0, 3, 0), (2, 1, 4, 0)];
    wired(&nodes, &links, 4000.0)
}

fn entries(program: &genmsi_core::Program, result: &genmsi_core::RunResult, label: &str) -> u64 {
    let index = program
        .blocks
        .iter()
        .position(|block| block.label.as_deref() == Some(label))
        .unwrap_or_else(|| panic!("no block labelled {label}"));
    result.blocks[index].entries
}

fn share_of_salida1(prob: &'static str) -> f64 {
    let program = branch_model(prob);
    let result = run(&program, 4000.0, 1);
    let yes = entries(&program, &result, "SI") as f64;
    let no = entries(&program, &result, "NO") as f64;
    assert!(
        (yes + no - 2000.0).abs() < 1.0,
        "{prob}: every generated transaction must leave through one of the two \
         outputs, got {yes} + {no}"
    );
    yes / (yes + no)
}

#[test]
fn transfer_sends_the_probability_share_to_salida1() {
    // 2000 draws at p = 0.55: sd = 0.011, so +-0.04 is beyond 3.5 sigma.
    let share = share_of_salida1("55%");
    assert!(
        (share - 0.55).abs() < 0.04,
        "Prob 55% must send 55% to Salida1, measured {share:.3}"
    );
    // The inverted rule would put ~0.45 there, far outside that band.
}

#[test]
fn transfer_reads_a_bare_prob_number_as_thousandths() {
    // The original prints `800` for the model that stores `800`, and 80% of that
    // model's 367 transactions take Salida1, so a bare number is thousandths.
    assert!(
        (share_of_salida1("550") - 0.55).abs() < 0.04,
        "Prob 550 must mean 55%, not 550% or 0.55%"
    );
    let share = share_of_salida1("1");
    assert!(
        share < 0.01,
        "Prob 1 must mean 0.1% of the traffic on Salida1, measured {share:.4}"
    );
}

/// The same rule, read off a committed baseline instead of a synthetic model.
#[test]
fn transfer_share_matches_the_monorriel_baseline() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let model = genmsi_format::load(root.join("fixtures/models/monorriel__monorriel.gmdl"))
        .expect("monorriel loads");
    let program = compile_model(&model).expect("monorriel compiles");

    // The baseline: `Entrada1 TRANSFER 600,A_Terminate,A_Advance3` with 289
    // entries, 176 at `A_Advance3` (Salida1) and 113 at `A_Terminate` (Salida2).
    let mut yes = 0.0;
    let mut no = 0.0;
    for seed in 1..=5 {
        let result = Simulation::new(
            &program,
            SimConfig {
                seed,
                stop_time: Some(3600.0),
                ..SimConfig::default()
            },
        )
        .run()
        .expect("monorriel runs");
        yes += entries(&program, &result, "A_Advance3") as f64;
        no += entries(&program, &result, "A_Terminate") as f64;
    }
    let share = yes / (yes + no);
    assert!(
        (share - 0.60).abs() < 0.06,
        "the baseline splits 176/113 = 60% to `Salida1` (`A_Advance3`), our \
         five replications give {share:.3}"
    );
}
