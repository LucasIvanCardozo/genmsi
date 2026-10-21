//! Entity operands: literal names and the GPSS `*parameter` indirect form.

mod support;

use support::{linear, module, run};

/// A `FacilityUse`/`FacilityInt`/`StorageUse` whose `Queue` operand starts with
/// `*` takes the queue name from a parameter, exactly like its `Facility` and
/// `Storage` operands do. The original renders it as `QUEUE *Bondi` and reports
/// one queue per parameter value.
///
/// `Parcial 6-11-23` needs this: `QUEUE *Bondi` must become queues `1`..`10`
/// (one per bus), and the baseline reports exactly those queues.
#[test]
fn indirect_queue_operand_resolves_through_the_parameter() {
    let program = linear(
        &[
            module(
                "Generate",
                "Gen",
                &[("Media", "1"), ("Modificador", "0"), ("Cantidad", "5")],
            ),
            module(
                "Assign",
                "A_Bus",
                &[("Parametro", "Bus"), ("Operador", "="), ("Valor", "7")],
            ),
            module(
                "FacilityUse",
                "F",
                &[
                    ("Facility", "cajero"),
                    ("Queue", "*Bus"),
                    ("Media", "2"),
                    ("Modificador", "0"),
                ],
            ),
            module("Terminate", "T", &[("Unidades", "0")]),
        ],
        100.0,
    );
    let result = run(&program, 100.0, 5);
    let mut names: Vec<&str> = result.entities.queues.keys().map(String::as_str).collect();
    names.sort();
    assert_eq!(
        names,
        vec!["7"],
        "the indirect queue must be named after the parameter value"
    );
}
