//! The listing of the statements that the original writes and ours did not.
//!
//! Two pendings of the seventh phase, measured against the original:
//!
//! - The `ALTER` writes the destination of its failure output ("No alteró")
//!   when that output is connected. The original's own listing for the probe
//!   `AlterMiembro` is:
//!
//!   ```text
//!   A_Alter         ALTER    E       UnGrupo,1,Param,99,Param,0,A_Terminate1
//!   ```
//!
//! - The `LOGIC` block has a third operator, `I`, that inverts the state of
//!   the switch. The manual says it ("Si el operador lógico es I, el estado
//!   de la llave lógica especificada por el operando A es invertido") and the
//!   original's `op_Swich` list offers three: `S` (set), `R` (reset) and
//!   `I` (invertir).

mod support;

use genmsi_core::Program;
use genmsi_core::gpss::render;
use support::{node, run, wired};

/// The statements of a program, with their padding removed.
fn statements(program: &Program) -> Vec<String> {
    render(program)
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect()
}

/// The statement of a program that carries the name `name`.
fn named(program: &Program, name: &str) -> String {
    statements(program)
        .into_iter()
        .find(|line| {
            line.split_whitespace()
                .next()
                .is_some_and(|first| first == name)
        })
        .unwrap_or_else(|| panic!("no statement carries the name {name}"))
}

/// A model that alters a member of a group, like the probe `AlterMiembro`.
///
/// The failure output of the `Alter` ("No alteró") goes to `A_Terminate1`
/// when `failure` names it, and stays unconnected otherwise.
fn alter_model(failure: Option<&'static str>) -> Program {
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A_JoinMe"))],
            &[("Media", "1"), ("Modificador", "0")],
        ),
        node(
            2,
            "JoinMe",
            Some("A_JoinMe"),
            &[("Salida1", Some("A_Alter"))],
            &[("Grupo", "UnGrupo")],
        ),
        node(
            3,
            "Alter",
            Some("A_Alter"),
            &[("Salida1", Some("A_Terminate")), ("Salida2", failure)],
            &[
                ("Grupo", "UnGrupo"),
                ("Cantidad", "1"),
                ("AltParam", "Param"),
                ("AltValor", "99"),
                ("CompParam", "Param"),
                ("CompValue", "0"),
                ("CompOp", "="),
            ],
        ),
        node(4, "Terminate", Some("A_Terminate"), &[], &[("Unidades", "0")]),
        node(
            5,
            "Terminate",
            Some("A_Terminate1"),
            &[],
            &[("Unidades", "0")],
        ),
    ];
    let mut links = vec![(1, 0, 2, 0), (2, 0, 3, 0), (3, 0, 4, 0)];
    if failure.is_some() {
        links.push((3, 1, 5, 0));
    }
    wired(&nodes, &links, 100.0)
}

/// The `ALTER` names the destination of its failure output when the output is
/// connected: that destination is the last operand of the statement.
#[test]
fn the_alter_writes_the_destination_of_its_failure_output() {
    let program = alter_model(Some("A_Terminate1"));
    assert_eq!(
        named(&program, "A_Alter"),
        "A_Alter ALTER E UnGrupo,1,Param,99,Param,0,A_Terminate1"
    );
}

/// Without a failure output the `ALTER` keeps the six operands it had: the
/// destination is only written when there is one.
#[test]
fn the_alter_without_a_failure_output_keeps_its_six_operands() {
    let program = alter_model(None);
    assert_eq!(
        named(&program, "A_Alter"),
        "A_Alter ALTER E UnGrupo,1,Param,99,Param,0"
    );
}

/// A model with one `LOGIC` block that carries the given operator.
fn logic_model(operator: &'static str) -> Program {
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A_Logic"))],
            // One transaction: an even number of inversions would leave the
            // switch as it was.
            &[("Cantidad", "1"), ("Media", "1"), ("Modificador", "0")],
        ),
        node(
            2,
            "Logic",
            Some("A_Logic"),
            &[("Salida1", Some("A_Terminate"))],
            &[("LogicSwich", "1"), ("Op", operator)],
        ),
        node(3, "Terminate", Some("A_Terminate"), &[], &[("Unidades", "0")]),
    ];
    let links = [(1, 0, 2, 0), (2, 0, 3, 0)];
    wired(&nodes, &links, 100.0)
}

/// The third operator of the `LOGIC` block is the inversion, and the listing
/// writes it as the original does: `LOGIC I 1`.
#[test]
fn the_logic_writes_the_inversion_as_the_original_does() {
    let program = logic_model("Invertir");
    assert_eq!(named(&program, "A_Logic"), "A_Logic LOGIC I 1");
}

/// The two operators that already worked keep their letters.
#[test]
fn the_logic_keeps_the_letters_of_the_other_two_operators() {
    assert_eq!(named(&logic_model("Set"), "A_Logic"), "A_Logic LOGIC S 1");
    assert_eq!(
        named(&logic_model("Reset"), "A_Logic"),
        "A_Logic LOGIC R 1"
    );
}

/// The inversion changes the state of the switch: a switch that starts reset
/// ends set.
#[test]
fn the_logic_inverts_the_state_of_the_switch() {
    let program = logic_model("Invertir");
    let result = run(&program, 100.0, 1);
    let switch = result
        .logic_switches
        .iter()
        .find(|switch| switch.name == "1")
        .expect("the switch of the block");
    assert!(
        switch.value,
        "the switch starts reset and the inversion sets it"
    );
}
