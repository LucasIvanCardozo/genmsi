//! Compatibility harness against the original tool's report.
//!
//! The baselines under `fixtures/outputs/*.report.txt` are single sample runs of
//! the Delphi original (decision D6: semantic equivalence within statistical
//! tolerance — not bit-exactness). This module parses those reports and compares
//! them with replications of the Rust engine.
//!
//! Comparison rule (documented in `docs/oracle-comparison.md`): a metric matches
//! when its relative deviation from the baseline is below the metric's tolerance
//! **or** the baseline value lies inside the 95% confidence interval of our
//! replication mean. Counts use a 20% tolerance, time/occupancy metrics 35%.

use std::collections::BTreeMap;

use crate::error::EngineError;
use crate::program::Program;
use crate::replication::{ReplicationConfig, ReplicationResult, replicate};
use crate::sim::{EndReason, SimConfig};

/// One block row of a baseline report.
#[derive(Debug, Clone)]
pub struct BaselineBlock {
    /// Report line number.
    pub line: u32,
    /// Entry count.
    pub entries: u64,
    /// Current count.
    pub current: u64,
    /// Retry count.
    pub retry: u64,
    /// GPSS label (`LOC`).
    pub label: String,
    /// GPSS block type.
    pub kind: String,
}

/// One facility row of a baseline report.
#[derive(Debug, Clone)]
pub struct BaselineFacility {
    /// Entity name.
    pub name: String,
    /// Entries.
    pub entries: f64,
    /// Utilization.
    pub utilization: f64,
    /// Average time.
    pub average_time: f64,
}

/// One queue row of a baseline report.
#[derive(Debug, Clone)]
pub struct BaselineQueue {
    /// Entity name.
    pub name: String,
    /// Maximum content.
    pub max: f64,
    /// Entries.
    pub entries: f64,
    /// Entries that waited zero.
    pub zero_entries: f64,
    /// Average content.
    pub average_content: f64,
    /// Average time.
    pub average_time: f64,
}

/// One storage row of a baseline report.
#[derive(Debug, Clone)]
pub struct BaselineStorage {
    /// Entity name.
    pub name: String,
    /// Capacity.
    pub capacity: f64,
    /// Average content.
    pub average_content: f64,
    /// Utilization.
    pub utilization: f64,
    /// Entries.
    pub entries: f64,
}

/// Everything extracted from one baseline report.
#[derive(Debug, Clone, Default)]
pub struct BaselineReport {
    /// End time of the run.
    pub end_time: f64,
    /// Block table rows.
    pub blocks: Vec<BaselineBlock>,
    /// Facilities.
    pub facilities: Vec<BaselineFacility>,
    /// Queues.
    pub queues: Vec<BaselineQueue>,
    /// Storages.
    pub storages: Vec<BaselineStorage>,
}

