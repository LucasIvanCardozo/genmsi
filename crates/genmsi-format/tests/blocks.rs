//! A new block is born with the default values of its class.
//!
//! The original editor hands a new block the defaults of its class (a clock
//! starts counting at one hour). The catalogue carries those defaults in
//! `catalogue::DEFAULTS`, and `build_block` fills them in; every property the
//! table does not name stays empty.
//!
//! The classes the original declares and our catalogue does not offer are
//! pinned here too: the ports and the defaults come from the original's own
//! class library (`Genmsi_20230711.backup/gClasses.xml`). The macro code of a
//! class (its `MAKE` line) declares the properties of the class with the
//! default value it carries, and every one of them is pinned below.

use genmsi_format::catalogue::{build_block, class_spec, listed_options, CLASSES, DEFAULTS};
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
    // The `Terminate` class declares its units without a value: the original
    // leaves them empty, and so does the catalogue.
    let block = build_block("Terminate", 0, None).expect("Terminate is in the catalogue");
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
    // The values the file carries are the ones the original writes: a `=`
    // is `%3D` and a `*` is `%2A`.
    assert_eq!(value("op"), "%3D");
    assert_eq!(value("valorComp"), "%2A1");
    // The editor shows them as the original writes them.
    let shown = |name: &str| {
        block
            .props
            .iter()
            .find(|prop| prop.name == name)
            .unwrap_or_else(|| panic!("the class declares {name}"))
            .value()
    };
    assert_eq!(shown("op"), "=");
    assert_eq!(shown("valorComp"), "*1");

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

