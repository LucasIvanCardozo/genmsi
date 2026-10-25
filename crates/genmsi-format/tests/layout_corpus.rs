//! The corpus of the composition.
//!
//! The composition is measured over every reference model of the repository,
//! in the two pictures that matter: a model with no geometry (the one an
//! agent builds) and a model whose author left a geometry (the one the
//! "Reacomodar" button recomposes).
//!
//! Three things are checked:
//!
//! 1. the measure is the one the baseline says (`tests/layout-baseline.json`):
//!    a change to the algorithm shows in the baseline as a change in the
//!    numbers, and the baseline is refreshed on purpose, never by accident;
//! 2. every block stays inside the container that owns it: a block out of
//!    its frame is a broken composition, in every picture;
//! 3. the composition is deterministic: the same model produces the same
//!    geometry, twice.

use std::collections::BTreeMap;
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

/// What the corpus measures, by model.
fn measure_corpus() -> BTreeMap<String, (Composition, Composition)> {
    let mut rows = BTreeMap::new();
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
        rows.insert(
            name,
            (
                measure(&bare, &relayout(&bare)),
                measure(&model, &relayout(&model)),
            ),
        );
    }
    rows
}

/// A count as the number the baseline speaks.
fn count(value: usize) -> i64 {
    i64::try_from(value).expect("the count fits")
}

/// A coordinate as the number the baseline speaks.
fn coordinate(value: i32) -> i64 {
    i64::from(value)
}

/// The baseline the composition is compared against.
fn baseline() -> serde_json::Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout-baseline.json");
    let text = std::fs::read_to_string(&path).expect("the baseline is committed");
    serde_json::from_str(&text).expect("the baseline is JSON")
}

#[test]
fn the_composition_measures_as_the_baseline_says() {
    let measured = measure_corpus();
    let baseline = baseline();
    let models = baseline["models"]
        .as_array()
        .expect("the baseline has the models");
    let mut differences: Vec<String> = Vec::new();
    for entry in models {
        let name = entry["model"].as_str().expect("the model has a name");
        let Some((from_scratch, recomposed)) = measured.get(name) else {
            differences.push(format!("{name}: the corpus does not have it"));
            continue;
        };
        for (label, one, expected) in [
            ("from scratch", from_scratch, &entry["from_scratch"]),
            ("recomposed", recomposed, &entry["recomposed"]),
        ] {
            let values: [(&str, i64); 8] = [
                ("blocks", count(one.blocks)),
                ("edges", count(one.edges)),
                ("overlaps", count(one.overlaps)),
                ("outside", count(one.outside)),
                ("crossings", count(one.crossings)),
                ("isolated", count(one.isolated)),
                ("width", coordinate(one.width)),
                ("height", coordinate(one.height)),
            ];
            for (field, value) in values {
                let said = expected[field].as_i64().expect("the baseline has the field");
                if said != value {
                    differences.push(format!(
                        "{name} ({label}): {field} is {value}, the baseline says {said}"
                    ));
                }
            }
        }
    }
    assert!(
        differences.is_empty(),
        "the composition does not measure as the baseline says:\n{}\n\
         refresh the baseline on purpose with \
         `cargo run -p genmsi-format --example layout_measure -- --write`",
        differences.join("\n")
    );
}

#[test]
fn every_block_stays_inside_the_container_that_owns_it() {
    for (name, (from_scratch, recomposed)) in measure_corpus() {
        assert_eq!(
            from_scratch.outside, 0,
            "{name}: blocks fell out of their container (from scratch)"
        );
        assert_eq!(
            recomposed.outside, 0,
            "{name}: blocks fell out of their container (recomposed)"
        );
    }
}

#[test]
fn the_composition_is_deterministic() {
    for path in corpus() {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(model) = genmsi_format::parse_str(&text) else {
            continue;
        };
        let bare = without_geometry(&model);
        assert_eq!(
            relayout(&bare),
            relayout(&bare),
            "{}: the composition is not deterministic",
            path.display()
        );
        assert_eq!(
            relayout(&model),
            relayout(&model),
            "{}: the recomposition is not deterministic",
            path.display()
        );
    }
}
