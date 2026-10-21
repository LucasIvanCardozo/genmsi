//! Replications and 95% confidence intervals.
//!
//! Each replication gets an independent seed derived from the base seed, so a
//! replication study is reproducible (D5) and the samples are independent as
//! the interval assumes.

use std::fmt::Write as _;

use serde::Serialize;

use crate::error::EngineError;
use crate::program::Program;
use crate::rng::mix_u64;
use crate::sim::{RunResult, SimConfig, Simulation};

/// Settings of a replication study.
#[derive(Debug, Clone, Copy)]
pub struct ReplicationConfig {
    /// Number of independent runs.
    pub runs: usize,
    /// Base seed; run `i` uses `mix(base_seed, i)`.
    pub base_seed: u64,
    /// Per-run configuration (trace is disabled for replications).
    pub run: SimConfig,
}

impl Default for ReplicationConfig {
    fn default() -> Self {
        Self {
            runs: 20,
            base_seed: 1,
            run: SimConfig::default(),
        }
    }
}

/// One metric with its replication summary.
#[derive(Debug, Clone, Serialize)]
pub struct MetricSummary {
    /// Section (`facility`, `queue`, `storage`, `block`, `run`).
    pub section: String,
    /// Entity name, or empty for run-level metrics.
    pub entity: String,
    /// Metric name.
    pub metric: String,
    /// Number of samples.
    pub samples: usize,
    /// Sample mean.
    pub mean: f64,
    /// Sample standard deviation (`n - 1`).
    pub stddev: f64,
    /// Lower bound of the 95% confidence interval of the mean.
    pub ci_low: f64,
    /// Upper bound of the 95% confidence interval of the mean.
    pub ci_high: f64,
}

impl MetricSummary {
    /// Half-width of the computed interval.
    pub fn half_width(&self) -> f64 {
        (self.ci_high - self.ci_low) / 2.0
    }

    /// Relative half-width against the mean (0 when the mean is zero).
    pub fn relative_half_width(&self) -> f64 {
        if self.mean.abs() < 1e-12 {
            0.0
        } else {
            self.half_width() / self.mean.abs()
        }
    }
}

/// Result of a replication study.
#[derive(Debug, Clone, Serialize)]
pub struct ReplicationResult {
    /// Base seed of the study.
    pub base_seed: u64,
    /// Number of runs completed.
    pub runs: usize,
    /// Individual run results.
    pub results: Vec<RunResult>,
    /// Summaries of the key metrics.
    pub metrics: Vec<MetricSummary>,
}

/// Two-sided 95% Student-t critical value.
///
/// Uses the exact table for small samples and the normal approximation above
/// 30 degrees of freedom, which is where the difference is below 0.3%.
pub fn t_critical(samples: usize) -> f64 {
    const TABLE: [f64; 30] = [
        12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179,
        2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064,
        2.060, 2.056, 2.052, 2.048, 2.045, 2.042,
    ];
    if samples < 2 {
        return f64::INFINITY;
    }
    let degrees = samples - 1;
    if degrees <= 30 {
        TABLE[degrees - 1]
    } else {
        1.96 + 2.4 / degrees as f64
    }
}

/// Run `config.runs` independent replications and summarize the key metrics.
pub fn replicate(
    program: &Program,
    config: ReplicationConfig,
) -> Result<ReplicationResult, EngineError> {
    if config.runs == 0 {
        return Err(EngineError::Runtime(
            "a replication study needs at least one run".to_string(),
        ));
    }
    let mut results = Vec::with_capacity(config.runs);
    for index in 0..config.runs {
        let seed = mix_u64(config.base_seed, index as u32);
        let mut run = config.run;
        run.seed = seed;
        run.trace = false;
        results.push(Simulation::new(program, run).run()?);
    }
    let metrics = summarize(&results);
    Ok(ReplicationResult {
        base_seed: config.base_seed,
        runs: results.len(),
        results,
        metrics,
    })
}

