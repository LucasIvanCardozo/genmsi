//! Report, JSON and CSV rendering of a run.
//!
//! The text report follows the original's GPSS layout (decision D11: minimum
//! parity, better internals): header counts, block table with
//! `ENTRY/CURRENT/RETRY`, then facilities, queues, storages, user chains,
//! tables and savevalues. Numbers use a comma decimal separator like the
//! Spanish original.

use std::fmt::Write as _;

use serde::Serialize;
use serde_json::json;

use crate::error::EngineError;
use crate::expr::{Expr, NameSource, ProgramExpr, RefKind};
use crate::program::{
    AssignOp, BlockKind, ExecBlock, GateCondition, LogicOp, Program, RelOp, SelectMode,
    TransferSpec,
};
use crate::sim::{BlockStats, RunResult};

/// Format a number the way the original report does.
pub fn number(value: f64) -> String {
    if !value.is_finite() {
        return "0,000".to_string();
    }
    if value.fract().abs() < 1e-9 && value.abs() < 1e12 {
        return format!("{}", value as i64);
    }
    let text = format!("{value:.3}");
    text.replace('.', ",")
}

/// Render the human-readable report.
pub fn render(result: &RunResult, program: &Program) -> String {
    let mut out = String::with_capacity(8 * 1024);
    let _ = writeln!(out, "                             Reporte MicroVisualSIS");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        " START_TIME    END_TIME     BLOCKS    FACILITIES    STORAGES"
    );
    let _ = writeln!(
        out,
        "{:>10}  {:>10}  {:>10}  {:>12}  {:>10}",
        number(result.start_time),
        number(result.end_time),
        program.blocks.len(),
        result.entities.facilities.len(),
        result.entities.storages.len()
    );
    let _ = writeln!(out);
    let _ = writeln!(out, " SEMILLA    {}", result.seed);
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        " LINE    ENTRY CURRENT RETRY            LOC  BLOCK_TYPE"
    );
    let _ = writeln!(out, "         COUNT   COUNT");
    let mut numeric_label = 0;
    for (index, block) in program.blocks.iter().enumerate() {
        let stats = result.blocks.get(index).cloned().unwrap_or_default();
        let loc = match &block.label {
            Some(label) => label.clone(),
            None => {
                numeric_label += 1;
                numeric_label.to_string()
            }
        };
        let _ = writeln!(
            out,
            "{:>5} {:>8} {:>7} {:>5} {:>14}  {}",
            index + 1,
            stats.entries,
            stats.current,
            stats.retries,
            loc,
            type_and_operands(block, program)
        );
    }

    if !result.entities.facilities.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "FACILITY       ENTRIES  UTIL.  AVE_TIME  AVAILABLE  OWNER PEND INTER RETRY DELAY"
        );
        let elapsed = result.elapsed().max(1e-9);
        for facility in result.entities.facilities.values() {
            let _ = writeln!(
                out,
                "  {:<14} {:>7} {:>7} {:>9} {:>10} {:>6} {:>4} {:>5} {:>5} {:>5}",
                facility.name.to_uppercase(),
                facility.entries,
                number(facility.utilization(elapsed)),
                number(facility.average_time()),
                if facility.available { 1 } else { 0 },
                facility.owner.unwrap_or(0),
                // The original keeps the transactions that wait for the
                // facility in its `DELAY` column and leaves `PEND` at zero.
                0,
                facility.inter,
                facility.retry,
                facility.waiters.len(),
            );
        }
    }

    if !result.entities.queues.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "QUEUE            MAX  CONT. ENTRIES ENTRIES(0) AVE.CONT. AVE.TIME AVE.(-0) RETRY"
        );
        let elapsed = result.elapsed().max(1e-9);
        for queue in result.entities.queues.values() {
            let _ = writeln!(
                out,
                "  {:<14} {:>5} {:>6} {:>7} {:>10} {:>9} {:>9} {:>8} {:>5}",
                queue.name.to_uppercase(),
                queue.max,
                queue.content,
                queue.entries,
                queue.zero_entries,
                number(queue.average_content(elapsed)),
                number(queue.average_time()),
                number(queue.average_time_nonzero()),
                queue.retry,
            );
        }
    }

    if !result.entities.storages.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "STORAGE           CAP. REMAIN. MIN.  MAX. ENTRIES AVL. AVE.C.  UTIL. RETRY DELAY"
        );
        let elapsed = result.elapsed().max(1e-9);
        for storage in result.entities.storages.values() {
            let _ = writeln!(
                out,
                "  {:<16} {:>4} {:>7} {:>5} {:>5} {:>7} {:>4} {:>7} {:>6} {:>5} {:>5}",
                storage.name.to_uppercase(),
                storage.capacity,
                storage.remaining(),
                storage.min_use,
                storage.max_use,
                storage.entries,
                1,
                number(storage.average_content(elapsed)),
                number(storage.utilization(elapsed)),
                storage.retry,
                storage.delay,
            );
        }
    }

    if !result.entities.chains.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "USER_CHAIN       CHAIN_SIZE   RETRY   AVE.CONT   ENTRIES    MAX     AVE.TIME"
        );
        let elapsed = result.elapsed().max(1e-9);
        for chain in result.entities.chains.values() {
            let _ = writeln!(
                out,
                "  {:<16} {:>10} {:>7} {:>10} {:>9} {:>6} {:>12}",
                chain.name.to_uppercase(),
                chain.content,
                0,
                number(chain.average_content(elapsed)),
                chain.entries,
                chain.max,
                number(chain.average_time()),
            );
        }
    }

    for name in &result.table_names {
        let Some(table) = result.entities.tables.get(name) else {
            continue;
        };
        let _ = writeln!(out);
        let _ = writeln!(out, "TABLAS");
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "{:<16} TABLE           {}, {}, {}",
            table.name, number(table.first), number(table.width), table.intervals
        );
        let _ = writeln!(out, "    MEDIA              : {}", number(table.mean()));
        let _ = writeln!(
            out,
            "    DESVIACION ESTANDAR: {}",
            number(table.stddev())
        );
        let _ = writeln!(out, "    CANTIDAD DE VALORES: {}", table.samples);
        let _ = writeln!(out, "    VALOR MAXIMO       : {}", number(table.max_value));
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "    INTERVALO         FRECUENCIA     %     %ACUMULADO     %REMANENTE"
        );
        let samples = table.samples.max(1) as f64;
        let mut cumulative = 0.0;
        for (index, count) in table.frequencies.iter().enumerate() {
            let share = *count as f64 / samples * 100.0;
            cumulative += share;
            let low = if index == 0 {
                0.0
            } else {
                table.first + (index as f64 - 1.0) * table.width + 1.0
            };
            let high = table.first + index as f64 * table.width;
            let _ = writeln!(
                out,
                "    {:<5} - {:<8} {:>7} {:>10} {:>10} {:>12}",
                number(low),
                number(high),
                count,
                number(share),
                number(cumulative),
                number(100.0 - cumulative)
            );
        }
    }

    for name in &result.qtable_names {
        let Some(table) = result.entities.qtables.get(name) else {
            continue;
        };
        let _ = writeln!(out);
        let _ = writeln!(out, "QTABLES");
        let _ = writeln!(
            out,
            "{:<16} QTABLE          {}, {}, {}",
            table.name, number(table.first), number(table.width), table.intervals
        );
        let _ = writeln!(out, "    MEDIA              : {}", number(table.mean()));
        let _ = writeln!(
            out,
            "    DESVIACION ESTANDAR: {}",
            number(table.stddev())
        );
        let _ = writeln!(out, "    CANTIDAD DE VALORES: {}", table.samples);
    }

    if !result.entities.savevalues.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(out, "SAVEVALUE                 VALUE");
        for (name, value) in &result.entities.savevalues {
            let _ = writeln!(out, "  {:<18} {:>10}", name, number(*value));
        }
    }

    if !result.diagnostics.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(out, "DIAGNOSTICOS");
        for message in &result.diagnostics {
            let _ = writeln!(out, "  {message}");
        }
    }

    out
}

