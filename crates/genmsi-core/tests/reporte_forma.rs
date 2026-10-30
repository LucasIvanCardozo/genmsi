//! The shape of the report, row by row, against the original's own reports.
//!
//! Every assertion of this file is pinned to a row of the original's report
//! for the bench of a class (`scripts/wine-oracle/benches/<Class>.gmdl` and
//! the report the original leaves in `scripts/wine-oracle/out/`): the column
//! `LOC` holds the name of the statement, the rows of the statements write
//! the operands the original writes, the report has the sections the original
//! has, and the numbers have the shape the original gives them.
//!
//! The nine differences this file fixes are the ones of the notes
//! `odd/tasks/notas-reporte-forma.md`.

mod support;

use std::path::{Path, PathBuf};

use genmsi_core::report::render;
use genmsi_core::{SimConfig, Simulation};
use support::{node, run, wired};

/// The path of the bench of a class.
fn bench_path(class: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/wine-oracle/benches")
        .join(format!("{class}.gmdl"))
}

/// The report of the bench of a class.
fn report(class: &str) -> String {
    let path = bench_path(class);
    let model = genmsi_format::load(&path).expect("the bench loads");
    let program = genmsi_core::compile_model(&model).expect("the bench compiles");
    let result = Simulation::new(&program, SimConfig::default())
        .run()
        .expect("the bench runs");
    render(&result, &program)
}

/// The mode letters the original prints in a column of their own.
const MODES: &[&str] = &[
    "LS", "LR", "SE", "SNE", "SF", "SNF", "NU", "U", "MIN", "MAX", "LE", "GE", "NE", "E", "L", "G", "S", "R",
];

/// One row of the block table: the fields the original prints.
///
/// The row starts with four numeric columns; the `LOC` field runs fourteen
/// columns from there and the `BLOCK_TYPE` cell follows, with the letter of
/// the mode (when the statement has one) in a column of its own.
fn row(report: &str, loc: &str) -> (String, String, String, String, String, String) {
    let line = report
        .lines()
        .find(|line| {
            line.trim_start().starts_with(|c: char| c.is_ascii_digit())
                && line.get(29..).is_some_and(|rest| {
                    rest.split_whitespace()
                        .next()
                        .is_some_and(|field| field == loc)
                })
        })
        .unwrap_or_else(|| panic!("no block row with LOC {loc}\n{report}"));
    let mut fields = line.split_whitespace();
    let line = fields.next().unwrap_or_default().to_string();
    let entries = fields.next().unwrap_or_default().to_string();
    let current = fields.next().unwrap_or_default().to_string();
    let retry = fields.next().unwrap_or_default().to_string();
    let _ = fields.next(); // the `LOC` field itself
    let block_type = fields.next().unwrap_or_default().to_string();
    let rest: Vec<&str> = fields.collect();
    // The statements with a mode give the letter their own column.
    let (mode, operands) = match rest.split_first() {
        Some((first, tail)) if MODES.contains(first) => (*first, tail.join(",")),
        _ => ("", rest.join(",")),
    };
    (line, entries, current, retry, block_type, format!("{mode}|{operands}"))
}

/// The lines of a section of the report, from the one that names it.
fn section(report: &str, name: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut inside = false;
    for line in report.lines() {
        if line.starts_with(name) {
            inside = true;
            lines.push(line.to_string());
            continue;
        }
        if inside {
            if line.trim().is_empty() {
                break;
            }
            lines.push(line.to_string());
        }
    }
    lines
}

/// The first difference 1: the column `LOC` holds the name of the statement.
///
/// The original (`scripts/wine-oracle/benches/GateF.gmdl` and its report):
/// ```
///     1       64       1     0              1  GENERATE        60,30
///     2       63       0     0          GateF  GATE    NU      Recurso
///     3       63       0     0    A_Terminate  TERMINATE
///     4        2       1     0              4  GENERATE        3600
///     5        1       0     0              5  TERMINATE       1
/// ```
/// The name is the one of the statement (the gates take the name of their
/// module) and the statements without a name carry the number they have in
/// the program.
#[test]
fn the_loc_column_holds_the_name_of_the_statement() {
    if !benches_are_there() {
        return;
    }
    let text = report("GateF");
    assert_eq!(row(&text, "GateF").4, "GATE");
    assert_eq!(row(&text, "A_Terminate").4, "TERMINATE");
    // The statements without a name carry the number they have in the
    // program: the first one is the `1` and the clock's the `4`.
    let first = row(&text, "1");
    assert_eq!(first.4, "GENERATE");
    assert_eq!(first.5, "|60,30");
    let clock = row(&text, "4");
    assert_eq!(clock.4, "GENERATE");
    assert_eq!(clock.5, "|3600");
}

