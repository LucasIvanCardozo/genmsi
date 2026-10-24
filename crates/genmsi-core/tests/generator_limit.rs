//! What `Cantidad` limits: the total the generator creates.
//!
//! The catalogue publishes the description of the class, and the description
//! says what `Cantidad` does. The engine has to do what the description says,
//! because the description is the only thing an agent reads before it writes
//! the block:
//!
//! - an **empty** value creates transactions forever;
//! - a number limits the total;
//! - **zero** creates none.
//!
//! The reference models agree with the first two: 98 of the `Cantidad`
//! properties of the twenty-five models are empty, and none of them is zero.

use genmsi_core::{Program, RunResult, SimConfig, Simulation, program::compile};
use genmsi_format::load_str;

/// A generator with the given `Cantidad`, a terminate that takes nothing from
/// the counter of the run, and a clock that ends the run.
fn program(cantidad: &str) -> Program {
    let xml = format!(
        r#"<gmodel versioncode="1"><gclasses/><modules>
<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C"><links>
<link frommoduleid="1" fromioid="1" tomoduleid="2" toioid="1"/>
</links><inputs/><outputs/></module>
<module moduleid="1" moduleclass="Generate" owner="0" caption="g" name="g" kind="A"><outputs><output ioid="1" name="Salida1" caption="" mylabel="T"/></outputs><props><property name="Media" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Cantidad" hint="" value="{cantidad}" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="2" moduleclass="Terminate" owner="0" caption="t" name="t" kind="A"><inputs><input ioid="1" name="Entrada1" caption="" mylabel="T"/></inputs><props><property name="Unidades" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="3" moduleclass="Clock" owner="0" caption="c" name="c" kind="A"><props><property name="Frecuencia" hint="" value="3600" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
</modules></gmodel>"#
    );
    compile(&load_str(&xml).expect("the model parses"), "generator").expect("the model compiles")
}

/// Run the model with a fixed seed.
fn run(program: &Program) -> RunResult {
    Simulation::new(
        program,
        SimConfig {
            seed: 1,
            ..SimConfig::default()
        },
    )
    .run()
    .expect("the model runs")
}

/// The transactions the generator made.
///
/// The count is the generator's own row of the report: the clock is a
/// generator too, and its one transaction is not the model's.
fn created(result: &RunResult) -> u64 {
    result.blocks[0].entries
}

#[test]
fn an_empty_limit_creates_transactions_forever() {
    let result = run(&program(""));
    assert!(
        created(&result) > 100,
        "the generator keeps creating transactions: {}",
        created(&result)
    );
}

#[test]
fn a_limit_of_zero_creates_none() {
    let result = run(&program("0"));
    assert_eq!(
        created(&result),
        0,
        "a limit of zero is a total of zero, not a generator without a limit"
    );
}

#[test]
fn a_number_limits_the_total() {
    let result = run(&program("7"));
    assert_eq!(
        created(&result),
        7,
        "the generator made what it was told"
    );
}
