//! The rows and the sections of the report that still had to be written like
//! the original's.
//!
//! Every assertion is pinned to the original's own report: the benches under
//! `scripts/wine-oracle/benches/` are the models that exercise one class
//! each, and the original wrote its report for them in
//! `/tmp/bench-out-new/<Class>.report.txt` (the run of
//! `scripts/wine-oracle/run_oracle.sh`). The assertions check the whole text
//! of the row (the `BLOCK_TYPE` cell, the letter of the mode and the whole
//! operand list), never a fragment of it.

use std::path::{Path, PathBuf};

use genmsi_core::{SimConfig, Simulation};

mod support;

use support::{node, run, wired};

/// The column where the `LOC` field of a row of the block table starts.
///
/// The field is right-aligned in fifteen columns and ends at the column
/// forty-three; a name that does not fit in it eats the space that separates
/// it from the `RETRY` column, the way the original writes it (the row of
/// the bench `MultifurcacionP` reads `0MultifurcacionP`).
const LOC_COLUMN: usize = 28;

/// The path of one bench model.
fn bench(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/wine-oracle/benches")
        .join(format!("{name}.gmdl"))
}

/// Compile, run and render one bench model.
fn render_bench(name: &str) -> String {
    let path = bench(name);
    let model = genmsi_format::load(&path).expect("the bench loads");
    let program = genmsi_core::compile_model(&model).expect("the bench compiles");
    let result = Simulation::new(&program, SimConfig::default())
        .run()
        .expect("the bench runs");
    genmsi_core::render(&result, &program)
}

/// The text of the row of the block table whose `LOC` is `loc`: the
/// `BLOCK_TYPE` cell, the letter of the mode and the operand list.
///
/// The row of the original is `    2       57       0     0        A_Alter  ALTER    E       ungrupo,1,param,0,otro,1`:
/// the four counters come first and the rest of the row is what the
/// assertion fixes.
fn row(report: &str, loc: &str) -> String {
    for line in report.lines() {
        let Some(rest) = line.get(LOC_COLUMN..) else {
            continue;
        };
        let rest = rest.trim_start();
        let Some((row_loc, tail)) = rest.split_once("  ") else {
            continue;
        };
        if row_loc.trim() == loc {
            return tail.trim_end().to_string();
        }
    }
    panic!("no block row with LOC {loc}\n{report}");
}

/// The number of the line of the listing that the row of the block table
/// whose `LOC` is `loc` stands for.
fn line_of(report: &str, loc: &str) -> usize {
    for line in report.lines() {
        let Some(rest) = line.get(LOC_COLUMN..) else {
            continue;
        };
        let rest = rest.trim_start();
        let Some((row_loc, _)) = rest.split_once("  ") else {
            continue;
        };
        if row_loc.trim() == loc {
            return line
                .split_whitespace()
                .next()
                .expect("the line has a number")
                .parse()
                .expect("the number of the line is a number");
        }
    }
    panic!("no block row with LOC {loc}\n{report}");
}

/// The lines of the report that start with `prefix`.
fn lines_with(report: &str, prefix: &str) -> Vec<String> {
    report
        .lines()
        .filter(|line| line.starts_with(prefix))
        .map(|line| line.to_string())
        .collect()
}

#[test]
fn the_alter_row_writes_the_operator_the_group_the_count_and_the_two_parameters() {
    if !benches_are_there() {
        return;
    }
    // Original (`/tmp/bench-out-new/Alter.report.txt`):
    // `    2       57       0     0        A_Alter  ALTER    E       ungrupo,1,param,0,otro,1`
    // The operator of the comparison, the group, the number of transactions
    // altered, the parameter altered and its value, and the parameter
    // compared and its value.
    let report = render_bench("Alter");
    assert_eq!(
        row(&report, "A_Alter"),
        "ALTER    E       ungrupo,1,param,0,otro,1"
    );
}

