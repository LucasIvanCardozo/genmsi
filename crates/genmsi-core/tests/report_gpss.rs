//! The report's block table must speak the original's GPSS dialect.
//!
//! Every assertion is pinned to a row of the single-sample baselines under
//! `fixtures/outputs/*.report.txt`: block classes keep their GPSS statement
//! names (`GATE`, not `GATEL`), statements with a mode (`GATE`, `TEST`,
//! `LOGIC`, `SELECT`, `REMOVE`, `UNLINK`, `SCAN`) give the letter its own
//! column before the comma list, and branching blocks carry the LOC of their
//! linked outputs.

use std::path::{Path, PathBuf};

use genmsi_core::{SimConfig, Simulation};

/// Block-table rows start at column 29 (four numeric fields); the `LOC` field
/// runs 14 columns from there and the `BLOCK_TYPE` cell follows. Labels longer
/// than the field overflow it (baseline `A_ExamineNumber`), so rows are parsed
/// from the token stream instead of fixed offsets.
const BLOCK_TABLE: usize = 29;

/// GPSS mode letters the original prints in their own column between the
/// block type and the operand list.
const MODES: &[&str] = &[
    "LS", "LR", "SE", "SNE", "NU", "U", "MIN", "MAX", "LE", "GE", "NE", "E", "L", "G", "S", "R",
];

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

/// The parsed tail of one block-table row: `(LOC, BLOCK_TYPE, mode, operands)`.
/// The mode is empty for statements without a mode column.
fn fields(line: &str) -> Option<(&str, &str, &str, &str)> {
    if !line.trim_start().starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    // Column 29 starts the `LOC` field; the four numeric columns are behind it.
    let rest = line.get(BLOCK_TABLE..)?.trim_start();
    let (loc, tail) = rest.split_once("  ")?;
    let (block_type, tail) = tail.split_once(char::is_whitespace)?;
    let tail = tail.trim_start();
    // The mode letter (when the statement has one) sits in its own column,
    // so at least two spaces separate type, mode and the operand list.
    let (mode, operands) = match tail.split_once(char::is_whitespace) {
        Some((token, list)) if MODES.contains(&token) => (token, list),
        _ => ("", tail),
    };
    Some((loc.trim(), block_type, mode, operands.trim()))
}

/// The block-table row whose `LOC` field is exactly `loc`, as
/// `(BLOCK_TYPE, mode, operands)`.
fn block_row(report: &str, loc: &str) -> (String, String, String) {
    report
        .lines()
        .filter_map(fields)
        .find(|(row_loc, ..)| *row_loc == loc)
        .map(|(_, block_type, mode, operands)| {
            (block_type.to_string(), mode.to_string(), operands.to_string())
        })
        .unwrap_or_else(|| panic!("no block row with LOC {loc}\n{report}"))
}

#[test]
fn gate_logic_switch_rows_match_the_baseline() {
    // Baseline: `GateL  GATE    LS      Puerta` (TP6__Clase 6_1_PuertaBanco).
    let report = render_model("TP6__Clase 6_1_PuertaBanco");
    assert_eq!(block_row(&report, "GateL"), ("GATE".into(), "LS".into(), "Puerta".into()));
}

#[test]
fn gate_storage_not_empty_row_matches_the_baseline() {
    // Baseline: `HornoConComidas  GATE    SNE     Comidas` (entregaComida).
    let report = render_model("entregaComida__entregaComida");
    let line = report
        .lines()
        .filter_map(fields)
        .find(|(loc, _, mode, _)| *loc == "A_GateS" && *mode == "SNE")
        .expect("the gate requiring a non-empty storage");
    assert_eq!(line.1, "GATE");
    assert_eq!(line.3, "Comidas");
}

#[test]
fn gate_facility_free_row_keeps_the_else_target() {
    // Baseline: `GateF  GATE    NU      Juan,A_Advance`
    // (TP6__Clase 6_3_JefeEducado).
    let report = render_model("TP6__Clase 6_3_JefeEducado");
    assert_eq!(
        block_row(&report, "GateF"),
        ("GATE".into(), "NU".into(), "Juan,A_Advance".into())
    );
}

