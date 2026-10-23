//! A new block is born with the default values of its class.
//!
//! The original editor hands a new block the defaults of its class (a clock
//! starts counting at one hour). The catalogue carries those defaults in
//! `catalogue::DEFAULTS`, and `build_block` fills them in; every property the
//! table does not name stays empty.
//!
//! The classes the original declares and our catalogue does not offer are
//! pinned here too: the ports and the defaults come from the original's own
//! class library (`Genmsi_20230711.backup/gClasses.xml`).

use genmsi_format::catalogue::{build_block, class_spec, CLASSES, DEFAULTS};
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

// The classes the original declares and our catalogue did not offer.
//
// Their shape comes from the original's own class library
// (`Genmsi_20230711.backup/gClasses.xml`): the ports the class graph
// declares and the properties its macro code declares, with the default
// value the macro code carries.

/// The ports of a class, as `(id, name)` pairs.
fn ports(spec: &genmsi_format::catalogue::ClassSpec, inputs: bool) -> Vec<(u32, &str)> {
    let list = if inputs {
        spec.inputs
    } else {
        spec.outputs
    };
    list.iter().map(|port| (port.ioid, port.name)).collect()
}

#[test]
fn the_catalogue_offers_the_four_classes_the_original_declares() {
    // The `Preempt` class: one input, the main output and the output of the
    // owner it interrupts.
    let preempt = class_spec("Preempt").expect("the catalogue offers Preempt");
    assert_eq!(
        ports(preempt, true),
        vec![(1, "Entrada1")],
        "the Preempt class takes one input"
    );
    assert_eq!(
        ports(preempt, false),
        vec![(2, "Salida1"), (3, "SalidaAlt")],
        "the owner of the interrupted facility goes to SalidaAlt"
    );
    assert_eq!(
        preempt
            .properties
            .iter()
            .map(|property| property.name)
            .collect::<Vec<_>>(),
        vec!["Facility", "ModoArrebato", "ModoInterrupcion", "Remanente"],
        "the properties the macro code of the class declares"
    );

    // The `Return` class: the facility it gives back.
    let returns = class_spec("Return").expect("the catalogue offers Return");
    assert_eq!(ports(returns, true), vec![(1, "Entrada1")]);
    assert_eq!(ports(returns, false), vec![(2, "Salida1")]);
    assert_eq!(
        returns
            .properties
            .iter()
            .map(|property| property.name)
            .collect::<Vec<_>>(),
        vec!["Facility"]
    );

    // The two multifurcations: one input and five outputs.
    for (class, caption) in [
        ("Multifurcacion", "%5BProb1%5D"),
        ("MultifurcacionP", "%5BParametro%5D%3D1"),
    ] {
        let spec = class_spec(class).expect("the catalogue offers the class");
        assert_eq!(ports(spec, true), vec![(1, "Entrada")], "{class}");
        assert_eq!(
            ports(spec, false),
            vec![
                (2, "Salida1"),
                (3, "SalidaProb2"),
                (4, "SalidaProb3"),
                (5, "SalidaProb4"),
                (6, "SalidaProb5"),
            ],
            "{class}"
        );
        assert_eq!(spec.outputs[0].caption, caption, "{class}");
    }
}