/// The name of a statement is not the label of the port it comes in.
///
/// The notes of the lot one (`odd/tasks/notas-fase7-lote1.md`, change 4)
/// ask for this: a `GateF` whose input port is labelled `A_GateF` writes the
/// row with the name of its statement, `GateF`.
#[test]
fn the_loc_is_the_name_of_the_statement_and_not_the_label_of_the_port() {
    if !benches_are_there() {
        return;
    }
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A_GateF"))],
            &[("Media", "10"), ("Modificador", "0")],
        ),
        node(
            2,
            "GateF",
            Some("A_GateF"),
            &[("Salida1", Some("A_Terminate"))],
            &[("Facility", "Recurso"), ("op", "Libre")],
        ),
        node(3, "Terminate", Some("A_Terminate"), &[], &[]),
    ];
    let links = [(1, 0, 2, 0), (2, 0, 3, 0)];
    let program = wired(&nodes, &links, 3600.0);
    let result = run(&program, 100.0, 1);
    let text = render(&result, &program);
    assert!(
        text.lines().any(|line| line.contains("GateF  GATE")),
        "the row of the gate carries the name of the statement:\n{text}"
    );
    assert!(
        !text.lines().any(|line| line.contains("A_GateF  GATE")),
        "the row of the gate does not carry the label of the port:\n{text}"
    );
}

/// The difference 2: the row of the `PREEMPT` writes its four operands.
///
/// The original (`scripts/wine-oracle/benches/FacilityInt.gmdl` and its
/// report):
/// ```
///     3       64       0     0              3  PREEMPT         Recurso,PR,,Remanente
/// ```
#[test]
fn the_preempt_row_writes_its_four_operands() {
    if !benches_are_there() {
        return;
    }
    let text = report("FacilityInt");
    assert_eq!(row(&text, "3").4, "PREEMPT");
    assert_eq!(row(&text, "3").5, "|Recurso,PR,,Remanente");
}

/// The difference 3: the row of the `SELECT` writes the letter of the
/// operator of the class.
///
/// The original:
/// ```
/// A_SelectS    SELECT   SNF     param,1,10                (bench SelectS)
/// A_SelectL    SELECT   LS      param,1,10                (bench SelectL)
/// A_SelectSNA  SELECT   E       param,1,10,*1,Q,          (bench SelectSNA)
/// ```
#[test]
fn the_select_rows_write_the_letter_of_the_operator() {
    if !benches_are_there() {
        return;
    }
    let text = report("SelectS");
    assert_eq!(row(&text, "A_SelectS").4, "SELECT");
    assert_eq!(row(&text, "A_SelectS").5, "SNF|param,1,10");
    let text = report("SelectL");
    assert_eq!(row(&text, "A_SelectL").4, "SELECT");
    assert_eq!(row(&text, "A_SelectL").5, "LS|param,1,10");
    let text = report("SelectSNA");
    assert_eq!(row(&text, "A_SelectSNA").4, "SELECT");
    assert_eq!(row(&text, "A_SelectSNA").5, "E|param,1,10,*1,Q,");
}

/// The difference 4: the row of the `REMOVE` of the `RemoveMe` class has no
/// letter of comparison.
///
/// The original:
/// ```
/// A_RemoveMe     REMOVE           UnGrupo                  (bench RemoveMe)
/// A_RemoveTrans  REMOVE   E       UnGrupo,1,,otro,1         (bench RemoveTrans)
/// ```
#[test]
fn the_remove_rows_write_the_group_and_the_comparison() {
    if !benches_are_there() {
        return;
    }
    let text = report("RemoveMe");
    assert_eq!(row(&text, "A_RemoveMe").4, "REMOVE");
    assert_eq!(row(&text, "A_RemoveMe").5, "|UnGrupo");
    let text = report("RemoveTrans");
    assert_eq!(row(&text, "A_RemoveTrans").4, "REMOVE");
    assert_eq!(row(&text, "A_RemoveTrans").5, "E|UnGrupo,1,,otro,1");
}

