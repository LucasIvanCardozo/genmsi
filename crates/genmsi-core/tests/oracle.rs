//! The oracle harness must stay usable and the committed comparison document
//! must stay in sync with the fixture set.

use std::path::{Path, PathBuf};

use genmsi_core::oracle::{compare_model_text, parse_baseline_report};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn models() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(root().join("fixtures/models"))
        .expect("fixtures/models")
        .map(|entry| entry.expect("entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "gmdl"))
        .collect();
    paths.sort();
    paths
}

fn baseline_path(model: &Path) -> PathBuf {
    let stem = model.file_stem().unwrap().to_string_lossy().to_string();
    root().join(format!(
        "fixtures/outputs/{}.report.txt",
        stem.replace(' ', "_")
    ))
}

#[test]
fn every_valid_baseline_parses_into_metrics() {
    let mut parsed = 0;
    for model in models() {
        let text = std::fs::read_to_string(baseline_path(&model)).expect("baseline exists");
        let report = match parse_baseline_report(&text) {
            Ok(report) => report,
            Err(_) => continue, // one of the five error outputs
        };
        parsed += 1;
        assert!(report.end_time > 0.0, "{}", model.display());
        assert!(!report.blocks.is_empty(), "{}", model.display());
    }
    assert_eq!(parsed, 20, "expected 20 report baselines, parsed {parsed}");
}

#[test]
fn error_baselines_are_not_reports() {
    for text in [
        "Error en linea     9 -> Bloque inválido",
        "Advertencia -> No hay bloques TERMINATE con parámetro A\n\nError al intentar iniciar la simulación.",
        "Advertencia -> No hay bloques TERMINATE con parámetro A\n\nError de ejecución al iniciar la simulación.\nERROR!!",
        "Advertencia -> No hay bloques TERMINATE con parámetro A\n\nTiempo de simulación: 39,  Transacción activa: 3\nERROR!!Se intentó obtener la linea del fuente de un bloque inexistente.",
        "Advertencia -> No hay bloques TERMINATE con parámetro A\n\nError en linea    35 -> El parámetro B no debe ser nulo",
    ] {
        assert!(
            parse_baseline_report(text).is_err(),
            "error output parsed as a report: {text}"
        );
    }
}

#[test]
fn fast_models_match_the_baseline() {
    for name in [
        "Problema 1__semaforo-1",
        "entregaComida__entregaComida",
        "TP7__Clase 7_1_Link_Unlink",
        "TP7__Clase 7_2_ProductorConsumidor",
    ] {
        let path = root().join(format!("fixtures/models/{name}.gmdl"));
        let model = genmsi_format::load(&path).expect("loads");
        let text = std::fs::read_to_string(baseline_path(&path)).expect("baseline");
        let comparison = compare_model_text(name, &model, &text, 3).expect("comparison runs");
        assert_eq!(
            comparison.status(),
            "match",
            "{name}: matched {}/{} metrics",
            comparison.matched(),
            comparison.metrics.len()
        );
    }
}

#[test]
fn comparison_document_covers_every_model() {
    let doc = std::fs::read_to_string(root().join("docs/oracle-comparison.md"))
        .expect("docs/oracle-comparison.md exists");
    assert!(doc.contains("# Oracle comparison against the original GenMSI"));
    for model in models() {
        let stem = model.file_stem().unwrap().to_string_lossy().to_string();
        assert!(doc.contains(&stem), "document does not mention {stem}");
    }
    assert!(doc.contains("TP9__hack"), "document must cover TP9__hack");
    assert!(
        doc.contains("not") && doc.contains("replicated"),
        "document must state that the TP9__hack bug is not replicated"
    );
}
