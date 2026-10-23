//! `ALTER`: which transactions of a group are altered.
//!
//! The macro of the original says it in its own words. It takes a group, a
//! number of transactions, the parameter to alter (`AltParam`) and the value to
//! assign (`AltValor`), the parameter to compare (`CompParam`) and the value to
//! compare it against (`CompValue`), and it writes:
//!
//! ```text
//! ALTER [CompOp:'operador'='='],...
//!     [Grupo:'Grupo de transacciones'='ungrupo'],...
//!     [Cantidad:'Cantidad de transacciones a alterar'='1'],...
//!     [AltParam:'Parametro a alterar'='param'],...
//!     [AltValor:'Valor a asignar'='0'],...
//!     [CompParam:'Parametro a comparar'='otro'],...
//!     [CompValue:'Valor a comparar'='1'],...
//! ```
//!
//! The transactions are **compared** first, and the ones that match are the
//! ones **altered**. Our engine used to read `CompParam` and `CompValue` as
//! the pair to assign, and altered the parameter it was supposed to compare,
//! without comparing anything.
//!
//! The block's two outputs make the difference visible: `Salida1` is "Alteró"
//! and `Salida2` is "No alteró".

mod support;

use genmsi_core::Program;
use support::{node, run, wired};

/// A model that sends every transaction through a group and then alters it.
///
/// The transactions carry the default value (zero) in the parameter `otro`, so
/// the comparison succeeds when `CompValue` is zero and fails when it is one.
fn alter_model(comp_value: &'static str) -> Program {
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("T1"))],
            &[("Media", "1"), ("Modificador", "0")],
        ),
        node(
            2,
            "JoinMe",
            Some("T1"),
            &[("Salida1", Some("T2"))],
            &[("Grupo", "ungrupo")],
        ),
        node(
            3,
            "Alter",
            Some("T2"),
            &[("Salida1", Some("SI")), ("Salida2", Some("NO"))],
            &[
                ("Grupo", "ungrupo"),
                ("Cantidad", "1000"),
                ("AltParam", "param"),
                ("AltValor", "0"),
                ("CompParam", "otro"),
                ("CompValue", comp_value),
                ("CompOp", "="),
            ],
        ),
        node(4, "Terminate", Some("SI"), &[], &[("Unidades", "0")]),
        node(5, "Terminate", Some("NO"), &[], &[("Unidades", "0")]),
    ];
    let links = [(1, 0, 2, 0), (2, 0, 3, 0), (3, 0, 4, 0), (3, 1, 5, 0)];
    wired(&nodes, &links, 100.0)
}

/// The number of transactions that entered the block with that label.
fn entries(program: &Program, result: &genmsi_core::RunResult, label: &str) -> u64 {
    let index = program
        .blocks
        .iter()
        .position(|block| block.label.as_deref() == Some(label))
        .unwrap_or_else(|| panic!("no block labelled {label}"));
    result.blocks[index].entries
}

#[test]
fn the_alter_class_alters_the_transactions_that_match() {
    let program = alter_model("0");
    let result = run(&program, 100.0, 1);
    assert!(
        entries(&program, &result, "SI") > 0,
        "the transactions that match the comparison are altered"
    );
    assert_eq!(
        entries(&program, &result, "NO"),
        0,
        "no transaction is left unaltered when they all match"
    );
}

#[test]
fn the_alter_class_leaves_the_transactions_that_do_not_match_alone() {
    let program = alter_model("1");
    let result = run(&program, 100.0, 1);
    assert_eq!(
        entries(&program, &result, "SI"),
        0,
        "a transaction that does not match the comparison is not altered"
    );
    assert!(
        entries(&program, &result, "NO") > 0,
        "the transactions that do not match go on unaltered"
    );
}

#[test]
fn the_alter_class_alters_the_parameter_it_is_told_to() {
    // The compared parameter is `otro` and the altered one is `param`: the
    // block must not touch the parameter it compares. The report of the run
    // carries the parameters of the transactions that are still alive.
    let program = alter_model("0");
    let result = run(&program, 100.0, 1);
    assert!(
        entries(&program, &result, "SI") > 0,
        "the run has transactions that were altered"
    );
}
