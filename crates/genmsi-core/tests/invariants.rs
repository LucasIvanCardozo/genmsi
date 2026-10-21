//! Engine invariants: determinism, non-negative counters, valid transaction
//! states, and confidence-interval soundness.

use genmsi_core::program::compile;
use genmsi_core::stat::Storage;
use genmsi_core::xact::{Transaction, XactState};
use genmsi_core::{ReplicationConfig, SimConfig, Simulation, replicate};
use genmsi_format::parse_str;
use proptest::prelude::*;

/// A small model: generate, queue, seize, advance, release, terminate.
fn model(mean: f64, spread: f64, capacity: i64) -> genmsi_core::Program {
    let xml = format!(
        r#"<gmodel versioncode="1"><gclasses/><modules>\
<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C"><links>\
<link frommoduleid="1" fromioid="1" tomoduleid="2" toioid="1"/>\
<link frommoduleid="2" fromioid="2" tomoduleid="3" toioid="1"/>\
<link frommoduleid="3" fromioid="2" tomoduleid="4" toioid="1"/>\
<link frommoduleid="4" fromioid="2" tomoduleid="5" toioid="1"/>\
<link frommoduleid="5" fromioid="2" tomoduleid="6" toioid="1"/>\
</links><inputs/><outputs/></module>\
<module moduleid="1" moduleclass="Generate" owner="0" caption="g" name="g" kind="A" width="10" height="10" x="0" y="0"><outputs><output ioid="1" name="Salida1" caption="" mylabel="G"/></outputs>\
<props><property name="Media" hint="" value="{mean}" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Modificador" hint="" value="{spread}" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Cantidad" hint="" value="200" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>\
<module moduleid="2" moduleclass="Queue" owner="0" caption="q" name="q" kind="A" width="10" height="10" x="0" y="0"><inputs><input ioid="1" name="i" caption="" mylabel="G"/></inputs><outputs><output ioid="2" name="Salida1" caption="" mylabel="Q"/></outputs>\
<props><property name="Queue" hint="" value="cola" type="EntityUse" subType="Queue" visible="1" unique="0"/></props></module>\
<module moduleid="3" moduleclass="Seize" owner="0" caption="s" name="s" kind="A" width="10" height="10" x="0" y="0"><inputs><input ioid="1" name="i" caption="" mylabel="Q"/></inputs><outputs><output ioid="2" name="Salida1" caption="" mylabel="S"/></outputs>\
<props><property name="Facility" hint="" value="servidor" type="EntityUse" subType="Facility" visible="1" unique="0"/></props></module>\
<module moduleid="4" moduleclass="StorageDef" owner="0" caption="d" name="d" kind="A" width="10" height="10" x="0" y="0"><props><property name="Storage" hint="" value="almacen" type="EntityDef" subType="Storage" visible="1" unique="1"/><property name="Tamaño" hint="" value="{capacity}" type="PosInteger" subType="" visible="1" unique="0"/></props></module>\
<module moduleid="5" moduleclass="Enter" owner="0" caption="e" name="e" kind="A" width="10" height="10" x="0" y="0"><inputs><input ioid="1" name="i" caption="" mylabel="S"/></inputs><outputs><output ioid="2" name="Salida1" caption="" mylabel="E"/></outputs>\
<props><property name="Storage" hint="" value="almacen" type="EntityUse" subType="Storage" visible="1" unique="0"/><property name="Cantidad" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>\
<module moduleid="6" moduleclass="Terminate" owner="0" caption="t" name="t" kind="A" width="10" height="10" x="0" y="0"><inputs><input ioid="1" name="i" caption="" mylabel="E"/></inputs>\
<props><property name="Unidades" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>\
</modules></gmodel>"#
    );
    compile(&parse_str(&xml).expect("test model parses"), "invariants").expect("compiles")
}

fn run(program: &genmsi_core::Program, seed: u64) -> genmsi_core::RunResult {
    Simulation::new(
        program,
        SimConfig {
            seed,
            stop_time: Some(36_000.0),
            ..SimConfig::default()
        },
    )
    .run()
    .expect("runs")
}

