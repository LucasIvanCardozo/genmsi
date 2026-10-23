//! The three classes the original declares and the catalogue did not offer
//! until now: `MFunction`, `MFunctionDist` and `RemoveNumber`.
//!
//! The original's own answer is the specification of each one:
//!
//! - the `MFunction` class defines several functions over the same input
//!   attribute, one row of its matrix per function. The original writes one
//!   `FUNCTION` statement per name, with the values of the row:
//!
//!   ```
//!   FuncA           FUNCTION        P$Param,E3
//!   1,11/2,12/3,13
//!   FuncB           FUNCTION        P$Param,E3
//!   1,21/2,22/3,23
//!   ```
//!
//! - the `MFunctionDist` class defines several distribution functions, one
//!   row of its matrix of results per function. The original writes one
//!   `FUNCTION` statement per name, with the running sum of the
//!   probabilities of the row:
//!
//!   ```
//!   Func1           FUNCTION        RN4,D3
//!   0.2,1/0.5,2/1,3
//!   Func2           FUNCTION        RN4,D3
//!   0.25,1/0.6,2/1,3
//!   ```
//!
//! - the `RemoveNumber` class asks a numeric group for a number. The
//!   original writes `REMOVE E <grupo>,,<número>` (and the destination of
//!   the failure output as the seventh operand when that output is
//!   connected). The listing of this class cannot be pinned yet: the
//!   statement needs an arm of its own in `crates/genmsi-core/src/gpss.rs`,
//!   which is not part of this work. What the engine does with the number
//!   is in `odd/tasks/notas-w3-clases.md`.

mod support;

use genmsi_core::Program;
use genmsi_core::gpss::render;
use support::{block, node, run, wired};

/// The statements of a program, with their padding removed.
fn statements(program: &Program) -> Vec<String> {
    render(program)
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect()
}

/// The `MFunction` class with the defaults of the original's class library.
fn mfunction() -> Program {
    let nodes = [
        node(
            1,
            "MFunction",
            None,
            &[],
            &[
                ("NombresFunc", "%7BFuncA%3BFuncB%7D"),
                ("Atributo", "P%3AParam"),
                ("Matriz", "%7B%7B11%3B12%3B13%7D%3B%7B21%3B22%3B23%7D%7D"),
            ],
        ),
    ];
    wired(&nodes, &[], 3600.0)
}

/// The `MFunctionDist` class with the defaults of the original's class
/// library.
fn mfunctiondist() -> Program {
    let nodes = [
        node(
            1,
            "MFunctionDist",
            None,
            &[],
            &[
                ("NombresDist", "%7BFunc1%3BFunc2%7D"),
                ("Semilla", "4"),
                (
                    "Resultados",
                    "%7B%0A%7B20%25%3B30%25%3B50%25%7D%3B%0A%7B25%25%3B35%25%3B40%25%7D%0A%7D",
                ),
            ],
        ),
    ];
    wired(&nodes, &[], 3600.0)
}

#[test]
fn the_mfunction_writes_a_function_for_every_name_of_its_list() {
    // The original's own listing of the bench of the class:
    //
    //   FuncA           FUNCTION        P$Param,E3
    //   1,11/2,12/3,13
    //   FuncB           FUNCTION        P$Param,E3
    //   1,21/2,22/3,23
    assert_eq!(
        statements(&mfunction()),
        vec![
            "FuncA FUNCTION P$Param,E3",
            "1,11/2,12/3,13",
            "FuncB FUNCTION P$Param,E3",
            "1,21/2,22/3,23",
            "GENERATE 3600",
            "TERMINATE 1",
        ]
    );
}

#[test]
fn the_mfunction_dist_writes_a_distribution_for_every_name_of_its_list() {
    // The original's own listing of the bench of the class:
    //
    //   Func1           FUNCTION        RN4,D3
    //   0.2,1/0.5,2/1,3
    //   Func2           FUNCTION        RN4,D3
    //   0.25,1/0.6,2/1,3
    assert_eq!(
        statements(&mfunctiondist()),
        vec![
            "Func1 FUNCTION RN4,D3",
            "0.2,1/0.5,2/1,3",
            "Func2 FUNCTION RN4,D3",
            "0.25,1/0.6,2/1,3",
            "GENERATE 3600",
            "TERMINATE 1",
        ]
    );
}