// Every default the original's class library declares.
//
// The macro code of a class (its `MAKE` line) declares the properties of the
// class with the default value it carries. The value is the one the file
// carries: percent-encoded, the way the original writes it.
const CLASS_LIBRARY_DEFAULTS: &[(&str, &str, &str)] = &[
    ("Advance", "Media", "10"),
    ("Advance", "Modificador", "5"),
    ("Alter", "AltOpcion", "Parametro"),
    ("Alter", "AltParam", "param"),
    ("Alter", "AltValor", "0"),
    ("Alter", "Cantidad", "1"),
    ("Alter", "CompOp", "%3D"),
    ("Alter", "CompOpcion", "Parametro"),
    ("Alter", "CompParam", "otro"),
    ("Alter", "CompValue", "1"),
    ("Alter", "Grupo", "ungrupo"),
    ("Alter", "Modo", "Algunos"),
    ("Assemble", "Cantidad", "1"),
    ("Assign", "Operador", "%3D"),
    ("Assign", "Parametro", "Param"),
    ("Assign", "Valor", "0"),
    ("Clock", "Frecuencia", "3600"),
    ("Counter", "Incremento", "1"),
    ("Counter", "Param", "Nro"),
    ("Counter", "SaveValue", "Count"),
    ("Depart", "Queue", "Qrecurso"),
    ("Enter", "Cantidad", "1"),
    ("Enter", "Storage", "AStorage"),
    ("Examine", "Grupo", "UnGrupo"),
    ("ExamineNumber", "Grupo", "UnGrupo"),
    ("ExamineNumber", "Numero", "0"),
    ("FacilityInt", "Facility", "Recurso"),
    ("FacilityInt", "Media", "10"),
    ("FacilityInt", "Modificador", "5"),
    ("FacilityInt", "ModoArrebato", "NoRemover"),
    ("FacilityInt", "ModoInterrupcion", "Prioridad"),
    ("FacilityInt", "Queue", "ColaInt"),
    ("FacilityInt", "Remanente", "Remanente"),
    ("FacilityUse", "Facility", "Recurso"),
    ("FacilityUse", "Media", "10"),
    ("FacilityUse", "Modificador", "5"),
    ("FacilityUse", "Queue", "Cola"),
    ("Function", "Atributo", "P%3AParam"),
    ("Function", "NombreFunc", "Func"),
    ("Function", "Tabla", "%7B1%7C1%3B2%7C2%7D"),
    ("Function", "Tipo", "Discreta"),
    ("FunctionDist", "Distribucion", "%7B50%25%3B50%25%7D"),
    ("FunctionDist", "NombreDist", "FuncDist"),
    ("FunctionDist", "Semilla", "3"),
    ("FunctionExpon", "NombreFunc", "Expon"),
    ("FunctionExpon", "Seed", "2"),
    ("GateF", "Facility", "Recurso"),
    ("GateF", "op", "Libre"),
    ("GateL", "LogicSwich", "1"),
    ("GateL", "op", "Set"),
    ("GateM", "Rotulo", "Rotulo"),
    ("GateM", "op", "Maching"),
    ("GateS", "Storage", "Almacen"),
    ("GateS", "op", "NoLleno"),
    ("Gather", "Cantidad", "1"),
    ("Generate", "Media", "60"),
    ("Generate", "Modificador", "30"),
    ("InitiaSV", "SaveValue", "1"),
    ("InitiaSV", "Valor", "0"),
    ("InitialLS", "LogicSwich", "1"),
    ("InitialLS", "Valor", "Set"),
    ("JoinMe", "Grupo", "UnGrupo"),
    ("JoinNumber", "Grupo", "UnGrupo"),
    ("JoinNumber", "Numero", "0"),
    ("Leave", "Cantidad", "1"),
    ("Leave", "Storage", "AStorage"),
    ("Link", "Cadena", "Cadena"),
    ("Link", "Modo", "FIFO"),
    ("Logic", "LogicSwich", "1"),
    ("Logic", "Op", "Set"),
    ("Loop", "Parametro", "Param"),
    ("Mark", "Parametro", "Param"),
    ("Match", "RotuloMatch", "BBB"),
    ("Match", "RotuloPropio", "AAA"),
    // The three classes the catalogue did not offer, whose defaults the
    // class library of the original declares (the macro code of each class
    // carries them).
    ("MFunction", "Atributo", "P%3AParam"),
    ("MFunction", "Matriz", "%7B%7B11%3B12%3B13%7D%3B%7B21%3B22%3B23%7D%7D"),
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
    ("Preempt", "Facility", "Recurso"),
    ("Preempt", "ModoArrebato", "NoRemover"),
    ("Preempt", "ModoInterrupcion", "Prioridad"),
    ("Preempt", "Remanente", "Remanente"),
    ("Priority", "Prioridad", "0"),
    ("QTable", "Cantidad", "60"),
    ("QTable", "NombreTabla", "QTabla"),
    ("QTable", "PrimerIntervalo", "0"),
    ("QTable", "Queue", "qRecurso"),
    ("QTable", "TamFrecuencia", "600"),
    ("Queue", "Queue", "QRecurso"),
    ("Release", "Facility", "Recurso"),
    ("RemoveMe", "Grupo", "UnGrupo"),
    ("RemoveTrans", "Cantidad", "1"),
    ("RemoveTrans", "CompOp", "%3D"),
    ("RemoveTrans", "CompOpcion", "Parametro"),
    ("RemoveTrans", "CompParam", "otro"),
    ("RemoveTrans", "CompValue", "1"),
    ("RemoveTrans", "Grupo", "UnGrupo"),
    ("RemoveTrans", "Modo", "Algunos"),
    ("Return", "Facility", "Recurso"),
    ("SaveValue", "Op", "%3D"),
    ("SaveValue", "SaveValue", "1"),
    ("SaveValue", "Valor", "0"),
    ("Scan", "CompOp", "%3D"),
    ("Scan", "CompOpcion", "Parametro"),
    ("Scan", "CompParam", "otro"),
    ("Scan", "CompValue", "1"),
    ("Scan", "DeOpción", "Parametro"),
    ("Scan", "DeParam", "Param"),
    ("Scan", "EnOpcion", "Parametro"),
    ("Scan", "EnParam", "Param"),
    ("Scan", "Grupo", "UnGrupo"),
    ("Seize", "Facility", "Recurso"),
    ("Select", "ClaseSNA", "Q"),
    ("Select", "Param", "param"),
    ("Select", "maximo", "10"),
    ("Select", "minimo", "1"),
    ("Select", "op", "Minimo"),
    ("SelectF", "Param", "param"),
    ("SelectF", "maximo", "10"),
    ("SelectF", "minimo", "1"),
    ("SelectF", "op", "Libre"),
    ("SelectL", "Param", "param"),
    ("SelectL", "maximo", "10"),
    ("SelectL", "minimo", "1"),
    ("SelectL", "op", "Set"),
    ("SelectS", "Param", "param"),
    ("SelectS", "maximo", "10"),
    ("SelectS", "minimo", "1"),
    ("SelectS", "op", "NoLleno"),
    ("SelectSNA", "ClaseSNA", "Q"),
    ("SelectSNA", "Param", "param"),
    ("SelectSNA", "maximo", "10"),
    ("SelectSNA", "minimo", "1"),
    ("SelectSNA", "valorComp", "%2A1"),
    ("Split", "Cantidad", "1"),
    ("StorageDef", "Storage", "Almacen"),
    ("StorageDef", "Tamaño", "10"),
    ("StorageUse", "Media", "10"),
    ("StorageUse", "Modificador", "5"),
    ("StorageUse", "Queue", "qAlmacen"),
    ("StorageUse", "Storage", "Almacen"),
    ("Table", "Atributo", "P%3AAtrib"),
    ("Table", "Cantidad", "10"),
    ("Table", "NombreTabla", "Tabla"),
    ("Table", "PrimerIntervalo", "0"),
    ("Table", "TamFrecuencia", "10"),
    ("Tabulate", "NombreTabla", "Tabla"),
    ("Test", "1_Valor", "1"),
    ("Test", "2_OP", "%3D"),
    ("Test", "3_Valor", "1"),
    ("Unlink", "Cadena", "Cadena"),
    ("Unlink", "Cantidad", "1"),
    ("Unlink", "CompOp", "%3D"),
    ("Unlink", "CompOpcion", "Parametro"),
    ("Unlink", "CompParam", "Otro"),
    ("Unlink", "CompValue", "0"),
    ("Unlink", "Modo", "Algunos"),
    ("Variable", "NombreVar", "vari"),
    ("VariableA", "Maximo", "10"),
    ("VariableA", "Minimo", "1"),
    ("VariableA", "NombreVar", "Azar"),
    ("VariableA", "Seed", "80"),
    ("VariableN", "Desvio", "1"),
    ("VariableN", "Media", "60"),
    ("VariableN", "NombreFunc", "NORMAL"),
    ("VariableN", "NombreVar", "Normal"),
    ("VariableN", "Seed", "90"),
];

