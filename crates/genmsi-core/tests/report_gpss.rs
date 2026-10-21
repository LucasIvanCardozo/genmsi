//! The report's block table must speak the original's GPSS dialect.
//!
//! Every assertion is pinned to a row of the single-sample baselines under
//! `fixtures/outputs/*.report.txt`: block classes keep their GPSS statement
//! names (`GATE`, not `GATEL`), mode letters replace the internal enum names,
//! and conditional blocks carry the LOC of their linked failure output.

use std::path::{Path, PathBuf};

use genmsi_core::{SimConfig, Simulation};

/// Line layout of the block table (`render`): 4 numeric fields, the right
/// aligned 14-wide `LOC` field, two spaces, the 14-wide `BLOCK_TYPE` field,
/// two spaces, then the operands. Labels longer than the field overflow it
/// (baseline `A_ExamineNumber`), so the row is split on the column gaps.
const BLOCK_TABLE: std::ops::RangeFrom<usize> = 29..;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Compile, run and render one reference model.
fn render_model(name: &str) -> String {
    let path = root().join(format!("fixtures/models/{name}.gmdl"));
    let model = genmsi_format::load(&path).expect("model loads");
    let program = genmsi_core::compile_model(&model).expect("model compiles");
    let result = Simulation::new(&program, SimConfig::default())
        .run()
        .expect("model runs");
    genmsi_core::render(&result, &program)
}

/// The `LOC`, `BLOCK_TYPE` and operand fields of one block-table row.
fn fields(line: &str) -> Option<(&str, &str, &str)> {
    let (loc, rest) = line.get(BLOCK_TABLE)?.trim_start().split_once("  ")?;
    let (block_type, operands) = rest.split_once("  ")?;
    Some((loc, block_type.trim(), operands.trim()))
}

/// The block-table row whose `LOC` field is exactly `loc`, as
/// `(BLOCK_TYPE, operands)`.
fn block_row(report: &str, loc: &str) -> (String, String) {
    let (_, block_type, operands) = report
        .lines()
        .find_map(|line| {
            let (row_loc, block_type, operands) = fields(line)?;
            (row_loc == loc).then_some((row_loc, block_type, operands))
        })
        .unwrap_or_else(|| panic!("no block row with LOC {loc}\n{report}"));
    (block_type.to_string(), operands.to_string())
}

#[test]
fn gate_logic_switch_rows_match_the_baseline() {
    // Baseline: `GateL  GATE  LS  Puerta` (TP6__Clase 6_1_PuertaBanco).
    let report = render_model("TP6__Clase 6_1_PuertaBanco");
    assert_eq!(block_row(&report, "GateL"), ("GATE".into(), "LS,Puerta".into()));
}

#[test]
fn gate_storage_not_empty_row_matches_the_baseline() {
    // Baseline: `HornoConComidas  GATE  SNE  Comidas` (entregaComida).
    let report = render_model("entregaComida__entregaComida");
    let line = report
        .lines()
        .filter_map(fields)
        .find(|(loc, _, operands)| *loc == "A_GateS" && operands.starts_with("SNE,"))
        .expect("the gate requiring a non-empty storage");
    assert_eq!(line.1, "GATE");
    assert_eq!(line.2, "SNE,Comidas");
}

#[test]
fn gate_facility_free_row_keeps_the_else_target() {
    // Baseline: `GateF  GATE  NU  Juan,A_Advance` (TP6__Clase 6_3_JefeEducado).
    let report = render_model("TP6__Clase 6_3_JefeEducado");
    assert_eq!(
        block_row(&report, "GateF"),
        ("GATE".into(), "NU,Juan,A_Advance".into())
    );
}

#[test]
fn gate_else_target_is_reported_when_the_failure_output_is_linked() {
    // Baseline: `GateL1  GATE  LS  PuertaMuseo,A_FacilityUse3`
    // (Parcial 6-11-23__Veron - Parcial).
    let report = render_model("Parcial 6-11-23__Veron - Parcial");
    assert_eq!(
        block_row(&report, "GateL1"),
        ("GATE".into(), "LS,PuertaMuseo,A_FacilityUse3".into())
    );
}

#[test]
fn select_first_free_row_matches_the_baseline() {
    // Baseline: `A_SelectF  SELECT  NU  Caja,1,10,,,A_Select`
    // (Parcial 6-11-23__Veron - Parcial).
    let report = render_model("Parcial 6-11-23__Veron - Parcial");
    assert_eq!(
        block_row(&report, "A_SelectF"),
        ("SELECT".into(), "NU,Caja,1,10,,,A_Select".into())
    );
}

#[test]
fn test_and_logic_rows_use_the_gpss_letters() {
    // Baseline: `A_Test  TEST  E  P$Cuadra,50,A_Test1` and
    // `A_Logic  LOGIC  S  *semaforo`; indirect entities keep the `*` prefix
    // (`GateL  GATE  LS  *Cuadra`, Problema 1__semaforo-1).
    let report = render_model("Problema 1__semaforo-1");
    assert_eq!(
        block_row(&report, "A_Test"),
        ("TEST".into(), "E,P:Cuadra,50,A_Test1".into())
    );
    assert_eq!(
        block_row(&report, "A_Logic"),
        ("LOGIC".into(), "S,*semaforo".into())
    );
    assert_eq!(block_row(&report, "GateL"), ("GATE".into(), "LS,*Cuadra".into()));
    assert_eq!(block_row(&report, "GateL1"), ("GATE".into(), "LR,*Cuadra".into()));
}

#[test]
fn counter_select_join_examine_and_remove_rows_match_the_baseline() {
    // Baselines (TP10__tp10): `GateL  GATE  LR  Fin,A_Counter1`,
    // `A_Counter1  SAVEVALUE  CountEspera+,1`,
    // `A_Test  TEST  GE  P$CantPescados,1,A_Terminate7`,
    // `A_JoinNumber  JOIN  GPesos,P$PesoPescado`,
    // `A_ExamineNumber  EXAMINE  GPesos,P$PesoPescado,A_Assign7`,
    // `A_RemoveTrans  REMOVE  E  GPescando,1,,IdPescador,P$NroPescador`.
    let report = render_model("TP10__tp10");
    assert_eq!(
        block_row(&report, "GateL"),
        ("GATE".into(), "LR,Fin,A_Counter1".into())
    );
    assert_eq!(
        block_row(&report, "A_Counter1"),
        ("SAVEVALUE".into(), "CountEspera+,1".into())
    );
    assert_eq!(
        block_row(&report, "A_Test"),
        ("TEST".into(), "GE,P:CantPescados,1,A_Terminate7".into())
    );
    assert_eq!(
        block_row(&report, "A_JoinNumber"),
        ("JOIN".into(), "GPesos,P:PesoPescado".into())
    );
    assert_eq!(
        block_row(&report, "A_ExamineNumber"),
        ("EXAMINE".into(), "GPesos,P:PesoPescado,A_Assign7".into())
    );
    assert_eq!(
        block_row(&report, "A_RemoveTrans"),
        ("REMOVE".into(), "E,GPescando,1,,IdPescador,P:NroPescador".into())
    );
}
