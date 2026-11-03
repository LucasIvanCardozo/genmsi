//! The trace the agent can filter, and the summary of a run.
//!
//! The trace of a run is a hose: a model of five hundred blocks makes
//! thousands of events, and the agent that wants the events of one class has
//! to dump them all and filter them on its side. The engine knows the class,
//! the block of the document and the time of every event as it records it,
//! so the filter belongs there: the filter is applied **when the event is
//! recorded**, and the limit of the trace counts the events that pass it.
//!
//! The summary answers the question the agent asks the ninety percent of the
//! time ("how many times did the flow go through here?"): how many entries a
//! block had, and the first and the last time it had them. It is computed
//! whether the trace is on or not, and it does not change the run.
//!
//! The trace also numbers the statements of the compiled program, and not
//! the blocks of the document: the event carries the `moduleid` of the block
//! it comes from, so "what block is the 218" has an answer of one line.

use genmsi_core::documents::{
    TRACE_SCHEMA, TRACE_SUMMARY_SCHEMA, to_pretty_json, trace_document, trace_summary,
};
use genmsi_core::program::compile;
use genmsi_core::{Program, RunResult, SimConfig, Simulation, TraceFilter};
use genmsi_format::parse_str;

mod support;

use support::{linear, module};

/// A model with a generator, a queue, an assignment and a termination.
///
/// The four modules are the ones of the document: the generator is the
/// module `1`, the queue the module `2`, the assignment the module `3` and
/// the termination the module `4`. The clock that the helper adds is the
/// module `5`.
fn model() -> Program {
    linear(
        &[
            module("Generate", "", &[("Media", "10"), ("Cantidad", "4")]),
            module("Queue", "Q1", &[("Queue", "cola")]),
            module("Assign", "A1", &[("Parametro", "p"), ("Valor", "7")]),
            module("Terminate", "T1", &[("Unidades", "1")]),
        ],
        1000.0,
    )
}

/// Run the model with the trace on and the given filter.
///
/// The run ends with the fourth transaction: the `START` of the run is four
/// termination units, so the four transactions of the generator go through
/// the whole flow.
fn traced(program: &Program, filter: TraceFilter, limit: usize) -> RunResult {
    Simulation::new(
        program,
        SimConfig {
            trace: true,
            trace_limit: limit,
            trace_filter: filter,
            start: 4,
            stop_time: Some(1000.0),
            ..SimConfig::default()
        },
    )
    .run()
    .expect("runs")
}

/// Run the model with the summary on and the trace off.
fn summarized(program: &Program) -> RunResult {
    Simulation::new(
        program,
        SimConfig {
            trace_summary: true,
            start: 4,
            stop_time: Some(1000.0),
            ..SimConfig::default()
        },
    )
    .run()
    .expect("runs")
}

#[test]
fn the_trace_event_carries_the_node_of_the_block_of_the_document() {
    let program = model();
    let result = traced(&program, TraceFilter::default(), 1000);
    assert!(!result.trace.is_empty(), "the run has events");
    for event in &result.trace {
        let statement = &program.blocks[event.block];
        assert_eq!(
            event.node, statement.module_id,
            "the event names the module of the statement"
        );
    }
    // The queue is the second module of the document.
    let queue = result
        .trace
        .iter()
        .find(|event| event.class == "Queue")
        .expect("the run passes through the queue");
    assert_eq!(queue.node, 2);
}

#[test]
fn a_filter_by_class_keeps_only_the_events_of_that_class() {
    let program = model();
    let filter = TraceFilter {
        classes: vec!["Queue".to_string()],
        ..TraceFilter::default()
    };
    let result = traced(&program, filter, 1000);
    assert!(!result.trace.is_empty(), "the filter keeps the queue events");
    assert!(
        result.trace.iter().all(|event| event.class == "Queue"),
        "the filter keeps only the events of the class"
    );
}

#[test]
fn the_limit_of_the_trace_counts_the_events_that_pass_the_filter() {
    let program = model();
    let whole = traced(&program, TraceFilter::default(), 1000);
    let filter = TraceFilter {
        classes: vec!["Terminate".to_string()],
        ..TraceFilter::default()
    };
    // The first events of the run are the ones of the generator: the limit
    // of two records must keep the first two *terminations*, and not the
    // first two events of the run.
    let result = traced(&program, filter, 2);
    assert_eq!(result.trace.len(), 2, "the limit counts the events that pass");
    assert!(
        result.trace.iter().all(|event| event.class == "Terminate"),
        "the events that pass are the ones that were asked for"
    );
    assert!(
        whole
            .trace
            .iter()
            .take(2)
            .all(|event| event.class != "Terminate"),
        "the first events of the run are not the ones that were asked for"
    );
}