/// The rows the class library does not declare, and where they come from.
const EXTRAS: &[(&str, &str, &str)] = &[
    // The random seed the macro code names (`RN:2`): the seed number is the
    // value of the property that carries it.
    ("FunctionExpon", "2", "2"),
    // The macro code of the two multifurcations names these properties
    // without declaring them: the values come from the model the original
    // ships, the one that exercises the class.
    ("Multifurcacion", "MultifurcacionCH", "MultifurcacionCH"),
    ("Multifurcacion", "MultifurcacionGO", "MultifurcacionGO"),
    ("Multifurcacion", "Prob1", "20%25"),
    ("Multifurcacion", "Prob2", "20%25"),
    ("Multifurcacion", "Prob3", "20%25"),
    ("Multifurcacion", "Prob4", "20%25"),
    ("Multifurcacion", "Prob5", "20%25"),
    ("MultifurcacionP", "MultifurcacionGO", "MultifurcacionGO"),
    ("MultifurcacionP", "Parametro", "Param"),
    ("MultifurcacionP", "Prob1", "20%25"),
    ("MultifurcacionP", "Prob2", "20%25"),
    ("MultifurcacionP", "Prob3", "20%25"),
    ("MultifurcacionP", "Prob4", "20%25"),
    ("MultifurcacionP", "Prob5", "20%25"),
    // The `SelectSNA` class declares no operator: the original falls back to
    // the first option of the list.
    ("SelectSNA", "op", "%3D"),
];

#[test]
fn every_default_of_the_class_library_is_the_one_the_catalogue_gives() {
    for (class, property, value) in CLASS_LIBRARY_DEFAULTS {
        assert_eq!(
            default_of(class, property),
            Some(*value),
            "{class}.{property}: the original's class library gives it {value}"
        );
        let spec =
            class_spec(class).unwrap_or_else(|| panic!("the catalogue offers the class {class}"));
        assert!(
            spec.properties.iter().any(|p| p.name == *property),
            "{class}: the catalogue declares the property {property}"
        );
    }
}

