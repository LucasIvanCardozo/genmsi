//! Measure the granularity of the clock: what the original counts, what we
//! count, and how the two can be told apart.
//!
//! The original (GenMSI 1.2020.07.28, the Delphi desktop simulator) keeps its
//! simulated time in whole units. Its pending events (`FEC_XACT_NUMBER`), its
//! interarrival times and its facility hold times are whole numbers. This
//! engine keeps the time in `f64`, so the same models produce fractions.
//!
//! The example measures the difference against the original's own reports:
//!
//! 1. the pending events of every bench (`FEC_XACT_NUMBER`): the original
//!    prints whole times, this engine's raw times carry fractions;
//! 2. the totals the reports imply: the busy time of a facility
//!    (`AVE_TIME x ENTRIES`) and the sum of the values of a table
//!    (`MEDIA x CANTIDAD`). A total that is a whole number of units is what
//!    tells an integer clock from a continuous one: a difference of two
//!    rounded times is a whole number either way, but a sum of many of them
//!    is not;
//! 3. the interarrival times of a probe with a fast generator behind a long
//!    `ADVANCE` (the pending events of the original give the arrival times);
//! 4. the first arrival of a generator with dispersion (the `MARK` block of
//!    the original writes the clock into a parameter and a `TABLE` collects
//!    the readings), with a Kolmogorov-Smirnov test against the uniform the
//!    engine draws its intervals from.
//!
//! Usage:
//!
//! ```text
//! cargo run --release -p genmsi-core --example reloj -- [options]
//!
//!   --oracle <dir>     the original's reports of the benches (default /tmp/all-oracle)
//!   --fixtures <dir>   the original's reports of the reference models (default fixtures/outputs)
//!   --probes <dir>     the probe models and their reports (default /tmp/reloj-probes)
//!   --benches <dir>    the bench models (default scripts/wine-oracle/benches)
//!   --seed <n>         the seed of our runs (default 7)
//!   --seeds <n>        how many seeds the first-arrival sample uses (default 10)
//! ```
//!
//! The probe models are the ones this work unit built (the notes of the unit
//! carry the script that writes them); the original's reports of the probes
//! come from `scripts/wine-oracle/run_oracle.sh`.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use genmsi_core::{SimConfig, simulate};

// ---------------------------------------------------------------------------
// The text of the original's report.
// ---------------------------------------------------------------------------

/// A number the original prints with a comma as the decimal separator.
fn original_number(text: &str) -> Option<f64> {
    text.replace(',', ".").parse::<f64>().ok()
}

/// One row of the `FEC_XACT_NUMBER` section of a report.
struct PendingRow {
    /// The time the event is scheduled for (`BDT`).
    time: f64,
    /// The block the event belongs to (`CURRENT`).
    block: u64,
}

/// The events a report still has pending.
fn pending_rows(report: &str) -> Vec<PendingRow> {
    let mut rows = Vec::new();
    let mut inside = false;
    for line in report.lines() {
        if line.starts_with("FEC_XACT_NUMBER") {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 4 {
            continue;
        }
        let (Ok(_), Some(time), Ok(block)) = (
            fields[0].parse::<u64>(),
            original_number(fields[2]),
            fields[3].parse::<u64>(),
        ) else {
            continue;
        };
        rows.push(PendingRow { time, block });
    }
    rows
}

/// The numbers of the blocks of a report: the line of the report and the
/// class of the block.
///
/// The original numbers the blocks of a model from one, and the
/// `FEC_XACT_NUMBER` section names the block of every pending event with
/// that number.
fn block_lines(report: &str) -> Vec<(u64, String)> {
    let mut lines = Vec::new();
    for line in report.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 6 {
            continue;
        }
        let (Ok(number), class) = (fields[0].parse::<u64>(), fields[5].to_string()) else {
            continue;
        };
        if !class.chars().next().is_some_and(|first| first.is_ascii_alphabetic()) {
            continue;
        }
        lines.push((number, class));
    }
    lines
}