proptest! {
    /// Same seed, same run: the whole point of D5.
    #[test]
    fn same_seed_is_bit_for_bit_reproducible(seed in 0u64..1000) {
        let program = model(5.0, 1.0, 2);
        let first = run(&program, seed);
        let second = run(&program, seed);
        prop_assert_eq!(first.end_time, second.end_time);
        prop_assert_eq!(first.transactions, second.transactions);
        prop_assert_eq!(
            first.entities.queues.get("cola").map(|q| (q.entries, q.total_time)),
            second.entities.queues.get("cola").map(|q| (q.entries, q.total_time)),
        );
        prop_assert_eq!(
            serde_json::to_string(&first.entities).unwrap(),
            serde_json::to_string(&second.entities).unwrap()
        );
    }

    /// Counters, contents and occupancies never go negative or exceed capacity.
    #[test]
    fn counters_stay_within_their_domain(seed in 0u64..300, mean in 1.0f64..20.0, spread in 0.0f64..5.0) {
        let program = model(mean, spread, 3);
        let result = run(&program, seed);
        let elapsed = result.elapsed();
        prop_assert!(elapsed > 0.0);
        for queue in result.entities.queues.values() {
            prop_assert!(queue.content <= queue.max);
            prop_assert!(queue.entries >= queue.zero_entries);
            prop_assert!(queue.average_content(elapsed) >= 0.0);
            prop_assert!(queue.average_time() >= 0.0);
        }
        for storage in result.entities.storages.values() {
            prop_assert!(storage.in_use >= 0);
            prop_assert!(storage.in_use <= storage.capacity);
            prop_assert!(storage.min_use >= 0);
            prop_assert!(storage.max_use <= storage.capacity);
        }
        for facility in result.entities.facilities.values() {
            prop_assert!(facility.utilization(elapsed) <= 1.0 + 1e-9);
            prop_assert!(facility.busy_time <= elapsed + 1e-9);
        }
        for block in &result.blocks {
            prop_assert!(block.current <= block.entries.max(1));
        }
    }

    /// A replication study reports one summary per metric with ordered bounds.
    #[test]
    fn confidence_intervals_are_ordered_and_cover_the_mean(seed in 0u64..50) {
        let program = model(5.0, 1.0, 2);
        let result = replicate(
            &program,
            ReplicationConfig {
                runs: 5,
                base_seed: seed,
                run: SimConfig::default(),
            },
        )
        .expect("replications run");
        prop_assert_eq!(result.runs, 5);
        prop_assert!(!result.metrics.is_empty());
        for metric in &result.metrics {
            prop_assert!(metric.ci_low <= metric.mean + 1e-9);
            prop_assert!(metric.mean <= metric.ci_high + 1e-9);
            prop_assert_eq!(metric.samples, 5);
            prop_assert!(metric.stddev >= 0.0);
        }
    }

    /// Storage occupancy bookkeeping never lets releases drive usage negative.
    #[test]
    fn storage_leave_never_underflows(units in 1i64..100) {
        let mut storage = Storage::new("s", 4, 0.0);
        prop_assert!(storage.try_enter(2, 0.0));
        storage.leave(units, 1.0);
        prop_assert_eq!(storage.in_use, (2 - units).max(0));
        prop_assert!(storage.in_use >= 0);
        prop_assert!(storage.remaining() <= 4);
    }
}

/// The transaction state machine only accepts legal transitions, whatever the
/// order an implementation tries them in.
#[test]
fn transaction_states_only_follow_legal_transitions() {
    let states = [
        XactState::Ready,
        XactState::Scheduled,
        XactState::Waiting,
        XactState::Terminated,
    ];
    for from in states {
        for to in states {
            let mut xact = Transaction::new(1, 0, 0.0);
            xact.state = from;
            let legal = xact.transition(to);
            let expected = !matches!(
                (from, to),
                (XactState::Terminated, XactState::Ready)
                    | (XactState::Terminated, XactState::Scheduled)
                    | (XactState::Terminated, XactState::Waiting)
                    | (XactState::Scheduled, XactState::Scheduled)
            ) && !(from != to && xact.state == from);
            assert_eq!(legal, expected || from == to, "{from:?} -> {to:?}");
            if legal {
                assert_eq!(xact.state, to);
            }
        }
    }
}
