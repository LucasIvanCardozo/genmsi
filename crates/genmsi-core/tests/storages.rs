//! Storage semantics: the capacity of a storage, the units one transaction
//! takes, and what happens when the model never declares the storage.
//!
//! The oracle is the original `genmsi.exe` under Wine (`scripts/wine-oracle`),
//! which was run on three hand-built models with the same shape as the ones
//! below:
//!
//! - `StorageDef almacen Tamaño=1` + `StorageUse almacen Cantidad=1`:
//!   the original reports `ALMACEN  1  0  0  1  6  1  0,992  0,992  0  114`
//!   (capacity 1, maximum occupancy 1, 114 refusals) and `END_TIME 3600`.
//! - `Tamaño=2`, `Cantidad=1`: capacity 2, maximum occupancy 2.
//! - `Tamaño=1`, `Cantidad=2`: capacity 1, maximum occupancy 0, 120 refusals:
//!   two units do not fit in a storage of one unit, so nobody gets in.
//!
//! - An `ENTER` of a storage that no `StorageDef` declares stops the original
//!   with `ERROR!!Referencia a una entidad storage inexistente. (Storage: -1).`
//!   The engine refuses the same model.

mod support;

use genmsi_core::SimConfig;
use genmsi_core::program::compile;
use genmsi_core::report::render;
use genmsi_core::{Simulation, program::Program};
use genmsi_format::parse_str;
use support::{Node, node, run, try_wired};

/// A model with a generator, a `StorageUse` and a `StorageDef`.
///
/// `capacity` is the `Tamaño` of the storage (its capacity, in units) and
/// `units` the `Cantidad` of the `StorageUse` (the units one transaction
/// takes and gives back).
fn storage_model(capacity: &'static str, units: &'static str, hold: &'static str) -> Vec<Node> {
    vec![
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A_StorageUse"))],
            &[("Media", "10"), ("Modificador", "0")],
        ),
        node(
            2,
            "StorageUse",
            Some("A_StorageUse"),
            &[("Salida1", Some("A_Terminate"))],
            &[
                ("Storage", "almacen"),
                ("Cantidad", units),
                ("Media", hold),
                ("Modificador", "0"),
                ("Queue", "qAlmacen"),
            ],
        ),
        node(
            3,
            "Terminate",
            Some("A_Terminate"),
            &[],
            &[("Unidades", "0")],
        ),
        node(
            4,
            "StorageDef",
            None,
            &[],
            &[("Storage", "almacen"), ("Tamaño", capacity)],
        ),
    ]
}

fn links() -> Vec<(u32, usize, u32, usize)> {
    vec![(1, 0, 2, 0), (2, 0, 3, 0)]
}

/// A storage of one unit holds one transaction at a time.
///
/// `Tamaño` is the capacity, in units, and `Cantidad` the units one
/// transaction takes: with one of each, the storage admits one transaction
/// and the others wait. This is what the user expects from a storage of one.
#[test]
fn a_storage_of_one_unit_holds_one_transaction_at_a_time() {
    let nodes = storage_model("1", "1", "100");
    let program = try_wired(&nodes, &links(), 1_000.0).expect("the model compiles");
    let result = run(&program, 1_000.0, 7);
    let storage = &result.entities.storages["almacen"];
    assert_eq!(storage.capacity, 1, "the `Tamaño` is the capacity, in units");
    assert_eq!(
        storage.max_use, 1,
        "two transactions were inside the storage at the same time"
    );
    assert!(storage.entries > 0, "no transaction ever got in");
    // The original's storage row counts no refusal: its `RETRY` column is
    // zero and the transactions that wait are in its `DELAY` column.
    assert_eq!(storage.retry, 0, "the storage counted a refusal");
    assert!(
        storage.delay > 0,
        "the transactions that did not fit are not counted as waiting"
    );
}

