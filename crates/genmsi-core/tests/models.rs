//! Every reference model must load, the runnable ones must simulate, and the
//! five models the original refuses must fail with the same error class.

use std::path::{Path, PathBuf};

use genmsi_core::{SimConfig, compile_model};
use genmsi_format::{FormatError, load, load_str, parse_str, validate_runnable};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn models() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(fixtures().join("fixtures/models"))
        .expect("fixtures/models")
        .map(|entry| entry.expect("entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "gmdl"))
        .collect();
    paths.sort();
    paths
}

/// The five models whose baseline is an error message instead of a report.
const REFUSED: [&str; 5] = [
    "Parcial 8-11-21__ParcialMikelModelos",
    "Problema 2__puenteLevadizo",
    "TP6__Clase 6_0_TestGate",
    "TP9__hack",
    "TP9__tp9",
];

fn stem(path: &Path) -> String {
    path.file_stem().unwrap().to_string_lossy().to_string()
}

#[test]
fn all_reference_models_parse() {
    let paths = models();
    assert_eq!(paths.len(), 25);
    for path in &paths {
        genmsi_format::parse_file(path)
            .unwrap_or_else(|err| panic!("{}: {err}", path.display()));
    }
}

#[test]
fn twenty_four_models_are_structurally_valid() {
    let refused = 0;
    let mut invalid = 0;
    for path in models() {
        match load(&path) {
            Ok(_) => {}
            Err(err) => {
                invalid += 1;
                assert!(
                    err.to_string().contains("invalid block"),
                    "{}: unexpected structural error {err}",
                    path.display()
                );
            }
        }
    }
    assert_eq!(invalid, 1, "exactly one model is structurally invalid");
    let _ = refused;
}

#[test]
fn the_twenty_valid_models_compile_to_a_program() {
    for path in models() {
        let name = stem(&path);
        if REFUSED.contains(&name.as_str()) {
            continue;
        }
        let model = load(&path).expect("loads");
        let program = compile_model(&model).unwrap_or_else(|err| panic!("{name}: {err}"));
        assert!(!program.blocks.is_empty(), "{name}: empty program");
    }
}

#[test]
fn the_five_refused_models_produce_the_expected_error_class() {
    for path in models() {
        let name = stem(&path);
        if !REFUSED.contains(&name.as_str()) {
            continue;
        }
        let model = match load(&path) {
            Ok(model) => model,
            Err(err) => {
                // Structurally invalid: load already rejected it (the original
                // reports this class as "Bloque inválido").
                let text = err.to_string();
                assert!(
                    text.contains("invalid block"),
                    "{name}: expected a structural error, got {text}"
                );
                continue;
            }
        };
        let error = match validate_runnable(&model) {
            Err(err) => err.to_string(),
            Ok(()) => compile_model(&model)
                .err()
                .map(|err| err.to_string())
                .unwrap_or_else(|| panic!("{name}: the engine accepted a model the original refuses")),
        };
        match name.as_str() {
            "TP9__tp9" => assert!(error.contains("operand B"), "{name}: {error}"),
            _ => assert!(
                error.contains("TERMINATE"),
                "{name}: expected the termination error, got {error}"
            ),
        }
    }
}

#[test]
fn invalid_qtable_name_is_a_structural_error_with_node_context() {
    let path = fixtures().join("fixtures/models/Parcial 8-11-21__ParcialMikelModelos.gmdl");
    let err = load(&path).expect_err("invalid block name must be rejected");
    assert!(
        matches!(err, FormatError::Structural { .. }),
        "expected structural error, got {err:?}"
    );
    let text = err.to_string();
    assert!(text.contains("invalid block"), "{text}");
    assert!(text.contains("tiempo demora"), "{text}");
}

#[test]
fn every_valid_model_runs_to_its_horizon() {
    for path in models() {
        let name = stem(&path);
        if REFUSED.contains(&name.as_str()) {
            continue;
        }
        let model = load(&path).expect("loads");
        let program = compile_model(&model).unwrap_or_else(|err| panic!("{name}: {err}"));
        let config = SimConfig {
            seed: 11,
            // A bounded horizon keeps the suite fast; the oracle example uses the
            // baseline's own END_TIME.
            stop_time: Some(3600.0),
            max_events: 2_000_000,
            ..SimConfig::default()
        };
        let result = genmsi_core::Simulation::new(&program, config)
            .run()
            .unwrap_or_else(|err| panic!("{name}: {err}"));
        assert!(result.end_time > 0.0, "{name}: no simulated time");
        assert_eq!(result.blocks.len(), program.blocks.len(), "{name}");
        for block in &result.blocks {
            // Engine invariant: a block never holds more transactions than it saw
            // enter (a generator holds its pending arrival before its first tick).
            assert!(
                block.current <= block.entries.max(1),
                "{name}: block has {} current but {} entries",
                block.current,
                block.entries
            );
        }
    }
}

#[test]
fn tp9_hack_does_not_replicate_the_original_crash() {
    // The original aborts with "Se intentó obtener la linea del fuente de un
    // bloque inexistente"; the engine reports the modelling error instead.
    let path = fixtures().join("fixtures/models/TP9__hack.gmdl");
    let model = load(&path).expect("structurally valid");
    let err = compile_model(&model).expect_err("no termination path");
    let text = err.to_string();
    assert!(text.contains("TERMINATE"), "{text}");
    assert!(!text.contains("inexistente"), "{text}");
}

#[test]
fn load_str_matches_load_file() {
    let path = fixtures().join("fixtures/models/TP2__problema1.gmdl");
    let text = std::fs::read_to_string(&path).expect("readable");
    let from_file = load(&path).expect("loads");
    let from_text = load_str(&text).expect("loads");
    assert_eq!(from_file, from_text);
    assert_eq!(parse_str(&text).unwrap(), from_file);
}
