//! The properties a block carries and the class that declares them.
//!
//! A property the class does not declare is a typo, and the engine reads it
//! as nothing: the storage keeps no capacity and the model runs without
//! saying why. The format's answer is a finding that names the property the
//! block probably meant.
//!
//! The reference models are the corpus that keeps the finding honest: they
//! are the original's own files, and every property they carry belongs to
//! the class of its block.

use std::path::{Path, PathBuf};

use genmsi_format::properties::undeclared_properties;
use genmsi_format::{parse_str, properties::property_warnings};

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

/// A `StorageDef` with the capacity written without the accent.
fn storage_with_a_typo() -> String {
    r#"<gmodel versioncode="1"><gclasses/><modules>
<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C"><links/><inputs/><outputs/></module>
<module moduleid="1" moduleclass="StorageDef" owner="0" caption="Almacen" name="Almacen" kind="A"><props><property name="Storage" hint="" value="SalaEspera" type="EntityDef" subType="Storage" visible="1" unique="1"/><property name="Tamano" hint="" value="30" type="PosInteger" subType="" visible="1" unique="0"/></props></module>
</modules></gmodel>"#
    .to_string()
}

#[test]
fn a_property_the_class_does_not_declare_is_a_finding() {
    let model = parse_str(&storage_with_a_typo()).expect("the model parses");
    let found = undeclared_properties(&model);
    assert_eq!(found.len(), 1, "{found:?}");
    let finding = &found[0];
    assert_eq!(finding.block, 1);
    assert_eq!(finding.class, "StorageDef");
    assert_eq!(finding.property, "Tamano");
}

#[test]
fn the_finding_names_the_property_it_meant() {
    let model = parse_str(&storage_with_a_typo()).expect("the model parses");
    let warnings = property_warnings(&model);
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    let warning = &warnings[0];
    assert!(warning.contains("Tamano"), "{warning}");
    assert!(warning.contains("Tamaño"), "{warning}");
    assert!(warning.contains("StorageDef"), "{warning}");
}

#[test]
fn a_property_of_the_class_is_not_a_finding() {
    let model = parse_str(&storage_with_a_typo().replace("Tamano", "Tamaño")).expect("the model parses");
    assert!(
        undeclared_properties(&model).is_empty(),
        "the property the class declares is not a finding"
    );
}

#[test]
fn the_reference_models_carry_no_finding() {
    for path in models() {
        let text = std::fs::read_to_string(&path).expect("model is UTF-8");
        let model = parse_str(&text).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        let found = undeclared_properties(&model);
        assert!(
            found.is_empty(),
            "{}: {} carries a property its class does not declare: {found:?}",
            path.display(),
            found.len()
        );
    }
}
