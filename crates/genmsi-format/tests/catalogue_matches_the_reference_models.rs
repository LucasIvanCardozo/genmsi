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

// The three classes the original declares and the catalogue did not offer
// until now: `MFunction`, `MFunctionDist` and `RemoveNumber`.
//
// Their shape comes from the original's class library
// (`Genmsi_20230711.backup/gClasses.xml`): the ports its graph declares and
// the properties its macro code declares, with the default value the macro
// code carries.
//
// - `MFunction`: no ports. `NombresFunc` (a list of names, `{FuncA;FuncB}`),
//   `Atributo` (`P:Param`) and `Matriz` (`{{11;12;13};{21;22;23}}`);
// - `MFunctionDist`: no ports. `NombresDist` (`{Func1;Func2}`), `Semilla`
//   (`4`) and `Resultados` (`{{20%;30%;50%};{25%;35%;40%}}`, with the line
//   breaks the macro code writes inside the value);
// - `RemoveNumber`: one input (`Entrada1`) and two outputs (`Salida1` and
//   `Salida2`, "No existe el Nº"). `Op` (`=`), `Grupo` (`UnGrupo`) and
//   `Numero` (`0`).

#[test]
fn the_three_classes_the_original_declares_are_offered_with_its_defaults() {
    // The `MFunction` class: the functions it defines over one attribute.
    let mfunction = class_spec("MFunction").expect("the catalogue offers MFunction");
    assert!(mfunction.inputs.is_empty(), "the class declares no ports");
    assert!(mfunction.outputs.is_empty(), "the class declares no ports");
    assert_eq!(mfunction.properties.len(), 3);
    assert_eq!(mfunction.properties[0].name, "Atributo");
    assert_eq!(mfunction.properties[0].ty, "TSNA");
    assert_eq!(mfunction.properties[0].subtype, "SNA");
    assert_eq!(mfunction.properties[1].name, "Matriz");
    assert_eq!(mfunction.properties[1].ty, "TSNAMATRIX");
    assert_eq!(mfunction.properties[2].name, "NombresFunc");
    assert_eq!(mfunction.properties[2].ty, "TEntityDefList");
    assert_eq!(mfunction.properties[2].subtype, "Function");
    assert!(
        mfunction.properties[2].unique,
        "a list of names defines the functions"
    );

    // The `MFunctionDist` class: the distribution functions it defines.
    let mfunctiondist = class_spec("MFunctionDist").expect("the catalogue offers MFunctionDist");
    assert!(mfunctiondist.inputs.is_empty(), "the class declares no ports");
    assert!(mfunctiondist.outputs.is_empty(), "the class declares no ports");
    assert_eq!(mfunctiondist.properties.len(), 3);
    assert_eq!(mfunctiondist.properties[0].name, "NombresDist");
    assert_eq!(mfunctiondist.properties[0].ty, "TEntityDefList");
    assert_eq!(mfunctiondist.properties[0].subtype, "Function");
    assert_eq!(mfunctiondist.properties[1].name, "Resultados");
    assert_eq!(mfunctiondist.properties[1].ty, "FloatPercentMatrix");
    assert_eq!(mfunctiondist.properties[2].name, "Semilla");
    assert_eq!(mfunctiondist.properties[2].ty, "PosInteger");

    // The `RemoveNumber` class: the number it asks a numeric group for.
    let removenumber = class_spec("RemoveNumber").expect("the catalogue offers RemoveNumber");
    assert_eq!(removenumber.inputs.len(), 1);
    assert_eq!(removenumber.inputs[0].ioid, 1);
    assert_eq!(removenumber.inputs[0].name, "Entrada1");
    assert_eq!(removenumber.outputs.len(), 2);
    assert_eq!(removenumber.outputs[0].ioid, 2);
    assert_eq!(removenumber.outputs[0].name, "Salida1");
    assert_eq!(removenumber.outputs[1].ioid, 3);
    assert_eq!(removenumber.outputs[1].name, "Salida2");
    assert_eq!(removenumber.outputs[1].caption, "No%20existe%20el%20N%BA");
    assert_eq!(removenumber.properties.len(), 3);
    assert_eq!(removenumber.properties[0].name, "Grupo");
    assert_eq!(removenumber.properties[0].ty, "EntityUse");
    assert_eq!(removenumber.properties[0].subtype, "NumericGroup");
    assert_eq!(removenumber.properties[1].name, "Numero");
    assert_eq!(removenumber.properties[1].ty, "TSNA");
    assert_eq!(removenumber.properties[2].name, "Op");
    assert_eq!(removenumber.properties[2].ty, "Listed");
    assert_eq!(removenumber.properties[2].subtype, "op_RelacMaxMin");
}

#[test]
fn the_three_classes_the_original_declares_carry_its_defaults() {
    // The defaults of the original's class library, percent-encoded the way
    // the file carries them.
    for (class, property, value) in [
        ("MFunction", "Atributo", "P%3AParam"),
        (
            "MFunction",
            "Matriz",
            "%7B%7B11%3B12%3B13%7D%3B%7B21%3B22%3B23%7D%7D",
        ),
        ("MFunction", "NombresFunc", "%7BFuncA%3BFuncB%7D"),
        ("MFunctionDist", "NombresDist", "%7BFunc1%3BFunc2%7D"),
        (
            "MFunctionDist",
            "Resultados",
            "%7B%0A%7B20%25%3B30%25%3B50%25%7D%3B%0A%7B25%25%3B35%25%3B40%25%7D%0A%7D",
        ),
        ("MFunctionDist", "Semilla", "4"),
        ("RemoveNumber", "Grupo", "UnGrupo"),
        ("RemoveNumber", "Numero", "0"),
        ("RemoveNumber", "Op", "%3D"),
    ] {
        assert_eq!(
            genmsi_format::catalogue::default_value(class, property),
            Some(value),
            "{class}.{property}: the default of the original's class library"
        );
    }
}

#[test]
fn a_new_block_of_the_three_classes_carries_the_defaults() {
    use genmsi_format::{build_block, catalogue::default_value};

    for class in ["MFunction", "MFunctionDist", "RemoveNumber"] {
        let block = build_block(class, 0, None).expect("the class is in the catalogue");
        for property in &block.props {
            let expected = default_value(class, &property.name).unwrap_or_default();
            assert_eq!(
                property.raw_value, expected,
                "{class}.{}: the block does not come with the default",
                property.name
            );
        }
    }
}

