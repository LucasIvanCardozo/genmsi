//! Helpers shared by the block-semantics regression tests.
//!
//! Each test builds a tiny model instead of using a fixture: the point is to pin
//! one rule in isolation, so the model must contain nothing else.

#![allow(dead_code)]

use genmsi_core::{Program, RunResult, SimConfig, Simulation, program::compile};
use genmsi_format::parse_str;

/// One block of a test model.
pub struct Mod {
    class: &'static str,
    label: &'static str,
    props: Vec<(&'static str, &'static str)>,
}

pub fn module(
    class: &'static str,
    label: &'static str,
    props: &[(&'static str, &'static str)],
) -> Mod {
    Mod {
        class,
        label,
        props: props.to_vec(),
    }
}

/// Compile a linear model: every block's `Salida1` feeds the next block. The
/// trailing `Clock` module provides the termination tick, like the original's
/// `CLOCK` block, so the model needs no positive `TERMINATE` operand.
pub fn linear(modules: &[Mod], clock: f64) -> Program {
    let nodes: Vec<Node> = modules
        .iter()
        .enumerate()
        .map(|(index, module)| Node {
            id: index as u32 + 1,
            class: module.class,
            input: (index > 0).then_some(module.label),
            outputs: modules
                .get(index + 1)
                .map(|next| vec![("Salida1", Some(next.label))])
                .unwrap_or_default(),
            props: module.props.clone(),
        })
        .collect();
    let links: Vec<(u32, usize, u32, usize)> = (1..modules.len() as u32)
        .map(|id| (id, 0, id + 1, 0))
        .collect();
    wired(&nodes, &links, clock)
}

/// A block of a hand-wired test model.
pub struct Node {
    /// Module id; links refer to it.
    pub id: u32,
    /// Module class (`Generate`, `Seize`, `GateL`, ...).
    pub class: &'static str,
    /// Connection label of the block's single input port.
    pub input: Option<&'static str>,
    /// `(output port name, connection label)` per output, in port order.
    pub outputs: Vec<(&'static str, Option<&'static str>)>,
    /// Module properties.
    pub props: Vec<(&'static str, &'static str)>,
}

pub fn node(
    id: u32,
    class: &'static str,
    input: Option<&'static str>,
    outputs: &[(&'static str, Option<&'static str>)],
    props: &[(&'static str, &'static str)],
) -> Node {
    Node {
        id,
        class,
        input,
        outputs: outputs.to_vec(),
        props: props.to_vec(),
    }
}

/// Compile a hand-wired model: nodes plus explicit links
/// `(from module, from port index, to module, to port index)`.
pub fn wired(nodes: &[Node], links: &[(u32, usize, u32, usize)], clock: f64) -> Program {
    let mut xml = String::from(r#"<gmodel versioncode="1"><gclasses/><modules>"#);
    xml.push_str(
        r#"<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C"><links>"#,
    );
    for (from, port, to, _) in links {
        xml.push_str(&format!(
            r#"<link frommoduleid="{from}" fromioid="{}" tomoduleid="{to}" toioid="1"/>"#,
            port + 2
        ));
    }
    xml.push_str("</links><inputs/><outputs/></module>");
    for node in nodes {
        xml.push_str(&format!(
            r#"<module moduleid="{}" moduleclass="{}" owner="0" caption="c" name="{}" kind="A" width="10" height="10" x="0" y="0">"#,
            node.id, node.class, node.class
        ));
        match node.input {
            Some(label) => xml.push_str(&format!(
                r#"<inputs><input ioid="1" name="Entrada1" caption="" mylabel="{label}"/></inputs>"#
            )),
            None => xml.push_str("<inputs/>"),
        }
        if node.outputs.is_empty() {
            xml.push_str("<outputs/>");
        } else {
            xml.push_str("<outputs>");
            for (index, (name, label)) in node.outputs.iter().enumerate() {
                xml.push_str(&format!(
                    r#"<output ioid="{}" name="{name}" caption="" mylabel="{}"/>"#,
                    index + 2,
                    label.unwrap_or("")
                ));
            }
            xml.push_str("</outputs>");
        }
        xml.push_str("<props>");
        for (name, value) in &node.props {
            xml.push_str(&format!(
                r#"<property name="{name}" hint="" value="{value}" type="TSNA" subType="SNA" visible="1" unique="0"/>"#
            ));
        }
        xml.push_str("</props></module>");
    }
    let clock_id = nodes.iter().map(|node| node.id).max().unwrap_or(0) as usize + 1;
    xml.push_str(&clock_xml(clock_id, clock));
    xml.push_str("</modules></gmodel>");
    compile(&parse_str(&xml).expect("test model parses"), "wired").expect("test model compiles")
}

fn clock_xml(id: usize, clock: f64) -> String {
    format!(
        r#"<module moduleid="{id}" moduleclass="Clock" owner="0" caption="c" name="Clock" kind="A" width="10" height="10" x="0" y="0"><props><property name="Frecuencia" hint="" value="{clock}" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>"#
    )
}

pub fn run(program: &Program, stop_time: f64, seed: u64) -> RunResult {
    Simulation::new(
        program,
        SimConfig {
            seed,
            stop_time: Some(stop_time),
            max_events: 5_000_000,
            ..SimConfig::default()
        },
    )
    .run()
    .expect("runs")
}

/// Locate a block by its connection label, or by class for generators (which
/// have no input port and therefore no label, like the original's numeric LOC).
pub fn block(program: &Program, label: &str) -> usize {
    program
        .blocks
        .iter()
        .position(|block| block.label.as_deref() == Some(label))
        .or_else(|| {
            program
                .blocks
                .iter()
                .position(|block| block.class == label)
        })
        .unwrap_or_else(|| panic!("no block labelled {label}"))
}
