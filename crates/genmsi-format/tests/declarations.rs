//! The entities a model uses must be declared somewhere in the model.
//!
//! The original editor says so in the description of the classes that use an
//! entity ("El Storage debe estar declarado (esta clase no lo declara)"), and
//! its entity picker only offers the entities the model declares. These tests
//! pin the format's answer to that rule: `undeclared_entities` names the
//! entity a block uses that nothing declares, `declaration_warnings` turns
//! each finding into a message that says what to fix, and
//! `validate_declarations` is the strict form of the same check.

use std::path::{Path, PathBuf};

use genmsi_format::{
    GmdlModel, UndeclaredEntity, declaration_warnings, parse_str, undeclared_entities, validate,
    validate_declarations,
};

/// A model with a `main` container and the given activity blocks.
fn model(blocks: &str) -> GmdlModel {
    let xml = format!(
        "<gmodel versioncode=\"1\"><gclasses/><modules>\
         <module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"m\" name=\"m\" kind=\"C\"><links/><inputs/><outputs/></module>\
         {blocks}</modules></gmodel>"
    );
    parse_str(&xml).expect("test model parses")
}

/// An activity block with the given properties.
fn activity(id: u32, class: &str, props: &str) -> String {
    format!(
        "<module moduleid=\"{id}\" moduleclass=\"{class}\" owner=\"0\" caption=\"c\" name=\"n\" kind=\"A\">\
         <props>{props}</props></module>"
    )
}

/// A property of the given type and subtype.
fn prop(name: &str, value: &str, ty: &str, subtype: &str) -> String {
    format!(
        "<property name=\"{name}\" hint=\"\" value=\"{value}\" type=\"{ty}\" subType=\"{subtype}\" visible=\"1\" unique=\"0\"/>"
    )
}

/// A block that uses a storage.
fn storage_use(id: u32, storage: &str) -> String {
    activity(
        id,
        "StorageUse",
        &prop("Storage", storage, "EntityUse", "Storage"),
    )
}

/// A block that declares a storage.
fn storage_def(id: u32, storage: &str) -> String {
    activity(
        id,
        "StorageDef",
        &prop("Storage", storage, "EntityDef", "Storage"),
    )
}

#[test]
fn a_storage_use_without_a_storage_def_is_marked() {
    let model = model(&storage_use(1, "Almacen"));

    let findings = undeclared_entities(&model);
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let finding = &findings[0];
    assert_eq!(finding.kind, "Storage");
    assert_eq!(finding.name, "Almacen");
    assert_eq!(finding.class, "StorageUse");
    assert_eq!(finding.property, "Storage");
    assert_eq!(finding.block, 1, "the finding points at the using block");
    assert!(
        finding.declared.is_empty(),
        "the model declares no storage at all: {:?}",
        finding.declared
    );

    let err = validate_declarations(&model).expect_err("the model is refused");
    let message = err.to_string();
    assert!(message.contains("Almacen"), "{message}");
    assert!(message.contains("StorageDef"), "{message}");
}

#[test]
fn a_storage_use_with_a_storage_def_is_accepted() {
    let model = model(&format!(
        "{}{}",
        storage_use(1, "Almacen"),
        storage_def(2, "Almacen")
    ));

    assert!(
        undeclared_entities(&model).is_empty(),
        "{:#?}",
        undeclared_entities(&model)
    );
    assert!(declaration_warnings(&model).is_empty());
    validate_declarations(&model).expect("the model declares what it uses");
    validate(&model).expect("the model is structurally valid");
}

#[test]
fn the_warning_says_what_is_declared_and_how_to_fix() {
    let model = model(&format!(
        "{}{}",
        storage_def(1, "acreditacon"),
        storage_use(2, "acreditacion")
    ));

    let warnings = declaration_warnings(&model);
    assert_eq!(warnings.len(), 1, "{warnings:#?}");
    let warning = &warnings[0];
    assert!(warning.contains("acreditacion"), "{warning}");
    assert!(warning.contains("acreditacon"), "{warning}");
    assert!(warning.contains("StorageDef"), "{warning}");
}

#[test]
fn an_expression_or_a_placeholder_is_not_checked() {
    // `*Cuadra` is an expression the model computes at run time and
    // `[storage]` is a placeholder: neither names an entity, so neither can
    // be declared.
    let model = model(&format!(
        "{}{}",
        storage_use(1, "*Cuadra"),
        storage_use(2, "[storage]")
    ));

    assert!(
        undeclared_entities(&model).is_empty(),
        "{:#?}",
        undeclared_entities(&model)
    );
    assert!(declaration_warnings(&model).is_empty());
    assert!(validate_declarations(&model).is_ok());
}

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
fn the_reference_models_use_no_undeclared_storage() {
    let mut storage_findings: Vec<(String, UndeclaredEntity)> = Vec::new();
    for path in models() {
        let name = path
            .file_name()
            .expect("model file name")
            .to_string_lossy()
            .into_owned();
        let text = std::fs::read_to_string(&path).expect("model is UTF-8");
        // `parse_str` and not `load`: one reference model fails the
        // structural validation on purpose (its QTable name has a space),
        // and the declaration check must run on it anyway.
        let model = parse_str(&text).unwrap_or_else(|err| panic!("{name}: {err}"));
        for finding in undeclared_entities(&model) {
            if finding.kind == "Storage" {
                storage_findings.push((name.clone(), finding));
            }
        }
    }

    let summary: Vec<String> = storage_findings
        .iter()
        .map(|(name, finding)| format!("{name}: {}", finding.message()))
        .collect();
    assert_eq!(
        storage_findings.len(),
        1,
        "the reference models use exactly one undeclared storage, found:\n{}",
        summary.join("\n")
    );

    let (name, finding) = &storage_findings[0];
    assert_eq!(
        name, "Parcial 8-11-21__ParcialMikelModelos.gmdl",
        "the undeclared storage is in another model: {}",
        finding.message()
    );
    assert_eq!(finding.kind, "Storage");
    assert_eq!(finding.class, "StorageUse");
    assert_eq!(finding.name, "acreditacion");
    assert_eq!(
        finding.declared,
        vec!["acreditacon".to_string()],
        "the model declares a different storage"
    );
}