/// GPSS block type shown in the report's `BLOCK_TYPE` column.
///
/// Most GenMSI classes keep their own uppercased name, but a few expand into
/// differently named GPSS statements. The mapping below is proven by these
/// baseline rows in `fixtures/outputs/*.report.txt`:
///
/// - `GateL  GATE  LS  Puerta` (`TP6__Clase 6_1_PuertaBanco`) and
///   `GateF  GATE  NU  Juan,A_Advance` (`TP6__Clase 6_3_JefeEducado`): the
///   three gate classes all print `GATE`.
/// - `Multifurcacion  TRANSFER  ,FN$MultifurcacionGO` (`TP10__tp10`).
/// - `A_SelectF  SELECT  NU  Caja,1,10,,,A_Select` and
///   `A_Select  SELECT  MIN  Caja,1,10,,Q` (`Parcial 6-11-23__Veron - Parcial`,
///   `TP4__supermercado`, `TP5__burgerShop`).
/// - `A_JoinNumber  JOIN  GPesos,P$PesoPescado` and
///   `A_JoinMe  JOIN  EsperaServiciosVarios` (`TP10__tp10`, `TP8__mecanicos`).
/// - `A_ExamineNumber  EXAMINE  GPesos,P$PesoPescado,A_Assign7` (`TP10__tp10`).
/// - `A_RemoveTrans  REMOVE  E  EsperaServiciosVarios,1,,IDAutomovil,P$IDAutomovil`
///   (`TP10__tp10`).
/// - `A_Counter  SAVEVALUE  CAutomovil+,1` (`TP8__mecanicos`, `TP10__tp10`):
///   the original has no `COUNTER` statement, a `Counter` module expands to
///   `SAVEVALUE`.
pub fn block_type(block: &ExecBlock) -> std::borrow::Cow<'_, str> {
    match block.class.as_str() {
        "GateL" | "GateS" | "GateF" => std::borrow::Cow::Borrowed("GATE"),
        // The `Match` class expands into a `TRANSFER` and a `MATCH`.
        "MatchTransfer" => std::borrow::Cow::Borrowed("TRANSFER"),
        "Multifurcacion" => std::borrow::Cow::Borrowed("TRANSFER"),
        "SelectF" => std::borrow::Cow::Borrowed("SELECT"),
        "JoinNumber" | "JoinMe" => std::borrow::Cow::Borrowed("JOIN"),
        "ExamineNumber" => std::borrow::Cow::Borrowed("EXAMINE"),
        "RemoveTrans" => std::borrow::Cow::Borrowed("REMOVE"),
        "Counter" => std::borrow::Cow::Borrowed("SAVEVALUE"),
        other => std::borrow::Cow::Owned(other.to_uppercase()),
    }
}

