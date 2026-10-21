//! Logic-switch initial state and `INITIAL LS`.
//!
//! A logic switch starts *reset*, like published GPSS, unless `INITIAL LS`
//! (the editor's `InitialLS` module, property `Valor`) overrides it. The editor
//! writes that property as `Valor`, not `Op`: reading the wrong name made every
//! `InitialLS` module a no-op.

mod support;

use genmsi_core::{Program, RunResult, SimConfig, Simulation, program::compile};
use genmsi_format::parse_str;
use support::block;

fn program(initial_state: Option<&str>) -> Program {
    let initial = match initial_state {
        Some(state) => format!(
            r#"<module moduleid="8" moduleclass="InitialLS" owner="0" caption="i" name="i" kind="A" width="10" height="10" x="0" y="0"><props><property name="LogicSwich" hint="" value="S" type="EntityDef" subType="LogicSwich" visible="1" unique="1"/><property name="Valor" hint="" value="{state}" type="Listed" subType="op_Swich" visible="1" unique="0"/></props></module>"#
        ),
        None => String::new(),
    };
    let xml = format!(
        r#"<gmodel versioncode="1"><gclasses/><modules>\
<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C"><links>\
<link frommoduleid="1" fromioid="1" tomoduleid="2" toioid="1"/>\
<link frommoduleid="2" fromioid="2" tomoduleid="3" toioid="1"/>\
<link frommoduleid="4" fromioid="1" tomoduleid="5" toioid="1"/>\
<link frommoduleid="5" fromioid="2" tomoduleid="6" toioid="1"/>\
<link frommoduleid="6" fromioid="2" tomoduleid="7" toioid="1"/>\
</links><inputs/><outputs/></module>\
<module moduleid="1" moduleclass="Generate" owner="0" caption="g" name="g" kind="A" width="10" height="10" x="0" y="0"><outputs><output ioid="1" name="Salida1" caption="" mylabel="A"/></outputs>\
<props><property name="Media" hint="" value="10" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Modificador" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Cantidad" hint="" value="5" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>\
<module moduleid="2" moduleclass="GateL" owner="0" caption="gate" name="gate" kind="A" width="10" height="10" x="0" y="0"><inputs><input ioid="1" name="Entrada1" caption="" mylabel="A"/></inputs><outputs><output ioid="2" name="Salida1" caption="" mylabel="B"/></outputs>\
<props><property name="LogicSwich" hint="" value="S" type="EntityUse" subType="LogicSwich" visible="1" unique="0"/><property name="op" hint="" value="Set" type="Listed" subType="op_Logic_Swich" visible="1" unique="0"/></props></module>\
<module moduleid="3" moduleclass="Terminate" owner="0" caption="t" name="t" kind="A" width="10" height="10" x="0" y="0"><inputs><input ioid="1" name="Entrada1" caption="" mylabel="B"/></inputs>\
<props><property name="Unidades" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>\
<module moduleid="4" moduleclass="Generate" owner="0" caption="g2" name="g2" kind="A" width="10" height="10" x="0" y="0"><outputs><output ioid="1" name="Salida1" caption="" mylabel="C"/></outputs>\
<props><property name="Media" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Cantidad" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>\
<module moduleid="5" moduleclass="Advance" owner="0" caption="a" name="a" kind="A" width="10" height="10" x="0" y="0"><inputs><input ioid="1" name="Entrada1" caption="" mylabel="C"/></inputs><outputs><output ioid="2" name="Salida1" caption="" mylabel="D"/></outputs>\
<props><property name="Media" hint="" value="100" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Modificador" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>\
<module moduleid="6" moduleclass="Logic" owner="0" caption="l" name="l" kind="A" width="10" height="10" x="0" y="0"><inputs><input ioid="1" name="Entrada1" caption="" mylabel="D"/></inputs><outputs><output ioid="2" name="Salida1" caption="" mylabel="E"/></outputs>\
<props><property name="LogicSwich" hint="" value="S" type="EntityUse" subType="LogicSwich" visible="1" unique="0"/><property name="Op" hint="" value="Set" type="Listed" subType="op_Swich" visible="1" unique="0"/></props></module>\
<module moduleid="7" moduleclass="Terminate" owner="0" caption="t2" name="t2" kind="A" width="10" height="10" x="0" y="0"><inputs><input ioid="1" name="Entrada1" caption="" mylabel="E"/></inputs>\
<props><property name="Unidades" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>\
<module moduleid="9" moduleclass="Clock" owner="0" caption="c" name="c" kind="A" width="10" height="10" x="0" y="0"><props><property name="Frecuencia" hint="" value="1000" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>\
{initial}</modules></gmodel>"#
    );
    compile(&parse_str(&xml).expect("parses"), "logic").expect("compiles")
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

/// Without an `INITIAL LS` the switch starts reset, so `GATE LS` holds the
/// transaction until the model sets it.
#[test]
fn logic_switches_start_reset() {
    let program = program(None);
    let result = run(&program);
    let gate = block(&program, "A");
    assert_eq!(result.blocks[gate].entries, 5, "generated");
    assert!(
        result.blocks[gate].retries > 0,
        "a switch that nothing set must hold the transaction"
    );
    assert_eq!(result.blocks[gate].current, 0, "the later LOGIC S releases it");
}

/// `INITIAL LS` (property `Valor`) overrides the default: an initial *set*
/// switch lets the gate pass at once.
#[test]
fn initial_ls_valor_overrides_the_default() {
    let program = program(Some("Set"));
    let result = run(&program);
    let gate = block(&program, "A");
    assert_eq!(
        result.blocks[gate].retries, 0,
        "a switch initialized set must not hold the transaction"
    );
    assert_eq!(
        result.blocks[gate].current, 0,
        "the later LOGIC S releases every waiter"
    );
    assert_eq!(result.blocks[block(&program, "B")].entries, 5);
}
