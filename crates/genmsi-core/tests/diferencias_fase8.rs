//! The four differences of form that phase eight closes, one test each.
//!
//! Every test pins one rule of the original, measured against it: the
//! reports the original left in `fixtures/outputs/` and the listings the
//! original wrote under Wine (`scripts/wine-oracle/`).
//!
//! - **The rows of the connections.** The original writes one row per
//!   statement of the listing, and the connections the listing writes when
//!   the flow breaks are rows too: the `TP10` report has 69 rows and ours
//!   had 66, the `TP3` report has 108 and ours had 103. The connection row
//!   carries the number the statement has in the listing in its `LOC` column
//!   (`48 TRANSFER ,A_Assign6`).
//!
//! - **The order of the storages.** The original writes the storages of its
//!   report in the order the model declares them (`MIGRACION`,
//!   `ENTREGAEQUIPAJE`, `SALIDA` in the `TP3` report) and we wrote them in
//!   the alphabetical one.
//!
//! - **The label of the section of the tables.** The original writes `TABLAS`
//!   once, at the beginning of the section, and we wrote it again before
//!   every table.
//!
//! - **The generator with an empty mean.** The original writes
//!   `GENERATE ,,,1` (the `TP10` report, row 52) and we wrote
//!   `GENERATE 0,,,1`: the mean that the model does not set is empty, not
//!   zero.

mod support;

use genmsi_core::Program;
use genmsi_core::gpss::render;
use genmsi_core::report::render as render_report;
use support::{node, run, wired};

/// The statements of a program, with their padding removed.
fn statements(program: &Program) -> Vec<String> {
    render(program)
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect()
}

/// The rows of the block table of a report: the line, the `LOC` and the text
/// of every row.
fn block_rows(report: &str) -> Vec<(usize, String, String)> {
    let mut rows = Vec::new();
    let mut in_table = false;
    for line in report.lines() {
        if line.contains("ENTRY CURRENT RETRY") {
            in_table = true;
            continue;
        }
        if !in_table {
            continue;
        }
        if line.trim().is_empty() {
            break;
        }
        let number: usize = line
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .parse()
            .unwrap_or_default();
        let Some(row) = line.get(28..) else {
            continue;
        };
        let (loc, text) = row
            .trim_start()
            .split_once("  ")
            .unwrap_or((row.trim_start(), ""));
        rows.push((number, loc.trim().to_string(), text.trim_end().to_string()));
    }
    rows
}

/// A model whose flow breaks: the second `Test` sends the transaction to a
/// block the listing visited before, so the listing writes a connection.
///
/// The listing walks the flow in depth: the first `Test` sends the
/// transaction to the `A_Terminate` of the first branch, the second `Test`
/// sends it to the same block, and the listing visits the second `Test`
/// after it, so the connection that routes the transaction back to the
/// `A_Terminate` is a statement of its own.
fn broken_flow() -> Program {
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A"))],
            &[("Media", "10"), ("Modificador", "0")],
        ),
        node(
            2,
            "Test",
            Some("A"),
            &[("Salida1", Some("C")), ("Salida2", Some("B"))],
            &[("1_Valor", "P%3AParam"), ("2_OP", "%3D"), ("3_Valor", "1")],
        ),
        node(
            3,
            "Test",
            Some("B"),
            &[("Salida1", Some("C")), ("Salida2", Some("D"))],
            &[("1_Valor", "P%3AParam"), ("2_OP", "%3D"), ("3_Valor", "0")],
        ),
        node(4, "Terminate", Some("C"), &[], &[("Unidades", "0")]),
        node(5, "Terminate", Some("D"), &[], &[("Unidades", "0")]),
    ];
    wired(
        &nodes,
        &[(1, 0, 2, 0), (2, 0, 4, 0), (2, 1, 3, 0), (3, 0, 4, 0), (3, 1, 5, 0)],
        3600.0,
    )
}

/// The connections of the listing are rows of the block table.
///
/// The original writes one row per statement of the listing, and the
/// connection the listing writes when the flow breaks is a statement of its
/// own: the `TP10` report has 69 rows (ours had 66) and the `TP3` report
/// has 108 (ours had 103).
#[test]
fn the_connections_of_the_listing_are_rows_of_the_block_table() {
    let program = broken_flow();
    let listing = statements(&program);
    assert!(
        listing
            .iter()
            .any(|line| line.starts_with("TRANSFER ,A_Terminate")),
        "the listing writes the connection: {listing:?}"
    );

    let result = run(&program, 3600.0, 1);
    let report = render_report(&result, &program);
    let rows = block_rows(&report);
    let connection = rows
        .iter()
        .find(|(_, _, text)| text.starts_with("TRANSFER"))
        .unwrap_or_else(|| panic!("the report has no row of the connection: {report}"));
    assert_eq!(
        connection.2, "TRANSFER        ,A_Terminate",
        "the row of the connection: {report}"
    );
    // The connection has no name of its own: the `LOC` column holds the
    // number the statement has in the listing.
    assert!(
        connection.1.chars().all(|c| c.is_ascii_digit()) && !connection.1.is_empty(),
        "the connection has the number of its statement: {report}"
    );
    // The report writes one row per statement of the listing.
    assert_eq!(
        rows.len(),
        listing.len(),
        "the report writes one row per statement of the listing: {report}"
    );
}