/// The GPSS comparison letters used by `TEST`:
/// `TEST E P$SectorVisitado,3,A_Test1`, `TEST NE P$ImpacienciaMedia,0,A_JoinMe`,
/// `TEST L P$Tipo,6,Entrada11`, `TEST G P$Donde,3,A_Unlink1`,
/// `TEST GE P$CantPescados,1,A_Terminate7` (baseline rows in
/// `fixtures/outputs/*.report.txt`).
fn rel_op_letter(op: RelOp) -> &'static str {
    match op {
        RelOp::Lt => "L",
        RelOp::Le => "LE",
        RelOp::Eq => "E",
        RelOp::Ne => "NE",
        RelOp::Ge => "GE",
        RelOp::Gt => "G",
    }
}

/// GPSS `LOC` of the block at `index`: its label, or the running number
/// `render` assigns to unlabeled blocks.
fn loc_label(program: &Program, index: usize) -> String {
    let mut numeric = 0;
    program
        .blocks
        .iter()
        .take(index + 1)
        .map(|block| match &block.label {
            Some(label) => label.clone(),
            None => {
                numeric += 1;
                numeric.to_string()
            }
        })
        .last()
        .unwrap_or_default()
}

/// LOC of the target of the block at the port called `port`, when linked.
/// Gate, Test, SelectF and ExamineNumber modules route their "else" branch
/// through `Salida2`; the original prints that target as the last operand
/// (`GateF  GATE  NU  Juan,A_Advance`). When the compiler did not record the
/// port names, `Salida2` falls back to the second output.
fn port_target_label(program: &Program, block: &ExecBlock, port: &str) -> Option<String> {
    let position = block
        .port_names
        .iter()
        .position(|name| name == port)
        .or_else(|| (port == "Salida2" && block.next.len() > 1).then_some(1))?;
    block
        .next
        .get(position)
        .copied()
        .flatten()
        .map(|target| loc_label(program, target))
}

/// First operand of `TRANSFER`: the probability in thousandths like the
/// original (`800` for `80%`), the raw expression for function/variable
/// probabilities (`FN$PValue`), and nothing for the unconditional form.
fn transfer_probability(spec: &TransferSpec) -> String {
    if spec.thousandths {
        return spec.probability.to_string();
    }
    match spec.probability.ast() {
        // The compiler folds an empty `Prob` into 1.0; the original prints an
        // empty first operand there (`TRANSFER  ,A_Enter`).
        Expr::Num(1.0) => String::new(),
        Expr::Num(probability) => format!("{}", (probability * 1000.0).round() as i64),
        _ => spec.probability.to_string(),
    }
}

/// Whether an operand is the number one.
///
/// The original writes the units of a queue or a storage only when the model
/// gives them: `QUEUE qAlmacen,2` has two, `ENTER Migracion` has none.
fn is_one_unit(expr: &ProgramExpr) -> bool {
    expr.source().parse::<f64>() == Ok(1.0)
}

