//! The shape of the GPSS listing and of the block table: five rules that the
//! original writes and that the listing or the report did not.
//!
//! Every assertion here is pinned to the original's own answer: the listing
//! of the original for the models under `fixtures/` and `scripts/wine-oracle/`,
//! and the report the original leaves in `/tmp` (the benches of the oracle).

mod support;

use genmsi_core::Program;
use genmsi_core::gpss::render;
use genmsi_core::report::render as render_report;
use support::{node, wired};

/// The statements of a program, with their padding removed.
fn statements(program: &Program) -> Vec<String> {
    render(program)
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect()
}

/// The statement that carries the name `name`.
fn named(program: &Program, name: &str) -> String {
    statements(program)
        .into_iter()
        .find(|line| line.split_whitespace().next().is_some_and(|first| first == name))
        .unwrap_or_else(|| panic!("no statement carries the name {name}"))
}

/// The rows of the block table of a report, with their padding removed.
fn block_rows(report: &str) -> Vec<String> {
    report
        .lines()
        .skip_while(|line| !line.contains("BLOCK_TYPE"))
        .skip(2)
        .take_while(|line| !line.trim().is_empty())
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect()
}

/// The counts of the header of a report: the line below the labels.
fn header(report: &str) -> String {
    let mut lines = report.lines();
    let values = lines
        .find(|line| line.contains("BLOCKS"))
        .and_then(|_| lines.next())
        .unwrap_or_default();
    values.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn the_examine_writes_the_alternative_destination() {
    // The original writes the block the transaction goes to when it is not
    // in the group: `EXAMINE UnGrupo,,A_Terminate1`
    // (scripts/wine-oracle/out/Examine.gpss.txt). Without it the original
    // refuses to compile the model ("Faltan parámetros").
    let nodes = vec![
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A"))],
            &[("Media", "60"), ("Modificador", "0")],
        ),
        node(
            2,
            "Examine",
            Some("A"),
            &[("Salida1", Some("B")), ("Salida2", Some("C"))],
            &[("Grupo", "UnGrupo")],
        ),
        node(3, "Terminate", Some("B"), &[], &[]),
        node(4, "Terminate", Some("C"), &[], &[]),
    ];
    let links = vec![(1, 0, 2, 0), (2, 0, 3, 0), (2, 1, 4, 0)];
    let program = wired(&nodes, &links, 3600.0);
    assert_eq!(
        named(&program, "A_Examine"),
        "A_Examine EXAMINE UnGrupo,,A_Terminate1"
    );
}

#[test]
fn the_gate_on_a_logic_switch_writes_the_reset_letter() {
    // The two operators of a logic switch are `LS` (set) and `LR` (reset):
    // the original writes `GateL1  GATE  LR  *Cuadra`
    // (fixtures/outputs/Problema_1__semaforo-1.gpss.txt, line 71).
    let model = |op: &'static str| {
        let nodes = vec![
            node(
                1,
                "Generate",
                None,
                &[("Salida1", Some("A"))],
                &[("Media", "60"), ("Modificador", "0")],
            ),
            node(
                2,
                "GateL",
                Some("A"),
                &[("Salida1", Some("B")), ("Salida2", None)],
                &[("LogicSwich", "1"), ("op", op)],
            ),
            node(3, "Terminate", Some("B"), &[], &[]),
        ];
        let links = vec![(1, 0, 2, 0), (2, 0, 3, 0)];
        wired(&nodes, &links, 3600.0)
    };
    assert_eq!(named(&model("Set"), "GateL"), "GateL GATE LS 1");
    assert_eq!(named(&model("Reset"), "GateL"), "GateL GATE LR 1");
}

