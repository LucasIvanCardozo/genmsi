//! The gate that does not open and the inversion of the logic switch.
//!
//! Both rules come from the original, measured under Wine
//! (`scripts/wine-oracle/run_oracle.sh`):
//!
//! - The gate of the bench `GateL` (`Generate` → `GateL` → `Terminate`, the
//!   logic switch `1` never set) writes the row `0 0 0`: the transaction that
//!   fails the test of the gate does not enter the gate, and its attempt is
//!   counted on the switch the gate looks at, the way the original counts it
//!   in its `LOGICSWITCH` section.
//!
//! - The third operator of the `LOGIC` block, the inversion, flips the state
//!   of the switch. The original's report of the probes `LogicInvertir1`,
//!   `LogicInvertir2`, `LogicInvertir3` and `LogicInvertir4` (the same model
//!   with one, two, three and four inversions, the switch starting set) leaves
//!   the switch in `0`, `1`, `0` and `1`, and the probes
//!   `LogicInvertirSinInicial1` and `LogicInvertirSinInicial3` (the same
//!   model without the `INITIAL LS`, so the switch starts reset) leave it in
//!   `1` and `1`.

mod support;

use genmsi_core::Program;
use genmsi_core::report::render;
use support::{block, node, run, wired};

/// The bench `GateL`: a generator, the gate on the logic switch `1` and a
/// terminate. Nothing in the model sets the switch, so the gate never opens
/// and every transaction waits in it.
fn gate_model() -> Program {
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("GateL"))],
            &[("Media", "60"), ("Modificador", "30")],
        ),
        node(
            2,
            "GateL",
            Some("GateL"),
            &[("Salida1", Some("A_Terminate")), ("Salida2", None)],
            &[("LogicSwich", "1"), ("op", "Set")],
        ),
        node(3, "Terminate", Some("A_Terminate"), &[], &[]),
    ];
    wired(&nodes, &[(1, 0, 2, 0), (2, 0, 3, 0)], 3600.0)
}

/// The row of the gate in the report of the run.
fn gate_row(program: &Program) -> String {
    let result = run(program, 3600.0, 1);
    let text = render(&result, program);
    text.lines()
        .find(|line| line.contains("GATE"))
        .unwrap_or_else(|| panic!("the report has no row of the gate"))
        .to_string()
}

/// A gate that does not open counts nothing.
///
/// The original's row of the gate of the bench `GateL` is `0 0 0`: the
/// transaction that fails the test does not enter the block, and the attempt
/// is counted on the switch the gate looks at, not on the block.
#[test]
fn the_gate_that_does_not_open_counts_nothing() {
    let program = gate_model();
    let result = run(&program, 3600.0, 1);
    let gate = block(&program, "GateL");
    assert_eq!(result.blocks[gate].entries, 0, "the gate counts no entry");
    assert_eq!(result.blocks[gate].current, 0, "the gate holds nobody");
    assert_eq!(result.blocks[gate].retries, 0, "the gate counts no retry");
    // The row of the report is the three numbers of the block.
    let row = gate_row(&program);
    let numbers: Vec<&str> = row.split_whitespace().skip(1).take(3).collect();
    assert_eq!(numbers, ["0", "0", "0"], "the row of the gate: {row}");
}

/// The attempt that fails the test of the gate is counted on the switch.
///
/// This is where the original counts it: its `LOGICSWITCH` section writes the
/// number of the attempts that failed the test of the switch.
#[test]
fn the_attempt_that_fails_the_gate_is_counted_on_the_switch() {
    let program = gate_model();
    let result = run(&program, 3600.0, 1);
    let switch = result
        .logic_switches
        .iter()
        .find(|switch| switch.name == "1")
        .expect("the switch of the gate");
    assert!(
        switch.retry > 0,
        "the switch counts the attempts that fail the test of the gate"
    );
    // Nothing reaches the terminate: the gate never opens.
    let terminate = block(&program, "A_Terminate");
    assert_eq!(result.blocks[terminate].entries, 0);
}

/// A model with `count` inversions of the logic switch `1`, which starts set.
fn logic_model(count: &'static str) -> Program {
    logic_model_with_operator("Invertir", count, true)
}

/// The same model, with the operator and the initial state the test names.
///
/// Without the `INITIAL LS` the switch starts reset, the way every switch of
/// the original does; the `INITIAL LS` of the original only knows how to
/// declare a switch that starts set.
fn logic_model_with_operator(
    operator: &'static str,
    count: &'static str,
    starts_set: bool,
) -> Program {
    let mut nodes = Vec::new();
    let mut links = Vec::new();
    let mut next = 1u32;
    if starts_set {
        nodes.push(node(
            next,
            "InitialLS",
            None,
            &[],
            &[("LogicSwich", "1"), ("Valor", "Set")],
        ));
        next += 1;
    }
    nodes.push(node(
        next,
        "Generate",
        None,
        &[("Salida1", Some("A_Logic"))],
        &[("Media", "1"), ("Modificador", "0"), ("Cantidad", count)],
    ));
    links.push((next, 0, next + 1, 0));
    next += 1;
    nodes.push(node(
        next,
        "Logic",
        Some("A_Logic"),
        &[("Salida1", Some("A_Terminate"))],
        &[("LogicSwich", "1"), ("Op", operator)],
    ));
    links.push((next, 0, next + 1, 0));
    next += 1;
    nodes.push(node(next, "Terminate", Some("A_Terminate"), &[], &[]));
    wired(&nodes, &links, 3600.0)
}

/// The state of the logic switch `1` after the run.
fn switch_value(program: &Program) -> bool {
    let result = run(program, 3600.0, 1);
    result
        .logic_switches
        .iter()
        .find(|switch| switch.name == "1")
        .expect("the switch of the block")
        .value
}

/// The third operator of the `LOGIC` block inverts the switch.
///
/// The switch starts reset and one inversion leaves it set: the original's
/// report of the probe `LogicInvertirSinInicial1` writes `1` in its
/// `LOGICSWITCH` section, and a `Reset` would have left it in `0`.
#[test]
fn the_logic_inverts_the_switch_after_an_odd_number_of_inversions() {
    let program = logic_model_with_operator("Invertir", "1", false);
    assert!(
        switch_value(&program),
        "the switch starts reset and one inversion sets it"
    );
}

/// The inversion is an inversion and not a reset: the switch that starts set
/// is set again after two inversions.
///
/// The original's report of the probe `LogicInvertir2` writes `1` in its
/// `LOGICSWITCH` section, and a `Reset` would have written `0`.
#[test]
fn the_logic_inverts_the_switch_after_an_even_number_of_inversions() {
    let program = logic_model("2");
    assert!(
        switch_value(&program),
        "the switch starts set and two inversions set it again"
    );
}

/// The inversion is an inversion and not a `Set`: the switch that starts set
/// is reset by one inversion.
///
/// The original's report of the probe `LogicInvertir1` writes `0` in its
/// `LOGICSWITCH` section, and a `Set` would have left it in `1`.
#[test]
fn the_logic_inverts_the_switch_that_starts_set() {
    let program = logic_model("1");
    assert!(
        !switch_value(&program),
        "the switch starts set and one inversion resets it"
    );
}

/// The two operators that are not the inversion keep their meaning.
#[test]
fn the_logic_keeps_the_meaning_of_the_other_two_operators() {
    let set = logic_model_with_operator("Set", "1", true);
    assert!(switch_value(&set), "the `Set` operator sets the switch");
    let reset = logic_model_with_operator("Reset", "1", true);
    assert!(
        !switch_value(&reset),
        "the `Reset` operator resets the switch"
    );
}