#[test]
fn gate_else_target_is_reported_when_the_failure_output_is_linked() {
    // Baseline: `GateL1  GATE    LS      PuertaMuseo,A_FacilityUse3`
    // (Parcial 6-11-23__Veron - Parcial).
    let report = render_model("Parcial 6-11-23__Veron - Parcial");
    assert_eq!(
        block_row(&report, "GateL1"),
        ("GATE".into(), "LS".into(), "PuertaMuseo,A_FacilityUse3".into())
    );
}

#[test]
fn select_first_free_row_matches_the_baseline() {
    // Baseline: `A_SelectF  SELECT   NU      Caja,1,10,,,A_Select`
    // (Parcial 6-11-23__Veron - Parcial).
    let report = render_model("Parcial 6-11-23__Veron - Parcial");
    assert_eq!(
        block_row(&report, "A_SelectF"),
        ("SELECT".into(), "NU".into(), "Caja,1,10,,,A_Select".into())
    );
}

#[test]
fn test_and_logic_rows_use_the_gpss_letters() {
    // Baselines (Problema 1__semaforo-1): `A_Test  TEST    E       P$Cuadra,50,A_Test1`,
    // `A_Logic  LOGIC   S       *semaforo`; indirect entities keep the `*`
    // prefix (`GateL  GATE    LS      *Cuadra`).
    let report = render_model("Problema 1__semaforo-1");
    assert_eq!(
        block_row(&report, "A_Test"),
        ("TEST".into(), "E".into(), "P:Cuadra,50,A_Test1".into())
    );
    assert_eq!(
        block_row(&report, "A_Logic"),
        ("LOGIC".into(), "S".into(), "*semaforo".into())
    );
    assert_eq!(
        block_row(&report, "GateL"),
        ("GATE".into(), "LS".into(), "*Cuadra".into())
    );
    assert_eq!(
        block_row(&report, "GateL1"),
        ("GATE".into(), "LR".into(), "*Cuadra".into())
    );
}

#[test]
fn counter_select_join_examine_and_remove_rows_match_the_baseline() {
    // Baselines (TP10__tp10): `GateL  GATE    LR      Fin,A_Counter1`,
    // `A_Counter1  SAVEVALUE       CountEspera+,1`,
    // `A_Test  TEST    GE      P$CantPescados,1,A_Terminate7`,
    // `A_JoinNumber  JOIN            GPesos,P$PesoPescado`,
    // `A_ExamineNumber  EXAMINE         GPesos,P$PesoPescado,A_Assign7`,
    // `A_RemoveTrans  REMOVE   E       GPescando,1,,IdPescador,P$NroPescador`.
    let report = render_model("TP10__tp10");
    assert_eq!(
        block_row(&report, "GateL"),
        ("GATE".into(), "LR".into(), "Fin,A_Counter1".into())
    );
    assert_eq!(
        block_row(&report, "A_Counter1"),
        ("SAVEVALUE".into(), "".into(), "CountEspera+,1".into())
    );
    assert_eq!(
        block_row(&report, "A_Test"),
        ("TEST".into(), "GE".into(), "P:CantPescados,1,A_Terminate7".into())
    );
    assert_eq!(
        block_row(&report, "A_JoinNumber"),
        ("JOIN".into(), "".into(), "GPesos,P:PesoPescado".into())
    );
    assert_eq!(
        block_row(&report, "A_ExamineNumber"),
        ("EXAMINE".into(), "".into(), "GPesos,P:PesoPescado,A_Assign7".into())
    );
    assert_eq!(
        block_row(&report, "A_RemoveTrans"),
        ("REMOVE".into(), "E".into(), "GPescando,1,,IdPescador,P:NroPescador".into())
    );
}

#[test]
fn transfer_rows_render_the_probability_in_thousandths_and_both_targets() {
    // Baseline (ejercicio_4): `Entrada1  TRANSFER        550,A_FacilityUse11,A_Advance`
    // (model `Prob = 55%`); the probability branch is the `Salida2` target and
    // the main branch the `Salida1` target. monorriel renders `600` from
    // `Prob = 60%` (`Entrada1  TRANSFER        600,A_Terminate,A_Advance3`).
    let report = render_model("ejercicio_4__ejercicio_4");
    assert_eq!(
        block_row(&report, "Entrada1"),
        ("TRANSFER".into(), "".into(), "550,A_FacilityUse11,A_Advance".into())
    );
    let report = render_model("monorriel__monorriel");
    assert_eq!(
        block_row(&report, "Entrada1"),
        ("TRANSFER".into(), "".into(), "600,A_Terminate,A_Advance3".into())
    );
}