/// The number of the block of a class in a report.
fn block_of(report: &str, class: &str) -> Option<u64> {
    block_lines(report)
        .into_iter()
        .find(|(_, name)| name == class)
        .map(|(number, _)| number)
}

/// One row of the `FACILITY` section of a report.
struct FacilityRow {
    name: String,
    entries: u64,
    utilization: f64,
    average_time: f64,
}

/// The facilities of a report.
fn facility_rows(report: &str) -> Vec<FacilityRow> {
    let mut rows = Vec::new();
    let mut inside = false;
    for line in report.lines() {
        if line.starts_with("FACILITY") {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        if line.trim().is_empty() {
            break;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 4 {
            continue;
        }
        rows.push(FacilityRow {
            name: fields[0].to_string(),
            entries: fields[1].parse().unwrap_or(0),
            utilization: original_number(fields[2]).unwrap_or(0.0),
            average_time: original_number(fields[3]).unwrap_or(0.0),
        });
    }
    rows
}

/// The statistics of the first table of a report.
struct TableStats {
    mean: f64,
    stddev: f64,
    count: u64,
    /// The tabulated values, read back from the histogram.
    values: Vec<f64>,
}

/// The table of a report: its mean, its deviation and the values its
/// histogram holds.
///
/// The histogram has one row per class and the count of the row is the number
/// of values that fell in it. The classes of a table of width one are the
/// values themselves, so the sample can be read back exactly.
fn table_stats(report: &str) -> Option<TableStats> {
    let section = report.split("TABLAS").nth(1)?;
    let mut mean = None;
    let mut stddev = None;
    let mut count = None;
    let mut values = Vec::new();
    let mut inside = false;
    for line in section.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("INTERVALO") {
            inside = true;
            continue;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            let value = value.trim();
            match key.trim() {
                "MEDIA" => mean = original_number(value),
                "DESVIACION ESTANDAR" => stddev = original_number(value),
                "CANTIDAD DE VALORES TABULADOS" => count = value.parse().ok(),
                _ => {}
            }
            continue;
        }
        if !inside {
            continue;
        }
        let fields: Vec<&str> = trimmed.split_whitespace().collect();
        if fields.len() < 4 || fields[1] != "-" {
            continue;
        }
        let (Ok(low), Ok(high), Ok(frequency)) = (
            fields[0].parse::<f64>(),
            fields[2].parse::<f64>(),
            fields[3].parse::<usize>(),
        ) else {
            continue;
        };
        let value = if (low - high).abs() < f64::EPSILON {
            low
        } else {
            (low + high) / 2.0
        };
        values.extend(std::iter::repeat_n(value, frequency));
    }
    Some(TableStats {
        mean: mean?,
        stddev: stddev?,
        count: count?,
        values,
    })
}

// ---------------------------------------------------------------------------
// The statistics.
// ---------------------------------------------------------------------------

/// Whether a time is a whole number of units.
fn is_whole(value: f64) -> bool {
    (value - value.round()).abs() < 1e-9
}

/// Whether a total is a whole number of units, to the precision the report
/// prints it with.
///
/// The report prints an average with three decimals, so the total it implies
/// is only known to within half a unit of the last decimal of the average,
/// times the number of samples.
fn whole_within(total: f64, samples: f64) -> bool {
    (total - total.round()).abs() <= 0.0005 * samples + 1e-9
}

/// The Kolmogorov-Smirnov statistic of a sample against a uniform, and its
/// asymptotic p-value (the Stephens correction).
fn ks_uniform(sample: &[f64], low: f64, high: f64) -> (f64, f64) {
    if sample.is_empty() || high <= low {
        return (0.0, 1.0);
    }
    let mut sorted = sample.to_vec();
    sorted.sort_by(f64::total_cmp);
    let n = sorted.len() as f64;
    let uniform = |value: f64| ((value - low) / (high - low)).clamp(0.0, 1.0);
    let mut statistic = 0.0_f64;
    for (index, value) in sorted.iter().enumerate() {
        let below = index as f64 / n;
        let up_to = (index + 1) as f64 / n;
        let expected = uniform(*value);
        statistic = statistic
            .max((below - expected).abs())
            .max((up_to - expected).abs());
    }
    let root = n.sqrt();
    let lambda = (root + 0.12 + 0.11 / root) * statistic;
    let mut p = 0.0;
    for k in 1..=100 {
        let term = (-2.0 * (k as f64).powi(2) * lambda * lambda).exp();
        p += if k % 2 == 1 { term } else { -term };
    }
    (statistic, (2.0 * p).clamp(0.0, 1.0))
}

/// The mean and the deviation of a sample.
fn mean_and_stddev(sample: &[f64]) -> (f64, f64) {
    if sample.is_empty() {
        return (0.0, 0.0);
    }
    let mean = sample.iter().sum::<f64>() / sample.len() as f64;
    if sample.len() < 2 {
        return (mean, 0.0);
    }
    let variance = sample.iter().map(|value| (value - mean).powi(2)).sum::<f64>()
        / (sample.len() - 1) as f64;
    (mean, variance.sqrt())
}

/// The distinct values of a sample, sorted.
fn distinct(sample: &[f64]) -> Vec<f64> {
    let mut values = sample.to_vec();
    values.sort_by(f64::total_cmp);
    values.dedup_by(|left, right| (*left - *right).abs() < 1e-9);
    values
}

/// The differences between the consecutive values of a sorted sample.
fn differences(sorted: &[f64]) -> Vec<f64> {
    sorted.windows(2).map(|pair| pair[1] - pair[0]).collect()
}

/// The lowest and the highest value of a sample.
fn range(sample: &[f64]) -> (f64, f64) {
    let low = sample.iter().copied().fold(f64::INFINITY, f64::min);
    let high = sample.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    (low, high)
}

/// A number the way the original writes it.
fn format_number(value: f64) -> String {
    format!("{value:.3}").replace('.', ",")
}

// ---------------------------------------------------------------------------
// The runs of this engine.
// ---------------------------------------------------------------------------

/// The result of one run of a model.
struct Run {
    /// The block and the raw time of every event that is still pending.
    pending: Vec<(usize, f64)>,
    /// The report of the run, in the format of the original's.
    report: String,
    /// The block of the `ADVANCE` of the model, when it has one.
    advance: Option<usize>,
}

/// Run a model of this engine.
fn run_model(path: &Path, seed: u64) -> Result<Run, String> {
    let model = genmsi_format::load(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let (program, result) = simulate(
        &model,
        SimConfig {
            seed,
            stop_time: Some(3600.0),
            trace: true,
            trace_limit: 200_000,
            ..SimConfig::default()
        },
    )
    .map_err(|error| format!("{}: {error}", path.display()))?;
    let pending = result
        .pending
        .iter()
        .map(|event| (event.block, event.time))
        .collect();
    let advance = program
        .blocks
        .iter()
        .position(|block| block.class == "Advance");
    let report = genmsi_core::render(&result, &program);
    Ok(Run {
        pending,
        report,
        advance,
    })
}

// ---------------------------------------------------------------------------
// The options and the report.
// ---------------------------------------------------------------------------

/// What a report says about the totals of its times.
#[derive(Default)]
struct Totals {
    /// The facilities with uses.
    facilities: usize,
    /// The facilities whose busy time is a whole number of units.
    whole_facilities: usize,
    /// The tables with values.
    tables: usize,
    /// The tables whose values sum to a whole number of units.
    whole_tables: usize,
}

impl Totals {
    /// Add the totals of another report.
    fn add(&mut self, other: &Totals) {
        self.facilities += other.facilities;
        self.whole_facilities += other.whole_facilities;
        self.tables += other.tables;
        self.whole_tables += other.whole_tables;
    }
}

/// The totals a report implies: the busy time of every facility and the sum
/// of the values of every table.
fn totals_of(report: &str) -> Totals {
    let mut totals = Totals::default();
    for row in facility_rows(report) {
        if row.entries == 0 {
            continue;
        }
        let total = row.average_time * row.entries as f64;
        if total <= 0.0 {
            // A facility nobody held is not a measurement.
            continue;
        }
        totals.facilities += 1;
        if whole_within(total, row.entries as f64) {
            totals.whole_facilities += 1;
        }
    }
    if let Some(table) = table_stats(report)
        && table.count > 0
    {
        totals.tables += 1;
        let total = table.mean * table.count as f64;
        if whole_within(total, table.count as f64) {
            totals.whole_tables += 1;
        }
    }
    totals
}

/// The options of the example.
struct Options {
    oracle: PathBuf,
    fixtures: PathBuf,
    models: PathBuf,
    probes: PathBuf,
    benches: PathBuf,
    seed: u64,
    seeds: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            oracle: PathBuf::from("/tmp/all-oracle"),
            fixtures: PathBuf::from("fixtures/outputs"),
            models: PathBuf::from("fixtures/models"),
            probes: PathBuf::from("/tmp/reloj-probes"),
            benches: PathBuf::from("scripts/wine-oracle/benches"),
            seed: 7,
            seeds: 10,
        }
    }
}

