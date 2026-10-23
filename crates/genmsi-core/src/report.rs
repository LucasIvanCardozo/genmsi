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
    AssignOp, BlockKind, ExecBlock, GateCondition, LogicOp, Program, RelOp, SelectMode, SelectState,
    TransferSpec,
};
use crate::sim::{BlockStats, GroupStat, LogicSwitchStat, RunResult};

/// Format a number the way the original report does.
///
/// The original writes three decimals and a comma, the separator of the
/// Spanish original: `0,000` and `9,636`. The whole numbers of the report
/// (the counts, the times of the header) are written by [`whole`].
pub fn number(value: f64) -> String {
    if !value.is_finite() {
        return "0,000".to_string();
    }
    let text = format!("{value:.3}");
    text.replace('.', ",")
}

/// Format a number the original writes without decimals when it is whole.
///
/// The counts of the report (the entries of a block, the capacity of a
/// storage, the value of a savevalue) and the times of its header are whole
/// numbers in the original.
pub fn whole(value: f64) -> String {
    if value.is_finite() && value.fract().abs() < 1e-9 && value.abs() < 1e12 {
        return format!("{}", value as i64);
    }
    number(value)
}

/// Format a number the way the oracle's tables print it.
///
/// The tables of the oracle compare numbers, and they are written with a dot
/// and without decimals when they are whole.
pub fn plain(value: f64) -> String {
    if !value.is_finite() {
        return "0".to_string();
    }
    if value.fract().abs() < 1e-9 && value.abs() < 1e12 {
        return format!("{}", value as i64);
    }
    format!("{value:.3}")
}

/// The value the original writes when the average it wanted has no divisor.
///
/// The original divides the integral of the entity by the number of its
/// entries, and when there are none it cannot compute the quotient: it
/// writes `0.000`, with a dot, the format Delphi gives a float it could not
/// compute (`QUEUE  1  0  64  64  0,000  0,000  0.000  0`).
pub fn not_computed() -> String {
    "0.000".to_string()
}

