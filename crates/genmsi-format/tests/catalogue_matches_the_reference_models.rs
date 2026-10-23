//! The catalogue must match the reference models.
//!
//! Direction pinned by this test: for every block of every reference model
//! whose class IS in the catalogue,
//!
//! 1. the block declares the catalogue's ports — same ids and names, and the
//!    model may declare fewer of them than its class does;
//! 2. every VISIBLE property the block declares exists in the catalogue's
//!    class with the same declared `type` and `subType` (one direction: the
//!    models may omit properties the catalogue declares).
//!
//! Blocks whose class is NOT in the catalogue (custom macros, plain
//! containers with an empty class, the model root) are skipped: they are not
//! offered by the editor and the catalogue makes no claim about them.

use std::path::{Path, PathBuf};

use genmsi_format::{class_spec, parse_str};

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
fn every_catalogued_block_matches_the_catalogue() {
    let mut checked = 0usize;
    for path in models() {
        let text = std::fs::read_to_string(&path).expect("model is UTF-8");
        let model = parse_str(&text).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        let name = path.display();
        for block in &model.blocks {
            let Some(spec) = class_spec(&block.class) else {
                continue;
            };
            checked += 1;

            // 1. Ports: the catalogue's ports, same ids and names. A model
            //    may declare fewer of them than its class does: the original
            //    keeps the ports a model declares (the `Multifurcacion` of the
            //    reference model has three of the five its class declares).
            let catalogue_inputs: Vec<(u32, &str)> =
                spec.inputs.iter().map(|p| (p.ioid, p.name)).collect();
            let catalogue_outputs: Vec<(u32, &str)> =
                spec.outputs.iter().map(|p| (p.ioid, p.name)).collect();
            let model_inputs: Vec<(u32, &str)> =
                block.inputs.iter().map(|p| (p.ioid, p.name.as_str())).collect();
            let model_outputs: Vec<(u32, &str)> =
                block.outputs.iter().map(|p| (p.ioid, p.name.as_str())).collect();
            assert!(
                catalogue_inputs.starts_with(&model_inputs),
                "{name}: block {} ({}) input ports differ from the catalogue: \
                 the model declares {model_inputs:?} and the catalogue {catalogue_inputs:?}",
                block.id, block.class
            );
            assert!(
                catalogue_outputs.starts_with(&model_outputs),
                "{name}: block {} ({}) output ports differ from the catalogue: \
                 the model declares {model_outputs:?} and the catalogue {catalogue_outputs:?}",
                block.id, block.class
            );

            // 2. Every visible property exists in the catalogue with the
            //    same declared type and subtype.
            for prop in &block.props {
                if !prop.visible {
                    continue;
                }
                let declared = spec
                    .properties
                    .iter()
                    .find(|p| p.name == prop.name)
                    .unwrap_or_else(|| {
                        panic!(
                            "{name}: block {} ({}) declares the visible property {:?} \
                             that the catalogue does not know",
                            block.id, block.class, prop.name
                        )
                    });
                assert_eq!(
                    prop.ty, declared.ty,
                    "{name}: block {} ({}), property {}: declared type differs",
                    block.id, block.class, prop.name
                );
                assert_eq!(
                    prop.subtype, declared.subtype,
                    "{name}: block {} ({}), property {}: declared subtype differs",
                    block.id, block.class, prop.name
                );
                // A property that defines an entity stays unique.
                assert_eq!(
                    prop.unique, declared.unique,
                    "{name}: block {} ({}), property {}: uniqueness differs",
                    block.id, block.class, prop.name
                );
            }
        }
    }
    assert!(
        checked > 0,
        "the reference models must exercise the catalogue"
    );
}