#[test]
fn the_alter_row_writes_the_destination_of_its_failure_output() {
    if !benches_are_there() {
        return;
    }
    // Original (`scripts/wine-oracle/probes/oracle/AlterMiembro.report.txt`):
    // `    4       62       0     0        A_Alter  ALTER    E       UnGrupo,1,Param,99,Param,0,A_Terminate1`
    // The destination of the failure output ("No alteró") is the last
    // operand of the statement, and only when that output is connected.
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/wine-oracle/probes/AlterMiembro.gmdl");
    let model = genmsi_format::load(&path).expect("the probe loads");
    let program = genmsi_core::compile_model(&model).expect("the probe compiles");
    let result = Simulation::new(&program, SimConfig::default())
        .run()
        .expect("the probe runs");
    let report = genmsi_core::render(&result, &program);
    assert_eq!(
        row(&report, "A_Alter"),
        "ALTER    E       UnGrupo,1,Param,99,Param,0,A_Terminate1"
    );
}

/// A model with one `LOGIC` block that carries the given operator.
fn logic_model(operator: &'static str) -> genmsi_core::Program {
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A_Logic"))],
            &[("Media", "1"), ("Modificador", "0")],
        ),
        node(
            2,
            "Logic",
            Some("A_Logic"),
            &[("Salida1", Some("A_Terminate"))],
            &[("LogicSwich", "1"), ("Op", operator)],
        ),
        node(3, "Terminate", Some("A_Terminate"), &[], &[("Unidades", "0")]),
    ];
    let links = [(1, 0, 2, 0), (2, 0, 3, 0)];
    wired(&nodes, &links, 100.0)
}

#[test]
fn the_logic_row_writes_the_third_operator_of_the_switch() {
    if !benches_are_there() {
        return;
    }
    // The original's list of operators of the switch has three: `S` (set),
    // `R` (reset) and `I` (invertir), and the report writes the letter of
    // the one the model asks for (`LOGIC   I       1`).
    let program = logic_model("Invertir");
    let result = run(&program, 100.0, 1);
    let report = genmsi_core::render(&result, &program);
    assert_eq!(row(&report, "A_Logic"), "LOGIC   I       1");
    let program = logic_model("Set");
    let result = run(&program, 100.0, 1);
    let report = genmsi_core::render(&result, &program);
    assert_eq!(row(&report, "A_Logic"), "LOGIC   S       1");
    let program = logic_model("Reset");
    let result = run(&program, 100.0, 1);
    let report = genmsi_core::render(&result, &program);
    assert_eq!(row(&report, "A_Logic"), "LOGIC   R       1");
}

#[test]
fn the_enter_and_the_leave_rows_write_the_units_of_the_storage() {
    if !benches_are_there() {
        return;
    }
    // Originals (`/tmp/bench-out-new/Enter.report.txt` and `Leave.report.txt`):
    // `    3       10       0     0        A_Enter  ENTER           AStorage,1`
    // `    4       63       0     0        A_Leave  LEAVE           AStorage,1`
    let report = render_bench("Enter");
    assert_eq!(row(&report, "A_Enter"), "ENTER           AStorage,1");
    let report = render_bench("Leave");
    assert_eq!(row(&report, "A_Enter"), "ENTER           AStorage,1");
    assert_eq!(row(&report, "A_Leave"), "LEAVE           AStorage,1");
}

#[test]
fn the_transfer_row_writes_the_probability_in_thousandths() {
    if !benches_are_there() {
        return;
    }
    // Original (`/tmp/bench-out-new/Transfer.report.txt`):
    // `    2       60       0     0       Entrada1  TRANSFER        1000,,A_Terminate`
    // The model gives `Prob = 100%` and the original writes it in thousandths.
    let report = render_bench("Transfer");
    assert_eq!(
        row(&report, "Entrada1"),
        "TRANSFER        1000,,A_Terminate"
    );
}

#[test]
fn the_multifurcation_rows_write_the_function_the_class_creates() {
    if !benches_are_there() {
        return;
    }
    // Original (`/tmp/bench-out-new/Multifurcacion.report.txt`):
    // `    6       60       0     0 Multifurcacion  TRANSFER        ,FN$MultifurcacionGO`
    // The original's macro code writes a `MULTITRANSFER` to the function
    // that maps the branch to its destination, and the report writes that
    // statement as a `TRANSFER` to the function.
    let report = render_bench("Multifurcacion");
    assert_eq!(
        row(&report, "Multifurcacion"),
        "TRANSFER        ,FN$MultifurcacionGO"
    );
    // The original wrote no report for the `MultifurcacionP` bench (the file
    // is empty), so this row follows the macro code of the class: the same
    // `TRANSFER` to the same function.
    let report = render_bench("MultifurcacionP");
    assert_eq!(
        row(&report, "MultifurcacionP"),
        "TRANSFER        ,FN$MultifurcacionGO"
    );
}