/// A model that reads a function and tells the value it gave.
///
/// The `MFunction` class takes the input attribute (`P:Param` by default) and
/// the row of the matrix is the value of the function for each input: the
/// first column is the input `1`, the second the input `2`, and so on.
fn reader(function: &'static str, expected: &'static str) -> Program {
    let nodes = [
        node(
            1,
            "MFunction",
            None,
            &[],
            &[
                ("NombresFunc", "%7BFuncA%3BFuncB%7D"),
                ("Atributo", "P%3AParam"),
                ("Matriz", "%7B%7B11%3B12%3B13%7D%3B%7B21%3B22%3B23%7D%7D"),
            ],
        ),
        node(
            2,
            "Generate",
            None,
            &[("Salida1", Some("A"))],
            &[("Media", "10"), ("Modificador", "0"), ("Cantidad", "1")],
        ),
        // The input of the function: the first column of the row.
        node(
            3,
            "Assign",
            Some("A"),
            &[("Salida1", Some("B"))],
            &[("Parametro", "Param"), ("Operador", "%3D"), ("Valor", "1")],
        ),
        // The value the function gives for that input.
        node(
            4,
            "Assign",
            Some("B"),
            &[("Salida1", Some("C"))],
            &[
                ("Parametro", "Param"),
                ("Operador", "%3D"),
                ("Valor", function),
            ],
        ),
        node(
            5,
            "Test",
            Some("C"),
            &[("Salida1", Some("T")), ("Salida2", Some("F"))],
            &[("1_Valor", "P%3AParam"), ("2_OP", "%3D"), ("3_Valor", expected)],
        ),
        node(6, "Terminate", Some("T"), &[], &[("Unidades", "0")]),
        node(7, "Terminate", Some("F"), &[], &[("Unidades", "0")]),
    ];
    let links = [(2, 0, 3, 0), (3, 0, 4, 0), (4, 0, 5, 0), (5, 0, 6, 0), (5, 1, 7, 0)];
    wired(&nodes, &links, 3600.0)
}

#[test]
fn the_mfunction_function_gives_the_value_of_the_row_of_its_name() {
    // `FuncA` is the first row of the matrix: the input `1` gives `11`.
    let program = reader("FN:FuncA", "11");
    let result = run(&program, 100.0, 7);
    assert_eq!(
        result.blocks[block(&program, "T")].entries,
        1,
        "the transaction takes the true branch of the test"
    );
    assert_eq!(
        result.blocks[block(&program, "F")].entries,
        0,
        "the value of the function is the one of the row"
    );

    // `FuncB` is the second row: the input `1` gives `21`.
    let program = reader("FN:FuncB", "21");
    let result = run(&program, 100.0, 7);
    assert_eq!(
        result.blocks[block(&program, "T")].entries,
        1,
        "the transaction takes the true branch of the test"
    );
    assert_eq!(
        result.blocks[block(&program, "F")].entries,
        0,
        "each function reads the row of its own name"
    );
}

/// A model that reads a distribution function and counts what it gives.
fn counter() -> Program {
    let nodes = [
        node(
            1,
            "MFunctionDist",
            None,
            &[],
            &[
                ("NombresDist", "%7BFunc1%3BFunc2%7D"),
                ("Semilla", "4"),
                (
                    "Resultados",
                    "%7B%0A%7B20%25%3B30%25%3B50%25%7D%3B%0A%7B25%25%3B35%25%3B40%25%7D%0A%7D",
                ),
            ],
        ),
        node(
            2,
            "Generate",
            None,
            &[("Salida1", Some("A"))],
            &[("Media", "1"), ("Modificador", "0"), ("Cantidad", "1000")],
        ),
        node(
            3,
            "Assign",
            Some("A"),
            &[("Salida1", Some("B"))],
            &[
                ("Parametro", "Param"),
                ("Operador", "%3D"),
                ("Valor", "FN:Func1"),
            ],
        ),
        // Three tests, one per result of the distribution.
        node(
            4,
            "Test",
            Some("B"),
            &[("Salida1", Some("T1")), ("Salida2", Some("B2"))],
            &[("1_Valor", "P%3AParam"), ("2_OP", "%3D"), ("3_Valor", "1")],
        ),
        node(
            5,
            "Test",
            Some("B2"),
            &[("Salida1", Some("T2")), ("Salida2", Some("B3"))],
            &[("1_Valor", "P%3AParam"), ("2_OP", "%3D"), ("3_Valor", "2")],
        ),
        node(
            6,
            "Test",
            Some("B3"),
            &[("Salida1", Some("T3")), ("Salida2", Some("T4"))],
            &[("1_Valor", "P%3AParam"), ("2_OP", "%3D"), ("3_Valor", "3")],
        ),
        node(7, "Terminate", Some("T1"), &[], &[("Unidades", "0")]),
        node(8, "Terminate", Some("T2"), &[], &[("Unidades", "0")]),
        node(9, "Terminate", Some("T3"), &[], &[("Unidades", "0")]),
        node(10, "Terminate", Some("T4"), &[], &[("Unidades", "0")]),
    ];
    let links = [
        (2, 0, 3, 0),
        (3, 0, 4, 0),
        (4, 0, 7, 0),
        (4, 1, 5, 0),
        (5, 0, 8, 0),
        (5, 1, 6, 0),
        (6, 0, 9, 0),
        (6, 1, 10, 0),
    ];
    wired(&nodes, &links, 3600.0)
}