#[test]
fn listed_options_cover_every_listed_property_of_the_models() {
    // Every `Listed` subtype the reference models declare must have options.
    for path in models() {
        let text = std::fs::read_to_string(&path).expect("model is UTF-8");
        let model = parse_str(&text).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        for block in &model.blocks {
            for prop in &block.props {
                if prop.ty == "Listed" && prop.visible {
                    let options = genmsi_format::listed_options(&prop.subtype);
                    assert!(
                        !options.is_empty(),
                        "{}: block {} ({}): Listed property {} has subtype {:?} \
                         without options",
                        path.display(),
                        block.id,
                        block.class,
                        prop.name,
                        prop.subtype
                    );
                    // An existing value of a Listed property is one of the
                    // options (blank means "not chosen yet"). Only ASCII
                    // values are checked strictly: the CP1252 files contain
                    // accented author spellings (e.g. "Parámetro") that the
                    // original editor accepted and the catalogue must not
                    // reproduce as option values.
                    let value = prop.value();
                    if !value.is_empty() && value.is_ascii() {
                        assert!(
                            options.contains(&value.as_str()),
                            "{}: block {} ({}): property {} holds {:?}, which is not \
                             one of {options:?}",
                            path.display(),
                            block.id,
                            block.class,
                            prop.name,
                            value
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn build_block_produces_the_reference_shape() {
    use genmsi_format::{build_block, Layout};

    // A Generate has one output, five properties and no inputs: its sections
    // are Outputs then Props, in that order.
    let block = build_block(
        "Generate",
        0,
        Some(Layout { width: 120, height: 48, x: 10, y: 20 }),
    )
    .expect("Generate is in the catalogue");
    assert_eq!(block.id, 0, "the caller assigns the real id");
    assert_eq!(block.class, "Generate");
    assert_eq!(block.kind, 'A');
    assert_eq!(block.owner, 0);
    assert_eq!(block.name, "Generate");
    assert_eq!(block.caption, "");
    assert_eq!(block.inputs.len(), 0);
    assert_eq!(block.outputs.len(), 1);
    assert_eq!(block.outputs[0].name, "Salida1");
    assert_eq!(block.outputs[0].ioid, 1);
    assert_eq!(block.outputs[0].mylabel, "");
    assert_eq!(block.props.len(), 5);
    // A new generator comes with the original's own defaults: a mean of 60 and
    // a spread of 30. The spread is what makes it a random source.
    let value = |name: &str| {
        block
            .props
            .iter()
            .find(|property| property.name == name)
            .map(|property| property.raw_value.clone())
            .unwrap_or_default()
    };
    assert_eq!(value("Media"), "60");
    assert_eq!(value("Modificador"), "30");
    assert_eq!(value("Cantidad"), "");
    assert_eq!(block.header, None);
    assert_eq!(block.source, None);
    assert_eq!(
        block.layout.map(|l| (l.width, l.height, l.x, l.y)),
        Some((120, 48, 10, 20))
    );

    // An unknown class is refused. `Multifurcacion` is a class of the
    // original and the catalogue offers it; a custom macro of a model is
    // not.
    let err = build_block("MacroPropio", 0, None).expect_err("custom macro");
    assert!(err.to_string().contains("MacroPropio"), "{err}");
}

// The three classes the original declares and the catalogue does not offer
// yet: `MFunction`, `MFunctionDist` and `RemoveNumber`.
//
// Their defaults come from the original's class library, and they wait for
// the day the catalogue shapes these classes:
//
// - `MFunction`: `NombresFunc` `{FuncA;FuncB}`, `Atributo` `P:Param` and
//   `Matriz` `{{11;12;13};{21;22;23}}`;
// - `MFunctionDist`: `NombresDist` `{Func1;Func2}`, `Semilla` `4` and
//   `Resultados` `{{20%;30%;50%};{25%;35%;40%}}`;
// - `RemoveNumber`: `Op` `=`, `Grupo` `UnGrupo` and `Numero` `0`.

#[test]
fn the_three_classes_the_original_declares_and_the_catalogue_does_not_offer() {
    for class in ["MFunction", "MFunctionDist", "RemoveNumber"] {
        assert!(
            class_spec(class).is_none(),
            "{class}: the catalogue offers the class now: give it the defaults \
             of the original's class library"
        );
    }
}
