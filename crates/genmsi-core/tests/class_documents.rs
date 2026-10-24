//! The documents that tell a client which classes exist and what a block of
//! one of them looks like.
//!
//! These are the documents an agent reads before it writes a model: the list
//! of the classes, the skeleton of a model and the XML of a block of each
//! class. The tests below read them back and check that what they carry is
//! what the format accepts.

use genmsi_core::{
    CLASSLIST_SCHEMA, CLASS_SCHEMA, class_document, class_list_document, to_pretty_json,
};

#[test]
fn the_class_list_names_every_class_and_the_skeleton() {
    let document = class_list_document();
    assert_eq!(document.classes.len(), 66, "the classes the editor offers");
    assert_eq!(document.classes[0].class, "Advance");
    assert!(!document.classes[0].title.is_empty());
    let generate = document
        .classes
        .iter()
        .find(|class| class.class == "Generate")
        .expect("the Generate class");
    assert_eq!(generate.title, "Generar");

    // The skeleton is a document the format reads: the agent starts from it.
    let model = genmsi_format::parse_str(&document.skeleton).expect("the skeleton parses");
    assert_eq!(model.blocks.len(), 1, "the skeleton has no blocks");
    assert!(model.root().is_some(), "the skeleton has the model root");
}

#[test]
fn the_class_list_names_its_schema() {
    let text = to_pretty_json(CLASSLIST_SCHEMA, &class_list_document());
    assert!(
        text.starts_with("{\n  \"schema\": \"genmsi.classlist/1\",\n"),
        "{text}"
    );
}

#[test]
fn a_class_document_carries_the_ports_the_properties_and_the_template() {
    let document = class_document("Generate").expect("the class is in the catalogue");
    assert_eq!(document.class, "Generate");
    assert_eq!(document.title, "Generar");
    assert!(document.inputs.is_empty(), "the generator has no inputs");
    assert_eq!(document.outputs.len(), 1);
    assert_eq!(document.outputs[0].ioid, 1);
    assert_eq!(document.outputs[0].name, "Salida1");
    let media = document
        .properties
        .iter()
        .find(|property| property.name == "Media")
        .expect("the Media property");
    assert_eq!(media.ty, "TSNA");
    assert_eq!(media.subtype, "SNA");
    assert!(document.template.starts_with("<module moduleid=\"0\""));
    assert!(document.template.contains("moduleclass=\"Generate\""));
}

#[test]
fn the_class_document_names_its_schema() {
    let document = class_document("Terminate").expect("the class is in the catalogue");
    let text = to_pretty_json(CLASS_SCHEMA, &document);
    assert!(
        text.starts_with("{\n  \"schema\": \"genmsi.class/1\",\n"),
        "{text}"
    );
}

#[test]
fn a_class_that_does_not_exist_has_no_document() {
    assert!(class_document("NotAClass").is_none());
    assert!(class_document("").is_none());
}

#[test]
fn the_template_of_every_class_is_a_block_the_format_reads() {
    let skeleton = class_list_document().skeleton;
    for class in class_list_document().classes {
        let document = class_document(&class.class).expect("the class is in the catalogue");
        let block = document
            .template
            .replace("moduleid=\"0\"", "moduleid=\"1\"");
        let text = skeleton.replace("</modules>", &format!("{block}</modules>"));
        let model = genmsi_format::parse_str(&text).expect("the template is a block of a model");
        genmsi_format::validate(&model).expect("the model is valid");
        assert_eq!(model.blocks.len(), 2);
    }
}