#[test]
fn the_mfunction_dist_function_follows_the_probabilities_of_its_row() {
    // The first row of the matrix of results is `{20%;30%;50%}`: the
    // function gives `1` a fifth of the time, `2` three tenths and `3` half
    // of the time.
    let program = counter();
    let result = run(&program, 3600.0, 7);
    let one = result.blocks[block(&program, "T1")].entries as f64;
    let two = result.blocks[block(&program, "T2")].entries as f64;
    let three = result.blocks[block(&program, "T3")].entries as f64;
    let total = one + two + three;
    assert_eq!(total, 1000.0, "every transaction reads the function");
    assert!(
        (one / total - 0.2).abs() < 0.05,
        "the first result comes out {one} times out of {total}"
    );
    assert!(
        (two / total - 0.3).abs() < 0.05,
        "the second result comes out {two} times out of {total}"
    );
    assert!(
        (three / total - 0.5).abs() < 0.05,
        "the third result comes out {three} times out of {total}"
    );
}

/// The `RemoveNumber` class of the original, with the defaults of its class
/// library.
fn removenumber(failure: bool) -> Program {
    let mut nodes = vec![
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A"))],
            &[("Media", "60"), ("Modificador", "30"), ("Cantidad", "2")],
        ),
        node(
            2,
            "RemoveNumber",
            Some("A"),
            &[
                ("Salida1", Some("B")),
                ("Salida2", failure.then_some("F")),
            ],
            &[("Op", "%3D"), ("Grupo", "UnGrupo"), ("Numero", "0")],
        ),
        node(3, "Terminate", Some("B"), &[], &[("Unidades", "0")]),
    ];
    let mut links = vec![(1, 0, 2, 0), (2, 0, 3, 0)];
    if failure {
        nodes.push(node(4, "Terminate", Some("F"), &[], &[("Unidades", "0")]));
        links.push((2, 1, 4, 0));
    }
    wired(&nodes, &links, 3600.0)
}

#[test]
fn the_remove_number_writes_a_remove_statement() {
    // The original writes `REMOVE E UnGrupo,,0` (and the destination of the
    // failure output as the seventh operand when that output is
    // connected). The statement has an arm of its own in
    // `crates/genmsi-core/src/gpss.rs`: the mode, the group, an empty limit
    // and the number.
    assert_eq!(
        statements(&removenumber(false)),
        vec![
            "GENERATE 60,30,,,2",
            "A_RemoveNumber REMOVE E UnGrupo,,0",
            "A_Terminate TERMINATE",
            "GENERATE 3600",
            "TERMINATE 1",
        ]
    );
}

#[test]
fn the_remove_number_lets_the_transaction_continue() {
    // The number is not in the group and the failure output is not
    // connected: the transaction goes on by the first output.
    let program = removenumber(false);
    let result = run(&program, 3600.0, 7);
    assert_eq!(
        result.blocks[block(&program, "B")].entries,
        2,
        "both transactions go on by the first output"
    );
}

#[test]
fn the_remove_number_sends_the_transaction_to_the_failure_output() {
    // The failure output of the class is "No existe el Nº": the number is
    // not in the group, so the transaction goes there.
    let program = removenumber(true);
    let result = run(&program, 3600.0, 7);
    assert_eq!(
        result.blocks[block(&program, "F")].entries,
        2,
        "the transactions go to the failure output"
    );
    assert_eq!(
        result.blocks[block(&program, "B")].entries,
        0,
        "and not to the first output"
    );
}
