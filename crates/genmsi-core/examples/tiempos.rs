//! The deviations of our numbers against the original's, metric by metric.
//!
//! The comparison is the one of the oracle (the same tolerances: 20% for the
//! counts, 35% for the times and the occupancies, or the baseline inside the
//! 95% interval of our replications), but this example prints the *detail*:
//! every metric, its baseline, our mean and its relative deviation. It is
//! the measure of how far the clock of the run is from the clock of the
//! original (`odd/tasks/notas-reloj.md`).
//!
//! Usage:
//!   cargo run --release -p genmsi-core --example tiempos -- <dir de modelos> <dir de reportes> [--runs N]
//!
//! The directory of the reports holds one `<modelo>.report.txt` per model.

use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let runs = args
        .iter()
        .position(|arg| arg == "--runs")
        .and_then(|index| args.get(index + 1))
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(3);
    let mut positional: Vec<PathBuf> = Vec::new();
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--runs" {
            index += 2;
            continue;
        }
        positional.push(PathBuf::from(&args[index]));
        index += 1;
    }
    if positional.len() < 2 {
        eprintln!("uso: tiempos <directorio de modelos> <directorio de reportes> [--runs N]");
        std::process::exit(2);
    }
    let models_dir = &positional[0];
    let reports_dir = &positional[1];

    let mut models: Vec<PathBuf> = match std::fs::read_dir(models_dir) {
        Ok(entries) => entries
            .map(|entry| entry.expect("entry").path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "gmdl"))
            .collect(),
        Err(error) => {
            eprintln!("no se puede leer {}: {error}", models_dir.display());
            std::process::exit(2);
        }
    };
    models.sort();

    let mut total_metrics = 0usize;
    let mut total_matched = 0usize;
    let mut worst: Vec<(String, String, f64)> = Vec::new();

    for path in &models {
        let stem = path
            .file_stem()
            .expect("file stem")
            .to_string_lossy()
            .to_string();
        let report_path = reports_dir.join(format!("{stem}.report.txt"));
        let report_path = if report_path.exists() {
            report_path
        } else {
            reports_dir.join(format!("{}.report.txt", stem.replace(' ', "_")))
        };
        let Ok(baseline_text) = std::fs::read_to_string(&report_path) else {
            println!("{stem:<45} sin reporte del original");
            continue;
        };
        let Ok(model) = genmsi_format::load(path) else {
            println!("{stem:<45} el modelo no se puede abrir");
            continue;
        };
        let comparison = match genmsi_core::oracle::compare_model_text(
            &stem,
            &model,
            &baseline_text,
            runs,
        ) {
            Ok(comparison) => comparison,
            Err(error) => {
                println!("{stem:<45} error de comparación: {error}");
                continue;
            }
        };
        total_metrics += comparison.metrics.len();
        total_matched += comparison.matched();
        // The largest deviation among the metrics with the tolerance of the
        // times (35%): the one that says whether the clock of the run is
        // closer to the clock of the original.
        let worst_time = comparison
            .metrics
            .iter()
            .filter(|metric| metric.metric != "entries")
            .map(|metric| metric.relative_error)
            .filter(|error| error.is_finite())
            .fold(0.0f64, |worst, error| worst.max(error));
        println!(
            "{:<45} {:>6}/{} {:>5.0}%  mayor desvío de tiempos: {:>5.1}%",
            stem,
            comparison.matched(),
            comparison.metrics.len(),
            comparison.match_ratio() * 100.0,
            worst_time * 100.0
        );
        for metric in &comparison.metrics {
            if metric.matched {
                continue;
            }
            let error = if metric.relative_error.is_finite() {
                metric.relative_error * 100.0
            } else {
                f64::INFINITY
            };
            println!(
                "    NO  {:<8} {:<22} {:<18} base={:<12} nuestro={:<12} desv={:.1}%",
                metric.section,
                metric.entity,
                metric.metric,
                genmsi_core::report::plain(metric.baseline),
                genmsi_core::report::plain(metric.ours),
                error
            );
            worst.push((stem.clone(), metric.metric.clone(), error));
        }
    }

    println!();
    println!("métricas comparadas: {total_metrics}");
    println!("coinciden: {total_matched}");
    println!("no coinciden: {}", total_metrics - total_matched);
    worst.sort_by(|left, right| right.2.partial_cmp(&left.2).unwrap_or(std::cmp::Ordering::Equal));
    println!("las desviaciones más grandes:");
    for (stem, metric, error) in worst.iter().take(15) {
        println!("    {stem:<45} {metric:<18} {error:.1}%");
    }
}
