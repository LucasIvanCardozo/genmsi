//! Transaction groups: `JOIN`, `REMOVE` and `SCAN`.

mod support;

use genmsi_core::Program;
use support::{block, node, run, wired};

/// A group of two waiting transactions plus one transaction that removes a
/// member. The members wait in a user chain, so any output port would be visible.
fn model() -> Program {
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A"))],
            &[("Media", "10"), ("Modificador", "0"), ("Cantidad", "2")],
        ),
        node(
            2,
            "JoinMe",
            Some("A"),
            &[("Salida1", Some("L"))],
            &[("Grupo", "G")],
        ),
        node(3, "Link", Some("L"), &[("Salida1", None)], &[("Cadena", "C")]),
        node(
            4,
            "Generate",
            None,
            &[("Salida1", Some("R"))],
            &[("Media", "50"), ("Modificador", "0"), ("Cantidad", "1")],
        ),
        node(
            5,
            "RemoveTrans",
            Some("R"),
            &[("Salida1", Some("T")), ("Salida2", None)],
            &[
                ("Grupo", "G"),
                ("Cantidad", "1"),
                ("Modo", "Algunos"),
                ("CompOp", "="),
                ("CompValue", "1"),
            ],
        ),
        node(6, "Terminate", Some("T"), &[], &[("Unidades", "0")]),
    ];
    let links = [
        (1, 0, 2, 0),
        (2, 0, 3, 0),
        (4, 0, 5, 0),
        (5, 0, 6, 0),
    ];
    wired(&nodes, &links, 100.0)
}

/// `RemoveTrans` detaches a member from the group; it does not resume it. The
/// removed transaction keeps waiting (it is still in its user chain) until an
/// `UNLINK` releases it — resuming it here makes it run the remover's own output
/// path, so every block after `REMOVE` sees twice the traffic.
///
/// `TP10` pairs `RemoveTrans` with the `Unlink` that follows it and the baseline
/// counts agree only under the "do not resume" reading: `A_RemoveTrans1` and
/// `A_Unlink1` both see 2584 entries, not 2584 + 2584.
#[test]
fn remove_trans_does_not_resume_the_removed_member() {
    let program = model();
    let result = run(&program, 100.0, 7);
    let remove = block(&program, "R");
    let terminate = block(&program, "T");
    let link = block(&program, "L");
    assert_eq!(result.blocks[remove].entries, 1, "one remover");
    assert_eq!(
        result.blocks[terminate].entries, 1,
        "only the remover continues; the removed member stays in its chain"
    );
    assert_eq!(
        result.blocks[link].current, 2,
        "both members are still linked"
    );
}
