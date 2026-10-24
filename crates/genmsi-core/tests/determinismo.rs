//! Una corrida tiene que ser función del programa y la semilla (decisión D5).
//!
//! El motor guardaba los espereros de las puertas en un `HashMap<usize,
//! Vec<XactId>>` y los recorría en el orden del hash, y **el orden de un
//! `HashMap` es aleatorio por proceso**: la misma corrida con la misma
//! semilla podía dar dos reportes distintos. El modelo
//! `TP6__Clase 6_2_UltimoCliente` lo muestra: tiene dos puertas (`GateL1`, que
//! mira la llave lógica `Puerta`, y `GateS`, que mira el almacén `Salon`) y el
//! número de intentos fallidos de la llave lógica salía 207 o 230 según el
//! proceso que hacía la corrida.
//!
//! Los tests de aquí:
//!
//! - la misma corrida con la misma semilla, repetida en el mismo proceso, da
//!   el mismo reporte;
//! - dos procesos distintos dan el mismo reporte;
//! - un modelo **sin** puertas no cambia: sus números son los de antes de la
//!   corrección, porque el recorrido de las puertas no lo toca.

use std::path::{Path, PathBuf};
use std::process::Command;

use genmsi_core::Program;
use genmsi_core::report::render;
use genmsi_core::{SimConfig, Simulation};

/// The model with the two gates: the one whose report changed with the order
/// of the hash.
const MODEL_WITH_GATES: &str = "TP6__Clase 6_2_UltimoCliente.gmdl";

/// A model without gates: its numbers must not move.
const MODEL_WITHOUT_GATES: &str = "ejercicio_1__ejercicio_1.gmdl";

/// The seed of every run of this file.
const SEED: u64 = 1;

/// The root of the repository, from the crate of the core.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Compile one of the fixture models.
fn compile_fixture(name: &str) -> Program {
    let path = root().join("fixtures/models").join(name);
    let model = genmsi_format::load(&path).expect("the model loads");
    genmsi_core::compile_model(&model).expect("the model compiles")
}

/// Run a program with the seed of this file and render its report.
fn report_of(program: &Program) -> String {
    let result = Simulation::new(
        program,
        SimConfig {
            seed: SEED,
            ..SimConfig::default()
        },
    )
    .run()
    .expect("the model runs");
    render(&result, program)
}

/// The same run, repeated in the same process, gives the same report.
///
/// The engine builds a fresh table of waiters on every run, and the seed of
/// the hash of a table is not the same in two tables: before the correction
/// the order of the gates (and with it the report) changed from one run to the
/// next. Sixteen runs are enough: the chance of sixteen runs giving the same
/// report by luck is one in thirty thousand.
#[test]
fn la_misma_corrida_repetida_da_el_mismo_reporte() {
    let program = compile_fixture(MODEL_WITH_GATES);
    let first = report_of(&program);
    for run in 1..16 {
        let again = report_of(&program);
        assert_eq!(
            first, again,
            "the run {run} of the same program with the same seed gave another report"
        );
    }
}

/// Two processes give the same report.
///
/// The order of the hash is random *per process*, so the repetition of the
/// previous test does not cover it: this one runs the model in two child
/// processes (the test binary itself, twice) and compares what they print.
#[test]
fn dos_procesos_dan_el_mismo_reporte() {
    let first = report_of_a_child_process();
    let second = report_of_a_child_process();
    assert_eq!(
        first, second,
        "two processes ran the same program with the same seed and gave another report"
    );
}

/// The report of the model, printed by a child process.
///
/// The child is this same test binary, with the environment variable that
/// asks the helper test to print the report of the model.
fn report_of_a_child_process() -> String {
    let output = Command::new(std::env::current_exe().expect("the test binary is here"))
        .args(["--exact", "la_corrida_del_proceso_hijo", "--nocapture"])
        .env("GENMSI_CHILD_REPORT", "1")
        .output()
        .expect("the child process runs");
    assert!(
        output.status.success(),
        "the child process failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let start = stdout.find(START_MARKER).expect("the child printed the report");
    let end = stdout[start..]
        .find(END_MARKER)
        .expect("the child printed the end of the report")
        + start;
    stdout[start + START_MARKER.len()..end].to_string()
}

/// The marks that say where the report of the child begins and ends.
const START_MARKER: &str = "-----INICIO DEL REPORTE-----";
const END_MARKER: &str = "-----FIN DEL REPORTE-----";

/// The helper of the test of the two processes.
///
/// Without the environment variable it does nothing: the test binary runs it
/// as any other test.
#[test]
fn la_corrida_del_proceso_hijo() {
    if std::env::var("GENMSI_CHILD_REPORT").is_err() {
        return;
    }
    let program = compile_fixture(MODEL_WITH_GATES);
    println!("{START_MARKER}");
    print!("{}", report_of(&program));
    println!("{END_MARKER}");
}

/// A model without gates keeps its numbers.
///
/// The correction touches the order in which the engine walks the gates with
/// waiters. A model without gates has no such waiters: its run goes through
/// the same code and comes out with the same numbers as before.
#[test]
fn un_modelo_sin_puertas_no_cambia() {
    let program = compile_fixture(MODEL_WITHOUT_GATES);
    let report = report_of(&program);
    // The numbers of the run of `ejercicio_1__ejercicio_1`, measured before
    // the correction of the order of the gates.
    assert!(
        report.contains("        0         3600        11           1            0"),
        "the horizon of the run moved:\n{report}"
    );
    assert!(
        report.contains("    1       64       1     0              1  GENERATE        60,30"),
        "the number of transactions of the run moved:\n{report}"
    );
}