/// Operand text of an expression, the way the original prints it.
///
/// A percentage literal is the editor's per-mille value (`70%` is the number
/// 700, see the `percent` rule in `expr/grammar.pest`), and the original prints
/// that number: `A_Assign43  ASSIGN  pValue,700` in `TP3__aeropuerto`, where the
/// model stores `70%`. Every other expression keeps its source text, so SNA
/// operands read back exactly as the model wrote them.
fn expr_operand(expr: &ProgramExpr) -> String {
    if expr.source().trim_end().ends_with('%')
        && let Expr::Num(value) = expr.ast()
    {
        return format!("{}", value.round() as i64);
    }
    // A bare operand that resolves to a parameter is the original's `*name`:
    // `A_Assign6  ASSIGN  cantPAux,*cantP` (`TP3__aeropuerto`), where the model
    // writes `cantP`. A prefixed form (`P:cantP`) keeps its source text.
    if let Expr::Ref {
        kind: RefKind::Parameter,
        name: NameSource::Literal(name),
    } = expr.ast()
        && expr.source().trim() == name.as_str()
    {
        return format!("*{name}");
    }
    expr.to_string()
}

/// Suffix the original appends to `ASSIGN`/`SAVEVALUE` parameters for the
/// combining operations (`Bondi+,1`); the plain assignment has none.
fn assign_suffix(op: AssignOp) -> &'static str {
    match op {
        AssignOp::Set => "",
        AssignOp::Add => "+",
        AssignOp::Sub => "-",
    }
}

