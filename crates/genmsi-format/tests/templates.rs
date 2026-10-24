//! The templates of the classes: the XML of a block the format reads as it is.
//!
//! A template is what an agent copies into a model, so it has to be the
//! shape the file really uses: the same ports, the same properties and the
//! same default values the class declares. The tests below read every
//! template back and compare it with the catalogue, so a template that drifts
//! from the class it belongs to fails here.

use genmsi_format::catalogue::{CLASSES, default_value};
use genmsi_format::{GmdlModel, SKELETON_MODEL, parse_str, to_string, validate};

/// The skeleton of a model with one block in it.
///
/// The template carries `moduleid="0"`, the id of the model root: whoever
/// drops the block into a model renumbers it, and so does this helper.
fn model_with(template: &str) -> GmdlModel {
    let block = template.replace("moduleid=\"0\"", "moduleid=\"1\"");
    let text = SKELETON_MODEL.replace("</modules>", &format!("{block}</modules>"));
    parse_str(&text).expect("the model parses")
}

#[test]
fn every_class_has_a_template() {
    assert_eq!(CLASSES.len(), 66, "the classes the editor offers");
    for spec in CLASSES {
        let template = spec.template();
        assert!(
            template.starts_with("<module moduleid=\"0\" moduleclass=\""),
            "{}: the template is a module element: {template}",
            spec.class
        );
        assert!(
            template.contains(&format!("moduleclass=\"{}\"", spec.class)),
            "{}: the template names its class: {template}",
            spec.class
        );
        assert!(
            template.contains("owner=\"0\""),
            "{}: the block belongs to the model root: {template}",
            spec.class
        );
        assert!(
            template.contains("kind=\"A\""),
            "{}: the block is an activity block: {template}",
            spec.class
        );
    }
}

#[test]
fn every_template_is_a_block_the_format_reads() {
    for spec in CLASSES {
        let model = model_with(&spec.template());
        validate(&model).expect("the template is a valid model");
        let block = model.block(1).expect("the block is in the model");
        assert_eq!(block.class, spec.class);
        assert_eq!(block.kind, 'A');
        assert_eq!(block.owner, 0, "the root owns the block");
        assert_eq!(block.inputs.len(), spec.inputs.len());
        assert_eq!(block.outputs.len(), spec.outputs.len());
        assert_eq!(block.props.len(), spec.properties.len());
        for (port, expected) in block.inputs.iter().zip(spec.inputs) {
            assert_eq!(port.ioid, expected.ioid);
            assert_eq!(port.name, expected.name);
            assert_eq!(port.caption, expected.caption);
            assert_eq!(port.mylabel, "", "the label is the link's business");
        }
        for (port, expected) in block.outputs.iter().zip(spec.outputs) {
            assert_eq!(port.ioid, expected.ioid);
            assert_eq!(port.name, expected.name);
            assert_eq!(port.caption, expected.caption);
            assert_eq!(port.mylabel, "", "the label is the link's business");
        }
        for (property, expected) in block.props.iter().zip(spec.properties) {
            assert_eq!(property.name, expected.name);
            assert_eq!(property.hint, expected.hint);
            assert_eq!(property.ty, expected.ty);
            assert_eq!(property.subtype, expected.subtype);
            assert_eq!(property.unique, expected.unique);
            assert!(property.visible, "the editor shows the property");
            assert_eq!(
                property.raw_value,
                default_value(spec.class, expected.name).unwrap_or(""),
                "{}: the default value of {}",
                spec.class,
                expected.name
            );
        }
    }
}

#[test]
fn the_template_is_the_text_the_writer_writes() {
    for spec in CLASSES {
        let template = spec.template();
        let model = model_with(&template);
        let written = to_string(&model);
        let block = template.replace("moduleid=\"0\"", "moduleid=\"1\"");
        assert!(
            written.contains(&block),
            "{}: the writer writes the template as it is",
            spec.class
        );
    }
}

#[test]
fn the_generate_template_carries_the_defaults_of_its_class() {
    let template = genmsi_format::catalogue::class_spec("Generate")
        .expect("the class is in the catalogue")
        .template();
    assert!(template.contains("name=\"Media\""), "{template}");
    assert!(template.contains("value=\"60\""), "{template}");
    assert!(template.contains("value=\"30\""), "{template}");
    // A property the class declares no default for comes empty.
    assert!(template.contains("name=\"Cantidad\" hint=\""));
}

#[test]
fn a_property_without_a_default_comes_empty() {
    let template = genmsi_format::catalogue::class_spec("Queue")
        .expect("the class is in the catalogue")
        .template();
    assert!(template.contains("value=\"QRecurso\""), "{template}");
}