#[test]
fn the_catalogue_gives_no_default_the_class_library_does_not_declare() {
    for (class, property, value) in DEFAULTS {
        let declared = CLASS_LIBRARY_DEFAULTS.contains(&(*class, *property, *value))
            || EXTRAS.contains(&(*class, *property, *value));
        assert!(
            declared,
            "{class}.{property}: the catalogue gives {value}, and the original's \
             class library declares no such default"
        );
    }
}

#[test]
fn the_defaults_are_written_the_way_the_original_writes_them() {
    // A value that the file has to encode is stored encoded: the original
    // writes `%3A` for a colon and `%7B1%7C1%3B2%7C2%7D` for the table of a
    // function.
    let block = build_block("Function", 0, None).expect("Function is in the catalogue");
    let raw = |name: &str| {
        block
            .props
            .iter()
            .find(|prop| prop.name == name)
            .unwrap_or_else(|| panic!("the class declares {name}"))
            .raw_value
            .clone()
    };
    let shown = |name: &str| {
        block
            .props
            .iter()
            .find(|prop| prop.name == name)
            .unwrap_or_else(|| panic!("the class declares {name}"))
            .value()
    };
    assert_eq!(raw("Atributo"), "P%3AParam");
    assert_eq!(raw("Tabla"), "%7B1%7C1%3B2%7C2%7D");
    // And the editor reads them back as the original writes them.
    assert_eq!(shown("Atributo"), "P:Param");
    assert_eq!(shown("Tabla"), "{1|1;2|2}");
}

#[test]
fn every_listed_default_is_one_of_the_options_of_its_subtype() {
    for (class, property, _) in DEFAULTS {
        let spec =
            class_spec(class).unwrap_or_else(|| panic!("the catalogue offers the class {class}"));
        let declared = spec
            .properties
            .iter()
            .find(|p| p.name == *property)
            .unwrap_or_else(|| panic!("{class} declares the property {property}"));
        if declared.ty != "Listed" {
            continue;
        }
        let block = build_block(class, 0, None).expect("the class is in the catalogue");
        let prop = block
            .props
            .iter()
            .find(|prop| prop.name == *property)
            .unwrap_or_else(|| panic!("the class declares {property}"));
        let options = listed_options(declared.subtype);
        assert!(
            options.contains(&prop.value().as_str()),
            "{class}.{property}: the default {:?} is not one of {options:?}",
            prop.value()
        );
    }
}

#[test]
fn the_editor_offers_the_six_operators_of_a_storage() {
    // The `op_Logic_Stor` list of the original has six operators, and the
    // editor offers the ones it offers:
    //
    //     SF:Lleno|SNF:NoLleno|SE:Vacio|SNE:NoVacio|SV:Disponible|SNV:NoDisponible
    //
    // (`Genmsi_20230711.backup/gmacroconfig.gmcfg`).  The `GateS` and the
    // `SelectS` take their `op` from that list.
    let options = listed_options("op_Logic_Stor");
    for operator in [
        "Lleno",
        "NoLleno",
        "Vacio",
        "NoVacio",
        "Disponible",
        "NoDisponible",
    ] {
        assert!(
            options.contains(&operator),
            "the editor does not offer {operator:?}: {options:?}"
        );
    }
}

