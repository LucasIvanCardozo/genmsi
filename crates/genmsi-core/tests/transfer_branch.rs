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

// The two multifurcations: the class that splits the traffic with the shares
// it stores and the class that routes by a parameter of the transaction.
//
// The original's own bench of the class (`scripts/wine-oracle/benches/
// Multifurcacion.gmdl`, run against the original by the wine oracle) says
// that the five branches take 9, 10, 16, 12 and 15 of the 62 transactions
// the run created: the shares the class declares (a fifth each) with the
// noise of the draw.

/// A `Multifurcacion` with the five shares of the original's class.
fn multifurcacion_model() -> genmsi_core::Program {
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("T1"))],
            &[("Media", "1"), ("Modificador", "0")],
        ),
        node(
            2,
            "Multifurcacion",
            Some("T1"),
            &[
                ("Salida1", Some("B1")),
                ("SalidaProb2", Some("B2")),
                ("SalidaProb3", Some("B3")),
                ("SalidaProb4", Some("B4")),
                ("SalidaProb5", Some("B5")),
            ],
            &[
                ("Prob1", "20%"),
                ("Prob2", "20%"),
                ("Prob3", "20%"),
                ("Prob4", "20%"),
                ("Prob5", "20%"),
            ],
        ),
        node(3, "Terminate", Some("B1"), &[], &[("Unidades", "0")]),
        node(4, "Terminate", Some("B2"), &[], &[("Unidades", "0")]),
        node(5, "Terminate", Some("B3"), &[], &[("Unidades", "0")]),
        node(6, "Terminate", Some("B4"), &[], &[("Unidades", "0")]),
        node(7, "Terminate", Some("B5"), &[], &[("Unidades", "0")]),
    ];
    let links = [(1, 0, 2, 0), (2, 0, 3, 0), (2, 1, 4, 0), (2, 2, 5, 0), (2, 3, 6, 0), (2, 4, 7, 0)];
    wired(&nodes, &links, 3600.0)
}

#[test]
fn the_multifurcacion_class_splits_the_traffic_in_the_shares_it_stores() {
    let program = multifurcacion_model();
    let result = run(&program, 2000.0, 1);
    let mut total = 0u64;
    let mut shares = Vec::new();
    for label in ["B1", "B2", "B3", "B4", "B5"] {
        let count = entries(&program, &result, label);
        total += count;
        shares.push(count as f64);
    }
    assert!(total > 1000, "the run must create enough transactions: {total}");
    // 2000 draws at p = 0.2: sd = 0.011, so +-0.03 is beyond 2.5 sigma.
    for (index, share) in shares.iter().enumerate() {
        let share = share / total as f64;
        assert!(
            (share - 0.2).abs() < 0.03,
            "branch {index} took {share:.3} of the traffic, a fifth each is the rule"
        );
    }
}

/// A `MultifurcacionP` whose parameter says which branch the transaction
/// takes.
fn multifurcacionp_model(value: &'static str) -> genmsi_core::Program {
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("T1"))],
            &[("Media", "1"), ("Modificador", "0")],
        ),
        node(
            2,
            "Assign",
            Some("T1"),
            &[("Salida1", Some("T2"))],
            &[("Parametro", "Param"), ("Operador", "="), ("Valor", value)],
        ),
        node(
            3,
            "MultifurcacionP",
            Some("T2"),
            &[
                ("Salida1", Some("B1")),
                ("SalidaProb2", Some("B2")),
                ("SalidaProb3", Some("B3")),
                ("SalidaProb4", Some("B4")),
                ("SalidaProb5", Some("B5")),
            ],
            &[("Parametro", "Param")],
        ),
        node(4, "Terminate", Some("B1"), &[], &[("Unidades", "0")]),
        node(5, "Terminate", Some("B2"), &[], &[("Unidades", "0")]),
        node(6, "Terminate", Some("B3"), &[], &[("Unidades", "0")]),
        node(7, "Terminate", Some("B4"), &[], &[("Unidades", "0")]),
        node(8, "Terminate", Some("B5"), &[], &[("Unidades", "0")]),
    ];
    let links = [
        (1, 0, 2, 0),
        (2, 0, 3, 0),
        (3, 0, 4, 0),
        (3, 1, 5, 0),
        (3, 2, 6, 0),
        (3, 3, 7, 0),
        (3, 4, 8, 0),
    ];
    wired(&nodes, &links, 3600.0)
}