fn summarize(results: &[RunResult]) -> Vec<MetricSummary> {
    let mut samples: Vec<(String, String, String, Vec<f64>)> = Vec::new();
    let push = |samples: &mut Vec<(String, String, String, Vec<f64>)>,
                section: &str,
                entity: &str,
                metric: &str,
                value: f64| {
        let key = (
            section.to_string(),
            entity.to_string(),
            metric.to_string(),
        );
        match samples.iter_mut().find(|(s, e, m, _)| s == &key.0 && e == &key.1 && m == &key.2) {
            Some((_, _, _, values)) => values.push(value),
            None => samples.push((key.0, key.1, key.2, vec![value])),
        }
    };

    for result in results {
        let elapsed = result.elapsed().max(1e-9);
        push(
            &mut samples,
            "run",
            "",
            "end_time",
            result.end_time,
        );
        push(
            &mut samples,
            "run",
            "",
            "transactions",
            result.transactions as f64,
        );
        for facility in result.entities.facilities.values() {
            push(
                &mut samples,
                "facility",
                &facility.name,
                "utilization",
                facility.utilization(elapsed),
            );
            push(
                &mut samples,
                "facility",
                &facility.name,
                "average_time",
                facility.average_time(),
            );
        }
        for queue in result.entities.queues.values() {
            push(
                &mut samples,
                "queue",
                &queue.name,
                "average_content",
                queue.average_content(elapsed),
            );
            push(
                &mut samples,
                "queue",
                &queue.name,
                "average_time",
                queue.average_time(),
            );
            push(
                &mut samples,
                "queue",
                &queue.name,
                "entries",
                queue.entries as f64,
            );
        }
        for storage in result.entities.storages.values() {
            push(
                &mut samples,
                "storage",
                &storage.name,
                "utilization",
                storage.utilization(elapsed),
            );
            push(
                &mut samples,
                "storage",
                &storage.name,
                "average_content",
                storage.average_content(elapsed),
            );
        }
    }

    let mut summaries = Vec::with_capacity(samples.len());
    for (section, entity, metric, values) in samples {
        let n = values.len();
        let mean = values.iter().sum::<f64>() / n as f64;
        let variance = if n > 1 {
            values
                .iter()
                .map(|value| (value - mean).powi(2))
                .sum::<f64>()
                / (n - 1) as f64
        } else {
            0.0
        };
        let stddev = variance.sqrt();
        let half = t_critical(n) * stddev / (n as f64).sqrt();
        summaries.push(MetricSummary {
            section,
            entity,
            metric,
            samples: n,
            mean,
            stddev,
            ci_low: mean - half,
            ci_high: mean + half,
        });
    }
    summaries.sort_by(|left, right| {
        (&left.section, &left.entity, &left.metric).cmp(&(
            &right.section,
            &right.entity,
            &right.metric,
        ))
    });
    summaries
}

/// Render the summary as a small table.
pub fn render_summary(result: &ReplicationResult) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "Replications: {} runs, base seed {}",
        result.runs, result.base_seed
    );
    let _ = writeln!(
        out,
        "{:<9} {:<18} {:<16} {:>10} {:>10} {:>20} {:>8}",
        "SECTION", "ENTITY", "METRIC", "MEAN", "STDDEV", "IC 95%", "REL"
    );
    for metric in &result.metrics {
        let _ = writeln!(
            out,
            "{:<9} {:<18} {:<16} {:>10} {:>10} {:>20} {:>8}",
            metric.section,
            metric.entity,
            metric.metric,
            crate::report::number(metric.mean),
            crate::report::number(metric.stddev),
            format!(
                "[{}, {}]",
                crate::report::number(metric.ci_low),
                crate::report::number(metric.ci_high)
            ),
            crate::report::number(metric.relative_half_width()),
        );
    }
    out
}

/// CSV export of the summary (`section,entity,metric,samples,mean,...`).
pub fn summary_csv(result: &ReplicationResult) -> String {
    let mut out = String::from("section,entity,metric,samples,mean,stddev,ci_low,ci_high\n");
    for metric in &result.metrics {
        let _ = writeln!(
            out,
            "{},{},{},{},{},{},{},{}",
            metric.section,
            csv(&metric.entity),
            metric.metric,
            metric.samples,
            metric.mean,
            metric.stddev,
            metric.ci_low,
            metric.ci_high
        );
    }
    out
}

fn csv(value: &str) -> String {
    if value.contains(',') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_critical_matches_the_table() {
        assert_eq!(t_critical(2), 12.706);
        assert_eq!(t_critical(21), 2.086);
        assert!(t_critical(100) < 2.0);
    }

    #[test]
    fn intervals_shrink_with_more_samples() {
        let small = summarize_from(&[1.0, 2.0, 3.0]);
        let large = summarize_from(&(1..=40).map(|value| value as f64).collect::<Vec<_>>());
        assert!(small.relative_half_width() > large.relative_half_width());
    }

    fn summarize_from(values: &[f64]) -> MetricSummary {
        let n = values.len();
        let mean = values.iter().sum::<f64>() / n as f64;
        let variance = if n > 1 {
            values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1) as f64
        } else {
            0.0
        };
        let stddev = variance.sqrt();
        let half = t_critical(n) * stddev / (n as f64).sqrt();
        MetricSummary {
            section: "run".into(),
            entity: String::new(),
            metric: "x".into(),
            samples: n,
            mean,
            stddev,
            ci_low: mean - half,
            ci_high: mean + half,
        }
    }
}