/// Render the human-readable report.
pub fn render(result: &RunResult, program: &Program) -> String {
    let mut out = String::with_capacity(8 * 1024);
    let table = block_table(program);
    let _ = writeln!(out, "                             Reporte MicroVisualSIS");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        " START_TIME    END_TIME     BLOCKS    FACILITIES    STORAGES"
    );
    let _ = writeln!(
        out,
        "{:>9}{:>13}{:>10}{:>12}{:>13}",
        whole(result.start_time),
        whole(result.end_time),
        table.rows.len(),
        result.entities.facilities.len(),
        result.entities.storages.len()
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        " LINE    ENTRY CURRENT RETRY            LOC  BLOCK_TYPE{:<32}",
        ""
    );
    let _ = writeln!(out, "         COUNT   COUNT   ");
    for row in table.rows.iter() {
        let stats = result.blocks.get(row.block).cloned().unwrap_or_default();
        let _ = writeln!(
            out,
            "{:>5} {:>8} {:>7} {:>5} {:>14}  {:<43}",
            row.line,
            stats.entries,
            stats.current,
            stats.retries,
            row.loc,
            row.text
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
                "  {:<17}{:>2}{:>8}{:>9}{:>8}{:>10}{:>5}{:>6}{:>6}{:>6}",
                facility.name.to_uppercase(),
                facility.entries,
                number(facility.utilization(elapsed)),
                // The average service time has no divisor when the
                // facility was never seized.
                if facility.entries == 0 {
                    not_computed()
                } else {
                    number(facility.average_time())
                },
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
        let mut queues: Vec<&crate::stat::Queue> = result.entities.queues.values().collect();
        queues.sort_by(|left, right| queue_order(&left.name, &right.name));
        for queue in queues {
            let _ = writeln!(
                out,
                "  {:<16}{:>2}{:>6}{:>8}{:>10}{:>10}{:>11}{:>9}{:>5}",
                queue.name.to_uppercase(),
                queue.max,
                queue.content,
                queue.entries,
                queue.zero_entries,
                number(queue.average_content(elapsed)),
                // The two averages of the waiting time have no divisor when
                // the queue took nobody, or when nobody waited.
                if queue.entries == 0 {
                    not_computed()
                } else {
                    number(queue.average_time())
                },
                if queue.entries == queue.zero_entries {
                    not_computed()
                } else {
                    number(queue.average_time_nonzero())
                },
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
                "  {:<17}{:>2}{:>7}{:>6}{:>6}{:>8}{:>4}{:>8}{:>8}{:>5}{:>6}",
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
                "  {:<17}{:>7}{:>8}{:>11}{:>10}{:>8}{:>13}",
                chain.name.to_uppercase(),
                chain.content,
                0,
                number(chain.average_content(elapsed)),
                chain.entries,
                chain.max,
                // The average time in the chain has no divisor when nobody
                // ever joined it.
                if chain.entries == 0 {
                    not_computed()
                } else {
                    number(chain.average_time())
                },
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
        // The header of the section is the statement of the table: its name,
        // the verb and the operands of the `TABLE` statement, the attribute
        // the table tabulates among them.
        let attribute = program
            .definitions
            .tables
            .get(name)
            .map(|definition| crate::gpss::sna(&definition.attribute))
            .unwrap_or_default();
        let operands = format!(
            "{},{},{},{}",
            attribute,
            whole(table.first),
            whole(table.width),
            table.intervals
        );
        let _ = writeln!(out, "{:<16}{:<16}{operands}  ", table.name, "TABLE");
        let _ = writeln!(out, "    {:<19}: {}", "MEDIA", number(table.mean()));
        let _ = writeln!(
            out,
            "    {:<19}: {}",
            "DESVIACION ESTANDAR",
            number(table.stddev())
        );
        // The tabulations that had to be retried: the original counts them
        // apart and none of them happens in a model that runs.
        let _ = writeln!(out, "    {:<19}: {}", "CANTIDAD EN RETRY", 0);
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "    INTERVALO         FRECUENCIA     %     %ACUMULADO     %REMANENTE"
        );
        let samples = table.samples.max(1) as f64;
        let mut cumulative = 0.0;
        let classes = table.frequencies.len().saturating_sub(1);
        // The last class of the table is the one of the values that do not
        // fit in any other: the original writes them apart, as the values
        // above the last limit.
        let rows: Vec<(String, String, String, u64)> = (0..table.frequencies.len())
            .map(|index| {
                let count = table.frequencies[index];
                let (low, separator, high) = if index == classes {
                    (
                        ">".to_string(),
                        "a".to_string(),
                        whole(table.first + (classes.saturating_sub(1) as f64) * table.width),
                    )
                } else {
                    let low = if index == 0 {
                        table.first
                    } else {
                        table.first + (index as f64 - 1.0) * table.width + 1.0
                    };
                    let high = table.first + index as f64 * table.width;
                    (whole(low), "-".to_string(), whole(high))
                };
                (low, separator, high, count)
            })
            .collect();
        for (index, (low, separator, high, count)) in rows.iter().enumerate() {
            let share = *count as f64 / samples * 100.0;
            cumulative += share;
            let _ = writeln!(
                out,
                "    {:<4}{:<3}{:<3}{:>15}{:>11}{:>12}{:>15}",
                low,
                separator,
                high,
                count,
                number(share),
                number(cumulative),
                number(100.0 - cumulative)
            );
            // The original stops the table when the classes add up to the
            // whole, and says how many empty classes are left.
            if cumulative >= 100.0 - 1e-9 && index + 1 < rows.len() {
                let _ = writeln!(
                    out,
                    "    Las frecuencias restantes ({}) son todas cero.",
                    rows.len() - index - 1
                );
                break;
            }
        }
        // The statistics of the values that did not fit: the original writes
        // them when there are any.
        if table.overflow > 0 {
            let _ = writeln!(out);
            let _ = writeln!(
                out,
                "    {:<29} : {}",
                "MEDIA OVERFLOW",
                number(table.overflow_mean())
            );
            let _ = writeln!(
                out,
                "    {:<29} : {}",
                "DESVIACION ESTANDAR OVERFLOW",
                number(table.overflow_stddev())
            );
        }
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "    {:<29} : {}",
            "CANTIDAD DE VALORES TABULADOS",
            table.samples
        );
        let _ = writeln!(
            out,
            "    {:<29} : {}",
            "VALOR MAXIMO TABULADO",
            number(table.max_value)
        );
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
        let _ = writeln!(out, "{:<26}{:>5}{:>11}", "SAVEVALUE", "VALUE", "RETRY");
        for (name, value) in &result.entities.savevalues {
            let _ = writeln!(
                out,
                "  {:<26}{:>2}{:>11}",
                name.to_uppercase(),
                whole(*value),
                0
            );
        }
    }

    if !result.logic_switches.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(out, "{:<26}{:>5}{:>11}", "LOGICSWITCH", "VALUE", "RETRY");
        let mut switches: Vec<&LogicSwitchStat> = result.logic_switches.iter().collect();
        switches.sort_by(|left, right| queue_order(&left.name, &right.name));
        for switch in switches {
            let _ = writeln!(
                out,
                "  {:<26}{:>2}{:>11}",
                switch.name.to_uppercase(),
                u8::from(switch.value),
                switch.retry
            );
        }
    }

    for (numeric, section) in [(false, "XACT_GROUP"), (true, "NUMERIC_GROUP")] {
        let groups: Vec<&GroupStat> = result
            .groups
            .iter()
            .filter(|group| group.numeric == numeric)
            .collect();
        if groups.is_empty() {
            continue;
        }
        let _ = writeln!(out);
        let _ = writeln!(out, "{:<21}{:>10}{:>11}", section, "GROUP_SIZE", "RETRY");
        for group in groups {
            let _ = writeln!(
                out,
                "  {:<26}{:>2}{:>11}",
                group.name.to_uppercase(),
                group.size,
                group.retry
            );
        }
    }

    if !result.pending.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "{:<16}{:>6}{:>9}{:>11}{:>7}{:>12}{:>8}",
            "FEC_XACT_NUMBER", "PRI", "BDT", "CURRENT", "NEXT", "PARAMETER", "VALUE"
        );
        for event in &result.pending {
            let _ = writeln!(
                out,
                "{:>14}{:>7}{:>11}{:>9}{:>8}",
                event.xact,
                event.priority,
                whole(event.time),
                table.numbers.get(event.block).copied().unwrap_or_default(),
                event
                    .next
                    .and_then(|block| table.numbers.get(block).copied())
                    .unwrap_or_default()
            );
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

/// One row of the block table.
struct BlockRow {
    /// The block the row counts.
    block: usize,
    /// The line of the listing the row stands for.
    line: usize,
    /// The `LOC` of the row.
    loc: String,
    /// The `BLOCK_TYPE` cell and the operand column.
    text: String,
}

/// The rows of the block table and the number of each statement.
struct BlockTable {
    /// The rows, in the order of the listing.
    rows: Vec<BlockRow>,
    /// The number of the statement of each block in the listing.
    numbers: Vec<usize>,
}

/// The rows of the block table.
///
/// The original lists one row per statement of the program, in the order the
/// listing writes them, and its `LOC` column holds the name of the statement
/// or, when the statement has none, the number it has in the listing.
///
/// The `Counter` class writes two statements (its macro code is a `SV` and an
/// `ASSI`): the `SAVEVALUE` that counts and the `ASSIGN` that copies the
/// counter into the parameter. The counter takes two rows, and the header
/// counts the rows.
fn block_table(program: &Program) -> BlockTable {
    let names = crate::gpss::block_names(program);
    let lines = crate::gpss::listing_lines(program);
    let order = crate::gpss::listing_order(program);
    let mut rows = Vec::with_capacity(program.blocks.len());
    let mut numbers = vec![0usize; program.blocks.len()];
    let mut statement = 0usize;
    for index in order {
        statement += 1;
        let name = names.get(index).cloned().unwrap_or_default();
        numbers[index] = statement;
        let loc = if name.is_empty() {
            statement.to_string()
        } else {
            name
        };
        rows.push(BlockRow {
            block: index,
            line: lines.get(index).copied().unwrap_or_default(),
            loc,
            text: type_and_operands(&program.blocks[index], &names),
        });
        if let BlockKind::Counter(spec) = &program.blocks[index].kind
            && let Some(parameter) = &spec.parameter
        {
            // The second statement of the counter has no name of its own.
            statement += 1;
            rows.push(BlockRow {
                block: index,
                line: lines.get(index).copied().unwrap_or_default() + 1,
                loc: statement.to_string(),
                text: format!("{:<16}{},X${}", "ASSIGN", parameter, spec.savevalue),
            });
        }
    }
    BlockTable { rows, numbers }
}

/// The order the original writes the entities in.
///
/// The original sorts them by name, but the names that are numbers are
/// compared as numbers: the queues `1, 2, …, 10` come in that order and not
/// in the alphabetical one (`1, 10, 2, …`).
fn queue_order(left: &str, right: &str) -> std::cmp::Ordering {
    match (left.parse::<f64>(), right.parse::<f64>()) {
        (Ok(left), Ok(right)) => left.partial_cmp(&right).unwrap_or(std::cmp::Ordering::Equal),
        (Ok(_), Err(_)) => std::cmp::Ordering::Less,
        (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
        (Err(_), Err(_)) => left.cmp(right),
    }
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

/// LOC of the target of the block at the port called `port`, when linked.
///
/// Gate, Test, SelectF and ExamineNumber modules route their "else" branch
/// through `Salida2`; the original prints that target as the last operand
/// (`GateF  GATE  NU  Juan,A_Advance`). When the compiler did not record the
/// port names, `Salida2` falls back to the second output.
fn port_target_label(names: &[String], block: &ExecBlock, port: &str) -> Option<String> {
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
        .map(|target| names.get(target).cloned().unwrap_or_default())
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
///
/// The names are the ones the listing gives to the blocks: the original
/// writes the name of the statement it points to.
fn operand_parts(
    block: &ExecBlock,
    names: &[String],
) -> (Option<&'static str>, Vec<String>) {
    let mut mode: Option<&'static str> = None;
    let refs: Vec<String> = match &block.kind {
        BlockKind::Generate(spec) => vec![
            expr_operand(&spec.mean),
            spec.modifier.as_ref().map(expr_operand).unwrap_or_default(),
            spec.offset.as_ref().map(expr_operand).unwrap_or_default(),
            spec.limit.map(whole).unwrap_or_default(),
            spec.priority.as_ref().map(expr_operand).unwrap_or_default(),
        ],
        BlockKind::Terminate(spec) => {
            if spec.units > 0.0 {
                vec![whole(spec.units)]
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
        BlockKind::Preempt(spec) => {
            // `PREEMPT         Recurso,PR,,Remanente`: the facility, the
            // mode of the interruption, the destination of the owner and the
            // parameter that keeps the time the owner had left.
            let priority = match spec.priority {
                crate::program::PreemptPriority::Preempting => "PR",
                crate::program::PreemptPriority::Unconditional => "",
            };
            vec![
                spec.facility.to_string(),
                priority.to_string(),
                if spec.route_owner { "1" } else { "" }.to_string(),
                spec.remaining.clone().unwrap_or_default(),
            ]
        }
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
            if let Some(label) = port_target_label(names, block, "Salida2") {
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
                refs.push(port_target_label(names, block, "Salida2").unwrap_or_default());
            }
            refs.push(port_target_label(names, block, "Salida1").unwrap_or_default());
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
            if let Some(label) = port_target_label(names, block, "Salida2") {
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
            // `LINK            Cadena,FIFO,A_Terminate`: the chain, the
            // insertion order and the block the transactions leave for.
            let mut refs = vec![
                spec.chain.to_string(),
                if spec.fifo { "FIFO" } else { "LIFO" }.to_string(),
            ];
            if let Some(target) = block.next.first().copied().flatten() {
                refs.push(names.get(target).cloned().unwrap_or_default());
            }
            refs
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
                port_target_label(names, block, "SalidaAlt").unwrap_or_default(),
                count,
                spec.compare_parameter.clone().unwrap_or_default(),
                spec.compare_value.as_ref().map(expr_operand).unwrap_or_default(),
            ];
            refs.push(port_target_label(names, block, "Salida2").unwrap_or_default());
            mode = Some("E");
            refs
        }
        BlockKind::Split(spec) => {
            // `SPLIT P$CantTuristas,A_Queue1` (Parcial 6-11-23 `Clona`): the
            // copy count and the LOC the copies are routed to (`Salida2`).
            vec![
                expr_operand(&spec.total),
                port_target_label(names, block, "Salida2").unwrap_or_default(),
            ]
        }
        BlockKind::Assemble(spec) => vec![expr_operand(&spec.count)],
        BlockKind::Match(spec) => vec![spec.conjugate.clone()],
        BlockKind::Loop(spec) => {
            // `LOOP Vitrinas,A_Advance7` (Parcial 6-11-23 `A_Loop1`): the
            // decremented parameter and the LOC of the back edge (`Salida2`).
            vec![
                spec.parameter.clone(),
                port_target_label(names, block, "Salida2").unwrap_or_default(),
            ]
        }
        BlockKind::Select(spec) => {
            // The row of a `SELECT` is the statement of the selection: the
            // letter of the operator and the operands the original writes
            // (`SNF param,1,10`, `LS param,1,10`, `E param,1,10,*1,Q,`,
            // `MIN param,1,10,,Q`, `NU param,1,10,,,A_Select`).
            let mut refs = vec![
                spec.parameter.clone(),
                expr_operand(&spec.low),
                expr_operand(&spec.high),
            ];
            match spec.state {
                Some(SelectState::LogicSet) => mode = Some("LS"),
                Some(SelectState::LogicReset) => mode = Some("LR"),
                Some(SelectState::StorageFull) => mode = Some("SF"),
                Some(SelectState::StorageNotFull) => mode = Some("SNF"),
                Some(SelectState::StorageEmpty) => mode = Some("SE"),
                Some(SelectState::StorageNotEmpty) => mode = Some("SNE"),
                // The storages that are in the "available" state (`SV`) and
                // the ones that are not (`SNV`).
                Some(SelectState::StorageAvailable) => mode = Some("SV"),
                Some(SelectState::StorageNotAvailable) => mode = Some("SNV"),
                Some(SelectState::EntityEqual) => {
                    // `E param,1,10,*1,Q,`: the value compared, the class of
                    // the entity and an empty operand after them.
                    mode = Some("E");
                    refs.push(
                        spec.compare_value
                            .as_ref()
                            .map(expr_operand)
                            .unwrap_or_default(),
                    );
                    refs.push(spec.class.clone());
                    refs.push(String::new());
                    return (mode, refs);
                }
                None => match spec.mode {
                    SelectMode::FirstFree => {
                        mode = Some("NU");
                        refs.push(String::new());
                        refs.push(String::new());
                        if let Some(label) = port_target_label(names, block, "Salida2") {
                            refs.push(label);
                        }
                    }
                    SelectMode::Min => {
                        mode = Some("MIN");
                        refs.push(String::new());
                        refs.push(spec.class.clone());
                    }
                    SelectMode::Max => {
                        mode = Some("MAX");
                        refs.push(String::new());
                        refs.push(spec.class.clone());
                    }
                },
            }
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
            if let Some(label) = port_target_label(names, block, "Salida2") {
                refs.push(label);
            }
            refs
        }
        BlockKind::Remove(spec) => {
            // `A_RemoveTrans  REMOVE  E  EsperaServiciosVarios,1,,otro,1`: the
            // equality mode of the comparison, group, count (`ALL` when every
            // member is removed), an unused operand, then the compared
            // parameter and value. The `RemoveMe` class takes the
            // transaction out of its group and writes no mode and no count.
            if spec.me {
                // The statement of the `RemoveMe` class keeps the mode
                // column empty: `REMOVE           UnGrupo`.
                return (Some(""), vec![spec.group.clone()]);
            }
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
            refs.push(port_target_label(names, block, "Salida2").unwrap_or_default());
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
    let names = crate::gpss::block_names(program);
    let (mode, refs) = operand_parts(block, &names);
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
fn type_and_operands(block: &ExecBlock, names: &[String]) -> String {
    let (mode, refs) = operand_parts(block, names);
    let mut trimmed = refs;
    if !keeps_the_trailing_empty(block) {
        while trimmed.last().is_some_and(|part| part.is_empty()) {
            trimmed.pop();
        }
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

/// Whether the statement of the block ends with an empty operand.
///
/// The `SELECTENT` of the original has one operand more than the other
/// selections and it is empty: `E param,1,10,*1,Q,`. The other statements
/// drop the empty operands they end with.
fn keeps_the_trailing_empty(block: &ExecBlock) -> bool {
    match &block.kind {
        BlockKind::Select(spec) => matches!(spec.state, Some(SelectState::EntityEqual)),
        _ => false,
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
        // The original writes three decimals and a comma in the averages.
        assert_eq!(number(1.5), "1,500");
        assert_eq!(number(2.0), "2,000");
        assert_eq!(number(0.2984), "0,298");
        // The whole numbers of the report have no decimals.
        assert_eq!(whole(2.0), "2");
        assert_eq!(whole(0.2984), "0,298");
        // The averages the original cannot compute keep the dot.
        assert_eq!(not_computed(), "0.000");
    }
}