#[test]
fn the_multifurcacionp_class_routes_by_the_parameter_of_the_transaction() {
    // The class library of the original routes by the parameter
    // (`MULTITRANSFER MultifurcacionGO,P:[Parametro],{...}`): the value of
    // the parameter says which branch the transaction takes, and the branch
    // writes its own number in the parameter.
    for (value, branch) in [("1", "B1"), ("2", "B2"), ("3", "B3"), ("4", "B4"), ("5", "B5")] {
        let program = multifurcacionp_model(value);
        let result = run(&program, 200.0, 1);
        let total: u64 = ["B1", "B2", "B3", "B4", "B5"]
            .iter()
            .map(|label| entries(&program, &result, label))
            .sum();
        assert!(total > 100, "the run must create transactions: {total}");
        for label in ["B1", "B2", "B3", "B4", "B5"] {
            let count = entries(&program, &result, label);
            if label == branch {
                assert_eq!(count, total, "the parameter {value} takes the branch {branch}");
            } else {
                assert_eq!(count, 0, "the parameter {value} takes no other branch");
            }
        }
    }
}

#[test]
fn the_multifurcacionp_class_writes_the_branch_it_took_in_the_parameter() {    // The captions of the original's class say what each branch does:
    // `[Parametro]=1`, `[Parametro]=2`, ... A transaction that arrives
    // without a value takes the first branch, and the branch writes its own
    // number in the parameter.
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("T1"))],
            &[("Media", "1"), ("Modificador", "0")],
        ),
        node(
            2,
            "MultifurcacionP",
            Some("T1"),
            &[
                ("Salida1", Some("T2")),
                ("SalidaProb2", Some("B2")),
                ("SalidaProb3", Some("B3")),
                ("SalidaProb4", Some("B4")),
                ("SalidaProb5", Some("B5")),
            ],
            &[("Parametro", "Param")],
        ),
        node(
            3,
            "SaveValue",
            Some("T2"),
            &[("Salida1", Some("T3"))],
            &[("SaveValue", "V"), ("Op", "="), ("Valor", "Param")],
        ),
        node(4, "Terminate", Some("T3"), &[], &[("Unidades", "0")]),
        node(5, "Terminate", Some("B2"), &[], &[("Unidades", "0")]),
        node(6, "Terminate", Some("B3"), &[], &[("Unidades", "0")]),
        node(7, "Terminate", Some("B4"), &[], &[("Unidades", "0")]),
        node(8, "Terminate", Some("B5"), &[], &[("Unidades", "0")]),
    ];
    let links = [
        (1, 0, 2, 0),
        (2, 0, 3, 0),
        (3, 0, 4, 0),
        (2, 1, 5, 0),
        (2, 2, 6, 0),
        (2, 3, 7, 0),
        (2, 4, 8, 0),
    ];
    let program = wired(&nodes, &links, 3600.0);
    let result = Simulation::new(
        &program,
        SimConfig {
            seed: 1,
            stop_time: Some(20.0),
            trace: true,
            ..SimConfig::default()
        },
    )
    .run()
    .expect("runs");
    let values: Vec<&str> = result
        .trace
        .iter()
        .filter(|event| event.event == "value")
        .map(|event| event.detail.as_str())
        .collect();
    assert!(!values.is_empty(), "the trace must publish the value");
    assert!(
        values.iter().all(|detail| detail.ends_with(" 1")),
        "the branch writes its own number in the parameter: {values:?}"
    );
}

