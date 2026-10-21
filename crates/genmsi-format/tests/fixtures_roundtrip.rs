//! The reference models under `fixtures/models/` must load and rewrite
//! byte-for-byte. This is the strongest available check that the semantic model
//! keeps every piece of information the original editor wrote, including canvas
//! layout, encoded hints and container headers.

use std::path::{Path, PathBuf};

use genmsi_format::{parse_str, to_string, validate};

fn models_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/models")
        .canonicalize()
        .expect("fixtures/models exists")
}

fn models() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(models_dir())
        .expect("models directory")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "gmdl"))
        .collect();
    paths.sort();
    paths
}

#[test]
fn all_reference_models_parse_and_validate() {
    let paths = models();
    assert_eq!(paths.len(), 25, "expected 25 reference models");
    for path in &paths {
        let text = std::fs::read_to_string(path).expect("model is UTF-8");
        let model = parse_str(&text).unwrap_or_else(|err| {
            panic!("{}: {err}", path.display());
        });
        validate(&model).unwrap_or_else(|err| {
            // Documented oracle rule (validate.rs): the original tool reports
            // `Error en linea 9 -> Bloque inválido` for this model's QTable
            // name "tiempo demora" (see its report.txt), so it must fail
            // load-time validation while still parsing and rewriting.
            if path.file_name().unwrap() == "Parcial 8-11-21__ParcialMikelModelos.gmdl"
                && err.to_string().contains("invalid block")
            {
                return;
            }
            panic!("{}: {err}", path.display());
        });
        assert!(model.root().is_some(), "{}: no root", path.display());
    }
}

#[test]
fn rewriting_every_reference_model_is_byte_identical() {
    for path in models() {
        let text = std::fs::read_to_string(&path).expect("model is UTF-8");
        let model = parse_str(&text).expect("model parses");
        let rewritten = to_string(&model);
        assert_eq!(
            rewritten,
            text,
            "{} rewrote with differences",
            path.display()
        );
    }
}

#[test]
fn rewriting_a_semantically_edited_model_changes_only_the_edited_value() {
    let path = models_dir().join("TP2__problema1.gmdl");
    let text = std::fs::read_to_string(&path).expect("model is UTF-8");
    let mut model = parse_str(&text).expect("model parses");
    let queue = model
        .activity_blocks()
        .find(|b| b.class == "Queue")
        .expect("TP2 has a Queue block")
        .id;
    model
        .block_mut(queue)
        .expect("queue block")
        .prop_mut("Queue")
        .expect("Queue property")
        .set_value("otra_cola");
    let rewritten = to_string(&model);

    let expected = text.replacen("value=\"centro\"", "value=\"otra_cola\"", 1);
    assert_eq!(rewritten, expected, "only the edited value may change");
}

#[test]
fn encoded_property_values_are_decoded_for_semantics() {
    let path = models_dir().join("Parcial 8-11-21__ParcialMikelModelos.gmdl");
    let text = std::fs::read_to_string(&path).expect("model is UTF-8");
    let model = parse_str(&text).expect("model parses");
    let qtable = model
        .activity_blocks()
        .find(|b| b.class == "QTable")
        .expect("model has a QTable");
    assert_eq!(
        qtable.prop_value("NombreTabla").as_deref(),
        Some("tiempo demora")
    );
    // The raw value keeps the editor's URL encoding for a byte-identical rewrite.
    assert_eq!(
        qtable.prop("NombreTabla").unwrap().raw_value,
        "tiempo%20demora"
    );
}

#[test]
fn custom_class_macro_code_is_preserved() {
    let path = models_dir().join("Parcial 6-11-23__Veron - Parcial.gmdl");
    let text = std::fs::read_to_string(&path).expect("model is UTF-8");
    let model = parse_str(&text).expect("model parses");
    assert_eq!(model.classes.len(), 2);
    assert_eq!(model.classes[0].class_name(), Some("Que area voy"));
    assert!(model.classes[0].raw.contains("macrocode"));
    assert_eq!(to_string(&model), text);
}