/// The number of the connection is the number of its statement.
///
/// The `LOC` column of the original's report holds the number the statement
/// has in the listing when the statement carries no name: the `TP10` report
/// writes `48 TRANSFER ,A_Assign6`, and the statement is the forty-eighth
/// of the listing.
#[test]
fn the_connection_carries_the_number_of_its_statement() {
    let program = broken_flow();
    let listing = statements(&program);
    let position = listing
        .iter()
        .position(|line| line.starts_with("TRANSFER"))
        .expect("the listing writes the connection")
        + 1;
    let result = run(&program, 3600.0, 1);
    let report = render_report(&result, &program);
    let rows = block_rows(&report);
    let connection = rows
        .iter()
        .find(|(_, _, text)| text.starts_with("TRANSFER"))
        .expect("the report has the row of the connection");
    assert_eq!(
        connection.1,
        position.to_string(),
        "the connection carries the number of its statement: {report}"
    );
}

/// The storages of the report come in the order the model declares them.
///
/// The original writes the storages of its report in the order of the
/// declaration (`MIGRACION`, `ENTREGAEQUIPAJE` and `SALIDA` in the `TP3`
/// report) and we wrote them in the alphabetical one.
#[test]
fn the_storages_of_the_report_come_in_the_order_of_the_declaration() {
    let nodes = [
        node(
            1,
            "StorageDef",
            None,
            &[],
            &[("Storage", "MIGRACION"), ("Tamaño", "3")],
        ),
        node(
            2,
            "StorageDef",
            None,
            &[],
            &[("Storage", "ENTREGAEQUIPAJE"), ("Tamaño", "12")],
        ),
        node(
            3,
            "StorageDef",
            None,
            &[],
            &[("Storage", "SALIDA"), ("Tamaño", "8")],
        ),
    ];
    let program = wired(&nodes, &[], 3600.0);
    let result = run(&program, 3600.0, 1);
    let report = render_report(&result, &program);
    let section = report
        .split_once("STORAGE           CAP.")
        .map(|(_, rest)| rest)
        .expect("the report has a section of the storages");
    let names: Vec<usize> = ["MIGRACION", "ENTREGAEQUIPAJE", "SALIDA"]
        .iter()
        .map(|name| {
            section
                .find(name)
                .unwrap_or_else(|| panic!("{name} is in the section of the storages"))
        })
        .collect();
    assert!(
        names.windows(2).all(|pair| pair[0] < pair[1]),
        "the storages come in the order of the declaration: {section}"
    );
}

/// The label of the section of the tables is written once.
///
/// The original writes `TABLAS` once, at the beginning of the section, and
/// `QTABLES` once as well (the `TP3` report has two queue tables and one
/// label); we wrote the label again before every table.
#[test]
fn the_label_of_the_section_of_the_tables_is_written_once() {
    let nodes = [
        node(
            1,
            "Table",
            None,
            &[],
            &[
                ("NombreTabla", "TBL1"),
                ("Atributo", "P:Atrib"),
                ("PrimerIntervalo", "0"),
                ("TamFrecuencia", "10"),
                ("Cantidad", "10"),
            ],
        ),
        node(
            2,
            "Table",
            None,
            &[],
            &[
                ("NombreTabla", "TBL2"),
                ("Atributo", "P:Atrib"),
                ("PrimerIntervalo", "0"),
                ("TamFrecuencia", "10"),
                ("Cantidad", "10"),
            ],
        ),
        node(
            3,
            "QTable",
            None,
            &[],
            &[
                ("NombreTabla", "QTBL1"),
                ("Queue", "Q1"),
                ("PrimerIntervalo", "0"),
                ("TamFrecuencia", "10"),
                ("Cantidad", "10"),
            ],
        ),
        node(
            4,
            "QTable",
            None,
            &[],
            &[
                ("NombreTabla", "QTBL2"),
                ("Queue", "Q2"),
                ("PrimerIntervalo", "0"),
                ("TamFrecuencia", "10"),
                ("Cantidad", "10"),
            ],
        ),
    ];
    let program = wired(&nodes, &[], 3600.0);
    let result = run(&program, 3600.0, 1);
    let report = render_report(&result, &program);
    assert_eq!(
        report.matches("TABLAS").count(),
        1,
        "the label of the section of the tables is written once: {report}"
    );
    assert_eq!(
        report.matches("QTABLES").count(),
        1,
        "the label of the section of the queue tables is written once: {report}"
    );
}