/// Parse a report produced by the original `genmsi.exe`.
pub fn parse_baseline_report(text: &str) -> Result<BaselineReport, String> {
    let mut report = BaselineReport::default();
    let mut lines = text.lines().peekable();
    let mut found_header = false;
    while let Some(line) = lines.next() {
        if line.contains("START_TIME") && line.contains("END_TIME") {
            found_header = true;
            let values = lines.next().ok_or("missing header values")?;
            let tokens: Vec<&str> = values.split_whitespace().collect();
            report.end_time = tokens
                .get(1)
                .and_then(|token| parse_number(token))
                .ok_or("missing END_TIME")?;
            break;
        }
    }
    if !found_header {
        return Err("no START_TIME/END_TIME header (not a report)".to_string());
    }

    let mut section = Section::Blocks;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let upper = trimmed.to_uppercase();
        if upper.starts_with("FACILITY") && upper.contains("ENTRIES") {
            section = Section::Facilities;
            continue;
        }
        if upper.starts_with("QUEUE") && upper.contains("AVE.TIME") {
            section = Section::Queues;
            continue;
        }
        if upper.starts_with("STORAGE") && upper.contains("CAP.") {
            section = Section::Storages;
            continue;
        }
        if upper.starts_with("USER_CHAIN")
            || upper.starts_with("TABLAS")
            || upper.starts_with("QTABLES")
            || upper.starts_with("SAVEVALUE")
            || upper.starts_with("LOGICSWITCH")
            || upper.starts_with("CEC_")
            || upper.starts_with("FEC_")
            || upper.starts_with("MATRIX")
            || upper.starts_with("DIAGNOSTICOS")
        {
            section = Section::Other;
            continue;
        }
        if upper.contains("START_TIME") || upper.starts_with("COUNT") || upper.starts_with("LINE") {
            continue;
        }
        match section {
            Section::Blocks => {
                if let Some(block) = parse_block_row(trimmed) {
                    report.blocks.push(block);
                }
            }
            Section::Facilities => {
                if let Some(row) = parse_facility_row(trimmed) {
                    report.facilities.push(row);
                }
            }
            Section::Queues => {
                if let Some(row) = parse_queue_row(trimmed) {
                    report.queues.push(row);
                }
            }
            Section::Storages => {
                if let Some(row) = parse_storage_row(trimmed) {
                    report.storages.push(row);
                }
            }
            Section::Other => {}
        }
    }
    Ok(report)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Blocks,
    Facilities,
    Queues,
    Storages,
    Other,
}

fn parse_block_row(line: &str) -> Option<BaselineBlock> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.len() < 6 {
        return None;
    }
    let line_no = tokens[0].parse::<u32>().ok()?;
    let entries = tokens[1].parse::<u64>().ok()?;
    let current = tokens[2].parse::<u64>().ok()?;
    let retry = tokens[3].parse::<u64>().ok()?;
    Some(BaselineBlock {
        line: line_no,
        entries,
        current,
        retry,
        label: tokens[4].to_string(),
        kind: tokens[5].to_string(),
    })
}

fn parse_facility_row(line: &str) -> Option<BaselineFacility> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.len() < 4 {
        return None;
    }
    Some(BaselineFacility {
        name: tokens[0].to_uppercase(),
        entries: parse_number(tokens[1])?,
        utilization: parse_number(tokens[2])?,
        average_time: parse_number(tokens[3])?,
    })
}

fn parse_queue_row(line: &str) -> Option<BaselineQueue> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.len() < 8 {
        return None;
    }
    Some(BaselineQueue {
        name: tokens[0].to_uppercase(),
        max: parse_number(tokens[1])?,
        entries: parse_number(tokens[3])?,
        zero_entries: parse_number(tokens[4])?,
        average_content: parse_number(tokens[5])?,
        average_time: parse_number(tokens[6])?,
    })
}

fn parse_storage_row(line: &str) -> Option<BaselineStorage> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.len() < 8 {
        return None;
    }
    Some(BaselineStorage {
        name: tokens[0].to_uppercase(),
        capacity: parse_number(tokens[1])?,
        average_content: parse_number(tokens[7])?,
        utilization: parse_number(tokens[8])?,
        entries: parse_number(tokens[5])?,
    })
}

fn parse_number(token: &str) -> Option<f64> {
    token.replace(',', ".").parse::<f64>().ok()
}

/// One compared metric.
#[derive(Debug, Clone)]
pub struct MetricComparison {
    /// Entity section (`facility`, `queue`, `storage`, `block`).
    pub section: &'static str,
    /// Entity name (uppercase, as the baseline prints it).
    pub entity: String,
    /// Metric name.
    pub metric: String,
    /// Baseline value.
    pub baseline: f64,
    /// Our replication mean.
    pub ours: f64,
    /// Lower confidence bound of our mean.
    pub ci_low: f64,
    /// Upper confidence bound of our mean.
    pub ci_high: f64,
    /// Relative deviation from the baseline.
    pub relative_error: f64,
    /// Whether the metric matched under the documented rule.
    pub matched: bool,
}