/// The difference 5: the row of the `LINK` writes the destination of the
/// statement.
///
/// The original (`scripts/wine-oracle/benches/Link.gmdl` and its report):
/// ```
///     2       56      55     0         A_Link  LINK            Cadena,FIFO,A_Terminate
/// ```
#[test]
fn the_link_row_writes_the_destination() {
    if !benches_are_there() {
        return;
    }
    let text = report("Link");
    assert_eq!(row(&text, "A_Link").4, "LINK");
    assert_eq!(row(&text, "A_Link").5, "|Cadena,FIFO,A_Terminate");
}

/// The difference 6: the report has the sections the original has.
///
/// The original:
/// ```
/// LOGICSWITCH               VALUE      RETRY   (bench GateL)
///   1                          0         62
/// XACT_GROUP           GROUP_SIZE      RETRY   (bench RemoveMe)
///   UNGRUPO                    0          0
/// NUMERIC_GROUP        GROUP_SIZE      RETRY   (bench JoinNumber)
///   UNGRUPO                    1          0
/// SAVEVALUE                 VALUE      RETRY   (bench Counter)
///   COUNT                     62          0
/// FEC_XACT_NUMBER    PRI      BDT    CURRENT   NEXT   PARAMETER   VALUE
/// ```
#[test]
fn the_report_has_the_sections_of_the_original() {
    if !benches_are_there() {
        return;
    }
    // The logic switches, with the attempts that failed their test.
    let text = report("GateL");
    let switches = section(&text, "LOGICSWITCH");
    assert_eq!(switches[0], "LOGICSWITCH               VALUE      RETRY");
    assert!(
        switches[1].starts_with("  1 "),
        "the switch of the gate: {:?}",
        switches[1]
    );

    // The groups of transactions and the groups of numbers.
    let text = report("RemoveMe");
    let groups = section(&text, "XACT_GROUP");
    assert_eq!(groups[0], "XACT_GROUP           GROUP_SIZE      RETRY");
    assert_eq!(groups[1], "  UNGRUPO                    0          0");
    let text = report("JoinNumber");
    let groups = section(&text, "NUMERIC_GROUP");
    assert_eq!(groups[0], "NUMERIC_GROUP        GROUP_SIZE      RETRY");
    assert_eq!(groups[1], "  UNGRUPO                    1          0");

    // The savevalues, with their retry and with their name in upper case.
    let text = report("Counter");
    let values = section(&text, "SAVEVALUE");
    assert_eq!(values[0], "SAVEVALUE                 VALUE      RETRY");
    assert!(values[1].starts_with("  COUNT "), "{:?}", values[1]);
    assert!(values[1].ends_with(" 0"), "{:?}", values[1]);

    // The events that are still pending when the run ends.
    let text = report("GateF");
    let pending = section(&text, "FEC_XACT_NUMBER");
    assert_eq!(
        pending[0],
        "FEC_XACT_NUMBER    PRI      BDT    CURRENT   NEXT   PARAMETER   VALUE"
    );
    assert_eq!(pending.len(), 3, "the pending events: {pending:?}");
    assert!(
        pending[1].split_whitespace().next().is_some_and(|number| {
            number.parse::<u64>().is_ok()
        }),
        "the row carries the number of the transaction: {:?}",
        pending[1]
    );
}

/// The difference 7: the row of a gate that does not open holds nothing.
///
/// The original (`scripts/wine-oracle/benches/GateL.gmdl` and its report):
/// ```
///     2        0       0     0          GateL  GATE    LS      1
/// LOGICSWITCH               VALUE      RETRY
///   1                          0         62
/// ```
/// The attempts that fail the test of the gate are counted on the switch the
/// gate tests and not on the block.
#[test]
fn a_gate_that_does_not_open_counts_nothing_and_the_switch_counts_the_attempts() {
    if !benches_are_there() {
        return;
    }
    let text = report("GateL");
    let gate = row(&text, "GateL");
    assert_eq!(gate.4, "GATE");
    assert_eq!((gate.1.as_str(), gate.2.as_str(), gate.3.as_str()), ("0", "0", "0"));
    let switches = section(&text, "LOGICSWITCH");
    let retry: u64 = switches[1]
        .split_whitespace()
        .last()
        .and_then(|value| value.parse().ok())
        .expect("the switch counts the attempts");
    assert!(retry > 0, "the switch counts the attempts: {:?}", switches[1]);
}

