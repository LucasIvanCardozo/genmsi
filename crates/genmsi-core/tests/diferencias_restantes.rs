//! The differences that were still open when this work unit started.
//!
//! Every test here pins one rule of the original, measured against it under
//! Wine (`scripts/wine-oracle/run_oracle.sh`) or against the answers the
//! original left in `fixtures/outputs/`:
//!
//! - **The order of the listing.** The original writes the blocks of the
//!   program in the order a transaction walks them, and the walk is a
//!   depth-first one: the branch of a port comes after the whole branch of
//!   the port before it. The listing of the model `TP10__tp10` is compared,
//!   statement by statement, with the block table of the original's own
//!   report.
//!
//! - **The row of the multifurcation.** The `LOC` column of the report is
//!   right-aligned in fifteen columns, and a name that does not fit eats the
//!   space that separates it from the `RETRY` column: the row of the
//!   `MultifurcacionP` reads `0MultifurcacionP` and not
//!   `0 MultifurcacionP`.
//!
//! - **The generator that waits at the gate.** The transaction that waits at
//!   a gate that does not open is still counted in the block it came from:
//!   the generator of the bench `GateL` reads `62 62` in the original's
//!   report, where we read `60 1`.
//!
//! - **The order of the tables.** The original writes the tables of its
//!   report the other way round than the model declares them: the probe
//!   `tablas_tres` (three tables, declared `TBL2`, `TBL1` and `TBL3`) gives a
//!   report that starts with `TBL3` and goes on with `TBL1` and `TBL2`.

mod support;

use std::path::Path;

use genmsi_core::Program;
use genmsi_core::gpss::render;
use genmsi_core::program::compile;
use genmsi_core::report::render as render_report;
use support::{block, node, run, wired};

/// The root of the repository, from the crate of the core.
fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The names the listing writes in the first column, in the order it writes
/// them.
///
/// The column is fifteen characters wide; the statements that carry no name
/// of their own (the connections the listing writes and the second statement
/// of a compound block) are left out, the way the `LOC` column of the
/// report leaves them out.
fn listing_names(program: &Program) -> Vec<String> {
    let order = genmsi_core::gpss::listing_order(program);
    let names = genmsi_core::gpss::block_names(program);
    order
        .iter()
        .filter_map(|index| {
            let name = names.get(*index)?.trim();
            (!name.is_empty()).then(|| name.to_string())
        })
        .collect()
}

/// The names the `LOC` column of the original's report holds, in the order of
/// the table.
///
/// The column is right-aligned in fifteen columns and ends at the column
/// forty-three; the statements with no name of their own carry the number
/// they have in the program, and those are left out.
fn report_names(path: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(path).expect("the original's report is there");
    let mut names = Vec::new();
    let mut in_table = false;
    for line in text.lines() {
        if line.contains("ENTRY CURRENT RETRY") {
            in_table = true;
            continue;
        }
        if !in_table {
            continue;
        }
        if line.trim().is_empty() {
            // The block table ends with an empty line.
            break;
        }
        let Some(row) = line.get(28..43) else {
            continue;
        };
        let loc = row.trim();
        if loc.is_empty() || loc.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        names.push(loc.to_string());
    }
    names
}

/// The listing walks the flow in depth.
///
/// The original writes the blocks of the program in the order a transaction
/// walks them, and the walk is a depth-first one: the branch of the second
/// port of a block comes after the whole branch of its first port, and a
/// block nothing leads to comes where the flow reaches it.
///
/// The model `TP10__tp10` has both: the `A_Unlink` has two outputs and the
/// `A_RemoveTrans` before it has one, and the `GENERATE` of the group
/// `EsperaHastaAbrir` comes before the `A_Logic` it feeds. The listing is
/// compared, statement by statement, with the block table of the original's
/// own report (`fixtures/outputs/TP10__tp10.report.txt`).
#[test]
fn the_listing_walks_the_flow_in_depth() {
    let model_path = root().join("fixtures/models/TP10__tp10.gmdl");
    let model = genmsi_format::load(&model_path).expect("the model loads");
    let program = compile(&model, "TP10").expect("the model compiles");
    let ours = listing_names(&program);
    let theirs = report_names(&root().join("fixtures/outputs/TP10__tp10.report.txt"));
    assert_eq!(
        ours, theirs,
        "the listing of the model does not walk the flow as the original does"
    );
}

/// The row of the multifurcation writes the name of the statement as the
/// original writes it.
///
/// The `LOC` column is right-aligned in fifteen columns and a name that does
/// not fit eats the space that separates it from the `RETRY` column: the
/// original's row of the bench `MultifurcacionP` reads
/// `    4       60       0     0MultifurcacionP  TRANSFER ,FN$MultifurcacionGO`.
#[test]
fn the_row_of_the_multifurcation_eats_the_space_of_its_column() {
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("MultifurcacionP"))],
            &[("Media", "60"), ("Modificador", "30")],
        ),
        node(
            2,
            "MultifurcacionP",
            Some("MultifurcacionP"),
            &[("Salida1", Some("A_Terminate"))],
            &[
                ("Parametro", "Param"),
                ("MultifurcacionGO", "MultifurcacionGO"),
            ],
        ),
        node(3, "Terminate", Some("A_Terminate"), &[], &[]),
    ];
    let program = wired(&nodes, &[(1, 0, 2, 0), (2, 0, 3, 0)], 3600.0);
    let result = run(&program, 3600.0, 1);
    let report = render_report(&result, &program);
    let row = report
        .lines()
        .find(|line| line.contains("MULTIFURCACIONP") || line.contains("MultifurcacionP"))
        .expect("the report has a row of the multifurcation");
    assert!(
        row.contains("0MultifurcacionP  TRANSFER        ,FN$MultifurcacionGO"),
        "the row of the multifurcation: {row:?}"
    );
}