#[test]
fn an_empty_filter_keeps_everything() {
    assert!(TraceFilter::default().keeps_everything());
    let by_class = TraceFilter {
        classes: vec!["Queue".to_string()],
        ..TraceFilter::default()
    };
    let by_node = TraceFilter {
        nodes: vec![2],
        ..TraceFilter::default()
    };
    let by_xact = TraceFilter {
        xact: Some(1),
        ..TraceFilter::default()
    };
    let from = TraceFilter {
        from: Some(1.0),
        ..TraceFilter::default()
    };
    let to = TraceFilter {
        to: Some(1.0),
        ..TraceFilter::default()
    };
    assert!(!by_class.keeps_everything());
    assert!(!by_node.keeps_everything());
    assert!(!by_xact.keeps_everything());
    assert!(!from.keeps_everything());
    assert!(!to.keeps_everything());
}

#[test]
fn a_filter_by_node_keeps_the_events_of_one_block_of_the_document() {
    let program = model();
    let filter = TraceFilter {
        nodes: vec![2],
        ..TraceFilter::default()
    };
    let result = traced(&program, filter, 1000);
    assert!(!result.trace.is_empty(), "the filter keeps the events of the block");
    assert!(
        result.trace.iter().all(|event| event.node == 2),
        "the filter keeps only the events of the block"
    );
    assert!(
        result.trace.iter().all(|event| event.class == "Queue"),
        "the block of the document is the queue"
    );
}

#[test]
fn a_filter_by_transaction_keeps_the_events_of_that_transaction() {
    let program = model();
    let whole = traced(&program, TraceFilter::default(), 1000);
    let xact = whole
        .trace
        .iter()
        .find(|event| event.class == "Queue")
        .expect("the run passes through the queue")
        .xact;
    let filter = TraceFilter {
        xact: Some(xact),
        ..TraceFilter::default()
    };
    let result = traced(&program, filter, 1000);
    assert!(!result.trace.is_empty(), "the filter keeps the events of the transaction");
    assert!(
        result.trace.iter().all(|event| event.xact == xact),
        "the filter keeps only the events of the transaction"
    );
}

#[test]
fn a_filter_by_time_window_keeps_the_events_of_that_window() {
    let program = model();
    let filter = TraceFilter {
        from: Some(20.0),
        to: Some(30.0),
        ..TraceFilter::default()
    };
    let result = traced(&program, filter, 1000);
    assert!(!result.trace.is_empty(), "the filter keeps the events of the window");
    assert!(
        result
            .trace
            .iter()
            .all(|event| event.time >= 20.0 && event.time <= 30.0),
        "the filter keeps only the events of the window"
    );
}

#[test]
fn an_empty_filter_keeps_the_same_trace_as_no_filter() {
    let program = model();
    let without = traced(&program, TraceFilter::default(), 1000);
    let with = traced(
        &program,
        TraceFilter {
            classes: Vec::new(),
            nodes: Vec::new(),
            xact: None,
            from: None,
            to: None,
        },
        1000,
    );
    assert_eq!(
        to_pretty_json(TRACE_SCHEMA, &trace_document(&without, 1000)),
        to_pretty_json(TRACE_SCHEMA, &trace_document(&with, 1000)),
        "an empty filter keeps every event"
    );
}

#[test]
fn the_summary_counts_the_entries_of_every_block() {
    let program = model();
    let result = summarized(&program);
    assert_eq!(
        result.summary.len(),
        program.blocks.len(),
        "the summary has one entry per block of the program"
    );
    let queue = result
        .summary
        .iter()
        .find(|block| block.class == "Queue")
        .expect("the summary names the queue");
    assert_eq!(queue.entries, 4, "the four transactions pass through the queue");
    assert_eq!(queue.first, 10.0, "the first transaction arrives at 10");
    assert_eq!(queue.last, 40.0, "the last transaction arrives at 40");
    assert_eq!(queue.node, 2, "the queue is the module 2 of the document");
    assert_eq!(queue.label.as_deref(), Some("Q1"), "the summary names the label");
}

#[test]
fn a_block_nobody_entered_is_in_the_summary_with_no_entries() {
    let program = model();
    let result = summarized(&program);
    // The clock of the model never ticks: the run ends before it.
    let clock = result
        .summary
        .iter()
        .find(|block| block.node == 5)
        .expect("the summary names the clock");
    assert_eq!(clock.entries, 0, "nobody entered the clock");
    assert_eq!(clock.first, 0.0, "a block nobody entered has no first time");
    assert_eq!(clock.last, 0.0, "a block nobody entered has no last time");
}

