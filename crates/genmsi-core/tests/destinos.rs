//! The destinations the original writes in the listing: the block a
//! transaction goes to when the test fails.
//!
//! Every assertion here is pinned to the original's own answer: the listing
//! the original wrote for the probes of `scripts/wine-oracle/probes/select/`
//! and for the model of the original in `fixtures/outputs/`.

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

/// A model with one selection whose failure output ("No selec") is
/// connected, and one that is not.
fn selection(class: &'static str, props: &[(&'static str, &'static str)]) -> (Program, Program) {
    let model = |failure: Option<&'static str>| {
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
                class,
                Some("A"),
                &[("Salida1", Some("B")), ("Salida2", failure)],
                props,
            ),
            node(3, "Terminate", Some("B"), &[], &[]),
            node(4, "Terminate", Some("C"), &[], &[]),
        ];
        let mut links = vec![(1, 0, 2, 0), (2, 0, 3, 0)];
        if failure.is_some() {
            links.push((2, 1, 4, 0));
        }
        wired(&nodes, &links, 3600.0)
    };
    (model(Some("C")), model(None))
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

/// The transactions that entered the block `label`, as the report says.
fn entries(report: &str, label: &str) -> i64 {
    block_rows(report)
        .iter()
        .find(|row| row.split_whitespace().any(|field| field == label))
        .and_then(|row| row.split_whitespace().nth(1).and_then(|count| count.parse().ok()))
        .unwrap_or_else(|| panic!("no row for {label} in the report"))
}

#[test]
fn the_selection_writes_the_destination_of_its_failure_output() {
    // The original writes the block the transaction goes to when the
    // selection finds nothing, as the last operand of the statement and
    // after two empty ones:
    //
    //     A_SelectS   SELECT  SE  param,1,10,,,A_Terminate1
    //
    // (the probe `sels_SE` of `scripts/wine-oracle/probes/select/`, and the
    // model of the original `fixtures/outputs/TP5__burgerShop.gpss.txt`:
    // "A_SelectF  SELECT  NU  caja,1,V$cantCajas,,,A_Select").
    let (connected, free) = selection(
        "SelectS",
        &[("Param", "param"), ("minimo", "1"), ("maximo", "10"), ("op", "Vacio")],
    );
    assert_eq!(
        named(&connected, "A_SelectS"),
        "A_SelectS SELECT SE param,1,10,,,A_Terminate1"
    );
    // Without the failure output the original writes no destination at all.
    assert_eq!(named(&free, "A_SelectS"), "A_SelectS SELECT SE param,1,10");

    let (connected, free) = selection(
        "SelectF",
        &[("Param", "param"), ("minimo", "1"), ("maximo", "10"), ("op", "Libre")],
    );
    assert_eq!(
        named(&connected, "A_SelectF"),
        "A_SelectF SELECT NU param,1,10,,,A_Terminate1"
    );
    assert_eq!(named(&free, "A_SelectF"), "A_SelectF SELECT NU param,1,10");

    let (connected, free) = selection(
        "SelectL",
        &[("Param", "param"), ("minimo", "1"), ("maximo", "10"), ("op", "Set")],
    );
    assert_eq!(
        named(&connected, "A_SelectL"),
        "A_SelectL SELECT LS param,1,10,,,A_Terminate1"
    );
    assert_eq!(named(&free, "A_SelectL"), "A_SelectL SELECT LS param,1,10");

    // The class that compares a value has a field of its own for the
    // destination, and it writes it even when it is empty
    // (`SELECT E param,1,10,*1,Q,`).
    let (connected, free) = selection(
        "SelectSNA",
        &[
            ("Param", "param"),
            ("minimo", "1"),
            ("maximo", "10"),
            ("op", "="),
            ("valorComp", "*1"),
            ("ClaseSNA", "Q"),
        ],
    );
    assert_eq!(
        named(&connected, "A_SelectSNA"),
        "A_SelectSNA SELECT E param,1,10,*1,Q,A_Terminate1"
    );
    assert_eq!(
        named(&free, "A_SelectSNA"),
        "A_SelectSNA SELECT E param,1,10,*1,Q,"
    );

    // The `Select` class has no failure output: its statement names the
    // class of the entity and nothing else.
    let (connected, free) = selection(
        "Select",
        &[
            ("Param", "param"),
            ("minimo", "1"),
            ("maximo", "10"),
            ("op", "Minimo"),
            ("ClaseSNA", "Q"),
        ],
    );
    assert_eq!(
        named(&connected, "A_Select"),
        "A_Select SELECT MIN param,1,10,,Q"
    );
    assert_eq!(named(&free, "A_Select"), "A_Select SELECT MIN param,1,10,,Q");
}