/// The generator counts the transactions that wait at the gate.
///
/// The original's report of the bench `GateL` (a generator, a gate that
/// never opens and a terminate) reads `58 58` in the row of the generator:
/// every transaction that waits at the gate is still counted in the block it
/// came from. The same rule of the `SEIZE`: the transaction goes back to the
/// block it came from.
#[test]
fn the_generator_counts_the_transactions_that_wait_at_the_gate() {
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
    let program = wired(&nodes, &[(1, 0, 2, 0), (2, 0, 3, 0)], 3600.0);
    let result = run(&program, 3600.0, 1);
    let generator = block(&program, "Generate");
    assert!(
        result.blocks[generator].current > 1,
        "the generator holds the transactions that wait at the gate: {:?}",
        result.blocks[generator]
    );
    assert_eq!(
        result.blocks[generator].current, result.blocks[generator].entries,
        "every transaction that waits at the gate is counted in the generator"
    );
    // The row of the report says the same: the current count is the entry
    // count.
    let report = render_report(&result, &program);
    let row = report
        .lines()
        .find(|line| line.contains("GENERATE"))
        .expect("the report has a row of the generator");
    let numbers: Vec<&str> = row.split_whitespace().skip(1).take(3).collect();
    assert_eq!(numbers[1], numbers[0], "the row of the generator: {row:?}");
}

/// The tables of the report come the other way round than the model declares
/// them.
///
/// The probe `tablas_tres` declares the tables `TBL2`, `TBL1` and `TBL3`, in
/// that order, and the original's report writes `TBL3`, `TBL1` and `TBL2`:
/// the last table declared is the first one of the report. The listing, on
/// the other hand, keeps the order of the declaration (`TBL2`, `TBL1`,
/// `TBL3`).
#[test]
fn the_tables_come_the_other_way_round_than_the_declaration() {
    let mut nodes = Vec::new();
    let mut links = Vec::new();
    // The three tables, declared `TBL2`, `TBL1` and `TBL3`.
    for (index, name) in ["TBL2", "TBL1", "TBL3"].iter().enumerate() {
        nodes.push(node(
            index as u32 + 1,
            "Table",
            None,
            &[],
            &[
                ("NombreTabla", name),
                ("Atributo", "P:Atrib"),
                ("PrimerIntervalo", "0"),
                ("TamFrecuencia", "10"),
                ("Cantidad", "10"),
            ],
        ));
    }
    // A generator that tabulates the value `5` into `TBL1`.
    nodes.push(node(
        4,
        "Generate",
        None,
        &[("Salida1", Some("A_Assign"))],
        &[("Media", "100"), ("Modificador", "0")],
    ));
    nodes.push(node(
        5,
        "Assign",
        Some("A_Assign"),
        &[("Salida1", Some("A_Tabulate"))],
        &[("Parametro", "Atrib"), ("Valor", "5"), ("Operador", "=")],
    ));
    nodes.push(node(
        6,
        "Tabulate",
        Some("A_Tabulate"),
        &[("Salida1", Some("A_Terminate"))],
        &[("NombreTabla", "TBL1"), ("Peso", "1")],
    ));
    nodes.push(node(7, "Terminate", Some("A_Terminate"), &[], &[]));
    links.push((4, 0, 5, 0));
    links.push((5, 0, 6, 0));
    links.push((6, 0, 7, 0));
    let program = wired(&nodes, &links, 3600.0);

    // The listing keeps the order of the declaration.
    let listing = render(&program);
    let declared: Vec<usize> = ["TBL2", "TBL1", "TBL3"]
        .iter()
        .map(|name| listing.find(name).unwrap_or_else(|| panic!("{name} is in the listing")))
        .collect();
    assert!(
        declared.windows(2).all(|pair| pair[0] < pair[1]),
        "the listing writes the tables in the order the model declares them: {listing}"
    );

    // The report writes them the other way round.
    let result = run(&program, 3600.0, 1);
    let report = render_report(&result, &program);
    let section = report
        .split_once("TABLAS")
        .map(|(_, rest)| rest)
        .expect("the report has a section of the tables");
    let written: Vec<usize> = ["TBL3", "TBL1", "TBL2"]
        .iter()
        .map(|name| {
            section
                .find(name)
                .unwrap_or_else(|| panic!("{name} is in the section of the tables"))
        })
        .collect();
    assert!(
        written.windows(2).all(|pair| pair[0] < pair[1]),
        "the report writes the tables the other way round: {report}"
    );
}