/// GPSS mode column of a block (`None` when its statement has none) and its
/// comma-separated operand list, both as the original prints them.
fn operand_parts(block: &ExecBlock, program: &Program) -> (Option<&'static str>, Vec<String>) {
    let mut mode: Option<&'static str> = None;
    let refs: Vec<String> = match &block.kind {
        BlockKind::Generate(spec) => vec![
            expr_operand(&spec.mean),
            spec.modifier.as_ref().map(expr_operand).unwrap_or_default(),
            spec.offset.as_ref().map(expr_operand).unwrap_or_default(),
            spec.limit.map(number).unwrap_or_default(),
            spec.priority.as_ref().map(expr_operand).unwrap_or_default(),
        ],
        BlockKind::Terminate(spec) => {
            if spec.units > 0.0 {
                vec![number(spec.units)]
            } else {
                vec![]
            }
        }
        BlockKind::Advance(spec) => vec![
            expr_operand(&spec.mean),
            spec.modifier.as_ref().map(expr_operand).unwrap_or_default(),
        ],
        BlockKind::Assign(spec) => vec![
            format!("{}{}", spec.parameter, assign_suffix(spec.op)),
            expr_operand(&spec.value),
        ],
        BlockKind::SaveValue(spec) => vec![
            format!("{}{}", spec.savevalue, assign_suffix(spec.op)),
            expr_operand(&spec.value),
        ],
        BlockKind::Counter(spec) => {
            // The original expands `Counter` to `SAVEVALUE name+,increment`
            // (`A_Counter  SAVEVALUE  CAutomovil+,1`, TP8/TP10 baselines).
            vec![
                format!("{}+", spec.savevalue),
                expr_operand(&spec.increment),
            ]
        }
        BlockKind::Queue(spec) | BlockKind::Depart(spec) => {
            // The original writes the units the queue holds: `QUEUE
            // qAlmacen,2`.
            let mut refs = vec![spec.queue.to_string()];
            if !is_one_unit(&spec.units) {
                refs.push(spec.units.to_string());
            }
            refs
        }
        BlockKind::Seize(spec) | BlockKind::Release(spec) | BlockKind::Return(spec) => {
            vec![spec.facility.to_string()]
        }
        BlockKind::Preempt(spec) => vec![spec.facility.to_string()],
        BlockKind::Enter(spec) | BlockKind::Leave(spec) => {
            // The units are written when the model gives them: the original's
            // `ENTER Migracion` has none, and its `ENTER almacen,2` has two.
            let mut refs = vec![spec.storage.to_string()];
            if !is_one_unit(&spec.units) {
                refs.push(spec.units.to_string());
            }
            refs
        }
        BlockKind::Test(spec) => {
            // `TEST    E       P$SectorVisitado,3,A_Test1`.
            let letter = rel_op_letter(spec.op);
            let mut refs = vec![expr_operand(&spec.left), expr_operand(&spec.right)];
            if let Some(label) = port_target_label(program, block, "Salida2") {
                refs.push(label);
            }
            mode = Some(letter);
            refs
        }
        BlockKind::Transfer(spec) => {
            // `TRANSFER 800,A_Advance5,A_Advance2` (Parcial 6-11-23): the
            // probability in thousandths, then the LOC of the `Salida2`
            // target, then the LOC of the `Salida1` target. A transfer with
            // a single output writes only that one (`TRANSFER ,AAA`).
            let mut refs = vec![transfer_probability(spec)];
            if block.port_names.iter().any(|port| port == "Salida2") {
                refs.push(port_target_label(program, block, "Salida2").unwrap_or_default());
            }
            refs.push(port_target_label(program, block, "Salida1").unwrap_or_default());
            refs
        }
        BlockKind::MultiTransfer(spec) => {
            // `Multifurcacion` generates a helper function in the original
            // (`TRANSFER  ,FN$MultifurcacionGO`, TP10); we keep the branch
            // weights the model actually stores.
            spec.weights.iter().map(expr_operand).collect()
        }
        BlockKind::Gate(spec) => {
            // `GateL  GATE  LS  Puerta`, `GateL  GATE  LR  Fin,A_Counter1`,
            // `HornoConComidas  GATE  SNE  Comidas`,
            // `GateF  GATE  NU  Juan,A_Advance`.
            let (letter, entity) = match &spec.condition {
                GateCondition::Logic { switch, state } => {
                    (if *state { "LS" } else { "LR" }, switch.to_string())
                }
                GateCondition::Storage { storage, op } => {
                    (op.letter(), storage.to_string())
                }
                GateCondition::Facility { facility, available } => {
                    (if *available { "NU" } else { "U" }, facility.to_string())
                }
            };
            let mut refs = vec![entity];
            if let Some(label) = port_target_label(program, block, "Salida2") {
                refs.push(label);
            }
            mode = Some(letter);
            refs
        }
        BlockKind::Logic(spec) => {
            // `LOGIC   R       Molinete` / `LOGIC   S       Molinete`.
            mode = Some(match spec.op {
                LogicOp::Set => "S",
                LogicOp::Reset => "R",
            });
            vec![spec.switch.to_string()]
        }
        BlockKind::Link(spec) => {
            // `LINK CPescando,FIFO` (TP10): the chain and the insertion order.
            vec![
                spec.chain.to_string(),
                if spec.fifo { "FIFO" } else { "LIFO" }.to_string(),
            ]
        }
        BlockKind::Unlink(spec) => {
            // `UNLINK E CPescando,A_Advance2,1,NroEspera,P$NroEspera,A_Terminate2`
            // (TP10 `A_Unlink1`): chain, then the LOC the removed members are
            // routed to (the `SalidaAlt` port, which is the one `sim.rs` sends
            // unlinked transactions through), then the count (`ALL` for
            // `Modo = Todos`), the matched parameter and value, and finally
            // the LOC of the no-match branch (`Salida2`, the port the engine
            // takes when nothing was unlinked).
            let count = if spec.all {
                "ALL".to_string()
            } else {
                spec.count.to_string()
            };
            let mut refs = vec![
                spec.chain.to_string(),
                port_target_label(program, block, "SalidaAlt").unwrap_or_default(),
                count,
                spec.compare_parameter.clone().unwrap_or_default(),
                spec.compare_value.as_ref().map(expr_operand).unwrap_or_default(),
            ];
            refs.push(port_target_label(program, block, "Salida2").unwrap_or_default());
            mode = Some("E");
            refs
        }
        BlockKind::Split(spec) => {
            // `SPLIT P$CantTuristas,A_Queue1` (Parcial 6-11-23 `Clona`): the
            // copy count and the LOC the copies are routed to (`Salida2`).
            vec![
                expr_operand(&spec.total),
                port_target_label(program, block, "Salida2").unwrap_or_default(),
            ]
        }
        BlockKind::Assemble(spec) => vec![expr_operand(&spec.count)],
        BlockKind::Match(spec) => vec![spec.conjugate.clone()],
        BlockKind::Loop(spec) => {
            // `LOOP Vitrinas,A_Advance7` (Parcial 6-11-23 `A_Loop1`): the
            // decremented parameter and the LOC of the back edge (`Salida2`).
            vec![
                spec.parameter.clone(),
                port_target_label(program, block, "Salida2").unwrap_or_default(),
            ]
        }
        BlockKind::Select(spec) => {
            // `A_SelectF  SELECT  NU  Caja,1,10,,,A_Select` (first free) and
            // `A_Select  SELECT  MIN  Caja,1,10,,Q` (smallest queue content):
            // parameter, range, an unused condition operand, then the scanned
            // entity class for MIN/MAX or the alternate destination for
            // first-free. The mode letter goes in the mode column.
            let letter = match spec.mode {
                SelectMode::FirstFree => "NU",
                SelectMode::Min => "MIN",
                SelectMode::Max => "MAX",
            };
            let mut refs = vec![
                spec.parameter.clone(),
                expr_operand(&spec.low),
                expr_operand(&spec.high),
            ];
            if spec.mode == SelectMode::FirstFree {
                refs.push(String::new());
                refs.push(String::new());
                if let Some(label) = port_target_label(program, block, "Salida2") {
                    refs.push(label);
                }
            } else {
                refs.push(String::new());
                refs.push(spec.kind.to_string());
            }
            mode = Some(letter);
            refs
        }
        BlockKind::Tabulate(spec) => {
            // `TABULATE TGondolas,1`: the weight, when the model sets `Peso`
            // (`TABULATE DinoFanaticos` keeps the operand empty without it).
            let mut refs = vec![spec.table.clone()];
            if let Some(weight) = &spec.weight {
                refs.push(expr_operand(weight));
            }
            refs
        }
        BlockKind::Priority(expr) => vec![expr_operand(expr)],
        BlockKind::Join(spec) => {
            // `A_JoinNumber  JOIN  GPesos,P$PesoPescado` (numbered) and
            // `A_JoinMe  JOIN  EsperaServiciosVarios` (the transaction itself).
            let mut refs = vec![spec.group.clone()];
            if let Some(number) = &spec.number {
                refs.push(expr_operand(number));
            }
            refs
        }
        BlockKind::Examine(spec) => {
            // `A_ExamineNumber  EXAMINE  GPesos,P$PesoPescado,A_Assign7`.
            let mut refs = vec![spec.group.clone(), expr_operand(&spec.number)];
            if let Some(label) = port_target_label(program, block, "Salida2") {
                refs.push(label);
            }
            refs
        }
        BlockKind::Remove(spec) => {
            // `A_RemoveTrans  REMOVE  E  EsperaServiciosVarios,1,,otro,1`: the
            // equality mode of the comparison, group, count (`ALL` when every
            // member is removed), an unused operand, then the compared
            // parameter and value.
            let count = if spec.all {
                "ALL".to_string()
            } else {
                spec.count.to_string()
            };
            let mut refs = vec![spec.group.clone(), count, String::new()];
            if let Some(parameter) = &spec.parameter {
                refs.push(parameter.clone());
            }
            if let Some(value) = &spec.value {
                refs.push(expr_operand(value));
            }
            mode = Some("E");
            refs
        }
        BlockKind::Scan(spec) => {
            // `SCAN E GPescando,IdPescador,P$NroPescador,IdPescador,Idescador,A_Advance1`
            // (TP10): the leading letter is the model's `CompOp` (`=` renders
            // as `E`, `Minimo` as `MIN`), then the group, the compared
            // parameter and value, and the copied parameter pair. The
            // compiler folds `Maximo` into `minimum = false`, so a MAX scan
            // would also print `E`; no fixture pins that case.
            let mut refs = vec![spec.group.clone()];
            refs.push(spec.compare_parameter.clone().unwrap_or_default());
            refs.push(spec.compare_value.as_ref().map(expr_operand).unwrap_or_default());
            refs.push(spec.from_parameter.clone().unwrap_or_default());
            refs.push(spec.to_parameter.clone().unwrap_or_default());
            refs.push(port_target_label(program, block, "Salida2").unwrap_or_default());
            mode = Some(if spec.minimum { "MIN" } else { "E" });
            refs
        }
        BlockKind::Alter(spec) => vec![spec.group.clone()],
        BlockKind::InitialLogic(spec) => vec![spec.switch.to_string()],
        BlockKind::NoOp | BlockKind::Buffer => Vec::new(),
    };
    (mode, refs)
}

