//! `FacilityInt` expansion: PREEMPT/ADVANCE/RETURN must continue the
//! transaction through the block's main output.

mod support;

use genmsi_core::{Program, RunResult, SimConfig, Simulation, program::compile};
use genmsi_format::parse_str;
use support::block;

/// The editor declares `FacilityInt`'s failure output (`SalidaAlt`) *before*
/// `Salida1` and leaves it unlinked, so "the first output port" is not the main
/// output: RETURN has to follow `Salida1`. Following the first port silently
/// ended the transaction instead, which killed every model that uses the block
/// (`Parcial 6-11-23`: the preempting manager never closed its loop).
fn model() -> Program {
    let xml = r#"<gmodel versioncode="1"><gclasses/><modules>\
<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C"><links>\
<link frommoduleid="1" fromioid="1" tomoduleid="2" toioid="1"/>\
<link frommoduleid="2" fromioid="2" tomoduleid="3" toioid="1"/>\
<link frommoduleid="3" fromioid="3" tomoduleid="4" toioid="1"/>\
<link frommoduleid="4" fromioid="2" tomoduleid="5" toioid="1"/>\
</links><inputs/><outputs/></module>\
<module moduleid="1" moduleclass="Generate" owner="0" caption="g" name="g" kind="A" width="10" height="10" x="0" y="0"><outputs><output ioid="1" name="Salida1" caption="" mylabel="A"/></outputs>\
<props><property name="Media" hint="" value="5" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Modificador" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Cantidad" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>\
<module moduleid="2" moduleclass="Advance" owner="0" caption="a" name="a" kind="A" width="10" height="10" x="0" y="0"><inputs><input ioid="1" name="Entrada1" caption="" mylabel="A"/></inputs><outputs><output ioid="2" name="Salida1" caption="" mylabel="B"/></outputs>\
<props><property name="Media" hint="" value="5" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Modificador" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>\
<module moduleid="3" moduleclass="FacilityInt" owner="0" caption="f" name="f" kind="A" width="10" height="10" x="0" y="0"><inputs><input ioid="1" name="Entrada1" caption="" mylabel="B"/></inputs>\
<outputs><output ioid="2" name="SalidaAlt" caption="fallo" mylabel=""/><output ioid="3" name="Salida1" caption="" mylabel="C"/></outputs>\
<props><property name="Facility" hint="" value="maquina" type="EntityUse" subType="Facility" visible="1" unique="0"/><property name="Queue" hint="" value="qq" type="EntityUse" subType="Queue" visible="1" unique="0"/><property name="Media" hint="" value="3" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Modificador" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Remanente" hint="" value="Remanente" type="Listed" subType="Parametro" visible="1" unique="0"/></props></module>\
<module moduleid="4" moduleclass="Assign" owner="0" caption="s" name="s" kind="A" width="10" height="10" x="0" y="0"><inputs><input ioid="1" name="Entrada1" caption="" mylabel="C"/></inputs><outputs><output ioid="2" name="Salida1" caption="" mylabel="D"/></outputs>\
<props><property name="Parametro" hint="" value="Intruso" type="Listed" subType="Parametro" visible="1" unique="0"/><property name="Operador" hint="" value="=" type="Listed" subType="op_Asignacion" visible="1" unique="0"/><property name="Valor" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>\
<module moduleid="5" moduleclass="Terminate" owner="0" caption="t" name="t" kind="A" width="10" height="10" x="0" y="0"><inputs><input ioid="1" name="Entrada1" caption="" mylabel="D"/></inputs>\
<props><property name="Unidades" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>\
<module moduleid="6" moduleclass="Clock" owner="0" caption="c" name="c" kind="A" width="10" height="10" x="0" y="0"><props><property name="Frecuencia" hint="" value="1000" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>\
</modules></gmodel>"#;
    compile(&parse_str(xml).expect("parses"), "preemption").expect("compiles")
}

fn run(program: &Program) -> RunResult {
    Simulation::new(
        program,
        SimConfig {
            seed: 3,
            stop_time: Some(1000.0),
            ..SimConfig::default()
        },
    )
    .run()
    .expect("runs")
}

#[test]
fn facility_int_continues_through_salida1_after_return() {
    let program = model();
    let result = run(&program);
    let returned: usize = program
        .blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| block.class == "Return")
        .map(|(index, _)| index)
        .collect::<Vec<_>>()[0];
    assert_eq!(result.blocks[returned].entries, 1, "the manager preempted");
    let assign = block(&program, "C");
    assert_eq!(
        result.blocks[assign].entries, 1,
        "RETURN must continue through Salida1, not end the transaction"
    );
    assert_eq!(result.blocks[block(&program, "D")].entries, 1);
}
