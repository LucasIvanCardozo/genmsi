//! What the catalogue says about the classes.
//!
//! The list of the classes and the detail of one class say the same thing
//! about it: the engine's glossary describes the class, and the description
//! travels with it. The tooltip of a property is text a reader can read, and
//! the template of a block keeps the encoded text the file stores.

use genmsi_core::{class_document, class_list_document};

/// The class the tests read.
const CLASS: &str = "Generate";

#[test]
fn the_list_and_the_detail_say_the_same_thing_about_a_class() {
    let listed = class_list_document();
    let entry = listed
        .classes
        .iter()
        .find(|class| class.class == CLASS)
        .expect("the class is in the catalogue");
    let detail = class_document(CLASS).expect("the class is in the catalogue");
    assert_eq!(entry.description, detail.description);
    assert!(
        !entry.description.is_empty(),
        "the class says what it does"
    );
}

#[test]
fn the_description_says_what_the_class_does() {
    let detail = class_document(CLASS).expect("the class is in the catalogue");
    assert!(
        detail.description.contains("transactions"),
        "the description says what the class does: {}",
        detail.description
    );
}

#[test]
fn a_class_the_glossary_does_not_cover_has_an_empty_description() {
    let listed = class_list_document();
    let entry = listed
        .classes
        .iter()
        .find(|class| class.class == "Alter")
        .expect("the class is in the catalogue");
    assert_eq!(entry.description, "");
    let detail = class_document("Alter").expect("the class is in the catalogue");
    assert_eq!(detail.description, "");
}

#[test]
fn the_tooltips_of_the_detail_are_text_a_reader_can_read() {
    let detail = class_document(CLASS).expect("the class is in the catalogue");
    for property in &detail.properties {
        assert!(
            !property.hint.contains('%'),
            "the tooltip of {} is still encoded: {}",
            property.name,
            property.hint
        );
    }
    let cantidad = detail
        .properties
        .iter()
        .find(|property| property.name == "Cantidad")
        .expect("the Cantidad property");
    assert_eq!(cantidad.hint, "Cantidad de Transacciones a generar");
}

#[test]
fn the_template_keeps_the_encoded_tooltip() {
    let detail = class_document(CLASS).expect("the class is in the catalogue");
    assert!(
        detail
            .template
            .contains("hint=\"Cantidad%20de%20Transacciones%20a%20generar\""),
        "the template is the text the file stores: {}",
        detail.template
    );
}