fn parse_options() -> Options {
    let mut options = Options::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().unwrap_or_default();
        match arg.as_str() {
            "--oracle" => options.oracle = PathBuf::from(value()),
            "--fixtures" => options.fixtures = PathBuf::from(value()),
            "--models" => options.models = PathBuf::from(value()),
            "--probes" => options.probes = PathBuf::from(value()),
            "--benches" => options.benches = PathBuf::from(value()),
            "--seed" => options.seed = value().parse().unwrap_or(options.seed),
            "--seeds" => options.seeds = value().parse().unwrap_or(options.seeds),
            other => eprintln!("unknown option: {other}"),
        }
    }
    options
}

/// The reports of a directory, sorted by name.
fn reports_of(directory: &Path) -> Vec<(String, String)> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut reports: Vec<(String, String)> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".report.txt"))
        })
        .filter_map(|path| {
            let name = path.file_stem()?.to_string_lossy().to_string();
            let text = std::fs::read_to_string(&path).ok()?;
            Some((name, text))
        })
        .collect();
    reports.sort_by(|left, right| left.0.cmp(&right.0));
    reports
}

/// The benches of a directory, paired with the reports of the original.
fn bench_pairs(benches: &Path, oracle: &Path) -> Vec<(String, String, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(benches) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "gmdl"))
        .filter_map(|path| {
            path.file_stem()
                .map(|stem| stem.to_string_lossy().to_string())
        })
        .collect();
    names.sort();
    names
        .into_iter()
        .filter_map(|name| {
            let path = benches.join(format!("{name}.gmdl"));
            let report = std::fs::read_to_string(oracle.join(format!("{name}.report.txt"))).ok()?;
            Some((name, report, path))
        })
        .collect()
}

