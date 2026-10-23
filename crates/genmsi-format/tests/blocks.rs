//! A new block is born with the default values of its class.
//!
//! The original editor hands a new block the defaults of its class (a clock
//! starts counting at one hour). The catalogue carries those defaults in
//! `catalogue::DEFAULTS`, and `build_block` fills them in; every property the
//! table does not name stays empty.

use genmsi_format::catalogue::{build_block, CLASSES, DEFAULTS};
use genmsi_format::{parse_str, to_string, validate, EditCommand, EditSession, Link};

/// The default the table gives to a property of a class, if it gives one.
fn default_of(class: &str, property: &str) -> Option<&'static str> {
    DEFAULTS
        .iter()
        .find(|(c, p, _)| *c == class && *p == property)
        .map(|(_, _, value)| *value)
}

#[test]
fn the_defaults_table_holds_the_clock_row() {
    assert!(
        DEFAULTS.contains(&("Clock", "Frecuencia", "3600")),
        "the clock counts in units of time: the original gives it a whole hour"
    );
    // The table is one row per class and property, and it never repeats a
    // pair: the last row for a pair would silently win.
    let mut pairs: Vec<(&str, &str)> = DEFAULTS.iter().map(|(c, p, _)| (*c, *p)).collect();
    let total = pairs.len();
    pairs.sort_unstable();
    pairs.dedup();
    assert_eq!(
        pairs.len(),
        total,
        "the defaults table has no repeated pair"
    );
}

#[test]
fn a_new_clock_block_comes_with_its_default_frequency() {
    let block = build_block("Clock", 0, None).expect("Clock is in the catalogue");
    let frequency = block
        .props
        .iter()
        .find(|prop| prop.name == "Frecuencia")
        .expect("the clock declares its frequency");
    assert_eq!(
        frequency.raw_value, "3600",
        "a new clock counts in units of time: it starts with a whole hour"
    );
    assert_eq!(
        frequency.value(),
        "3600",
        "the raw value is the written value"
    );
}

#[test]
fn a_new_block_of_a_class_without_defaults_is_born_empty() {
    let block = build_block("Advance", 0, None).expect("Advance is in the catalogue");
    assert!(!block.props.is_empty(), "the class declares properties");
    for prop in &block.props {
        assert_eq!(
            prop.raw_value, "",
            "{} has no default: it stays empty",
            prop.name
        );
    }
}

#[test]
fn only_the_properties_the_table_names_carry_a_default() {
    for class in CLASSES {
        let block = build_block(class.class, 0, None).expect("class is in the catalogue");
        for prop in &block.props {
            match default_of(class.class, &prop.name) {
                Some(value) => assert_eq!(
                    prop.raw_value, value,
                    "{}: the default of {} is {value}",
                    class.class, prop.name
                ),
                None => assert_eq!(
                    prop.raw_value, "",
                    "{}: {} has no default and stays empty",
                    class.class, prop.name
                ),
            }
        }
    }
}

// The connections of the editing API.
//
// A port has a single successor, so drawing a link over an occupied output
// replaces the link that starts there. An input port may receive from
// several blocks (21 of the 24 reference models do it), so drawing a link
// over an occupied input keeps the links that end there.

/// A small model: a queue, a depart and a terminate, chained.
fn small_model() -> genmsi_format::GmdlModel {
    parse_str(
        r#"<gmodel versioncode="1"><gclasses/><modules>\
<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C"><links><link frommoduleid="1" fromioid="2" tomoduleid="2" toioid="1"/><link frommoduleid="2" fromioid="2" tomoduleid="3" toioid="1"/></links><inputs/><outputs/></module>\
<module moduleid="1" moduleclass="Queue" owner="0" caption="q" name="Queue" kind="A"><inputs><input ioid="1" name="Entrada1" caption="Ingreso" mylabel=""/></inputs><outputs><output ioid="2" name="Salida1" caption="" mylabel="A_Queue"/></outputs><props/>\
</module>\
<module moduleid="2" moduleclass="Depart" owner="0" caption="d" name="Depart" kind="A"><inputs><input ioid="1" name="Entrada1" caption="Ingreso" mylabel="A_Queue"/></inputs><outputs><output ioid="2" name="Salida1" caption="" mylabel="A_Depart"/></outputs><props/>\
</module>\
<module moduleid="3" moduleclass="Terminate" owner="0" caption="t" name="Terminate" kind="A"><inputs><input ioid="1" name="Entrada1" caption="Ingreso" mylabel="A_Depart"/></inputs><props/>\
</module>\
</modules></gmodel>"#,
    )
    .expect("parses")
}