#[test]
fn a_new_block_of_the_four_classes_comes_with_the_defaults_of_the_original() {    // The `Preempt` class: the facility, the mode, the parameter and the way
    // the owner goes on.
    let block = build_block("Preempt", 0, None).expect("Preempt is in the catalogue");
    let value = |name: &str| {
        block
            .props
            .iter()
            .find(|prop| prop.name == name)
            .unwrap_or_else(|| panic!("the class declares {name}"))
            .raw_value
            .clone()
    };
    assert_eq!(value("Facility"), "Recurso");
    assert_eq!(value("ModoInterrupcion"), "Prioridad");
    assert_eq!(value("ModoArrebato"), "NoRemover");
    assert_eq!(value("Remanente"), "Remanente");

    // The `Return` class: the facility it gives back.
    let block = build_block("Return", 0, None).expect("Return is in the catalogue");
    assert_eq!(block.props.len(), 1);
    assert_eq!(block.props[0].name, "Facility");
    assert_eq!(block.props[0].raw_value, "Recurso");

    // The multifurcations: the functions they create and the five shares.
    for class in ["Multifurcacion", "MultifurcacionP"] {
        let block = build_block(class, 0, None).expect("the class is in the catalogue");
        for index in 1..=5 {
            let prob = block
                .props
                .iter()
                .find(|prop| prop.name == format!("Prob{index}"))
                .expect("the class declares the share");
            assert_eq!(prob.raw_value, "20%25", "{class}: the fifth of the traffic");
            assert_eq!(prob.value(), "20%", "{class}: the share as the editor shows it");
        }
        let go = block
            .props
            .iter()
            .find(|prop| prop.name == "MultifurcacionGO")
            .expect("the class creates the function it routes by");
        assert_eq!(go.raw_value, "MultifurcacionGO");
        assert!(go.unique, "{class}: the function is defined by the block");
    }
}

// The eight classes the original declares and the catalogue did not offer
// yet: `Examine`, `GateM`, `Mark`, `RemoveMe`, `SelectL`, `SelectS`,
// `SelectSNA` and `VariableN`.

#[test]
fn the_catalogue_offers_the_eight_classes_the_original_declares() {
    // The `Examine` class: the transaction is examined in a group.
    let examine = class_spec("Examine").expect("the catalogue offers Examine");
    assert_eq!(ports(examine, true), vec![(1, "Entrada1")]);
    assert_eq!(
        ports(examine, false),
        vec![(2, "Salida1"), (3, "Salida2")],
        "the transaction goes on by the first output and by the second one \
         when it does not belong to the group"
    );
    assert_eq!(
        examine
            .properties
            .iter()
            .map(|property| property.name)
            .collect::<Vec<_>>(),
        vec!["Grupo"]
    );

    // The `GateM` class: the gate evaluates the label of the transaction.
    let gate = class_spec("GateM").expect("the catalogue offers GateM");
    assert_eq!(ports(gate, true), vec![(1, "Entrada1")]);
    assert_eq!(ports(gate, false), vec![(2, "Salida1"), (3, "Salida2")]);
    assert_eq!(
        gate
            .properties
            .iter()
            .map(|property| property.name)
            .collect::<Vec<_>>(),
        vec!["Rotulo", "op"]
    );

    // The `Mark` class: the hour goes to a parameter.
    let mark = class_spec("Mark").expect("the catalogue offers Mark");
    assert_eq!(ports(mark, true), vec![(1, "Entrada1")]);
    assert_eq!(ports(mark, false), vec![(2, "Salida1")]);
    assert_eq!(
        mark
            .properties
            .iter()
            .map(|property| property.name)
            .collect::<Vec<_>>(),
        vec!["Parametro"]
    );

    // The `RemoveMe` class: the transaction leaves its group.
    let remove = class_spec("RemoveMe").expect("the catalogue offers RemoveMe");
    assert_eq!(ports(remove, true), vec![(1, "Entrada1")]);
    assert_eq!(ports(remove, false), vec![(2, "Salida1"), (3, "Salida2")]);
    assert_eq!(
        remove
            .properties
            .iter()
            .map(|property| property.name)
            .collect::<Vec<_>>(),
        vec!["Grupo"]
    );

    // The three selections: a logic switch, a storage and an entity.
    for class in ["SelectL", "SelectS", "SelectSNA"] {
        let spec = class_spec(class).expect("the catalogue offers the class");
        assert_eq!(ports(spec, true), vec![(1, "Entrada1")], "{class}");
        assert_eq!(
            ports(spec, false),
            vec![(2, "Salida1"), (3, "Salida2")],
            "{class}"
        );
    }
    assert_eq!(
        class_spec("SelectL")
            .expect("SelectL")
            .properties
            .iter()
            .map(|property| property.name)
            .collect::<Vec<_>>(),
        vec!["Param", "maximo", "minimo", "op"]
    );
    assert_eq!(
        class_spec("SelectS")
            .expect("SelectS")
            .properties
            .iter()
            .map(|property| property.name)
            .collect::<Vec<_>>(),
        vec!["Param", "maximo", "minimo", "op"]
    );
    assert_eq!(
        class_spec("SelectSNA")
            .expect("SelectSNA")
            .properties
            .iter()
            .map(|property| property.name)
            .collect::<Vec<_>>(),
        vec!["ClaseSNA", "Param", "maximo", "minimo", "op", "valorComp"]
    );

    // The `VariableN` class: a normal variable and the function it reads.
    let variable = class_spec("VariableN").expect("the catalogue offers VariableN");
    assert!(ports(variable, true).is_empty());
    assert!(ports(variable, false).is_empty());
    assert_eq!(
        variable
            .properties
            .iter()
            .map(|property| property.name)
            .collect::<Vec<_>>(),
        vec!["Desvio", "Media", "NombreFunc", "NombreVar", "Seed"]
    );
}