/// The header of a section.
fn section(out: &mut String, title: &str) {
    let _ = writeln!(out, "\n{title}");
    let _ = writeln!(out, "{}", "-".repeat(title.len()));
}

/// Print a share of whole numbers.
fn fraction(out: &mut String, label: &str, whole: usize, total: usize) {
    let share = if total == 0 {
        0.0
    } else {
        whole as f64 * 100.0 / total as f64
    };
    let _ = writeln!(
        out,
        "   {label:<34} {whole:>6} of {total:>6} ({share:>6.2} %)"
    );
}

fn main() {
    let options = parse_options();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "================================================================================"
    );
    let _ = writeln!(
        out,
        " The granularity of the clock: the original against this engine"
    );
    let _ = writeln!(
        out,
        "================================================================================"
    );
    let _ = writeln!(out, " the original's reports of the benches: {}", options.oracle.display());
    let _ = writeln!(
        out,
        "   the original's reports of the reference models: {}",
        options.fixtures.display()
    );
    let _ = writeln!(out, " our reference models:             {}", options.models.display());
    let _ = writeln!(out, " the probes: {}", options.probes.display());
    let _ = writeln!(out, " the benches: {}", options.benches.display());
    let _ = writeln!(out, " our seed: {}", options.seed);

    // -----------------------------------------------------------------------
    // 1. The pending events.
    // -----------------------------------------------------------------------
    section(&mut out, "1. THE PENDING EVENTS (FEC_XACT_NUMBER)");
    let pairs = bench_pairs(&options.benches, &options.oracle);
    let mut original_events = 0usize;
    let mut original_whole = 0usize;
    let mut our_events = 0usize;
    let mut our_whole = 0usize;
    let mut examples: Vec<String> = Vec::new();
    for (name, report, path) in &pairs {
        let times: Vec<f64> = pending_rows(report).iter().map(|row| row.time).collect();
        original_events += times.len();
        original_whole += times.iter().filter(|time| is_whole(**time)).count();
        let Ok(run) = run_model(path, options.seed) else {
            continue;
        };
        our_events += run.pending.len();
        our_whole += run
            .pending
            .iter()
            .filter(|(_, time)| is_whole(*time))
            .count();
        if examples.len() < 4 {
            let original: Vec<String> = times.iter().map(|time| format!("{time:.0}")).collect();
            let ours: Vec<String> = run
                .pending
                .iter()
                .map(|(_, time)| format!("{time:.3}"))
                .collect();
            examples.push(format!(
                "   {name}\n      original: {}\n      ours:     {}",
                original.join(" "),
                ours.join(" ")
            ));
        }
    }
    if pairs.is_empty() {
        let _ = writeln!(
            out,
            "   (no bench was compared: the original's reports are not in {} and the",
            options.oracle.display()
        );
        let _ = writeln!(
            out,
            "    benches are not in {}; run scripts/wine-oracle/make-benches.py and",
            options.benches.display()
        );
        let _ = writeln!(out, "    scripts/wine-oracle/run_oracle.sh to produce them)");
    } else {
        let _ = writeln!(
            out,
            "   {} models compared (the original's report and our run of the same bench)",
            pairs.len()
        );
        fraction(&mut out, "original: whole times", original_whole, original_events);
        fraction(&mut out, "ours: whole times", our_whole, our_events);
        for example in examples {
            let _ = writeln!(out, "{example}");
        }
    }

    // -----------------------------------------------------------------------
    // 2. The totals the reports imply.
    // -----------------------------------------------------------------------
    section(&mut out, "2. THE TOTALS THE REPORTS IMPLY");
    let _ = writeln!(
        out,
        "   a difference of two rounded times is a whole number either way, so the"
    );
    let _ = writeln!(
        out,
        "   totals are what tells an integer clock from a continuous one:"
    );
    let mut reports = reports_of(&options.oracle);
    reports.extend(reports_of(&options.fixtures));
    let mut original = Totals::default();
    let mut ours = Totals::default();
    let mut facility_examples: Vec<String> = Vec::new();
    for (name, report) in &reports {
        original.add(&totals_of(report));
        for row in facility_rows(report) {
            if row.entries == 0 {
                continue;
            }
            let total = row.average_time * row.entries as f64;
            if total <= 0.0 {
                continue;
            }
            if facility_examples.len() < 4 {
                facility_examples.push(format!(
                    "   {:<22} {} uses, AVE_TIME {}, UTIL. {} -> the total busy time is {total:.3}",
                    format!("{}:{}", name, row.name),
                    row.entries,
                    format_number(row.average_time),
                    format_number(row.utilization),
                ));
            }
        }
    }
    for (_, _, path) in &pairs {
        if let Ok(run) = run_model(path, options.seed) {
            ours.add(&totals_of(&run.report));
        }
    }
    // The reference models of the repository, whose reports the original wrote
    // as well.
    if let Ok(entries) = std::fs::read_dir(&options.models) {
        let mut paths: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "gmdl"))
            .collect();
        paths.sort();
        for path in paths {
            let Ok(run) = run_model(&path, options.seed) else {
                continue;
            };
            ours.add(&totals_of(&run.report));
        }
    }
    if original.facilities == 0 {
        let _ = writeln!(out, "   (no facility was measured)");
    } else {
        let _ = writeln!(
            out,
            "   the busy time of a facility: AVE_TIME x ENTRIES, in {} reports",
            reports.len()
        );
        fraction(
            &mut out,
            "original: whole busy times",
            original.whole_facilities,
            original.facilities,
        );
        fraction(
            &mut out,
            "ours: whole busy times",
            ours.whole_facilities,
            ours.facilities,
        );
        for example in facility_examples {
            let _ = writeln!(out, "{example}");
        }
    }
    if original.tables > 0 {
        let _ = writeln!(
            out,
            "   the sum of the values of a table: MEDIA x CANTIDAD, in {} tables",
            original.tables
        );
        let _ = writeln!(
            out,
            "   (a table whose values are counts sums to a whole number in both"
        );
        let _ = writeln!(
            out,
            "    engines; the tables that carry times are the ones that tell)"
        );
        fraction(
            &mut out,
            "original: whole sums",
            original.whole_tables,
            original.tables,
        );
        fraction(&mut out, "ours: whole sums", ours.whole_tables, ours.tables);
    }
    // -----------------------------------------------------------------------
    // 3. The interarrival times.
    // -----------------------------------------------------------------------
    section(&mut out, "3. THE INTERARRIVAL TIMES (probe relojEnteros)");
    let _ = writeln!(
        out,
        "   the probe is a GENERATE 5,2 behind an ADVANCE 1000: the pending events"
    );
    let _ = writeln!(
        out,
        "   of the original give the arrival times (the BDT minus the advance)"
    );
    let _ = writeln!(
        out,
        "   (the original prints its times without decimals, so this comparison sees"
    );
    let _ = writeln!(
        out,
        "    the differences of the printed times; the totals of section 2 are the"
    );
    let _ = writeln!(out, "    ones that see the clock itself)");
    let probe_model = options.probes.join("relojEnteros.gmdl");
    let probe_report = options.probes.join("oracle/relojEnteros.report.txt");
    match (
        std::fs::read_to_string(&probe_report),
        run_model(&probe_model, options.seed),
    ) {
        (Ok(report), Ok(run)) => {
            let advance = block_of(&report, "ADVANCE");
            let mut arrivals: Vec<f64> = pending_rows(&report)
                .iter()
                .filter(|row| advance.is_none_or(|block| row.block == block))
                .filter(|row| row.time > 1000.0)
                .map(|row| row.time - 1000.0)
                .collect();
            arrivals.sort_by(f64::total_cmp);
            let original_differences = differences(&arrivals);
            let whole = original_differences
                .iter()
                .filter(|value| is_whole(**value))
                .count();
            let (low, high) = range(&original_differences);
            let _ = writeln!(
                out,
                "   original: {} differences between the arrival times",
                original_differences.len()
            );
            fraction(
                &mut out,
                "original: whole differences",
                whole,
                original_differences.len(),
            );
            let _ = writeln!(
                out,
                "   original: the differences go from {low:.3} to {high:.3} (a generator of 5 with a dispersion of 2 draws 3..7)"
            );

            let mut our_arrivals: Vec<f64> = match run.advance {
                Some(block) => run
                    .pending
                    .iter()
                    .filter(|(number, _)| *number == block)
                    .map(|(_, time)| *time - 1000.0)
                    .collect(),
                None => Vec::new(),
            };
            our_arrivals.sort_by(f64::total_cmp);
            let our_differences = differences(&our_arrivals);
            let our_whole = our_differences
                .iter()
                .filter(|value| is_whole(**value))
                .count();
            let (our_low, our_high) = range(&our_differences);
            let _ = writeln!(
                out,
                "   ours:     {} pending events of the advance",
                our_arrivals.len()
            );
            fraction(
                &mut out,
                "ours: whole differences",
                our_whole,
                our_differences.len(),
            );
            let _ = writeln!(
                out,
                "   ours:     the differences go from {our_low:.3} to {our_high:.3}"
            );
        }
        _ => {
            let _ = writeln!(
                out,
                "   (the probe did not run: {} and {} are needed)",
                probe_model.display(),
                probe_report.display()
            );
        }
    }

    // -----------------------------------------------------------------------
    // 4. The first arrival with dispersion.
    // -----------------------------------------------------------------------
    section(&mut out, "4. THE FIRST ARRIVAL WITH DISPERSION (probe primeraLlegada)");
    let _ = writeln!(
        out,
        "   the probe holds 60 generators with one transaction each, a dispersion of 30"
    );
    let _ = writeln!(
        out,
        "   and a MARK that writes the clock into a parameter a TABLE collects"
    );
    let probe_model = options.probes.join("primeraLlegada.gmdl");
    let probe_report = options.probes.join("oracle/primeraLlegada.report.txt");
    let original_table = std::fs::read_to_string(&probe_report)
        .ok()
        .and_then(|report| table_stats(&report));
    if let Some(table) = &original_table {
        let (mean, stddev) = mean_and_stddev(&table.values);
        let (statistic, p) = ks_uniform(&table.values, 30.0, 90.0);
        let _ = writeln!(
            out,
            "   original: {} first arrivals, mean {mean:.3}, deviation {stddev:.3}, {} distinct values",
            table.values.len(),
            distinct(&table.values).len()
        );
        let _ = writeln!(
            out,
            "   original: the readings sum to {:.3} (a whole number of units: the clock reads whole units)",
            table.values.iter().sum::<f64>()
        );
        let _ = writeln!(
            out,
            "   original: the report says MEDIA {}, DESVIACION {} and {} values",
            format_number(table.mean),
            format_number(table.stddev),
            table.count
        );
        let _ = writeln!(
            out,
            "   original: KS against U(30,90): D = {statistic:.4}, p = {p:.4} (not rejected)"
        );
        let _ = writeln!(
            out,
            "   original: the reading 'the exact mean' would give 60 copies of 60 (deviation 0)"
        );
    } else {
        let _ = writeln!(
            out,
            "   (the original's report of the probe is not in {})",
            probe_report.display()
        );
    }
    let mut our_first: Vec<f64> = Vec::new();
    for seed in 0..options.seeds.max(1) as u64 {
        let Ok(run) = run_model(&probe_model, options.seed + seed) else {
            continue;
        };
        let Some(table) = table_stats(&run.report) else {
            continue;
        };
        our_first.extend(table.values);
    }
    if our_first.is_empty() {
        let _ = writeln!(
            out,
            "   (our run of the probe did not produce the table: {} is needed)",
            probe_model.display()
        );
    } else {
        let (mean, stddev) = mean_and_stddev(&our_first);
        let (statistic, p) = ks_uniform(&our_first, 30.0, 90.0);
        let _ = writeln!(
            out,
            "   ours:     {} first arrivals over {} seeds, mean {mean:.3}, deviation {stddev:.3}, {} distinct values",
            our_first.len(),
            options.seeds,
            distinct(&our_first).len()
        );
        let _ = writeln!(
            out,
            "   ours:     KS against U(30,90): D = {statistic:.4}, p = {p:.4}"
        );
    }

    print!("{out}");
}