/// The links of the model's root container.
fn links(session: &EditSession) -> Vec<Link> {
    session
        .model()
        .block(0)
        .expect("the root container")
        .links
        .clone()
}

/// Whether the model has a link between the given ports.
fn has_link(session: &EditSession, from: u32, from_ioid: u32, to: u32, to_ioid: u32) -> bool {
    links(session).iter().any(|link| {
        link.from_module == from
            && link.from_ioid == from_ioid
            && link.to_module == to
            && link.to_ioid == to_ioid
    })
}

#[test]
fn connecting_a_second_block_to_an_occupied_input_keeps_both_links() {
    let mut session = EditSession::new(small_model());
    let ids = session
        .apply(EditCommand::AddBlock {
            class: "Generate".into(),
            container: 0,
            x: 0,
            y: 0,
        })
        .expect("adds a Generate");
    let generate = ids[0];
    session
        .apply(EditCommand::Connect {
            from: generate,
            from_ioid: 1,
            to: 2,
            to_ioid: 1,
        })
        .expect("connects the Generate to the Depart");

    // The Depart's input had a link (the Queue): it keeps it, and the new
    // one arrives next to it.
    assert!(
        has_link(&session, 1, 2, 2, 1),
        "the link the Queue already had is kept"
    );
    assert!(
        has_link(&session, generate, 1, 2, 1),
        "the new link arrives at the same input"
    );
    assert_eq!(
        links(&session).len(),
        3,
        "the chain of the model and the new link"
    );
    // The model is still valid and the file keeps the two links.
    validate(session.model()).expect("the model with two links into an input is valid");
    let written = to_string(session.model());
    let reparsed = parse_str(&written).expect("the written model parses again");
    assert_eq!(reparsed.block(0).expect("root").links.len(), 3);
}

#[test]
fn connecting_a_second_block_to_an_occupied_output_replaces_the_link() {
    let mut session = EditSession::new(small_model());
    // The Queue's output already had a link (to the Depart).
    session
        .apply(EditCommand::Connect {
            from: 1,
            from_ioid: 2,
            to: 3,
            to_ioid: 1,
        })
        .expect("connects the Queue to the Terminate");

    let the_links = links(&session);
    assert!(
        !has_link(&session, 1, 2, 2, 1),
        "the link that started at the output is replaced"
    );
    assert!(
        has_link(&session, 1, 2, 3, 1),
        "the new link starts at that output"
    );
    assert!(
        has_link(&session, 2, 2, 3, 1),
        "the link that ended at the same input is kept"
    );
    assert_eq!(the_links.len(), 2, "one link per output port");
}

#[test]
fn undoing_a_connection_restores_the_links_it_kept() {
    let mut session = EditSession::new(small_model());
    let ids = session
        .apply(EditCommand::AddBlock {
            class: "Generate".into(),
            container: 0,
            x: 0,
            y: 0,
        })
        .expect("adds a Generate");
    let generate = ids[0];
    let before = to_string(session.model());
    session
        .apply(EditCommand::Connect {
            from: generate,
            from_ioid: 1,
            to: 2,
            to_ioid: 1,
        })
        .expect("connects the Generate to the Depart");
    assert_eq!(links(&session).len(), 3);

    session.undo().expect("undoes the connection");
    assert_eq!(
        to_string(session.model()),
        before,
        "undo restores the model exactly"
    );
    assert!(
        has_link(&session, 1, 2, 2, 1),
        "the link the Queue had is still there"
    );
    assert!(
        !has_link(&session, generate, 1, 2, 1),
        "the undone link is gone"
    );
    assert_eq!(links(&session).len(), 2, "the chain of the model is whole");
}

#[test]
fn adding_a_clock_through_the_editing_api_gives_it_the_default() {
    let model = parse_str(
        r#"<gmodel versioncode="1"><gclasses/><modules><module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C" genprefer="" exehead="" nonexehead="" exefoot="" nonexefoot=""><links/><inputs/><outputs/></module></modules></gmodel>"#,
    )
    .expect("parses");
    let mut session = EditSession::new(model);
    let ids = session
        .apply(EditCommand::AddBlock {
            class: "Clock".into(),
            container: 0,
            x: 10,
            y: 20,
        })
        .expect("adds");
    let block = session.model().block(ids[0]).expect("added");
    let frequency = block
        .props
        .iter()
        .find(|prop| prop.name == "Frecuencia")
        .expect("the clock declares its frequency");
    assert_eq!(
        frequency.raw_value, "3600",
        "the added clock starts at a whole hour"
    );
}