/// The generator with an empty mean writes an empty mean.
///
/// The original writes `GENERATE ,,,1` (the `TP10` report, row 52) where we
/// wrote `GENERATE 0,,,1`: the mean the model does not set is empty. The
/// limit is the fourth operand of the statement, as the original writes it
/// (`GENERATE 90,30,,100` for a generator of 90 units with 100 of them).
#[test]
fn the_generator_with_an_empty_mean_writes_an_empty_mean() {
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A"))],
            &[("Media", ""), ("Cantidad", "1")],
        ),
        node(2, "Terminate", Some("A"), &[], &[("Unidades", "0")]),
    ];
    let program = wired(&nodes, &[(1, 0, 2, 0)], 3600.0);
    assert!(
        statements(&program)
            .iter()
            .any(|line| line == "GENERATE ,,,1"),
        "the listing writes the empty mean: {:?}",
        statements(&program)
    );

    let result = run(&program, 3600.0, 1);
    let report = render_report(&result, &program);
    let row = report
        .lines()
        .find(|line| line.contains("GENERATE"))
        .expect("the report has a row of the generator");
    assert!(
        row.contains("GENERATE        ,,,1"),
        "the row of the generator: {row:?}"
    );
}

/// The storages of the report come in the order the program finishes with
/// them.
///
/// The original writes the storages of its report in that order: the `TP2`
/// report writes `ACREDITACION`, `VACUNACION`, `SALAESPERA` and `EXIT`, the
/// order the listing of the original leaves them in (its listing is in
/// `fixtures/outputs/TP2__problema1.gpss.txt`). The transaction of the model
/// enters the storages in another order, so the rule is not the order of the
/// entrance.
#[test]
fn the_storages_of_the_report_come_in_the_order_the_program_finishes_with_them() {
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A"))],
            &[("Media", "10"), ("Modificador", "0")],
        ),
        node(
            2,
            "Enter",
            Some("A"),
            &[("Salida1", Some("B"))],
            &[("Storage", "ALMACEN1"), ("Cantidad", "1")],
        ),
        node(
            3,
            "Leave",
            Some("B"),
            &[("Salida1", Some("C"))],
            &[("Storage", "ALMACEN1"), ("Cantidad", "1")],
        ),
        node(
            4,
            "Enter",
            Some("C"),
            &[("Salida1", Some("D"))],
            &[("Storage", "ALMACEN2"), ("Cantidad", "1")],
        ),
        node(
            5,
            "Enter",
            Some("D"),
            &[("Salida1", Some("E"))],
            &[("Storage", "ALMACEN3"), ("Cantidad", "1")],
        ),
        node(
            6,
            "Leave",
            Some("E"),
            &[("Salida1", Some("F"))],
            &[("Storage", "ALMACEN3"), ("Cantidad", "1")],
        ),
        node(
            7,
            "Leave",
            Some("F"),
            &[("Salida1", Some("T"))],
            &[("Storage", "ALMACEN2"), ("Cantidad", "1")],
        ),
        node(8, "Terminate", Some("T"), &[], &[("Unidades", "0")]),
        node(
            9,
            "StorageDef",
            None,
            &[],
            &[("Storage", "ALMACEN1"), ("Tamaño", "3")],
        ),
        node(
            10,
            "StorageDef",
            None,
            &[],
            &[("Storage", "ALMACEN2"), ("Tamaño", "3")],
        ),
        node(
            11,
            "StorageDef",
            None,
            &[],
            &[("Storage", "ALMACEN3"), ("Tamaño", "3")],
        ),
    ];
    let links = [
        (1, 0, 2, 0),
        (2, 0, 3, 0),
        (3, 0, 4, 0),
        (4, 0, 5, 0),
        (5, 0, 6, 0),
        (6, 0, 7, 0),
        (7, 0, 8, 0),
    ];
    let program = wired(&nodes, &links, 3600.0);
    let result = run(&program, 3600.0, 1);
    let report = render_report(&result, &program);
    let section = report
        .split_once("STORAGE           CAP.")
        .map(|(_, rest)| rest)
        .expect("the report has a section of the storages");
    // The transaction enters `ALMACEN1`, then `ALMACEN2`, then `ALMACEN3`, and
    // leaves them in another order: `ALMACEN1`, `ALMACEN3` and `ALMACEN2`.
    let names: Vec<usize> = ["ALMACEN1", "ALMACEN3", "ALMACEN2"]
        .iter()
        .map(|name| {
            section
                .find(name)
                .unwrap_or_else(|| panic!("{name} is in the section of the storages"))
        })
        .collect();
    assert!(
        names.windows(2).all(|pair| pair[0] < pair[1]),
        "the storages come in the order the program finishes with them: {section}"
    );
}

/// The limit of the generator is the fourth operand of the statement.
///
/// The original writes `GENERATE 90,30,,100` for a generator whose mean is
/// 90, whose deviation is 30 and whose `Cantidad` is 100: the limit is the
/// fourth operand and the priority the fifth.
#[test]
fn the_limit_of_the_generator_is_the_fourth_operand() {
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A"))],
            &[("Media", "90"), ("Modificador", "30"), ("Cantidad", "100")],
        ),
        node(2, "Terminate", Some("A"), &[], &[("Unidades", "0")]),
    ];
    let program = wired(&nodes, &[(1, 0, 2, 0)], 3600.0);
    assert!(
        statements(&program)
            .iter()
            .any(|line| line == "GENERATE 90,30,,100"),
        "the listing writes the limit in the fourth operand: {:?}",
        statements(&program)
    );
}
