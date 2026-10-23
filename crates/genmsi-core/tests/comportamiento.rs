//! The behaviour of the original that the first lot of the phase seven left
//! apart: the indicator of a chain, the `CURRENT` of an `ASSEMBLE`, the two
//! statements of a `MATCH`, the rows of a `SEIZE` and of its facility, and the
//! last transaction of a run.
//!
//! Every rule here comes from the reports of the original `genmsi.exe` under
//! Wine (`scripts/wine-oracle/`), which are quoted in the comment of each
//! test.

mod support;

use genmsi_core::report::render;
use support::{block, node, run, wired};

/// The row of the block table of the report that names `label`.
fn row(report: &str, label: &str) -> Vec<String> {
    report
        .lines()
        .find(|line| line.contains(&format!(" {label} ")) || line.contains(&format!(" {label}$")))
        .unwrap_or_default()
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

#[test]
fn the_link_sends_the_first_transaction_to_the_destination() {
    // The original (`scripts/wine-oracle/benches/Link.gmdl` and its report):
    //   2       56      55     0         A_Link  LINK            Cadena,FIFO,A_Terminate
    //   3        1       0     0    A_Terminate  TERMINATE
    //   USER_CHAIN       CHAIN_SIZE   RETRY   AVE.CONT   ENTRIES    MAX     AVE.TIME
    //     CADENA                55       0     26,466        55     55     1732,291
    //
    // The first transaction that reaches the chain finds its indicator off:
    // it goes on to the destination of the statement and does not join the
    // chain. The ones that find the indicator on wait in it. The average
    // time of the chain is the integral of its size over its entries.
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
            "Link",
            Some("A"),
            &[("Salida1", Some("B"))],
            &[("Cadena", "C")],
        ),
        node(3, "Terminate", Some("B"), &[], &[]),
    ];
    let links = [(1, 0, 2, 0), (2, 0, 3, 0)];
    let program = wired(&nodes, &links, 3600.0);
    let result = run(&program, 100.0, 1);

    let link = block(&program, "A");
    let terminate = block(&program, "B");
    // Both transactions reach the link, and only the first one goes on.
    assert_eq!(result.blocks[link].entries, 2);
    assert_eq!(result.blocks[terminate].entries, 1);
    // The second one waits in the chain.
    assert_eq!(result.blocks[link].current, 1);
    let chain = &result.entities.chains["C"];
    assert_eq!(chain.content, 1);
    assert_eq!(chain.entries, 1);
    // The average time of the chain is not zero: it is the integral of its
    // size over its entries (the original says 1732,291 in its own run).
    assert!(chain.average_time() > 0.0, "{}", chain.average_time());
}

#[test]
fn the_assemble_gives_the_transactions_back_when_the_set_is_complete() {
    // The original (`scripts/wine-oracle/benches/Assemble.gmdl` and its
    // report):
    //   2       58       0     0     A_Assemble  ASSEMBLE        1
    // The `CURRENT` of the block is zero: the transactions leave it when the
    // set is complete.
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
            "Assemble",
            Some("A"),
            &[("Salida1", Some("B"))],
            &[("Cantidad", "1")],
        ),
        node(3, "Terminate", Some("B"), &[], &[]),
    ];
    let links = [(1, 0, 2, 0), (2, 0, 3, 0)];
    let program = wired(&nodes, &links, 3600.0);
    let result = run(&program, 100.0, 1);

    let assemble = block(&program, "A");
    assert_eq!(result.blocks[assemble].entries, 2);
    assert_eq!(result.blocks[assemble].current, 0);
    assert_eq!(result.blocks[block(&program, "B")].entries, 2);
}

#[test]
fn the_match_is_two_statements_and_the_entries_are_in_the_transfer() {
    // The original (`scripts/wine-oracle/benches/Match.gmdl` and its
    // report):
    //   2       66      66     0        A_Match  TRANSFER        ,AAA
    //   3        0      66     0            AAA  MATCH           AAA
    // The class expands into a `TRANSFER` and a `MATCH`: the report lists the
    // two, and the entries are in the transfer.
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
            "Match",
            Some("A"),
            &[("Salida1", Some("B")), ("Salida2", None)],
            &[("RotuloPropio", "AAA"), ("RotuloMatch", "AAA")],
        ),
        node(3, "Terminate", Some("B"), &[], &[]),
    ];
    let links = [(1, 0, 2, 0), (2, 0, 3, 0)];
    let program = wired(&nodes, &links, 3600.0);
    let result = run(&program, 100.0, 1);

    let transfer = block(&program, "A");
    let matching = block(&program, "AAA");
    // The transfer holds the entries, the match holds the ones that wait.
    assert_eq!(result.blocks[transfer].entries, 2);
    assert_eq!(result.blocks[transfer].current, 2);
    assert_eq!(result.blocks[matching].entries, 0);
    assert_eq!(result.blocks[matching].current, 2);

    let report = render(&result, &program);
    let transfer_row = row(&report, "A");
    let match_row = row(&report, "AAA");
    assert!(transfer_row.contains(&"TRANSFER".to_string()), "{transfer_row:?}");
    assert!(match_row.contains(&"MATCH".to_string()), "{match_row:?}");
}