#[test]
fn the_loop_row_writes_the_empty_destination_of_the_iteration() {
    if !benches_are_there() {
        return;
    }
    // Original (`/tmp/bench-out-new/Loop.report.txt`):
    // `    2       59       0     0         A_Loop  LOOP            Param,`
    // The `LOOP` always has two operands: the parameter and the destination
    // of the iteration, which is empty when the model does not connect it.
    let report = render_bench("Loop");
    assert_eq!(row(&report, "A_Loop"), "LOOP            Param,");
}

#[test]
fn the_gate_of_a_label_writes_the_matching_letter() {
    if !benches_are_there() {
        return;
    }
    // Original (`/tmp/bench-out-new/GateM.report.txt`):
    // `    4        0       0     0          GateM  GATE    M       AAA`
    // The `GateM` class looks at a label: the letter is `M` (the
    // transaction has to match it) and not the `LS` of a logic switch.
    let report = render_bench("GateM");
    assert_eq!(row(&report, "GateM"), "GATE    M       AAA");
}

#[test]
fn the_qtables_section_writes_the_histogram_of_the_queue_table() {
    if !benches_are_there() {
        return;
    }
    // Original (`/tmp/bench-out-new/QTable.report.txt`):
    // ```
    // QTABLES
    // QTabla          QTABLE          qRecurso,0,600,60
    //     MEDIA              : 0,000
    //     DESVIACION ESTANDAR: 0,000
    //     CANTIDAD EN RETRY  : 0
    //
    //     INTERVALO         FRECUENCIA     %     %ACUMULADO     %REMANENTE
    //     0   -  0                0      0,000       0,000        100,000
    //     ...
    //     >   a  34800              0      0,000       0,000        100,000
    //
    //     CANTIDAD DE VALORES TABULADOS : 0
    //     VALOR MAXIMO TABULADO         : 0,000
    // ```
    // The header names the queue and the operands of the `QTABLE` statement,
    // and the histogram has the sixty classes of the table and the class of
    // the values that do not fit.
    let report = render_bench("QTable");
    assert!(report.contains("\nQTABLES\n"), "{report}");
    assert!(
        report.contains("QTabla          QTABLE          qRecurso,0,600,60"),
        "{report}"
    );
    assert!(
        report.contains("    CANTIDAD EN RETRY  : 0\n"),
        "{report}"
    );
    let histogram: Vec<&str> = report
        .lines()
        .filter(|line| line.starts_with("    0   -  0") || line.starts_with("    >   a  34800"))
        .collect();
    assert_eq!(histogram.len(), 2, "{report}");
    assert!(
        report.contains("    0   -  0                0      0,000       0,000        100,000"),
        "{report}"
    );
    assert!(
        report.contains("    >   a  34800              0      0,000       0,000        100,000"),
        "{report}"
    );
    // The sixty classes of the table and the class of the desborde.
    let classes = report
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with('>')
                || (trimmed.starts_with(|c: char| c.is_ascii_digit())
                    && trimmed.contains(" -  "))
        })
        .count();
    assert_eq!(classes, 60, "{report}");
    assert!(
        report.contains("    CANTIDAD DE VALORES TABULADOS : 0\n"),
        "{report}"
    );
    assert!(
        report.contains("    VALOR MAXIMO TABULADO         : 0,000\n"),
        "{report}"
    );
}