#[test]
fn a_new_block_of_the_eight_classes_comes_with_the_defaults_of_the_original() {
    // The group of the examination and of the removal.
    for class in ["Examine", "RemoveMe"] {
        let block = build_block(class, 0, None).expect("the class is in the catalogue");
        let group = block
            .props
            .iter()
            .find(|prop| prop.name == "Grupo")
            .expect("the class declares its group");
        assert_eq!(group.raw_value, "UnGrupo", "{class}");
    }

    // The gate: the label it evaluates and the operator.
    let block = build_block("GateM", 0, None).expect("GateM is in the catalogue");
    let value = |name: &str| {
        block
            .props
            .iter()
            .find(|prop| prop.name == name)
            .unwrap_or_else(|| panic!("the class declares {name}"))
            .raw_value
            .clone()
    };
    assert_eq!(value("Rotulo"), "Rotulo");
    assert_eq!(value("op"), "Maching");

    // The mark: the parameter that receives the hour.
    let block = build_block("Mark", 0, None).expect("Mark is in the catalogue");
    assert_eq!(block.props.len(), 1);
    assert_eq!(block.props[0].raw_value, "Param");

    // The three selections: the parameter, the range and the operator.
    for (class, op) in [("SelectL", "Set"), ("SelectS", "NoLleno")] {
        let block = build_block(class, 0, None).expect("the class is in the catalogue");
        let value = |name: &str| {
            block
                .props
                .iter()
                .find(|prop| prop.name == name)
                .unwrap_or_else(|| panic!("the class declares {name}"))
                .raw_value
                .clone()
        };
        assert_eq!(value("Param"), "param", "{class}");
        assert_eq!(value("minimo"), "1", "{class}");
        assert_eq!(value("maximo"), "10", "{class}");
        assert_eq!(value("op"), op, "{class}");
    }
    let block = build_block("SelectSNA", 0, None).expect("SelectSNA is in the catalogue");
    let value = |name: &str| {
        block
            .props
            .iter()
            .find(|prop| prop.name == name)
            .unwrap_or_else(|| panic!("the class declares {name}"))
            .raw_value
            .clone()
    };
    assert_eq!(value("ClaseSNA"), "Q");
    assert_eq!(value("Param"), "param");
    assert_eq!(value("minimo"), "1");
    assert_eq!(value("maximo"), "10");
    assert_eq!(value("op"), "=");
    assert_eq!(value("valorComp"), "*1");

    // The normal variable: the function it creates and the variable itself.
    let block = build_block("VariableN", 0, None).expect("VariableN is in the catalogue");
    let value = |name: &str| {
        block
            .props
            .iter()
            .find(|prop| prop.name == name)
            .unwrap_or_else(|| panic!("the class declares {name}"))
            .raw_value
            .clone()
    };
    assert_eq!(value("Media"), "60");
    assert_eq!(value("Desvio"), "1");
    assert_eq!(value("NombreFunc"), "NORMAL");
    assert_eq!(value("NombreVar"), "Normal");
    assert_eq!(value("Seed"), "90");
}