#[test]
fn adding_a_container_creates_a_submodule() {
    let mut session = EditSession::new(small_model());
    let ids = session
        .apply(EditCommand::AddBlock {
            class: "Container".into(),
            container: 0,
            x: 100,
            y: 200,
        })
        .expect("adds a container");
    let container_id = ids[0];
    let container = session.model().block(container_id).expect("exists");
    assert!(container.is_container());
    assert_eq!(container.inputs.len(), 1);
    assert_eq!(container.outputs.len(), 1);
    assert_eq!(container.layout.unwrap().x, 100);
    assert_eq!(container.layout.unwrap().y, 200);

    // Can add a child block inside this new container
    let child_ids = session
        .apply(EditCommand::AddBlock {
            class: "Generate".into(),
            container: container_id,
            x: 50,
            y: 50,
        })
        .expect("adds block inside container");
    let child = session.model().block(child_ids[0]).expect("child exists");
    assert_eq!(child.owner, container_id);

    // Can add a PortIn and PortOut inside this container.
    //
    // The ports of a container share one numbering: the container is born
    // with an entry port (1) and an exit port (2), so the port that comes
    // next is the 3, and the one after it the 4. An id never names two
    // ports of the same container.
    session
        .apply(EditCommand::AddBlock {
            class: "PortIn".into(),
            container: container_id,
            x: 0,
            y: 0,
        })
        .expect("adds port in");
    assert_eq!(session.model().block(container_id).unwrap().inputs.len(), 2);
    assert_eq!(session.model().block(container_id).unwrap().inputs[1].ioid, 3);
    assert_eq!(session.model().block(container_id).unwrap().inputs[1].name, "Entrada3");

    session
        .apply(EditCommand::AddBlock {
            class: "PortOut".into(),
            container: container_id,
            x: 0,
            y: 0,
        })
        .expect("adds port out");
    assert_eq!(session.model().block(container_id).unwrap().outputs.len(), 2);
    assert_eq!(session.model().block(container_id).unwrap().outputs[1].ioid, 4);
    assert_eq!(session.model().block(container_id).unwrap().outputs[1].name, "Salida4");

    // Cannot add PortIn at root (container 0)
    assert!(
        session
            .apply(EditCommand::AddBlock {
                class: "PortIn".into(),
                container: 0,
                x: 0,
                y: 0,
            })
            .is_err()
    );

    // Undo port out
    session.undo().expect("undo port out");
    assert_eq!(session.model().block(container_id).unwrap().outputs.len(), 1);

    // Undo port in
    session.undo().expect("undo port in");
    assert_eq!(session.model().block(container_id).unwrap().inputs.len(), 1);

    // Undo removes child, then container
    session.undo().expect("undo child");
    assert!(session.model().block(child_ids[0]).is_none());
    session.undo().expect("undo container");
    assert!(session.model().block(container_id).is_none());
}

// The ports of a container, and the links that cross its border.
//
// A container is a frame: the flow enters it through its own entry port and
// leaves it through its own exit port. The two ports carry different ids
// (the file parser refuses a block that has two ports with the same id) and
// a fresh port carries no label: the label of a port is the label of the
// flow that crosses it, and the flow is what the links decide.

/// A container with a block of the given class inside it: the session, the
/// container and the block.
fn container_with_a_child(class: &str) -> (EditSession, u32, u32) {
    let mut session = EditSession::new(small_model());
    let container = session
        .apply(EditCommand::AddBlock {
            class: "Container".into(),
            container: 0,
            x: 100,
            y: 200,
        })
        .expect("adds a container")[0];
    let child = session
        .apply(EditCommand::AddBlock {
            class: class.into(),
            container,
            x: 10,
            y: 10,
        })
        .expect("adds a block inside the container")[0];
    (session, container, child)
}

#[test]
fn a_new_container_has_an_entry_port_and_an_exit_port_with_different_ids() {
    let mut session = EditSession::new(small_model());
    let ids = session
        .apply(EditCommand::AddBlock {
            class: "Container".into(),
            container: 0,
            x: 0,
            y: 0,
        })
        .expect("adds a container");
    let container = session.model().block(ids[0]).expect("exists");
    assert_eq!(container.inputs.len(), 1, "the container has an entry port");
    assert_eq!(container.outputs.len(), 1, "the container has an exit port");
    assert_eq!(container.inputs[0].ioid, 1, "the entry port is the port 1");
    assert_eq!(container.outputs[0].ioid, 2, "the exit port is the port 2");
    assert_eq!(container.inputs[0].mylabel, "", "a fresh port carries no label");
    assert_eq!(container.outputs[0].mylabel, "", "a fresh port carries no label");
    // The two ids do not collide: the file the editor writes reads again.
    let written = to_string(session.model());
    let reparsed = parse_str(&written).expect("the written model parses again");
    let container = reparsed.block(ids[0]).expect("the container is there");
    assert_eq!(container.inputs[0].ioid, 1);
    assert_eq!(container.outputs[0].ioid, 2);
}