// The classes of the original that route a transaction by a condition of its
// own: the `Examine` class (the transaction belongs to a group or it does
// not), the `GateM` class (the label of the transaction matches the one the
// gate names) and the three selections (a logic switch, a storage and an
// entity of a class).
//
// The macro code of the original names the two outputs of the first two:
// `Salida1` is "En [Grupo]" and "No en [Grupo]" for the examination, and
// `Salida1` is the output of the transaction that matches the label.

/// A model with a group: the transactions join it and are then examined.
fn examination_model(join: bool) -> genmsi_core::Program {
    let mut nodes = vec![node(
        1,
        "Generate",
        None,
        &[("Salida1", Some("T1"))],
        &[("Media", "1"), ("Modificador", "0")],
    )];
    let mut links = vec![(1, 0, 2, 0)];
    let mut next_id = 2u32;
    let mut input = "T1";
    if join {
        nodes.push(node(
            next_id,
            "JoinMe",
            Some(input),
            &[("Salida1", Some("T2"))],
            &[("Grupo", "UnGrupo")],
        ));
        links.push((next_id, 0, next_id + 1, 0));
        next_id += 1;
        input = "T2";
    }
    let examine = next_id;
    nodes.push(node(
        examine,
        "Examine",
        Some(input),
        &[("Salida1", Some("YES")), ("Salida2", Some("NO"))],
        &[("Grupo", "UnGrupo")],
    ));
    next_id += 1;
    links.push((examine, 0, next_id, 0));
    nodes.push(node(
        next_id,
        "Terminate",
        Some("YES"),
        &[],
        &[("Unidades", "0")],
    ));
    next_id += 1;
    links.push((examine, 1, next_id, 0));
    nodes.push(node(
        next_id,
        "Terminate",
        Some("NO"),
        &[],
        &[("Unidades", "0")],
    ));
    wired(&nodes, &links, 4000.0)
}

#[test]
fn the_examine_class_goes_on_when_the_transaction_is_in_the_group() {
    // The class examines the transaction: its second output says "No en
    // [Grupo]", so a transaction that is not in the group takes it.
    let program = examination_model(true);
    let result = run(&program, 400.0, 1);
    let yes = entries(&program, &result, "YES");
    let no = entries(&program, &result, "NO");
    assert!(yes > 100, "the bench creates transactions: {yes}");
    assert_eq!(no, 0, "every transaction joined the group before examining it");
}

#[test]
fn the_examine_class_takes_the_second_output_when_the_transaction_is_not_in_the_group() {
    let program = examination_model(false);
    let result = run(&program, 400.0, 1);
    let yes = entries(&program, &result, "YES");
    let no = entries(&program, &result, "NO");
    assert!(no > 100, "the bench creates transactions: {no}");
    assert_eq!(yes, 0, "no transaction is in the group");
}

/// Two flows paired through `MATCH` blocks, and a gate that evaluates the
/// label the transaction carries.
fn gate_model(rotulo: &'static str) -> genmsi_core::Program {
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A1"))],
            &[("Media", "1"), ("Modificador", "0"), ("Cantidad", "20")],
        ),
        node(
            2,
            "Match",
            Some("A1"),
            &[("Salida1", Some("A2")), ("Salida2", None)],
            &[("RotuloPropio", "PA"), ("RotuloMatch", "PB")],
        ),
        node(
            3,
            "Generate",
            None,
            &[("Salida1", Some("B1"))],
            &[("Media", "1"), ("Modificador", "0"), ("Cantidad", "20")],
        ),
        node(
            4,
            "Match",
            Some("B1"),
            &[("Salida1", Some("B2")), ("Salida2", None)],
            &[("RotuloPropio", "PB"), ("RotuloMatch", "PA")],
        ),
        node(
            5,
            "GateM",
            Some("A2"),
            &[("Salida1", Some("YES")), ("Salida2", Some("NO"))],
            &[("Rotulo", rotulo), ("op", "Maching")],
        ),
        node(6, "Terminate", Some("YES"), &[], &[("Unidades", "0")]),
        node(7, "Terminate", Some("NO"), &[], &[("Unidades", "0")]),
    ];
    let links = [
        (1, 0, 2, 0),
        (3, 0, 4, 0),
        (2, 0, 5, 0),
        (4, 0, 5, 0),
        (5, 0, 6, 0),
        (5, 1, 7, 0),
    ];
    wired(&nodes, &links, 100.0)
}