/// Result of comparing one model.
#[derive(Debug, Clone)]
pub struct ModelComparison {
    /// Model file stem.
    pub model: String,
    /// Whether a baseline exists.
    pub has_baseline: bool,
    /// Whether the model compiled and ran.
    pub ran: bool,
    /// Failure message, when it did not run.
    pub error: Option<String>,
    /// Compared metrics.
    pub metrics: Vec<MetricComparison>,
    /// Baseline horizon used as the stop time.
    pub horizon: f64,
    /// Replications run.
    pub runs: usize,
}

impl ModelComparison {
    /// Metrics that matched.
    pub fn matched(&self) -> usize {
        self.metrics.iter().filter(|metric| metric.matched).count()
    }

    /// Ratio of matched metrics.
    pub fn match_ratio(&self) -> f64 {
        if self.metrics.is_empty() {
            0.0
        } else {
            self.matched() as f64 / self.metrics.len() as f64
        }
    }

    /// `match` (≥70%), `partial` (≥40%) or `gap`.
    pub fn status(&self) -> &'static str {
        if !self.ran {
            "error"
        } else if self.metrics.len() < 3 {
            "no-metrics"
        } else if self.match_ratio() >= 0.7 {
            "match"
        } else if self.match_ratio() >= 0.4 {
            "partial"
        } else {
            "gap"
        }
    }
}

/// Compare a compiled program against a parsed baseline.
pub fn compare_program(
    model: &str,
    program: &Program,
    baseline: &BaselineReport,
    runs: usize,
) -> ModelComparison {
    let config = ReplicationConfig {
        runs,
        base_seed: 7,
        run: SimConfig {
            stop_time: Some(baseline.end_time),
            ..SimConfig::default()
        },
    };
    let replication = match replicate(program, config) {
        Ok(replication) => replication,
        Err(err) => {
            return ModelComparison {
                model: model.to_string(),
                has_baseline: true,
                ran: false,
                error: Some(err.to_string()),
                metrics: Vec::new(),
                horizon: baseline.end_time,
                runs,
            };
        }
    };
    let labels: BTreeMap<String, usize> = program
        .blocks
        .iter()
        .enumerate()
        .filter_map(|(index, block)| block.label.clone().map(|label| (label, index)))
        .collect();
    let metrics = compare_metrics(baseline, &replication, Some(&labels));
    ModelComparison {
        model: model.to_string(),
        has_baseline: true,
        ran: true,
        error: None,
        metrics,
        horizon: baseline.end_time,
        runs,
    }
}