/// The units one transaction takes are the `Cantidad`.
///
/// A storage of two units with a `Cantidad` of one holds two transactions at
/// the same time; a storage of one unit with a `Cantidad` of two holds
/// nobody, because two units do not fit in one.
#[test]
fn the_units_of_a_transaction_are_the_cantidad_operand() {
    let nodes = storage_model("2", "1", "100");
    let program = try_wired(&nodes, &links(), 1_000.0).expect("the model compiles");
    let result = run(&program, 1_000.0, 7);
    let storage = &result.entities.storages["almacen"];
    assert_eq!(storage.capacity, 2);
    assert_eq!(
        storage.max_use, 2,
        "two units of capacity hold two transactions of one unit"
    );
    assert!(storage.entries > 0);

    let nodes = storage_model("1", "2", "100");
    let program = try_wired(&nodes, &links(), 1_000.0).expect("the model compiles");
    let result = run(&program, 1_000.0, 7);
    let storage = &result.entities.storages["almacen"];
    assert_eq!(storage.capacity, 1);
    assert_eq!(
        storage.max_use, 0,
        "a transaction of two units entered a storage of one unit"
    );
    assert_eq!(storage.entries, 0, "a transaction of two units got in");
    assert_eq!(storage.retry, 0, "the storage counted a refusal");
    assert!(
        storage.delay > 0,
        "the transactions that did not fit are not counted as waiting"
    );
}

/// The queue of a storage counts units, not transactions.
///
/// The original compiles the `StorageUse` as `ARRIVE [Queue],[Cantidad]` and
/// `DEPART [Queue],[Cantidad]`: the `Cantidad` is the number of places the
/// transaction takes in the queue. Its report agrees: a `StorageUse` with a
/// `Cantidad` of 2 and a queue of 120 transactions shows a queue of 240
/// places (`scripts/wine-oracle/probes/almacen`).
#[test]
fn the_queue_of_a_storage_counts_units() {
    let nodes = storage_model("4", "2", "100");
    let program = try_wired(&nodes, &links(), 1_000.0).expect("the model compiles");
    let result = run(&program, 1_000.0, 7);
    let queue = &result.entities.queues["qAlmacen"];
    let storage = &result.entities.storages["almacen"];
    assert!(
        queue.entries > storage.entries,
        "the queue must hold the transactions that did not get in"
    );
    assert_eq!(
        queue.content,
        2 * (queue.entries - storage.entries),
        "the queue must count two places per waiting transaction"
    );
    assert_eq!(
        queue.max % 2,
        0,
        "the queue's maximum must be a number of places"
    );
}

/// The `ENTER` row counts the entries that got in.
///
/// The original's `ENTER` row holds the entries that were admitted and
/// nothing else: the transactions that wait are in the block they came from
/// (in a `StorageUse`, the queue in front of the storage) and the refusals
/// are not counted anywhere in the block table.
#[test]
fn the_enter_row_counts_the_entries_that_got_in() {
    let nodes = storage_model("1", "1", "100");
    let program = try_wired(&nodes, &links(), 1_000.0).expect("the model compiles");
    let result = run(&program, 1_000.0, 7);
    let storage = &result.entities.storages["almacen"];
    let enter = program
        .blocks
        .iter()
        .position(|block| block.class == "Enter")
        .expect("the model has an `ENTER` block");
    let enter_stats = &result.blocks[enter];
    assert_eq!(
        enter_stats.entries, storage.entries,
        "the `ENTER` row must count the entries that got in"
    );
    assert_eq!(
        enter_stats.current, 0,
        "the `ENTER` row must not hold the transactions that wait"
    );
    assert_eq!(
        enter_stats.retries, 0,
        "the `ENTER` row must not count the refusals"
    );

    // The transactions that wait are counted in the queue they came from.
    let queue_block = program
        .blocks
        .iter()
        .position(|block| block.class == "Queue")
        .expect("the model has a `QUEUE` block");
    assert_eq!(
        result.blocks[queue_block].current, storage.delay,
        "the waiters must be counted in the queue in front of the storage"
    );
}

