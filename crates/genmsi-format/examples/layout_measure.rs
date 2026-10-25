//! The measure of the composition over the reference corpus.
//!
//! Two pictures are measured for every model of `fixtures/models`:
//!
//! - **from scratch**: the model with its geometry taken away, composed the
//!   way an agent's model is composed (the blocks get the default size);
//! - **recomposed**: the model as its author left it, with the positions
//!   recomposed the way the app's "Reacomodar" button recomposes them (the
//!   blocks keep the size they had).
//!
//! ```text
//! cargo run -p genmsi-format --example layout_measure
//! cargo run -p genmsi-format --example layout_measure -- --write
//! ```
//!
//! `--write` refreshes the baseline the corpus test compares against
//! (`tests/layout-baseline.json`). The baseline is evidence: a change in the
//! algorithm shows in it as a change in the numbers.

use std::path::{Path, PathBuf};

use genmsi_format::{Composition, GmdlModel, measure, relayout};

/// The corpus: the reference models of the repository.
fn corpus() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("the reference models are there")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "gmdl"))
        .collect();
    paths.sort();
    paths
}

/// The model with its geometry taken away.
fn without_geometry(model: &GmdlModel) -> GmdlModel {
    let mut stripped = model.clone();
    for block in &mut stripped.blocks {
        block.layout = None;
    }
    stripped
}

/// The measure of one model, in both pictures.
struct Both {
    from_scratch: Composition,
    recomposed: Composition,
}

/// The measure of every model of the corpus.
fn measure_corpus() -> Vec<(String, Both)> {
    let mut rows = Vec::new();
    for path in corpus() {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(model) = genmsi_format::parse_str(&text) else {
            continue;
        };
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default();
        let bare = without_geometry(&model);
        rows.push((
            name,
            Both {
                from_scratch: measure(&bare, &relayout(&bare)),
                recomposed: measure(&model, &relayout(&model)),
            },
        ));
    }
    rows
}

/// The totals of a column of the table.
fn total(rows: &[(String, Both)], from_scratch: bool) -> Composition {
    let mut total = Composition::default();
    for (_, both) in rows {
        let one = if from_scratch {
            both.from_scratch
        } else {
            both.recomposed
        };
        total.blocks += one.blocks;
        total.edges += one.edges;
        total.overlaps += one.overlaps;
        total.outside += one.outside;
        total.crossings += one.crossings;
        total.isolated += one.isolated;
        total.over_blocks += one.over_blocks;
    }
    total
}

/// Print the table of the corpus.
fn print_table(rows: &[(String, Both)]) {
    println!(
        "{:<48} {:>7} {:>7} {:>7} {:>6} {:>7} {:>7} {:>12} {:>6}",
        "model", "bloques", "aristas", "solapes", "fuera", "cruces", "aislados", "lienzo", "ratio"
    );
    for (name, both) in rows {
        let one = both.from_scratch;
        println!(
            "{:<48} {:>7} {:>7} {:>7} {:>6} {:>7} {:>7} {:>12} {:>6.1}",
            name,
            one.blocks,
            one.edges,
            one.overlaps,
            one.outside,
            one.crossings,
            one.isolated,
            format!("{}x{}", one.width, one.height),
            one.ratio()
        );
    }
    let from_scratch = total(rows, true);
    let recomposed = total(rows, false);
    println!();
    println!("from scratch: {}", summary(&from_scratch));
    println!("recomposed:   {}", summary(&recomposed));
}

/// One line with the numbers of a measure.
fn summary(one: &Composition) -> String {
    format!(
        "bloques {} aristas {} solapes {} fuera {} cruces {} aislados {} por-encima {}",
        one.blocks, one.edges, one.overlaps, one.outside, one.crossings, one.isolated,
        one.over_blocks
    )
}

/// The baseline document.
fn baseline_json(rows: &[(String, Both)]) -> String {
    let models: Vec<serde_json::Value> = rows
        .iter()
        .map(|(name, both)| {
            serde_json::json!({
                "model": name,
                "from_scratch": value(&both.from_scratch),
                "recomposed": value(&both.recomposed),
            })
        })
        .collect();
    let document = serde_json::json!({
        "note": "The measure of the composition over the reference corpus. \
                 The corpus test compares the composition against this baseline.",
        "models": models,
        "totals": {
            "from_scratch": value(&total(rows, true)),
            "recomposed": value(&total(rows, false)),
        },
    });
    let mut text = serde_json::to_string_pretty(&document).expect("the measure serializes");
    text.push('\n');
    text
}

/// One measure as JSON.
fn value(one: &Composition) -> serde_json::Value {
    serde_json::json!({
        "blocks": one.blocks,
        "edges": one.edges,
        "overlaps": one.overlaps,
        "outside": one.outside,
        "crossings": one.crossings,
        "isolated": one.isolated,
        "over_blocks": one.over_blocks,
        "width": one.width,
        "height": one.height,
    })
}

fn main() {
    let rows = measure_corpus();
    print_table(&rows);
    if std::env::args().any(|argument| argument == "--write") {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout-baseline.json");
        std::fs::write(&path, baseline_json(&rows)).expect("the baseline is written");
        println!();
        println!("baseline written: {}", path.display());
    }
}
