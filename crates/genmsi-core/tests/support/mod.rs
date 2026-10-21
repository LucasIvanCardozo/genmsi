//! Helpers shared by the block-semantics regression tests.
//!
//! Each test builds a tiny linear model instead of using a fixture: the point is
//! to pin one rule in isolation, so the model must contain nothing else.

#![allow(dead_code)]

use genmsi_core::{Program, RunResult, SimConfig, Simulation, program::compile};
use genmsi_format::parse_str;

/// One block of a linear test model.
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
    let mut xml = String::from(r#"<gmodel versioncode="1"><gclasses/><modules>"#);
    xml.push_str(
        r#"<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C"><links>"#,
    );
    for index in 0..modules.len() - 1 {
        xml.push_str(&format!(
            r#"<link frommoduleid="{}" fromioid="2" tomoduleid="{}" toioid="1"/>"#,
            index + 1,
            index + 2
        ));
    }
    xml.push_str("</links><inputs/><outputs/></module>");
    for (index, module) in modules.iter().enumerate() {
        let id = index + 1;
        let next = modules.get(index + 1).map(|module| module.label);
        xml.push_str(&format!(
            r#"<module moduleid="{id}" moduleclass="{}" owner="0" caption="c" name="{}" kind="A" width="10" height="10" x="0" y="0">"#,
            module.class, module.label
        ));
        if index > 0 {
            xml.push_str(&format!(
                r#"<inputs><input ioid="1" name="Entrada1" caption="" mylabel="{}"/></inputs>"#,
                module.label
            ));
        } else {
            xml.push_str("<inputs/>");
        }
        if let Some(next) = next {
            xml.push_str(&format!(
                r#"<outputs><output ioid="2" name="Salida1" caption="" mylabel="{next}"/></outputs>"#
            ));
        } else {
            xml.push_str("<outputs/>");
        }
        xml.push_str("<props>");
        for (name, value) in &module.props {
            xml.push_str(&format!(
                r#"<property name="{name}" hint="" value="{value}" type="TSNA" subType="SNA" visible="1" unique="0"/>"#
            ));
        }
        xml.push_str("</props></module>");
    }
    xml.push_str(&format!(
        r#"<module moduleid="{}" moduleclass="Clock" owner="0" caption="c" name="Clock" kind="A" width="10" height="10" x="0" y="0"><props><property name="Frecuencia" hint="" value="{clock}" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>"#,
        modules.len() + 1
    ));
    xml.push_str("</modules></gmodel>");
    compile(
        &parse_str(&xml).expect("test model parses"),
        "semantics",
    )
    .expect("test model compiles")
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