/// Compare a baseline report against replication summaries.
pub fn compare_metrics(
    baseline: &BaselineReport,
    replication: &ReplicationResult,
    labels: Option<&BTreeMap<String, usize>>,
) -> Vec<MetricComparison> {
    let mut index: BTreeMap<(String, String, String), &crate::replication::MetricSummary> =
        BTreeMap::new();
    for summary in &replication.metrics {
        index.insert(
            (
                summary.section.clone(),
                summary.entity.to_uppercase(),
                summary.metric.clone(),
            ),
            summary,
        );
    }
    let mut comparisons = Vec::new();
    let mut add = |section: &'static str,
                   entity: &str,
                   metric: &str,
                   baseline_value: f64,
                   tolerance: f64| {
        let key = (section.to_string(), entity.to_uppercase(), metric.to_string());
        let Some(summary) = index.get(&key) else {
            return;
        };
        let relative = if baseline_value.abs() < 1e-9 {
            if summary.mean.abs() < 1e-9 {
                0.0
            } else {
                f64::INFINITY
            }
        } else {
            (summary.mean - baseline_value).abs() / baseline_value.abs()
        };
        let inside_interval = baseline_value >= summary.ci_low && baseline_value <= summary.ci_high;
        comparisons.push(MetricComparison {
            section,
            entity: entity.to_uppercase(),
            metric: metric.to_string(),
            baseline: baseline_value,
            ours: summary.mean,
            ci_low: summary.ci_low,
            ci_high: summary.ci_high,
            relative_error: relative,
            matched: relative <= tolerance || inside_interval,
        });
    };

    for facility in &baseline.facilities {
        add("facility", &facility.name, "entries", facility.entries, 0.20);
        add(
            "facility",
            &facility.name,
            "utilization",
            facility.utilization,
            0.35,
        );
        add(
            "facility",
            &facility.name,
            "average_time",
            facility.average_time,
            0.35,
        );
    }
    for queue in &baseline.queues {
        add("queue", &queue.name, "entries", queue.entries, 0.20);
        add("queue", &queue.name, "max", queue.max, 0.35);
        add(
            "queue",
            &queue.name,
            "average_content",
            queue.average_content,
            0.35,
        );
        add("queue", &queue.name, "average_time", queue.average_time, 0.35);
    }
    for storage in &baseline.storages {
        add("storage", &storage.name, "entries", storage.entries, 0.20);
        add(
            "storage",
            &storage.name,
            "average_content",
            storage.average_content,
            0.35,
        );
        add(
            "storage",
            &storage.name,
            "utilization",
            storage.utilization,
            0.35,
        );
    }
    // Block entry counts, matched by GPSS label. Numeric labels are assigned by
    // each implementation in its own emission order, so only named labels are
    // comparable.
    if let Some(labels) = labels {
        for block in &baseline.blocks {
            if block.label.chars().all(|ch| ch.is_ascii_digit()) {
                continue;
            }
            let Some(index) = labels.get(&block.label) else {
                continue;
            };
            let samples: Vec<f64> = replication
                .results
                .iter()
                .map(|result| {
                    result
                        .blocks
                        .get(*index)
                        .map(|stats| stats.entries as f64)
                        .unwrap_or(0.0)
                })
                .collect();
            let n = samples.len();
            if n == 0 {
                continue;
            }
            let mean = samples.iter().sum::<f64>() / n as f64;
            let variance = if n > 1 {
                samples.iter().map(|value| (value - mean).powi(2)).sum::<f64>() / (n - 1) as f64
            } else {
                0.0
            };
            let half = crate::replication::t_critical(n) * variance.sqrt() / (n as f64).sqrt();
            let baseline_value = block.entries as f64;
            let relative = if baseline_value.abs() < 1e-9 {
                if mean.abs() < 1e-9 {
                    0.0
                } else {
                    f64::INFINITY
                }
            } else {
                (mean - baseline_value).abs() / baseline_value.abs()
            };
            let inside = baseline_value >= mean - half && baseline_value <= mean + half;
            comparisons.push(MetricComparison {
                section: "block",
                entity: block.label.clone(),
                metric: "entries".to_string(),
                baseline: baseline_value,
                ours: mean,
                ci_low: mean - half,
                ci_high: mean + half,
                relative_error: relative,
                matched: relative <= 0.20 || inside,
            });
        }
    }
    comparisons
}

