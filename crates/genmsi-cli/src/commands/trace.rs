//! `genmsi trace <model>`: run the model and print its event trace.
//!
//! The trace has no filters (decision of the coordinator: minimum surface).
//! `--limit` bounds how many records the engine keeps, and the human output
//! says so when the bound was reached, so a truncated trace is never mistaken
//! for a complete one.

use genmsi_core::{Simulation, TraceEvent};
use serde::Serialize;

use crate::cli::TraceArgs;
use crate::error::{CliError, ExitCode};
use crate::model;
use crate::output::{self, TRACE_SCHEMA};

/// Body of the `genmsi.trace/1` document.
#[derive(Debug, Serialize)]
pub struct TraceReport<'a> {
    /// Program title.
    pub title: &'a str,
    /// Seed used.
    pub seed: u64,
    /// Start time.
    pub start_time: f64,
    /// End time.
    pub end_time: f64,
    /// Why the run ended (`terminated`, `no_events`, `time_limit`).
    pub end_reason: &'a str,
    /// Record limit this run used.
    pub limit: usize,
    /// Records actually kept.
    pub records: usize,
    /// Whether the limit may have cut the trace.
    pub truncated: bool,
    /// Trace records, in simulation order.
    pub events: &'a [TraceEvent],
}

/// Human-readable form of a trace.
pub fn render(events: &[TraceEvent], limit: usize, truncated: bool) -> String {
    let mut out = String::new();
    for event in events {
        let xact = format!("T{}", event.xact);
        let line = format!(
            "{:>9.3}  {xact:<5} #{:<4} {:<14} {:<10} {}",
            event.time,
            event.block + 1,
            event.class,
            event.event,
            event.detail
        );
        out.push_str(line.trim_end());
        out.push('\n');
    }
    if truncated {
        out.push_str(&format!(
            "# trace stopped at the {limit} record limit (raise --limit)\n"
        ));
    }
    out
}

/// Run `genmsi trace`.
pub fn run(args: TraceArgs) -> Result<ExitCode, CliError> {
    let loaded = model::load(&args.model.path)?;
    let mut config = model::config(args.run.seed, args.run.until);
    config.trace = true;
    config.trace_limit = args.limit;
    let result = Simulation::new(&loaded.program, config)
        .run()
        .map_err(|error| CliError::model(&args.model.path, error))?;
    let records = result.trace.len();
    let truncated = records >= args.limit;
    if args.output.json {
        let document = TraceReport {
            title: &result.title,
            seed: result.seed,
            start_time: result.start_time,
            end_time: result.end_time,
            end_reason: result.end_reason.as_str(),
            limit: args.limit,
            records,
            truncated,
            events: &result.trace,
        };
        output::print_json(TRACE_SCHEMA, &document)?;
    } else {
        output::print(&render(&result.trace, args.limit, truncated))?;
    }
    Ok(ExitCode::Success)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(time: f64, xact: u64, block: usize, detail: &str) -> TraceEvent {
        TraceEvent {
            time,
            xact,
            block,
            class: "Queue".to_string(),
            label: None,
            event: "enter",
            detail: detail.to_string(),
        }
    }

    #[test]
    fn lines_are_fixed_width_for_snapshots() {
        let text = render(&[event(0.0, 1, 1, "-> COLA"), event(1.5, 2, 1, "")], 10, false);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(
            lines,
            [
                "    0.000  T1    #2    Queue          enter      -> COLA",
                // An empty detail leaves no trailing spaces.
                "    1.500  T2    #2    Queue          enter",
            ]
        );
    }

    #[test]
    fn a_reached_limit_is_reported() {
        let text = render(&[event(0.0, 1, 0, "")], 1, true);
        assert!(text.contains("# trace stopped at the 1 record limit"), "{text}");
    }
}
