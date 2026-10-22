//! The catalogue must match the reference models.
//!
//! Direction pinned by this test: for every block of every reference model
//! whose class IS in the catalogue,
//!
//! 1. the block declares EXACTLY the catalogue's ports — same ids and names,
//!    nothing more and nothing less (both directions);
//! 2. every VISIBLE property the block declares exists in the catalogue's
//!    class with the same declared `type` and `subType` (one direction: the
//!    models may omit properties the catalogue declares).
//!
//! Blocks whose class is NOT in the catalogue (custom macros like
//! `Multifurcacion`, plain containers with an empty class, the model root)
//! are skipped: they are not offered by the editor and the catalogue makes no
//! claim about them.

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

            // 1. Ports: exactly the catalogue's ports, same ids and names.
            let catalogue_inputs: Vec<(u32, &str)> =
                spec.inputs.iter().map(|p| (p.ioid, p.name)).collect();
            let catalogue_outputs: Vec<(u32, &str)> =
                spec.outputs.iter().map(|p| (p.ioid, p.name)).collect();
            let model_inputs: Vec<(u32, &str)> =
                block.inputs.iter().map(|p| (p.ioid, p.name.as_str())).collect();
            let model_outputs: Vec<(u32, &str)> =
                block.outputs.iter().map(|p| (p.ioid, p.name.as_str())).collect();
            assert_eq!(
                model_inputs, catalogue_inputs,
                "{name}: block {} ({}) input ports differ from the catalogue",
                block.id, block.class
            );
            assert_eq!(
                model_outputs, catalogue_outputs,
                "{name}: block {} ({}) output ports differ from the catalogue",
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
    assert!(block.props.iter().all(|p| p.raw_value.is_empty()));
    assert_eq!(block.header, None);
    assert_eq!(block.source, None);
    assert_eq!(
        block.layout.map(|l| (l.width, l.height, l.x, l.y)),
        Some((120, 48, 10, 20))
    );

    // An unknown class is refused.
    let err = build_block("Multifurcacion", 0, None).expect_err("custom macro");
    assert!(err.to_string().contains("Multifurcacion"), "{err}");
}