#[test]
fn the_summary_is_computed_with_the_trace_off() {
    let program = model();
    let result = summarized(&program);
    assert!(
        result.trace.is_empty(),
        "the run had no trace"
    );
    let queue = result
        .summary
        .iter()
        .find(|block| block.class == "Queue")
        .expect("the summary names the queue");
    assert_eq!(queue.entries, 4, "the summary does not need the trace");
}

#[test]
fn the_summary_does_not_change_the_run() {
    let program = model();
    let without = Simulation::new(
        &program,
        SimConfig {
            start: 4,
            stop_time: Some(1000.0),
            ..SimConfig::default()
        },
    )
    .run()
    .expect("runs");
    let with = summarized(&program);
    assert_eq!(
        genmsi_core::to_json(&without, &program).expect("json"),
        genmsi_core::to_json(&with, &program).expect("json"),
        "the summary does not change the report of the run"
    );
}

#[test]
fn the_summary_document_names_its_schema_and_carries_the_blocks() {
    let program = model();
    let result = summarized(&program);
    let text = to_pretty_json(TRACE_SUMMARY_SCHEMA, &trace_summary(&result));
    assert!(
        text.starts_with("{\n  \"schema\": \"genmsi.trace_summary/1\",\n"),
        "{text}"
    );
    assert!(text.contains("\"title\""), "{text}");
    assert!(text.contains("\"end_reason\""), "{text}");
    assert!(text.contains("\"blocks\""), "{text}");
    assert!(text.contains("\"entries\": 4"), "{text}");
}

/// A model written with one module per line, so the position of every module
/// in the file is known.
const WITH_LINES: &str = r#"<gmodel versioncode="1"><gclasses/><modules>
<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C"><links><link frommoduleid="1" fromioid="1" tomoduleid="2" toioid="1"/></links><inputs/><outputs/></module>
<module moduleid="1" moduleclass="Generate" owner="0" caption="g" name="g" kind="A"><outputs><output ioid="1" name="Salida1" caption="" mylabel="L1"/></outputs><props><property name="Media" hint="" value="10" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="2" moduleclass="Terminate" owner="0" caption="t" name="t" kind="A"><inputs><input ioid="1" name="i" caption="" mylabel="L1"/></inputs><props><property name="Unidades" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
</modules></gmodel>"#;

#[test]
fn the_explain_document_publishes_the_position_of_every_statement() {
    let model = parse_str(WITH_LINES).expect("the model parses");
    let program = compile(&model, "m").expect("the model compiles");
    let json = genmsi_core::program_json(&program);
    let blocks = json["blocks"].as_array().expect("the blocks");
    for (index, block) in blocks.iter().enumerate() {
        assert_eq!(block["index"], index, "the statement knows its index");
        let node = block["node"].as_u64().expect("the node") as u32;
        let source = model
            .block(node)
            .expect("the module of the document")
            .source
            .expect("the module has a position");
        assert_eq!(
            block["line"].as_u64().expect("the line") as usize,
            source.line,
            "the statement publishes the line of its module"
        );
        assert_eq!(
            block["column"].as_u64().expect("the column") as usize,
            source.column,
            "the statement publishes the column of its module"
        );
    }
    // The generator is the third line of the file, at its first column.
    assert_eq!(blocks[0]["line"], 3, "the generator is on the third line");
    assert_eq!(blocks[0]["column"], 1, "the generator is at the first column");
    assert_eq!(blocks[1]["line"], 4, "the termination is on the fourth line");
}

#[test]
fn the_summary_document_has_the_shape_the_client_reads() {
    let program = model();
    let result = summarized(&program);
    let document = trace_summary(&result);
    assert_eq!(document.title, "wired");
    assert_eq!(document.seed, 1);
    assert_eq!(document.start_time, 0.0);
    assert_eq!(document.end_time, 40.0);
    assert_eq!(document.end_reason, "terminated");
    assert_eq!(document.blocks.len(), program.blocks.len());
    let queue = document
        .blocks
        .iter()
        .find(|block| block.class == "Queue")
        .expect("the summary names the queue");
    assert_eq!(queue.block, 1, "the block is the index in the program");
    assert_eq!(queue.node, 2, "the node is the module of the document");
    assert_eq!(queue.label.as_deref(), Some("Q1"));
    assert_eq!(queue.entries, 4);
    assert_eq!(queue.first, 10.0);
    assert_eq!(queue.last, 40.0);
}
