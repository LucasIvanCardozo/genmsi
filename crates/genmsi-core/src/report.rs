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
use crate::program::{BlockKind, ExecBlock, Program};
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
            "{:>5} {:>8} {:>7} {:>5} {:>14}  {:<14}  {}",
            index + 1,
            stats.entries,
            stats.current,
            stats.retries,
            loc,
            block.class.to_uppercase(),
            operands(block)
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
                facility.waiters.len(),
                facility.inter,
                facility.retry,
                facility.gate_waiters.len(),
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

/// Render the operands of a block the way GPSS shows them.
pub fn operands(block: &ExecBlock) -> String {
    let refs: Vec<String> = match &block.kind {
        BlockKind::Generate(spec) => vec![
            spec.mean.to_string(),
            spec.modifier.as_ref().map(ToString::to_string).unwrap_or_default(),
            spec.offset.as_ref().map(ToString::to_string).unwrap_or_default(),
            spec.limit.map(number).unwrap_or_default(),
            spec.priority.as_ref().map(ToString::to_string).unwrap_or_default(),
        ],
        BlockKind::Terminate(spec) => {
            if spec.units > 0.0 {
                vec![number(spec.units)]
            } else {
                vec![]
            }
        }
        BlockKind::Advance(spec) => vec![
            spec.mean.to_string(),
            spec.modifier.as_ref().map(ToString::to_string).unwrap_or_default(),
        ],
        BlockKind::Assign(spec) => vec![spec.parameter.clone(), spec.value.to_string()],
        BlockKind::SaveValue(spec) => vec![spec.savevalue.clone(), spec.value.to_string()],
        BlockKind::Counter(spec) => vec![spec.savevalue.clone(), spec.increment.to_string()],
        BlockKind::Queue(spec) | BlockKind::Depart(spec) => vec![spec.queue.to_string()],
        BlockKind::Seize(spec) | BlockKind::Release(spec) | BlockKind::Return(spec) => {
            vec![spec.facility.to_string()]
        }
        BlockKind::Preempt(spec) => vec![spec.facility.to_string()],
        BlockKind::Enter(spec) | BlockKind::Leave(spec) => {
            vec![spec.storage.to_string(), spec.units.to_string()]
        }
        BlockKind::Test(spec) => vec![
            spec.left.to_string(),
            format!("{:?}", spec.op),
            spec.right.to_string(),
        ],
        BlockKind::Transfer(spec) => vec![spec.probability.to_string()],
        BlockKind::MultiTransfer(spec) => {
            spec.weights.iter().map(ToString::to_string).collect()
        }
        BlockKind::Gate(spec) => vec![format!("{:?}", spec.condition)],
        BlockKind::Logic(spec) => vec![spec.switch.to_string(), format!("{:?}", spec.op)],
        BlockKind::Link(spec) => vec![spec.chain.to_string()],
        BlockKind::Unlink(spec) => vec![spec.chain.to_string(), spec.count.to_string()],
        BlockKind::Split(spec) => vec![spec.total.to_string()],
        BlockKind::Assemble(spec) => vec![spec.count.to_string()],
        BlockKind::Match(spec) => vec![spec.conjugate.clone()],
        BlockKind::Loop(spec) => vec![spec.parameter.clone()],
        BlockKind::Select(spec) => vec![
            spec.parameter.clone(),
            spec.low.to_string(),
            spec.high.to_string(),
            format!("{:?}", spec.mode),
        ],
        BlockKind::Tabulate(spec) => vec![spec.table.clone()],
        BlockKind::Priority(expr) => vec![expr.to_string()],
        BlockKind::Join(spec) => vec![spec.group.clone()],
        BlockKind::Examine(spec) => vec![spec.group.clone(), spec.number.to_string()],
        BlockKind::Remove(spec) => vec![spec.group.clone()],
        BlockKind::Scan(spec) => vec![spec.group.clone()],
        BlockKind::Alter(spec) => vec![spec.group.clone()],
        BlockKind::InitialLogic(spec) => vec![spec.switch.to_string()],
        BlockKind::NoOp | BlockKind::Buffer => Vec::new(),
    };
    let mut trimmed = refs;
    while trimmed.last().is_some_and(|part| part.is_empty()) {
        trimmed.pop();
    }
    trimmed.join(",")
}

/// One block row in the JSON export.
#[derive(Debug, Serialize)]
pub struct JsonBlock<'a> {
    /// 1-based line the report shows.
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
    pub entries: u64,
    /// Current count.
    pub current: u64,
    /// Retry count.
    pub retries: u64,
}

/// JSON document exported by the CLI and stored by the MCP tool.
#[derive(Debug, Serialize)]
pub struct JsonReport<'a> {
    /// Program title.
    pub title: &'a str,
    /// Seed used.
    pub seed: u64,
    /// Start time.
    pub start_time: f64,
    /// End time.
    pub end_time: f64,
    /// End reason.
    pub end_reason: &'a str,
    /// Transactions created.
    pub transactions: u64,
    /// Events processed.
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
                operands: operands(block),
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
            "operands": operands(block),
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