#[test]
fn transfer_rows_keep_the_raw_expression_and_the_unconditional_form() {
    // Baselines (TP3__aeropuerto): `Entrada11  TRANSFER        X$qValue,Entrada13,A_Queue2`
    // keeps the SNA probability (`X:qValue` in our notation), and
    // `Entrada1  TRANSFER        150,,A_Advance2` (ejercicio_3, model
    // `Prob = 15%` with the `Salida2` output unlinked) keeps the empty
    // middle operand.
    let report = render_model("TP3__aeropuerto");
    assert_eq!(
        block_row(&report, "Entrada11"),
        ("TRANSFER".into(), "".into(), "X:qValue,Entrada13,A_Queue2".into())
    );
    let report = render_model("ejercicio_3__ejercicio_3");
    assert_eq!(
        block_row(&report, "Entrada1"),
        ("TRANSFER".into(), "".into(), "150,,A_Advance2".into())
    );
}

#[test]
fn assign_rows_suffix_the_combining_operator() {
    // Baselines (Parcial 6-11-23__Veron - Parcial):
    // `A_Assign6  ASSIGN          SectorVisitado+,1` (model `Operador = +`)
    // and `A_Assign7  ASSIGN          Dino,1` (no operator).
    let report = render_model("Parcial 6-11-23__Veron - Parcial");
    assert_eq!(
        block_row(&report, "A_Assign6"),
        ("ASSIGN".into(), "".into(), "SectorVisitado+,1".into())
    );
    assert_eq!(
        block_row(&report, "A_Assign7"),
        ("ASSIGN".into(), "".into(), "Dino,1".into())
    );
}

#[test]
fn unlink_rows_render_the_engine_branches_in_baseline_order() {
    // Baselines (TP10__tp10): `A_Unlink1  UNLINK   E       CPescando,A_Advance2,1,NroEspera,P$NroEspera,A_Terminate2`
    // and `A_Unlink2  UNLINK   E       CPescando,A_Test,ALL,Otro,0`. The
    // second operand is the `SalidaAlt` target — the port `sim.rs` routes the
    // removed transactions through — and the last one the `Salida2` target
    // taken when nothing matched (model module 25 links both).
    let report = render_model("TP10__tp10");
    assert_eq!(
        block_row(&report, "A_Unlink1"),
        (
            "UNLINK".into(),
            "E".into(),
            "CPescando,A_Advance2,1,NroEspera,P:NroEspera,A_Terminate2".into()
        )
    );
    assert_eq!(
        block_row(&report, "A_Unlink2"),
        ("UNLINK".into(), "E".into(), "CPescando,A_Test,ALL,Otro,0".into())
    );
}

#[test]
fn link_rows_render_the_fifo_mode() {
    // Baseline (TP10__tp10): `A_Link  LINK            CPescando,FIFO`.
    let report = render_model("TP10__tp10");
    assert_eq!(
        block_row(&report, "A_Link"),
        ("LINK".into(), "".into(), "CPescando,FIFO".into())
    );
}

#[test]
fn split_rows_render_the_copy_target() {
    // Baselines (Parcial 6-11-23__Veron - Parcial, TP10__tp10):
    // `Clona  SPLIT           P$CantTuristas,A_Queue1` and
    // `Clona  SPLIT           1,A_Advance11`; the copies take `Salida2`.
    let report = render_model("Parcial 6-11-23__Veron - Parcial");
    assert_eq!(
        block_row(&report, "Clona"),
        ("SPLIT".into(), "".into(), "P:CantTuristas,A_Queue1".into())
    );
    let report = render_model("TP10__tp10");
    assert_eq!(
        block_row(&report, "Clona"),
        ("SPLIT".into(), "".into(), "1,A_Advance11".into())
    );
}

#[test]
fn loop_rows_render_the_back_target() {
    // Baseline (Parcial 6-11-23__Veron - Parcial):
    // `A_Loop1  LOOP            Vitrinas,A_Advance7`; the back edge is the
    // `Salida2` target.
    let report = render_model("Parcial 6-11-23__Veron - Parcial");
    assert_eq!(
        block_row(&report, "A_Loop1"),
        ("LOOP".into(), "".into(), "Vitrinas,A_Advance7".into())
    );
}