#[test]
fn the_test_writes_the_destination_of_its_failure_output() {
    // The original writes the block the transaction goes to when the
    // comparison fails as the third operand of the statement:
    //
    //     A_Test   TEST  E  1,2,A_Terminate1
    //
    // (the probe `scripts/wine-oracle/probes/TestFalso.gmdl`, whose report
    // the original left with that line).
    let model = |failure: Option<&'static str>| {
        let nodes = vec![
            node(
                1,
                "Generate",
                None,
                &[("Salida1", Some("A"))],
                &[("Media", "60"), ("Modificador", "30")],
            ),
            node(
                2,
                "Test",
                Some("A"),
                &[("Salida1", Some("B")), ("Salida2", failure)],
                &[("1_Valor", "1"), ("2_OP", "="), ("3_Valor", "2")],
            ),
            node(3, "Terminate", Some("B"), &[], &[]),
            node(4, "Terminate", Some("C"), &[], &[]),
        ];
        let mut links = vec![(1, 0, 2, 0), (2, 0, 3, 0)];
        if failure.is_some() {
            links.push((2, 1, 4, 0));
        }
        wired(&nodes, &links, 3600.0)
    };
    assert_eq!(
        named(&model(Some("C")), "A_Test"),
        "A_Test TEST E 1,2,A_Terminate1"
    );
    // Without the failure output the original writes no destination at all
    // (`A_Test  TEST  E  1,1`, the bench of the oracle).
    assert_eq!(named(&model(None), "A_Test"), "A_Test TEST E 1,2");
}

#[test]
fn the_availability_of_a_storage_decides_the_selection() {
    // The manual of the original says of the two operators of the
    // availability of a storage: "El storage tiene que estar disponible
    // para que sea seleccionado" (`SV`) and "El storage no tiene que estar
    // disponible para que sea seleccionado" (`SNV`)
    // (`VISUALSIS.HLP`).
    //
    // The state of a storage starts available, and only the statements
    // `SAVAIL` and `SUNAVAIL` change it; the library of classes of the
    // original has none of them, so every storage of a model of the editor
    // is available: `SV` picks it and `SNV` picks none.
    //
    // The probes of `scripts/wine-oracle/probes/select/` say the same with
    // the original: with the storage `1` declared, the `SV` selection
    // leaves the number of the entity in the parameter and the `SNV` one
    // leaves a zero.
    let model = |op: &'static str| {
        let nodes = vec![
            node(
                1,
                "StorageDef",
                None,
                &[],
                &[("Storage", "1"), ("Tamaño", "10")],
            ),
            node(
                2,
                "Generate",
                None,
                &[("Salida1", Some("A"))],
                &[("Media", "60"), ("Modificador", "0")],
            ),
            node(
                3,
                "SelectS",
                Some("A"),
                &[("Salida1", Some("B")), ("Salida2", None)],
                &[("Param", "P"), ("minimo", "1"), ("maximo", "10"), ("op", op)],
            ),
            // The selection leaves the number of the entity it picked in the
            // parameter `P` (and a zero when it picks none), so the test says
            // which of the two happened.
            node(
                4,
                "Test",
                Some("B"),
                &[("Salida1", Some("A_Terminate")), ("Salida2", Some("A_Terminate1"))],
                &[("1_Valor", "P:P"), ("2_OP", "="), ("3_Valor", "1")],
            ),
            node(5, "Terminate", Some("A_Terminate"), &[], &[]),
            node(6, "Terminate", Some("A_Terminate1"), &[], &[]),
        ];
        let links = vec![(2, 0, 3, 0), (3, 0, 4, 0), (4, 0, 5, 0), (4, 1, 6, 0)];
        let program = wired(&nodes, &links, 3600.0);
        let report = render_report(&support::run(&program, 3600.0, 1), &program);
        (entries(&report, "A_Terminate"), entries(&report, "A_Terminate1"))
    };
    // The available storage is picked: every transaction leaves the number
    // of the entity in the parameter.
    let (picked, missed) = model("Disponible");
    assert!(picked > 0, "the SV selection picked nothing: {picked}/{missed}");
    assert_eq!(missed, 0, "the SV selection missed a transaction");
    // The unavailable one is not: nothing is picked and the parameter stays
    // at zero.
    let (picked, missed) = model("NoDisponible");
    assert_eq!(picked, 0, "the SNV selection picked something");
    assert!(missed > 0, "the SNV selection did not run");
}