#[test]
fn the_seize_row_counts_the_seizures_and_the_waiters_wait_where_they_came_from() {
    // The original (`scripts/wine-oracle/probes/almacen/facilidadContencion`
    // and its report):
    //   2      120     114     0  A_FacilityUse  QUEUE           Cola
    //   3        6       0     0              3  SEIZE           Recurso
    //   FACILITY       ENTRIES  UTIL.  AVE_TIME  AVAILABLE  OWNER PEND INTER RETRY DELAY
    //     RECURSO           6   0,992  595,000       1         7    0     0     0   114
    //
    // The `SEIZE` row counts the seizures that got the facility, its
    // `RETRY` stays at zero, and the transactions that wait are counted in
    // the block they came from. The facility holds them in its `DELAY`.
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A"))],
            &[("Media", "10"), ("Modificador", "0"), ("Cantidad", "3")],
        ),
        node(
            2,
            "FacilityUse",
            Some("A"),
            &[("Salida1", Some("B"))],
            &[
                ("Facility", "Recurso"),
                ("Media", "100"),
                ("Modificador", "0"),
                ("Queue", "Cola"),
            ],
        ),
        node(3, "Terminate", Some("B"), &[], &[]),
    ];
    let links = [(1, 0, 2, 0), (2, 0, 3, 0)];
    let program = wired(&nodes, &links, 3600.0);
    let result = run(&program, 50.0, 1);
    let report = render(&result, &program);

    // The blocks of the facility use: the queue, the seize, the depart, the
    // advance and the release.
    let seize = program
        .blocks
        .iter()
        .position(|block| block.class == "Seize")
        .expect("the facility use has a seize");
    let queue = program
        .blocks
        .iter()
        .position(|block| block.class == "Queue")
        .expect("the facility use has a queue");
    assert_eq!(result.blocks[seize].entries, 1, "only the seizure counts");
    assert_eq!(result.blocks[seize].current, 0);
    assert_eq!(result.blocks[seize].retries, 0);
    assert_eq!(result.blocks[queue].current, 2, "the waiters are in the queue");

    // The facility holds the waiters in its `DELAY` column, and `PEND` is
    // zero.
    let facility = report
        .lines()
        .find(|line| line.contains("RECURSO"))
        .unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>();
    assert_eq!(facility.last(), Some(&"2"), "{facility:?}");
    assert_eq!(facility[facility.len() - 2], "0", "{facility:?}");
}

#[test]
fn the_run_counts_the_arrival_of_the_hour_in_which_it_ends() {
    // The original (`scripts/wine-oracle/probes/almacen/almacenCola2` and
    // its report):
    //   2      121       1     0              1  GENERATE        30,0
    //   3      120     108     0     StorageUse  QUEUE           qAlmacen,2
    //   QUEUE            MAX  CONT. ENTRIES ENTRIES(0) AVE.CONT.   AVE.TIME AVE.(-0) RETRY
    //     QALMACEN       216   216     120         2   105,300   3159,000 3212,542    0
    //
    // The `ENTRY COUNT` of a generator holds the arrival that is already on
    // its way, and the run counts the arrival of the hour in which it ends:
    // the original processes the events of that hour before it stops.
    let nodes = [
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A"))],
            &[("Media", "30"), ("Modificador", "0")],
        ),
        node(
            2,
            "Queue",
            Some("A"),
            &[("Salida1", Some("B"))],
            &[("Queue", "Cola")],
        ),
        node(3, "Terminate", Some("B"), &[], &[]),
    ];
    let links = [(1, 0, 2, 0), (2, 0, 3, 0)];
    let program = wired(&nodes, &links, 3600.0);
    let result = run(&program, 4000.0, 1);

    // The generator made 120 transactions and has one more on its way.
    let generator = block(&program, "Generate");
    assert_eq!(result.blocks[generator].entries, 121);
    assert_eq!(result.blocks[generator].current, 1);
    // The arrival of the hour in which the run ends is counted.
    assert_eq!(result.entities.queues["Cola"].entries, 120);
    assert_eq!(result.end_time, 3600.0);
}
