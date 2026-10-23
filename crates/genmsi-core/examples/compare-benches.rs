//! Compare our numbers with the original's, class by class.
//!
//! A bench is a model that exercises one class of the original and nothing
//! else. The original's own report for it is the specification of what the
//! class does; this example runs our engine on the same bench and says
//! whether the numbers agree, with the same rules the reference models use
//! (20% for counts, 35% for times and occupancies, or the baseline inside the
//! 95% interval of our replications).
//!
//! Usage:
//!   cargo run --release -p genmsi-core --example compare-benches -- <benches dir> <results dir> [--runs N]
//!
//! The results directory holds the original's report for every bench, named
//! `<Class>.report.txt`; `scripts/wine-oracle/run-benches.sh` writes them.

use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let runs = args
        .iter()
        .position(|arg| arg == "--runs")
        .and_then(|index| args.get(index + 1))
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(5);

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
        eprintln!(
            "uso: compare-benches <directorio de bancos> <directorio de resultados> [--runs N]"
        );
        std::process::exit(2);
    }
    let benches_dir = positional[0].clone();
    let results_dir = positional[1].clone();

    let mut benches: Vec<PathBuf> = match std::fs::read_dir(&benches_dir) {
        Ok(entries) => entries
            .map(|entry| entry.expect("entry").path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "gmdl"))
            .collect(),
        Err(error) => {
            eprintln!("no se puede leer {}: {error}", benches_dir.display());
            std::process::exit(2);
        }
    };
    benches.sort();

    let mut compared = 0usize;
    let mut agreeing = 0usize;
    let mut missing = Vec::new();
    let mut without = Vec::new();

    for path in &benches {
        let stem = path
            .file_stem()
            .expect("file stem")
            .to_string_lossy()
            .to_string();
        let baseline_path = results_dir.join(format!("{stem}.report.txt"));
        let baseline_text = match std::fs::read_to_string(&baseline_path) {
            Ok(text) => text,
            Err(_) => {
                missing.push(stem);
                continue;
            }
        };
        if baseline_text.trim().is_empty() {
            without.push(stem);
            continue;
        }

        let model = match genmsi_format::load(path) {
            Ok(model) => model,
            Err(error) => {
                println!("{stem:<28} MODEL ERROR {error}");
                without.push(stem);
                continue;
            }
        };
        match genmsi_core::oracle::compare_model_text(&stem, &model, &baseline_text, runs) {
            Ok(comparison) => {
                compared += 1;
                if comparison.matched() == comparison.metrics.len() {
                    agreeing += 1;
                }
                println!(
                    "{:<28} {:>7} {:>4}/{:>3} {:>5.0}%  {}",
                    stem,
                    genmsi_core::report::number(comparison.horizon),
                    comparison.matched(),
                    comparison.metrics.len(),
                    comparison.match_ratio() * 100.0,
                    comparison.status()
                );
            }
            Err(error) => {
                println!("{stem:<28} COMPARE ERROR {error}");
                without.push(stem);
            }
        }
    }

    println!();
    println!("bancos con reporte del original: {compared}");
    println!("coinciden en todas las métricas: {agreeing}");
    if !without.is_empty() {
        println!("sin comparación posible: {}", without.len());
        for name in &without {
            println!("  {name}");
        }
    }
    if !missing.is_empty() {
        println!("sin reporte del original: {}", missing.len());
        for name in &missing {
            println!("  {name}");
        }
    }
}