/// A lone `ENTER` counts its waiters in the block they came from.
///
/// The original's report of a model with a plain `Enter` (no queue) shows the
/// transactions that wait in the row of the block before it, and its `ENTER`
/// row with no `CURRENT` at all.
#[test]
fn a_lone_enter_counts_its_waiters_where_they_came_from() {
    let nodes = vec![
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A_Enter"))],
            &[("Media", "10"), ("Modificador", "0")],
        ),
        node(
            2,
            "Enter",
            Some("A_Enter"),
            &[("Salida1", Some("A_Terminate"))],
            &[("Storage", "almacen"), ("Cantidad", "1")],
        ),
        node(
            3,
            "Terminate",
            Some("A_Terminate"),
            &[],
            &[("Unidades", "0")],
        ),
        node(
            4,
            "StorageDef",
            None,
            &[],
            &[("Storage", "almacen"), ("Tamaño", "1")],
        ),
    ];
    let program = try_wired(&nodes, &links(), 1_000.0).expect("the model compiles");
    let result = run(&program, 1_000.0, 7);
    let storage = &result.entities.storages["almacen"];
    let enter = program
        .blocks
        .iter()
        .position(|block| block.class == "Enter")
        .expect("the model has an `ENTER` block");
    assert_eq!(result.blocks[enter].entries, storage.entries);
    assert_eq!(result.blocks[enter].current, 0);
    // The generator holds the transactions that wait and the creation that is
    // still scheduled.
    let generator = program
        .blocks
        .iter()
        .position(|block| block.class == "Generate")
        .expect("the model has a generator");
    assert_eq!(
        result.blocks[generator].current as u64,
        storage.delay + 1,
        "the waiters must be counted in the block they came from"
    );
}

/// A storage the model does not declare is not a storage.
///
/// The original's `StorageUse` says it: "El Storage debe estar declarado (esta
/// clase no lo declara)". The original opens the model and stops the
/// simulation with "Referencia a una entidad storage inexistente"; the engine
/// compiles the model, says what is wrong, and stops the run at the same
/// place.
#[test]
fn a_storage_the_model_does_not_declare_is_reported_and_stops_the_run() {
    let mut nodes = storage_model("1", "1", "100");
    // Drop the `StorageDef`: the `StorageUse` now names a storage that no
    // declaration gives a capacity to.
    nodes.retain(|node| node.class != "StorageDef");
    let program = try_wired(&nodes, &links(), 1_000.0).expect("the model still compiles");
    let diagnostic = program
        .diagnostics
        .iter()
        .find(|message| message.contains("almacen"))
        .expect("the compiler names the storage");
    assert!(diagnostic.contains("StorageDef"), "{diagnostic}");

    let error = Simulation::new(
        &program,
        SimConfig {
            seed: 7,
            stop_time: Some(1_000.0),
            ..SimConfig::default()
        },
    )
    .run()
    .expect_err("the run must stop at the undeclared storage");
    let message = error.to_string();
    assert!(
        message.contains("almacen"),
        "the refusal does not name the storage: {message}"
    );
    assert!(
        message.contains("StorageDef"),
        "the refusal does not say what to do: {message}"
    );
}

/// The same refusal, for a name the model only resolves while it runs.
///
/// An indirect name (`*Param`) cannot be checked when the model compiles, so
/// the engine stops the run when the name turns out to be undeclared.
#[test]
fn an_undeclared_storage_found_while_the_model_runs_stops_the_run() {    let nodes = vec![
        node(
            1,
            "Generate",
            None,
            &[("Salida1", Some("A_Assign"))],
            &[("Media", "1"), ("Modificador", "0")],
        ),
        node(
            2,
            "Assign",
            Some("A_Assign"),
            &[("Salida1", Some("A_Enter"))],
            &[("Parametro", "Almacen"), ("Operador", "="), ("Valor", "2")],
        ),
        node(
            3,
            "Enter",
            Some("A_Enter"),
            &[("Salida1", Some("A_Terminate"))],
            &[("Storage", "*Almacen"), ("Cantidad", "1")],
        ),
        node(
            4,
            "Terminate",
            Some("A_Terminate"),
            &[],
            &[("Unidades", "0")],
        ),
        // Only the storage "1" is declared: the run asks for the storage "2".
        node(
            5,
            "StorageDef",
            None,
            &[],
            &[("Storage", "1"), ("Tamaño", "5")],
        ),
    ];
    let program = try_wired(&nodes, &links(), 1_000.0).expect("the model compiles");
    let error = Simulation::new(
        &program,
        SimConfig {
            seed: 7,
            stop_time: Some(1_000.0),
            ..SimConfig::default()
        },
    )
    .run()
    .expect_err("the run must stop at the undeclared storage");
    let message = error.to_string();
    assert!(
        message.contains("'2'"),
        "the refusal does not name the storage: {message}"
    );
}