/// Display string of a block's operands (JSON export and `explain_model`):
/// the GPSS mode letter, when the statement has one, followed by the
/// comma-separated operand list.
pub fn operands(block: &ExecBlock, program: &Program) -> String {
    let (mode, refs) = operand_parts(block, program);
    let list = refs.join(",");
    match mode {
        Some(mode) => format!("{mode} {list}"),
        None => list,
    }
}

/// The `BLOCK_TYPE` cell plus the operand column of one report row, laid out
/// like the original. Statements with a mode (`GATE`, `TEST`, `LOGIC`, `SCAN`,
/// `SELECT`, `REMOVE`, `UNLINK`) give the letter its own column: the type is
/// left-aligned to `max(8, len + 3)` and the mode to 8 more columns, so the
/// operand list starts at column 16 (`GATE    LS      Puerta`) or 17
/// (`REMOVE   E       GPescando,...`). Without a mode the list starts at
/// column 16 (`TRANSFER  800,A_Advance5,A_Advance2`).
fn type_and_operands(block: &ExecBlock, program: &Program) -> String {
    let (mode, refs) = operand_parts(block, program);
    let mut trimmed = refs;
    while trimmed.last().is_some_and(|part| part.is_empty()) {
        trimmed.pop();
    }
    let list = trimmed.join(",");
    let block_type = block_type(block);
    match mode {
        Some(mode) => {
            let type_width = (block_type.len() + 3).max(8);
            format!("{block_type:<type_width$}{mode:<8}{list}")
        }
        None => format!("{block_type:<16}{list}"),
    }
}