#[test]
fn the_group_sections_write_the_groups_that_the_statements_name() {
    if !benches_are_there() {
        return;
    }
    // Originals (`/tmp/bench-out-new/Examine.report.txt`,
    // `RemoveTrans.report.txt` and `ExamineNumber.report.txt`):
    // ```
    // XACT_GROUP           GROUP_SIZE      RETRY
    //   UNGRUPO                    0          0
    // ```
    // and the same for `NUMERIC_GROUP` with the `ExamineNumber` bench. The
    // original writes the group even when nobody belongs to it: the
    // statement names it.
    let report = render_bench("Examine");
    assert!(
        report.contains("XACT_GROUP           GROUP_SIZE      RETRY\n"),
        "{report}"
    );
    assert!(
        report.contains("  UNGRUPO                    0          0\n"),
        "{report}"
    );
    let report = render_bench("RemoveTrans");
    assert!(
        report.contains("XACT_GROUP           GROUP_SIZE      RETRY\n"),
        "{report}"
    );
    assert!(
        report.contains("  UNGRUPO                    0          0\n"),
        "{report}"
    );
    let report = render_bench("ExamineNumber");
    assert!(
        report.contains("NUMERIC_GROUP        GROUP_SIZE      RETRY\n"),
        "{report}"
    );
    assert!(
        report.contains("  UNGRUPO                    0          0\n"),
        "{report}"
    );
}

#[test]
fn the_test_row_writes_the_parameter_with_the_dollar_of_gpss() {
    if !benches_are_there() {
        return;
    }
    // Original (`fixtures/outputs/Problema 1__semaforo-1.report.txt`):
    // `   81        0      0     0         A_Test  TEST    E       P$Cuadra,50,A_Test1`
    // The model writes `P:Cuadra` and the original writes it in the
    // notation of GPSS, with a dollar.
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/Problema 1__semaforo-1.gmdl");
    let model = genmsi_format::load(&path).expect("the model loads");
    let program = genmsi_core::compile_model(&model).expect("the model compiles");
    let result = Simulation::new(&program, SimConfig::default())
        .run()
        .expect("the model runs");
    let report = genmsi_core::render(&result, &program);
    assert_eq!(
        row(&report, "A_Test"),
        "TEST    E       P$Cuadra,50,A_Test1"
    );
}

#[test]
fn the_line_column_is_the_line_of_the_program() {
    if !benches_are_there() {
        return;
    }
    // Original (`/tmp/bench-out-new/Enter.report.txt`):
    // ```
    //     2       62      52     0              1  GENERATE        60,30
    //     3       10       0     0        A_Enter  ENTER           AStorage,1
    // ```
    // The `STORAGE` of the model is the first line of the program, so the
    // generator is the second one: the `LINE` column counts the lines of the
    // program and not the statements of the model.
    let report = render_bench("Enter");
    assert_eq!(line_of(&report, "1"), 2);
    assert_eq!(line_of(&report, "A_Enter"), 3);
    assert_eq!(line_of(&report, "A_Terminate"), 4);
}

#[test]
fn the_pending_events_write_their_time_as_a_whole_number() {
    if !benches_are_there() {
        return;
    }
    // Original (`/tmp/bench-out-new/Alter.report.txt`):
    // ```
    // FEC_XACT_NUMBER    PRI      BDT    CURRENT   NEXT   PARAMETER   VALUE
    //             59      0       3624        1       1
    // ```
    // The original's events happen at whole times and the `BDT` column
    // writes them without decimals.
    let report = render_bench("Alter");
    let pending = lines_with(&report, "FEC_XACT_NUMBER");
    assert_eq!(pending.len(), 1, "{report}");
    let rows: Vec<&str> = report
        .lines()
        .skip_while(|line| !line.starts_with("FEC_XACT_NUMBER"))
        .skip(1)
        .collect();
    assert!(!rows.is_empty(), "{report}");
    for row in rows {
        let cells: Vec<&str> = row.split_whitespace().collect();
        assert_eq!(cells.len(), 5, "{row}");
        // The third cell is the time of the event.
        assert!(!cells[2].contains(','), "{row}");
        assert!(
            cells[2].parse::<u64>().is_ok(),
            "the time of the event is a whole number: {row}"
        );
    }
}

/// Whether the benches of the oracle are there.
///
/// They are generated artefacts (they need the original editor under wine,
/// and they are not tracked): a clone without them has no bench to check,
/// and the test skips itself instead of failing.
fn benches_are_there() -> bool {
    bench("GateF").exists()
}