/// Markdown table of all comparisons.
pub fn render_markdown(comparisons: &[ModelComparison], note: &str) -> String {
    let mut out = String::new();
    out.push_str("# Oracle comparison against the original GenMSI\n\n");
    out.push_str(note);
    out.push_str("\n\n## Summary\n\n");
    out.push_str("| Model | Horizon | Runs | Matched | Ratio | Status |\n");
    out.push_str("|---|---:|---:|---:|---:|---|\n");
    for comparison in comparisons {
        out.push_str(&format!(
            "| {} | {} | {} | {}/{} | {:.0}% | {} |\n",
            comparison.model,
            crate::report::number(comparison.horizon),
            comparison.runs,
            comparison.matched(),
            comparison.metrics.len(),
            comparison.match_ratio() * 100.0,
            comparison.status()
        ));
    }
    for comparison in comparisons {
        out.push_str(&format!("\n## {}\n\n", comparison.model));
        if let Some(error) = &comparison.error {
            out.push_str(&format!("Run failed: `{error}`\n"));
            continue;
        }
        out.push_str(&format!(
            "Horizon {} (baseline end time), {} replications, seed base 7.\n\n",
            crate::report::number(comparison.horizon),
            comparison.runs
        ));
        out.push_str("| Section | Entity | Metric | Baseline | Ours (mean) | 95% CI | Rel. error | Match |\n");
        out.push_str("|---|---|---|---:|---:|---|---:|---|\n");
        for metric in &comparison.metrics {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | [{}, {}] | {} | {} |\n",
                metric.section,
                metric.entity,
                metric.metric,
                crate::report::number(metric.baseline),
                crate::report::number(metric.ours),
                crate::report::number(metric.ci_low),
                crate::report::number(metric.ci_high),
                if metric.relative_error.is_finite() {
                    format!("{:.0}%", metric.relative_error * 100.0)
                } else {
                    "n/a".to_string()
                },
                if metric.matched { "yes" } else { "no" }
            ));
        }
    }
    out
}

/// Convenience: run a model file against its baseline report text.
pub fn compare_model_text(
    name: &str,
    model: &genmsi_format::GmdlModel,
    baseline_text: &str,
    runs: usize,
) -> Result<ModelComparison, EngineError> {
    let baseline = parse_baseline_report(baseline_text)
        .map_err(|err| EngineError::Runtime(format!("baseline {name}: {err}")))?;
    let program = crate::compile_model(model)?;
    Ok(compare_program(name, &program, &baseline, runs))
}

/// Whether a run ended for a reason other than the configured horizon.
pub fn ended_early(result: &crate::sim::RunResult) -> bool {
    matches!(result.end_reason, EndReason::NoEvents)
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPORT: &str = "                             Reporte MicroVisualSIS

 START_TIME    END_TIME     BLOCKS    FACILITIES    STORAGES
        0         3600        55           2            4

 LINE    ENTRY CURRENT RETRY            LOC  BLOCK_TYPE                                 
         COUNT   COUNT   
    6       60       1     0              1  GENERATE        60,30                      
    7       59       0     0        A_Queue  QUEUE           centro                     

FACILITY       ENTRIES  UTIL.  AVE_TIME  AVAILABLE  OWNER PEND INTER RETRY DELAY
  E_CARNET          7   0,298  153,286       1         0    0     0     0     0
  RECEPCION        59   0,392   23,932       1        62    0     0     0     0

QUEUE            MAX  CONT. ENTRIES ENTRIES(0) AVE.CONT. AVE.TIME AVE.(-0) RETRY
  CENTRO          33    30      59         0    22,285   1359,763 1359,763    0

STORAGE           CAP. REMAIN. MIN.  MAX. ENTRIES AVL. AVE.C.  UTIL. RETRY DELAY
  ACREDITACION      5      2     0     5      52   1   4,092   0,818    0     0
";

    #[test]
    fn parses_the_reference_report_format() {
        let report = parse_baseline_report(REPORT).expect("parses");
        assert_eq!(report.end_time, 3600.0);
        assert_eq!(report.blocks.len(), 2);
        assert_eq!(report.blocks[1].label, "A_Queue");
        assert_eq!(report.facilities.len(), 2);
        assert_eq!(report.facilities[0].name, "E_CARNET");
        assert!((report.facilities[0].utilization - 0.298).abs() < 1e-9);
        assert_eq!(report.queues.len(), 1);
        assert!((report.queues[0].average_time - 1359.763).abs() < 1e-9);
        assert_eq!(report.storages.len(), 1);
        assert!((report.storages[0].utilization - 0.818).abs() < 1e-9);
    }

    #[test]
    fn rejects_text_that_is_not_a_report() {
        assert!(parse_baseline_report("Error en linea 9 -> Bloque inválido").is_err());
    }
}