#[test]
fn the_selection_by_the_availability_of_a_storage_writes_its_letter() {
    // The `op_Logic_Stor` list of the original has six operators, and the
    // `SelectS` class takes its `op` from it: `SF` a full storage, `SNF` one
    // that is not full, `SE` an empty one, `SNE` one that is not empty, `SV`
    // one that is available and `SNV` one that is not
    // (`Genmsi_20230711.backup/gmacroconfig.gmcfg`, and the listing of the
    // original in `scripts/wine-oracle/probes/select/`).
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
    assert_eq!(
        named(&model("Disponible"), "A_SelectS"),
        "A_SelectS SELECT SV param,1,10"
    );
    assert_eq!(
        named(&model("NoDisponible"), "A_SelectS"),
        "A_SelectS SELECT SNV param,1,10"
    );
}

#[test]
fn the_examination_without_a_destination_is_reported() {
    // The manual of the original says of the third operand of the `EXAMINE`:
    // "Número de bloque. Destino alternativo para la transacción activa si
    // no es encontrado ningún miembro. Requerido" (`VISUALSIS.HLP`), and the
    // engine of the original refuses a model whose `EXAMINE` has no such
    // operand: "Error en linea 1 -> Faltan parámetros".
    //
    // This repository compiles the model anyway, so it says the finding out
    // loud.
    let model = |failure: Option<&'static str>| {
        let nodes = vec![
            node(
                1,
                "Generate",
                None,
                &[("Salida1", Some("A"))],
                &[("Media", "60"), ("Modificador", "30")],
            ),
            node(
                2,
                "Examine",
                Some("A"),
                &[("Salida1", Some("B")), ("Salida2", failure)],
                &[("Grupo", "UnGrupo")],
            ),
            node(3, "Terminate", Some("B"), &[], &[]),
            node(4, "Terminate", Some("C"), &[], &[]),
        ];
        let mut links = vec![(1, 0, 2, 0), (2, 0, 3, 0)];
        if failure.is_some() {
            links.push((2, 1, 4, 0));
        }
        wired(&nodes, &links, 3600.0)
    };
    let without = model(None);
    let diagnostic = without
        .diagnostics
        .iter()
        .find(|message| message.contains("Faltan parámetros"))
        .expect("the compiler names the examination");
    assert!(diagnostic.contains("Examine"), "{diagnostic}");
    // With the destination there is nothing to say.
    let with = model(Some("C"));
    assert!(
        !with
            .diagnostics
            .iter()
            .any(|message| message.contains("Faltan parámetros")),
        "{:?}",
        with.diagnostics
    );
}