#[test]
fn the_selection_by_the_state_of_a_storage_writes_its_letter() {
    // The `op_Logic_Stor` list of the original has four states, and the
    // operator of each is the one its code names: `SF` a full storage, `SNF`
    // one that is not full, `SE` an empty one and `SNE` one that is not empty.
    let model = |op: &'static str| {
        let nodes = vec![
            node(
                1,
                "Generate",
                None,
                &[("Salida1", Some("A"))],
                &[("Media", "60"), ("Modificador", "0")],
            ),
            node(
                2,
                "SelectS",
                Some("A"),
                &[("Salida1", Some("B")), ("Salida2", None)],
                &[("Param", "param"), ("minimo", "1"), ("maximo", "10"), ("op", op)],
            ),
            node(3, "Terminate", Some("B"), &[], &[]),
        ];
        let links = vec![(1, 0, 2, 0), (2, 0, 3, 0)];
        wired(&nodes, &links, 3600.0)
    };
    assert_eq!(named(&model("Lleno"), "A_SelectS"), "A_SelectS SELECT SF param,1,10");
    assert_eq!(
        named(&model("NoLleno"), "A_SelectS"),
        "A_SelectS SELECT SNF param,1,10"
    );
    assert_eq!(named(&model("Vacio"), "A_SelectS"), "A_SelectS SELECT SE param,1,10");
    assert_eq!(
        named(&model("NoVacio"), "A_SelectS"),
        "A_SelectS SELECT SNE param,1,10"
    );
}

#[test]
fn the_counter_takes_two_rows_in_the_block_table() {
    // The `Counter` class of the original writes two statements (a `SV` and
    // an `ASSI` in its macro code) and the block table lists the two:
    //
    //     2       62       0     0      A_Counter  SAVEVALUE       Count+,1
    //     3       62       0     0              3  ASSIGN          Nro,X$Count
    //
    // (scripts/wine-oracle/out/Counter.report.txt: the header says six
    // blocks where the model has five).
    let nodes = vec![
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A"))],
            &[("Media", "60"), ("Modificador", "0")],
        ),
        node(
            2,
            "Counter",
            Some("A"),
            &[("Salida1", Some("B"))],
            &[("SaveValue", "Count"), ("Incremento", "1"), ("Param", "Nro")],
        ),
        node(3, "Terminate", Some("B"), &[], &[]),
    ];
    let links = vec![(1, 0, 2, 0), (2, 0, 3, 0)];
    let program = wired(&nodes, &links, 3600.0);
    let report = render_report(&support::run(&program, 3600.0, 1), &program);
    let rows = block_rows(&report);
    let savevalue = rows
        .iter()
        .find(|row| row.contains("SAVEVALUE"))
        .expect("the row of the counter");
    assert!(savevalue.ends_with("SAVEVALUE Count+,1"), "{savevalue}");
    let assign = rows
        .iter()
        .find(|row| row.contains("ASSIGN"))
        .expect("the row of the assignment the counter writes");
    assert!(assign.ends_with("ASSIGN Nro,X$Count"), "{assign}");
    // The header counts the rows of the table: the counter takes two.
    assert_eq!(header(&report).split_whitespace().nth(2), Some("6"));
}

#[test]
fn the_select_writes_the_class_of_the_entity() {
    // The `entitySNAclass` list of the original has 38 classes, and the
    // `SELECT` statement carries the class as its last operand. The class `F`
    // is the occupancy of a facility: the original writes
    // `SELECT MIN param,1,10,,F`.
    let model = |class: &'static str| {
        let nodes = vec![
            node(
                1,
                "Generate",
                None,
                &[("Salida1", Some("A"))],
                &[("Media", "60"), ("Modificador", "0")],
            ),
            node(
                2,
                "Select",
                Some("A"),
                &[("Salida1", Some("B"))],
                &[
                    ("Param", "param"),
                    ("minimo", "1"),
                    ("maximo", "10"),
                    ("op", "Minimo"),
                    ("ClaseSNA", class),
                ],
            ),
            node(3, "Terminate", Some("B"), &[], &[]),
        ];
        let links = vec![(1, 0, 2, 0), (2, 0, 3, 0)];
        wired(&nodes, &links, 3600.0)
    };
    assert_eq!(named(&model("F"), "A_Select"), "A_Select SELECT MIN param,1,10,,F");
    assert_eq!(named(&model("Q"), "A_Select"), "A_Select SELECT MIN param,1,10,,Q");
    // Every class of the original's list compiles: the engine does not walk
    // all of them yet, but it does not refuse the model either.
    for class in [
        "SR", "FR", "CH", "CC", "CA", "QZ", "GN", "W", "N", "SC", "FC", "GT", "QC", "CM", "QM",
        "SM", "QA", "SA", "MP", "FV", "LR", "LS", "RN", "S", "F", "Q", "R", "BV", "FN", "V",
        "SV", "FT", "QT", "QX", "CT", "ST", "X", "P",
    ] {
        let program = model(class);
        assert_eq!(
            named(&program, "A_Select"),
            format!("A_Select SELECT MIN param,1,10,,{class}")
        );
    }
}