#[test]
fn the_gate_m_class_lets_the_transaction_go_on_when_its_label_matches() {
    // The class of the original waits for a label: the transaction that was
    // matched at the block `PA` carries the label `PA`, and the gate that
    // names it lets it go on.
    let program = gate_model("PA");
    let result = run(&program, 100.0, 1);
    let yes = entries(&program, &result, "YES");
    let no = entries(&program, &result, "NO");
    assert!(yes > 0, "the transactions matched at PA go on: {yes}");
    assert_eq!(
        yes, no,
        "half of the transactions carry the label PA and half carry PB"
    );
}

#[test]
fn the_gate_m_class_takes_the_second_output_when_the_label_does_not_match() {
    let program = gate_model("ZZZ");
    let result = run(&program, 100.0, 1);
    assert_eq!(
        entries(&program, &result, "YES"),
        0,
        "no transaction carries the label ZZZ"
    );
    assert!(entries(&program, &result, "NO") > 0, "the gate fails");
}


/// The `SelectF` class of the editor picks the first facility that is free
/// and writes its number in the parameter.
#[test]
fn the_select_f_class_picks_the_first_free_facility() {
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("T1"))],
            &[("Media", "1"), ("Modificador", "0")],
        ),
        node(
            2,
            "SelectF",
            Some("T1"),
            &[("Salida1", Some("T2")), ("Salida2", None)],
            &[
                ("Param", "param"),
                ("minimo", "1"),
                ("maximo", "10"),
                ("op", "Libre"),
            ],
        ),
        node(
            3,
            "Test",
            Some("T2"),
            &[("Salida1", Some("YES")), ("Salida2", Some("NO"))],
            &[("1_Valor", "P:param"), ("2_OP", "="), ("3_Valor", "1")],
        ),
        node(4, "Terminate", Some("YES"), &[], &[("Unidades", "0")]),
        node(5, "Terminate", Some("NO"), &[], &[("Unidades", "0")]),
    ];
    let links = [(1, 0, 2, 0), (2, 0, 3, 0), (3, 0, 4, 0), (3, 1, 5, 0)];
    let program = wired(&nodes, &links, 400.0);
    let result = run(&program, 400.0, 1);
    assert!(
        entries(&program, &result, "YES") > 100,
        "the class picks the first facility"
    );
    assert_eq!(
        entries(&program, &result, "NO"),
        0,
        "every transaction picks the same facility"
    );
}

/// A `MATCH` block whose conjugate is itself holds its transactions: the
/// report of the original for the bench of the class shows every transaction
/// waiting at the `MATCH` block instead of pairing with the others.
#[test]
fn a_match_block_does_not_pair_with_the_transactions_of_its_own_block() {
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A1"))],
            &[("Media", "1"), ("Modificador", "0")],
        ),
        node(
            2,
            "Match",
            Some("A1"),
            &[("Salida1", Some("A2")), ("Salida2", None)],
            &[("RotuloPropio", "AAA"), ("RotuloMatch", "AAA")],
        ),
        node(3, "Terminate", Some("A2"), &[], &[("Unidades", "0")]),
    ];
    let links = [(1, 0, 2, 0), (2, 0, 3, 0)];
    let program = wired(&nodes, &links, 400.0);
    let result = run(&program, 400.0, 1);
    assert_eq!(
        entries(&program, &result, "A2"),
        0,
        "no transaction is released: the conjugate is the block itself"
    );
    let waiting = program
        .blocks
        .iter()
        .position(|block| block.class == "Match")
        .map(|index| result.blocks[index].current)
        .expect("the model has a match block");
    assert!(
        waiting > 100,
        "every transaction waits at the match block: {waiting}"
    );
}
