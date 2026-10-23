//! The GPSS program of a compiled model, against the original's own answer.
//!
//! The original editor shows the program it compiles in its "Código GPSS"
//! tab. The benches under `scripts/wine-oracle/benches` are one model per
//! class, and the original's answer for each of them is what these tests
//! check: the verb, the operands and the name of every statement.

use std::path::{Path, PathBuf};

use genmsi_core::gpss::render;
use genmsi_core::program::Program;

/// Compile the bench of a class, the way the CLI does.
fn bench(class: &str) -> Program {
    let path = bench_path(class);
    let model = genmsi_format::load(&path).expect("the bench loads");
    genmsi_core::compile_model(&model).expect("the bench compiles")
}

fn bench_path(class: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/wine-oracle/benches")
        .join(format!("{class}.gmdl"))
}

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

/// The first statement of a program that starts with `prefix`.
fn starting(program: &Program, prefix: &str) -> String {
    statements(program)
        .into_iter()
        .find(|line| line.starts_with(prefix))
        .unwrap_or_else(|| panic!("no statement starts with {prefix}"))
}

#[test]
fn the_gates_write_a_gate_with_their_condition() {
    assert_eq!(named(&bench("GateF"), "GateF"), "GateF GATE NU Recurso");
    assert_eq!(named(&bench("GateL"), "GateL"), "GateL GATE LS 1");
    assert_eq!(named(&bench("GateS"), "GateS"), "GateS GATE SNF Almacen");
}

#[test]
fn the_transfer_names_its_statement_after_its_input_port() {
    assert_eq!(
        named(&bench("Transfer"), "Entrada1"),
        "Entrada1 TRANSFER 1000,,A_Terminate"
    );
}

#[test]
fn the_split_names_its_statement_after_what_it_does() {
    assert_eq!(named(&bench("Split"), "Clona"), "Clona SPLIT 1");
}

#[test]
fn the_storage_use_keeps_the_name_of_its_module() {
    assert_eq!(
        named(&bench("StorageUse"), "StorageUse"),
        "StorageUse QUEUE qAlmacen"
    );
}

#[test]
fn the_counter_writes_a_savevalue_and_an_assignment() {
    let program = bench("Counter");
    let statements = statements(&program);
    assert!(
        statements.contains(&"A_Counter SAVEVALUE Count+,1".to_string()),
        "{statements:#?}"
    );
    assert!(
        statements.contains(&"ASSIGN Nro,X$Count".to_string()),
        "{statements:#?}"
    );
}

#[test]
fn the_match_writes_the_transfer_to_its_conjugate() {
    let program = bench("Match");
    let statements = statements(&program);
    assert!(
        statements.contains(&"A_Match TRANSFER ,AAA".to_string()),
        "{statements:#?}"
    );
    assert!(
        statements.contains(&"AAA MATCH AAA".to_string()),
        "{statements:#?}"
    );
}

#[test]
fn two_blocks_of_the_same_class_take_different_names() {
    let program = bench("Unlink");
    let statements = statements(&program);
    assert!(
        statements.contains(&"A_Terminate TERMINATE".to_string()),
        "{statements:#?}"
    );
    assert!(
        statements.contains(&"A_Terminate1 TERMINATE".to_string()),
        "{statements:#?}"
    );
}

#[test]
fn the_definitions_are_written_before_the_blocks() {
    let program = bench("Function");
    let gpss = render(&program);
    let function = gpss.find("FUNCTION").expect("a function");
    let generate = gpss.find("GENERATE").expect("a generator");
    assert!(function < generate, "{gpss}");
    assert!(
        gpss.contains("Func            FUNCTION        P$Param,D2"),
        "{gpss}"
    );
    // The points of the function are written on a line of their own.
    assert!(gpss.contains("\n1,1/2,2\n"), "{gpss}");
}

#[test]
fn an_expression_is_written_with_the_dollar_of_gpss() {
    assert_eq!(
        named(&bench("Table"), "Tabla"),
        "Tabla TABLE P$Atrib,0,10,10"
    );
}

#[test]
fn the_initial_values_are_written_without_a_name() {
    assert_eq!(starting(&bench("InitiaSV"), "INITIAL"), "INITIAL X1,0");
    assert_eq!(starting(&bench("InitialLS"), "INITIAL"), "INITIAL LS1");
}

#[test]
fn the_blocks_write_the_operands_of_their_class() {
    for (class, expected) in [
        ("Alter", "A_Alter ALTER E ungrupo,1,param,0,otro,1"),
        ("Assemble", "A_Assemble ASSEMBLE 1"),
        ("Gather", "A_Gather GATHER 1"),
        ("ExamineNumber", "A_ExamineNumber EXAMINE UnGrupo,0,"),
        ("JoinMe", "A_JoinMe JOIN UnGrupo"),
        ("JoinNumber", "A_JoinNumber JOIN UnGrupo,0"),
        ("Link", "A_Link LINK Cadena,FIFO,A_Terminate"),
        ("Logic", "A_Logic LOGIC S 1"),
        ("Loop", "A_Loop LOOP Param,"),
        ("RemoveTrans", "A_RemoveTrans REMOVE E UnGrupo,1,,otro,1"),
        ("SaveValue", "A_SaveValue SAVEVALUE 1,0"),
        ("Scan", "A_Scan SCAN E UnGrupo,otro,1,Param,Param"),
        ("Select", "A_Select SELECT MIN param,1,10,,Q"),
        ("SelectF", "A_SelectF SELECT NU param,1,10"),
        ("Tabulate", "A_Tabulate TABULATE Tabla"),
        ("Test", "A_Test TEST E 1,1"),
        ("Unlink", "A_Unlink UNLINK E Cadena,A_Terminate1,1,Otro,0"),
        ("Variable", "vari VARIABLE 0"),
        (
            "VariableA",
            "Azar VARIABLE ((RN80#1000)+RN80)@(1+(10)-(1))+(1)",
        ),
    ] {
        let program = bench(class);
        let name = expected.split_whitespace().next().unwrap();
        assert_eq!(named(&program, name), expected, "the bench of {class}");
    }
}
