//! The facade hands the flag of the cancellation to the engine.
//!
//! The flag is optional: a run without it is the ordinary run, the one the
//! CLI and the oracle make, and it comes out exactly the same.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use genmsi_app_engine::{run_model_documents_bytes, simulate_model_bytes};
use genmsi_format::parse_str;

/// A model that keeps the engine busy for a while: a generator that makes a
/// transaction every hundredth of a second, and a clock of half an hour.
///
/// The model has no `Terminate` operand, so the run lasts until the clock
/// ends it: the engine has hundreds of thousands of events to work on, and
/// the flag has time to be raised in the middle of them.
fn busy() -> genmsi_format::GmdlModel {
    let xml = r#"<gmodel versioncode="1"><gclasses/><modules>
<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C"><links>
<link frommoduleid="1" fromioid="2" tomoduleid="2" toioid="1"/>
</links><inputs/><outputs/></module>
<module moduleid="1" moduleclass="Generate" owner="0" caption="Generate" name="Generate" kind="A" width="10" height="10" x="0" y="0"><inputs/><outputs><output ioid="2" name="Salida1" caption="" mylabel="T1"/></outputs><props><property name="Media" hint="" value="0.01" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="2" moduleclass="Terminate" owner="0" caption="Terminate" name="Terminate" kind="A" width="10" height="10" x="0" y="0"><inputs><input ioid="1" name="Entrada1" caption="" mylabel="T1"/></inputs><outputs/><props><property name="Unidades" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="3" moduleclass="Clock" owner="0" caption="Clock" name="Clock" kind="A" width="10" height="10" x="0" y="0"><props><property name="Frecuencia" hint="" value="1800" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
</modules></gmodel>"#;
    parse_str(xml).expect("the model parses")
}

/// The flag of the cancellation, and the thread that raises it.
fn raiser(delay: Duration) -> (Arc<AtomicBool>, thread::JoinHandle<()>) {
    let flag = Arc::new(AtomicBool::new(false));
    let raised = flag.clone();
    let handle = thread::spawn(move || {
        thread::sleep(delay);
        raised.store(true, Ordering::Relaxed);
    });
    (flag, handle)
}

#[test]
fn a_run_with_the_flag_ends_with_the_reason_of_the_cancellation() {
    let model = busy();

    // The whole run: how long the engine takes when nobody stops it.
    let whole_start = Instant::now();
    let whole = simulate_model_bytes(&model, 1, None, None).expect("runs");
    let whole_elapsed = whole_start.elapsed();
    let whole_report: serde_json::Value = serde_json::from_str(&whole).expect("parses");
    assert_eq!(whole_report["end_reason"], "terminated");

    // The same run, with the reader raising the flag in the middle of it.
    let (flag, handle) = raiser(Duration::from_millis(20));
    let cancelled_start = Instant::now();
    let cancelled = simulate_model_bytes(&model, 1, None, Some(flag)).expect("runs");
    let cancelled_elapsed = cancelled_start.elapsed();
    handle.join().expect("the flag is raised");

    let report: serde_json::Value = serde_json::from_str(&cancelled).expect("parses");
    assert_eq!(
        report["end_reason"], "cancelled",
        "the run did not end for the reason of the cancellation"
    );
    assert!(
        whole_elapsed > Duration::from_millis(50),
        "the whole run is too short to be cancelled in the middle: {whole_elapsed:?}"
    );
    assert!(
        cancelled_elapsed * 2 < whole_elapsed,
        "the cancelled run did not end much earlier: {cancelled_elapsed:?} against {whole_elapsed:?}"
    );
    // The result of a cancelled run is partial: it holds less of the run.
    assert!(
        report["transactions"].as_u64() < whole_report["transactions"].as_u64(),
        "the cancelled run holds the whole run: {:?}",
        report["transactions"]
    );
    assert!(
        report["events"].as_u64() < whole_report["events"].as_u64(),
        "the cancelled run processed the events of the whole run: {:?}",
        report["events"]
    );
}

#[test]
fn the_documents_of_a_run_with_the_flag_say_the_run_was_cancelled() {
    let model = busy();

    let (flag, handle) = raiser(Duration::from_millis(20));
    let documents = run_model_documents_bytes(&model, 1, None, 100, Some(flag)).expect("runs");
    handle.join().expect("the flag is raised");

    let report: serde_json::Value = serde_json::from_str(&documents.report).expect("parses");
    let trace: serde_json::Value = serde_json::from_str(&documents.trace).expect("parses");
    assert_eq!(report["end_reason"], "cancelled");
    assert_eq!(trace["end_reason"], "cancelled");
}

#[test]
fn a_run_without_the_flag_is_the_run_it_was() {
    let model = busy();

    // The run without a flag, and the run with a flag nobody raises: the two
    // are the same run, byte for byte.
    let plain = simulate_model_bytes(&model, 1, None, None).expect("runs");
    let flag = Arc::new(AtomicBool::new(false));
    let with_flag = simulate_model_bytes(&model, 1, None, Some(flag)).expect("runs");

    assert_eq!(plain, with_flag, "the flag changed the run");
}