/// One block row in the JSON export.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct JsonBlock<'a> {
    /// 1-based line the report shows.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub line: usize,
    /// Module id.
    pub node: u32,
    /// Block class.
    pub class: &'a str,
    /// GPSS label.
    pub label: Option<&'a str>,
    /// Operands, rendered.
    pub operands: String,
    /// Entry count.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub entries: u64,
    /// Current count.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub current: u64,
    /// Retry count.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub retries: u64,
}

/// JSON document exported by the CLI and stored by the MCP tool.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
pub struct JsonReport<'a> {
    /// Program title.
    pub title: &'a str,
    /// Seed used.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub seed: u64,
    /// Start time.
    pub start_time: f64,
    /// End time.
    pub end_time: f64,
    /// End reason.
    pub end_reason: &'a str,
    /// Transactions created.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub transactions: u64,
    /// Events processed.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub events: u64,
    /// Block table.
    pub blocks: Vec<JsonBlock<'a>>,
    /// Entity statistics.
    pub entities: &'a crate::stat::Entities,
    /// Diagnostics.
    pub diagnostics: &'a [String],
}

/// Build the JSON document for a run.
pub fn json_report<'a>(result: &'a RunResult, program: &'a Program) -> JsonReport<'a> {
    let mut numeric_label = 0usize;
    let blocks = program
        .blocks
        .iter()
        .enumerate()
        .map(|(index, block)| {
            let stats: BlockStats = result.blocks.get(index).cloned().unwrap_or_default();
            let label = match &block.label {
                Some(label) => Some(label.as_str()),
                None => {
                    numeric_label += 1;
                    None
                }
            };
            JsonBlock {
                line: index + 1,
                node: block.module_id,
                class: &block.class,
                label,
                operands: operands(block, program),
                entries: stats.entries,
                current: stats.current,
                retries: stats.retries,
            }
        })
        .collect();
    JsonReport {
        title: &result.title,
        seed: result.seed,
        start_time: result.start_time,
        end_time: result.end_time,
        end_reason: result.end_reason.as_str(),
        transactions: result.transactions,
        events: result.events,
        blocks,
        entities: &result.entities,
        diagnostics: &result.diagnostics,
    }
}

/// Serialize a run to pretty JSON.
pub fn to_json(result: &RunResult, program: &Program) -> Result<String, EngineError> {
    serde_json::to_string_pretty(&json_report(result, program))
        .map_err(|err| EngineError::Runtime(format!("cannot serialize report: {err}")))
}

/// Serialize a run to CSV (`section,entity,metric,value`).
pub fn to_csv(result: &RunResult, program: &Program) -> String {
    let mut out = String::from("section,entity,metric,value\n");
    let elapsed = result.elapsed().max(1e-9);
    let _ = writeln!(
        out,
        "run,{},start_time,{}",
        csv_escape(&result.title),
        result.start_time
    );
    let _ = writeln!(
        out,
        "run,{},end_time,{}",
        csv_escape(&result.title),
        result.end_time
    );
    let mut numeric_label = 0usize;
    for (index, block) in program.blocks.iter().enumerate() {
        let stats = result.blocks.get(index).cloned().unwrap_or_default();
        let loc = match &block.label {
            Some(label) => label.clone(),
            None => {
                numeric_label += 1;
                numeric_label.to_string()
            }
        };
        let _ = writeln!(
            out,
            "block,{},entries,{}",
            csv_escape(&loc),
            stats.entries
        );
    }
    for facility in result.entities.facilities.values() {
        let _ = writeln!(
            out,
            "facility,{},utilization,{}",
            csv_escape(&facility.name),
            facility.utilization(elapsed)
        );
        let _ = writeln!(
            out,
            "facility,{},average_time,{}",
            csv_escape(&facility.name),
            facility.average_time()
        );
    }
    for queue in result.entities.queues.values() {
        let _ = writeln!(
            out,
            "queue,{},average_content,{}",
            csv_escape(&queue.name),
            queue.average_content(elapsed)
        );
        let _ = writeln!(
            out,
            "queue,{},average_time,{}",
            csv_escape(&queue.name),
            queue.average_time()
        );
        let _ = writeln!(
            out,
            "queue,{},entries,{}",
            csv_escape(&queue.name),
            queue.entries
        );
    }
    for storage in result.entities.storages.values() {
        let _ = writeln!(
            out,
            "storage,{},utilization,{}",
            csv_escape(&storage.name),
            storage.utilization(elapsed)
        );
        let _ = writeln!(
            out,
            "storage,{},average_content,{}",
            csv_escape(&storage.name),
            storage.average_content(elapsed)
        );
    }
    out
}

fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// Serialize the whole program structure (for `explain_model` in Phase 3).
pub fn program_json(program: &Program) -> serde_json::Value {
    json!({
        "title": program.title,
        "start_counter": program.start_counter,
        "blocks": program.blocks.iter().map(|block| json!({
            "node": block.module_id,
            "class": block.class,
            "label": block.label,
            "operands": operands(block, program),
            "ports": block.port_names,
        })).collect::<Vec<_>>(),
        "storages": program.definitions.storages,
        "functions": program.definitions.functions.keys().collect::<Vec<_>>(),
        "variables": program.definitions.variables.keys().collect::<Vec<_>>(),
        "random_variables": program.definitions.random_variables.keys().collect::<Vec<_>>(),
        "tables": program.definitions.tables.keys().collect::<Vec<_>>(),
        "qtables": program.definitions.qtables.keys().collect::<Vec<_>>(),
        "diagnostics": program.diagnostics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::program::compile;
    use crate::sim::{SimConfig, Simulation};
    use genmsi_format::parse_str;

    fn model() -> Program {
        let xml = "<gmodel versioncode=\"1\"><gclasses/><modules>\
            <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"m\" name=\"m\" kind=\"C\"><links>\
            <link frommoduleid=\"1\" fromioid=\"1\" tomoduleid=\"2\" toioid=\"1\"/>\
            <link frommoduleid=\"2\" fromioid=\"2\" tomoduleid=\"3\" toioid=\"1\"/>\
            </links><inputs/><outputs/></module>\
            <module moduleid=\"1\" moduleclass=\"Generate\" owner=\"0\" caption=\"g\" name=\"g\" kind=\"A\"><outputs><output ioid=\"1\" name=\"Salida1\" caption=\"\" mylabel=\"L1\"/></outputs>\
            <props><property name=\"Media\" hint=\"\" value=\"10\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/><property name=\"Cantidad\" hint=\"\" value=\"3\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/></props></module>\
            <module moduleid=\"2\" moduleclass=\"Queue\" owner=\"0\" caption=\"q\" name=\"q\" kind=\"A\"><inputs><input ioid=\"1\" name=\"i\" caption=\"\" mylabel=\"L1\"/></inputs><outputs><output ioid=\"2\" name=\"Salida1\" caption=\"\" mylabel=\"L2\"/></outputs>\
            <props><property name=\"Queue\" hint=\"\" value=\"cola\" type=\"EntityUse\" subType=\"Queue\" visible=\"1\" unique=\"0\"/></props></module>\
            <module moduleid=\"3\" moduleclass=\"Terminate\" owner=\"0\" caption=\"t\" name=\"t\" kind=\"A\"><inputs><input ioid=\"1\" name=\"i\" caption=\"\" mylabel=\"L2\"/></inputs>\
            <props><property name=\"Unidades\" hint=\"\" value=\"1\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/></props></module>\
            </modules></gmodel>";
        compile(&parse_str(xml).unwrap(), "mini").expect("compiles")
    }

    #[test]
    fn report_has_the_gpss_sections() {
        let program = model();
        let result = Simulation::new(&program, SimConfig::default())
            .run()
            .expect("runs");
        let text = render(&result, &program);
        assert!(text.contains("START_TIME"), "{text}");
        assert!(text.contains("ENTRY CURRENT RETRY"), "{text}");
        assert!(text.contains("QUEUE            MAX"), "{text}");
        assert!(text.contains("COLA"), "{text}");
        insta::assert_snapshot!("report_mini", text);
    }

    #[test]
    fn json_and_csv_export_are_stable() {
        let program = model();
        let result = Simulation::new(&program, SimConfig::default())
            .run()
            .expect("runs");
        let json = to_json(&result, &program).expect("json");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(value["end_time"], 10.0);
        assert_eq!(value["blocks"][1]["class"], "Queue");
        let csv = to_csv(&result, &program);
        assert!(csv.starts_with("section,entity,metric,value\n"));
        assert!(csv.contains("queue,cola,average_time"));
    }

    #[test]
    fn numbers_use_comma_decimals() {
        assert_eq!(number(1.5), "1,500");
        assert_eq!(number(2.0), "2");
        assert_eq!(number(0.2984), "0,298");
    }
}