/// The difference 8: the table keeps the last class for the values that do
/// not fit in any other.
///
/// The original (`scripts/wine-oracle/benches/Table.gmdl` and its report):
/// ```
/// Tabla           TABLE           P$Atrib,0,10,10
///     MEDIA              : 0,000
///     DESVIACION ESTANDAR: 0,000
///     CANTIDAD EN RETRY  : 0
///
///     INTERVALO         FRECUENCIA     %     %ACUMULADO     %REMANENTE
///     0   -  0                0      0,000       0,000        100,000
///     1   -  10               0      0,000       0,000        100,000
///     ...
///     71  -  80               0      0,000       0,000        100,000
///     >   a  80               0      0,000       0,000        100,000
///
///     CANTIDAD DE VALORES TABULADOS : 0
///     VALOR MAXIMO TABULADO         : 0,000
/// ```
#[test]
fn the_table_keeps_the_last_class_for_the_overflow() {
    if !benches_are_there() {
        return;
    }
    let text = report("Table");
    let lines: Vec<&str> = text.lines().collect();
    let header = lines
        .iter()
        .find(|line| line.contains("TABLE"))
        .expect("the header of the table");
    assert!(
        header.contains("P$Atrib,0,10,10"),
        "the header carries the attribute of the table: {header}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("CANTIDAD EN RETRY  : 0")),
        "the table says how many values did not fit:\n{text}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("CANTIDAD DE VALORES TABULADOS : 0")),
        "the table says how many values it tabulated:\n{text}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("VALOR MAXIMO TABULADO         : 0,000")),
        "the table says the largest value it tabulated:\n{text}"
    );
    // The last class is the one of the values that do not fit: the original
    // writes them apart, as the values above the last limit.
    assert!(
        lines.iter().any(|line| line.contains(">   a  80")),
        "the table has the row of the values that do not fit:\n{text}"
    );
    assert!(
        !lines.iter().any(|line| line.contains("81  -  90")),
        "the table has no class for the values that do not fit:\n{text}"
    );
    assert!(
        lines.iter().any(|line| line.contains("71  -  80")),
        "the table has the last class that fits:\n{text}"
    );
}

/// The difference 9: the fine shape of the report.
///
/// The original writes three decimals and a comma in the averages
/// (`0,000`), the ones it cannot compute with a dot (`0.000`), the queues in
/// the order of their numbers (`1, 2, …, 10`) and it does not write the
/// seed of the run.
#[test]
fn the_fine_shape_of_the_report() {
    if !benches_are_there() {
        return;
    }
    // The averages of a queue: the content has a comma and the two averages
    // of the waiting time have a dot when the original cannot compute them.
    let text = report("Select");
    let queues = section(&text, "QUEUE");
    assert!(
        queues[1].contains("0,000") && queues[1].contains("0.000"),
        "the queue writes its averages: {:?}",
        queues[1]
    );
    // The queues come in the order of their numbers, not in the
    // alphabetical one.
    let names: Vec<&str> = queues[1..]
        .iter()
        .map(|line| line.split_whitespace().next().unwrap_or_default())
        .collect();
    assert_eq!(
        names,
        vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"],
        "the queues come in the order of their numbers: {names:?}"
    );
    // The original does not write the seed of the run.
    assert!(
        !text.contains("SEMILLA"),
        "the report does not write the seed:\n{text}"
    );
}

/// Whether the benches of the oracle are there.
///
/// They are generated artefacts (they need the original editor under wine,
/// and they are not tracked): a clone without them has no bench to check,
/// and the test skips itself instead of failing.
fn benches_are_there() -> bool {
    bench_path("GateF").exists()
}