/// The `Clock` spends one unit of the `START` every `Frecuencia` time units.
///
/// The original's `Clock` says: "Descuenta 1 unidad del START de la simulación
/// cada un cierto tiempo indicado". Its report ends at that time, and the
/// report says so.
#[test]
fn the_clock_spends_one_unit_of_start_per_frequency() {
    for frequency in [250.0, 1_000.0, 3_600.0] {
        let nodes = vec![
            node(
                1,
                "Generate",
                None,
                &[("Salida1", Some("A_Terminate"))],
                &[("Media", "10"), ("Modificador", "0")],
            ),
            node(
                2,
                "Terminate",
                Some("A_Terminate"),
                &[],
                &[("Unidades", "0")],
            ),
        ];
        let program = try_wired(&nodes, &[(1, 0, 2, 0)], frequency).expect("the model compiles");
        let result = run(&program, frequency * 10.0, 7);
        assert_eq!(
            result.end_time, frequency,
            "the run must end one `Frecuencia` after the start"
        );
        let report = render(&result, &program);
        let end_time = report
            .lines()
            .find(|line| line.trim_start().starts_with(|c: char| c.is_ascii_digit()))
            .expect("the report has a header line with the times");
        assert!(
            end_time.contains(&genmsi_core::report::number(frequency)),
            "the report does not show the end time: {end_time}"
        );
    }
}

/// A model that uses an undeclared storage says so, and the compiler does not
/// hide it.
#[test]
fn a_model_that_uses_an_undeclared_storage_says_so() {
    let xml = r#"<gmodel versioncode="1"><gclasses/><modules>\
<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C"><links>\
<link frommoduleid="1" fromioid="1" tomoduleid="2" toioid="1"/>\
</links><inputs/><outputs/></module>\
<module moduleid="1" moduleclass="StorageUse" owner="0" caption="s" name="s" kind="A" width="10" height="10" x="0" y="0">\
<inputs><input ioid="1" name="i" caption="" mylabel="G"/></inputs>\
<outputs><output ioid="2" name="Salida1" caption="" mylabel="T"/></outputs>\
<props><property name="Storage" hint="" value="almacen" type="EntityUse" subType="Storage" visible="1" unique="0"/><property name="Cantidad" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/><property name="Media" hint="" value="10" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>\
<module moduleid="2" moduleclass="Terminate" owner="0" caption="t" name="t" kind="A" width="10" height="10" x="0" y="0">\
<inputs><input ioid="1" name="i" caption="" mylabel="T"/></inputs>\
<props><property name="Unidades" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>\
</modules></gmodel>"#;
    let model = parse_str(xml).expect("the model parses");
    let program = compile(&model, "reported").expect("the model compiles");
    let message = program
        .diagnostics
        .iter()
        .find(|message| message.contains("almacen"))
        .expect("the compiler names the storage");
    assert!(message.contains("StorageDef"), "{message}");
}

/// The helper above compiles a model that the compiler accepts.
#[test]
fn a_declared_storage_compiles() {
    let nodes = storage_model("1", "1", "100");
    let program: Program = try_wired(&nodes, &links(), 1_000.0).expect("the model compiles");
    assert_eq!(program.definitions.storages.get("almacen"), Some(&1));
}
