//! The catalogue the editor offers must stay inside the classes the compiler
//! understands.
//!
//! The catalogue says what a block looks like in the file; the compiler says
//! what a block means. A class the compiler does not know would create a
//! block that simulates nothing, and the user would only find out when the
//! model runs — so the two lists are pinned together here.

use genmsi_core::known_classes;

/// Every class the catalogue offers is a class the compiler compiles.
#[test]
fn every_catalogue_class_is_known_by_the_compiler() {
    let known = known_classes();
    for class in genmsi_format::catalogue::CLASSES {
        assert!(
            known.contains(&class.class),
            "the catalogue offers {:?}, which the compiler does not know",
            class.class
        );
    }
}

/// The catalogue is not empty and every class declares its ports and its
/// properties with the ids the file uses.
#[test]
fn the_catalogue_declares_its_ports_with_unique_ids() {
    assert!(!genmsi_format::catalogue::CLASSES.is_empty());
    for class in genmsi_format::catalogue::CLASSES {
        let mut ids = Vec::new();
        for port in class.inputs.iter().chain(class.outputs.iter()) {
            assert!(
                !ids.contains(&port.ioid),
                "{}: the port id {} is used twice",
                class.class,
                port.ioid
            );
            ids.push(port.ioid);
            assert!(!port.name.is_empty(), "{}: a port has no name", class.class);
        }
        for property in class.properties {
            assert!(
                !property.name.is_empty(),
                "{}: a property has no name",
                class.class
            );
            assert!(
                !property.ty.is_empty() || !property.subtype.is_empty(),
                "{}: the property {} has neither a declared type nor a subtype",
                class.class,
                property.name
            );
        }
    }
}