#[test]
fn connecting_the_entry_port_of_a_container_to_a_block_inside_it_mints_the_label() {
    let (mut session, container, child) = container_with_a_child("Terminate");
    session
        .apply(EditCommand::Connect {
            from: container,
            from_ioid: 1,
            to: child,
            to_ioid: 1,
        })
        .expect("connects the entry port to the block inside");
    // The link that brings the flow into the container belongs to the
    // container.
    let links = &session.model().block(container).expect("container").links;
    assert_eq!(links.len(), 1, "the container holds the link that enters it");
    assert_eq!(
        (
            links[0].from_module,
            links[0].from_ioid,
            links[0].to_module,
            links[0].to_ioid
        ),
        (container, 1, child, 1),
        "the link starts at the entry port of the container"
    );
    // The two ends share the label the original editor mints for the block
    // that receives the flow.
    assert_eq!(
        session.model().block(container).unwrap().input(1).unwrap().mylabel,
        "A_Terminate"
    );
    assert_eq!(
        session.model().block(child).unwrap().input(1).unwrap().mylabel,
        "A_Terminate"
    );
}

#[test]
fn connecting_a_block_inside_a_container_to_its_exit_port_succeeds() {
    let (mut session, container, child) = container_with_a_child("Queue");
    session
        .apply(EditCommand::Connect {
            from: child,
            from_ioid: 2,
            to: container,
            to_ioid: 2,
        })
        .expect("connects the block to the exit port of the container");
    // The link that takes the flow out of the container belongs to the
    // container.
    let links = &session.model().block(container).expect("container").links;
    assert_eq!(links.len(), 1, "the container holds the link that leaves it");
    assert_eq!(
        (
            links[0].from_module,
            links[0].from_ioid,
            links[0].to_module,
            links[0].to_ioid
        ),
        (child, 2, container, 2),
        "the link ends at the exit port of the container"
    );
    // The label names the container the flow leaves: a container has no
    // class, so its name is what tells one from another.
    let name = session
        .model()
        .block(container)
        .expect("container")
        .name
        .clone();
    assert_eq!(
        session.model().block(container).unwrap().output(2).unwrap().mylabel,
        format!("A_{name}")
    );
    assert_eq!(
        session.model().block(child).unwrap().output(2).unwrap().mylabel,
        format!("A_{name}")
    );
}

#[test]
fn a_link_between_the_blocks_of_two_containers_is_refused() {
    let (mut session, container, _) = container_with_a_child("Queue");
    let other = session
        .apply(EditCommand::AddBlock {
            class: "Container".into(),
            container: 0,
            x: 300,
            y: 200,
        })
        .expect("adds another container")[0];
    session
        .apply(EditCommand::AddBlock {
            class: "Queue".into(),
            container: other,
            x: 10,
            y: 10,
        })
        .expect("adds a block inside the other container");
    // A block of one container and a block of another one do not share a
    // container: the editor refuses the link.
    let err = session
        .apply(EditCommand::Connect {
            from: child_of(&session, container),
            from_ioid: 2,
            to: child_of(&session, other),
            to_ioid: 1,
        })
        .expect_err("the blocks live in different containers");
    assert!(
        err.to_string()
            .contains("los bloques pertenecen a contenedores distintos"),
        "{err}"
    );
}

/// The block a container holds.
fn child_of(session: &EditSession, container: u32) -> u32 {
    session
        .model()
        .children_of(container)
        .next()
        .expect("the container holds a block")
        .id
}

#[test]
fn the_entry_port_of_a_container_can_be_connected_to_its_own_exit_port() {
    // The container passes the flow straight through: the entry port is the
    // source of the link and the exit port is its target.
    //
    // The model validator refuses a container that is connected to itself
    // (`crates/genmsi-format/src/validate.rs`), so the model this command
    // builds does not pass `validate` yet: the check belongs to the
    // validator, not to the editing API.
    let (mut session, container, _) = container_with_a_child("Queue");
    session
        .apply(EditCommand::Connect {
            from: container,
            from_ioid: 1,
            to: container,
            to_ioid: 2,
        })
        .expect("connects the entry port to the exit port");
    let links = &session.model().block(container).expect("container").links;
    assert_eq!(links.len(), 1, "the container holds the link that crosses it");
    assert_eq!(
        (
            links[0].from_module,
            links[0].from_ioid,
            links[0].to_module,
            links[0].to_ioid
        ),
        (container, 1, container, 2),
        "the link goes from the entry port to the exit port"
    );
}