#[test]
fn tabulate_rows_render_the_weight_when_the_model_sets_it() {
    // Baselines (monorriel, Parcial 6-11-23__Veron - Parcial):
    // `A_Tabulate2  TABULATE        TablaParada,1` (model `Peso = 1`) and
    // `A_Tabulate  TABULATE        DinoFanaticos` (`Peso` empty, no operand).
    let report = render_model("monorriel__monorriel");
    assert_eq!(
        block_row(&report, "A_Tabulate2"),
        ("TABULATE".into(), "".into(), "TablaParada,1".into())
    );
    let report = render_model("Parcial 6-11-23__Veron - Parcial");
    assert_eq!(
        block_row(&report, "A_Tabulate"),
        ("TABULATE".into(), "".into(), "DinoFanaticos".into())
    );
}

#[test]
fn savevalue_rows_suffix_the_combining_operator() {
    // Baseline (Parcial 6-11-23__Veron - Parcial):
    // `A_SaveValue  SAVEVALUE       Fichas+,1` (model `Op = +`).
    let report = render_model("Parcial 6-11-23__Veron - Parcial");
    assert_eq!(
        block_row(&report, "A_SaveValue"),
        ("SAVEVALUE".into(), "".into(), "Fichas+,1".into())
    );
}

#[test]
fn scan_rows_use_the_comp_op_letter() {
    // Baselines: `A_Scan  SCAN    E       GPescando,IdPescador,P$NroPescador,IdPescador,Idescador,A_Advance1`
    // (TP10__tp10, model `CompOp = "="` — note the original prints
    // `Idescador` where the model stores `EnParam = IdPescador`), and
    // `A_Scan  SCAN    MIN     EsperaServiciosVarios,IDAutomovil,,IDAutomovil,IDAutomovil,A_Advance2`
    // (TP8__mecanicos, model `CompOp = Minimo`).
    let report = render_model("TP10__tp10");
    assert_eq!(
        block_row(&report, "A_Scan"),
        (
            "SCAN".into(),
            "E".into(),
            "GPescando,IdPescador,P:NroPescador,IdPescador,IdPescador,A_Advance1".into()
        )
    );
    let report = render_model("TP8__mecanicos");
    assert_eq!(
        block_row(&report, "A_Scan"),
        (
            "SCAN".into(),
            "MIN".into(),
            "EsperaServiciosVarios,IDAutomovil,,IDAutomovil,IDAutomovil,A_Advance2".into()
        )
    );
}

#[test]
fn assign_rows_print_a_percentage_literal_as_its_per_mille_number() {
    // Baseline: `A_Assign43  ASSIGN  pValue,700` (TP3__aeropuerto) where the
    // model stores `70%`; the `percent` rule in `expr/grammar.pest` defines the
    // literal as 700, so the report prints the number, not the source text.
    let report = render_model("TP3__aeropuerto");
    assert_eq!(
        block_row(&report, "A_Assign43"),
        ("ASSIGN".into(), "".into(), "pValue,700".into())
    );
    assert_eq!(
        block_row(&report, "A_Assign431"),
        ("ASSIGN".into(), "".into(), "pValue,500".into())
    );
}

#[test]
fn assign_rows_print_a_bare_parameter_operand_with_the_indirection_star() {
    // Baseline: `A_Assign6  ASSIGN  cantPAux,*cantP` (TP3__aeropuerto) where the
    // model writes the bare name `cantP`: a bare operand that resolves to a
    // parameter is the original's `*name`. A prefixed form keeps its text.
    let report = render_model("TP3__aeropuerto");
    assert_eq!(
        block_row(&report, "A_Assign6"),
        ("ASSIGN".into(), "".into(), "cantPAux,*cantP".into())
    );
    // Baseline: `A_Assign  ASSIGN  TipoCliente,FN$ProbCliente`
    // (TP4__supermercado). A prefixed operand keeps the text the model wrote.
    let report = render_model("TP4__supermercado");
    assert_eq!(
        block_row(&report, "A_Assign"),
        ("ASSIGN".into(), "".into(), "TipoCliente,FN:ProbCliente".into())
    );
}
