//! The write of `genmsi layout --write`.
//!
//! The command rewrites a model the reader keeps on disk, so the write has
//! three promises:
//!
//! 1. it is **atomic**: a reader that watches the file never sees half a
//!    model (S6);
//! 2. it is **idempotent**: writing the same model twice does not change the
//!    bytes the second time (S6);
//! 3. it is **blind to the semantics**: the geometry it writes changes
//!    nothing of what the model means (S5), and it reaches a model the
//!    engine refuses to compile (S7).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

/// The root of the repository, so the paths in the documents are the same on
/// every machine.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Run the binary from the repository root.
fn genmsi(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_genmsi"))
        .current_dir(repo_root())
        .args(args)
        .output()
        .expect("the genmsi binary runs")
}

fn code(output: &Output) -> i32 {
    output.status.code().unwrap_or(-1)
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

/// A directory of its own for the model a test writes.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "genmsi-layout-write-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the scratch directory");
    dir
}

/// A model with no canvas geometry: the editor cannot draw it.
const WITHOUT_GEOMETRY: &str = r#"<gmodel versioncode="1"><gclasses/><modules>
<module moduleid="0" moduleclass="main" owner="0" caption="Modelo" name="Modelo1" kind="C">
<links><link frommoduleid="1" fromioid="1" tomoduleid="2" toioid="1"/><link frommoduleid="2" fromioid="2" tomoduleid="3" toioid="1"/></links><inputs/><outputs/>
</module>
<module moduleid="1" moduleclass="Generate" owner="0" caption="Generate" name="Generate" kind="A">
<outputs><output ioid="1" name="Salida1" caption="" mylabel="L1"/></outputs>
<props><property name="Media" hint="" value="10" type="TSNA" subType="SNA" visible="1" unique="0"/></props>
</module>
<module moduleid="2" moduleclass="Unlink" owner="0" caption="Unlink" name="Unlink" kind="A">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="L1"/></inputs>
<outputs><output ioid="2" name="Salida1" caption="" mylabel="L2"/><output ioid="3" name="SalidaAlt" caption="" mylabel=""/></outputs>
<props><property name="Media" hint="" value="10" type="TSNA" subType="SNA" visible="1" unique="0"/></props>
</module>
<module moduleid="3" moduleclass="Terminate" owner="0" caption="Terminate" name="Terminate" kind="A">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="L2"/></inputs>
<props><property name="Unidades" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props>
</module>
</modules></gmodel>"#;

/// A model that runs: the geometry is the only thing that changes.
const RUNNABLE: &str = r#"<gmodel versioncode="1"><gclasses/><modules>
<module moduleid="0" moduleclass="main" owner="0" caption="Modelo" name="Modelo1" kind="C">
<links><link frommoduleid="1" fromioid="1" tomoduleid="2" toioid="1"/></links><inputs/><outputs/>
</module>
<module moduleid="1" moduleclass="Generate" owner="0" caption="Generate" name="Generate" kind="A">
<outputs><output ioid="1" name="Salida1" caption="" mylabel="L1"/></outputs>
<props><property name="Media" hint="" value="10" type="TSNA" subType="SNA" visible="1" unique="0"/></props>
</module>
<module moduleid="2" moduleclass="Terminate" owner="0" caption="Terminate" name="Terminate" kind="A">
<inputs><input ioid="1" name="Entrada1" caption="" mylabel="L1"/></inputs>
<props><property name="Unidades" hint="" value="1" type="TSNA" subType="SNA" visible="1" unique="0"/></props>
</module>
</modules></gmodel>"#;

/// What a model says, without its canvas geometry: the blocks, their
/// properties, their ports and their links.
fn semantics(text: &str) -> Vec<String> {
    let model = genmsi_format::parse_str(text).expect("the model parses");
    model
        .blocks
        .iter()
        .map(|block| {
            let ports: Vec<String> = block
                .inputs
                .iter()
                .chain(block.outputs.iter())
                .map(|port| format!("{}:{}:{}", port.ioid, port.name, port.label().unwrap_or("")))
                .collect();
            let props: Vec<String> = block
                .props
                .iter()
                .map(|prop| format!("{}={}", prop.name, prop.value()))
                .collect();
            let links: Vec<String> = block
                .links
                .iter()
                .map(|link| {
                    format!(
                        "{}:{}->{}:{}",
                        link.from_module, link.from_ioid, link.to_module, link.to_ioid
                    )
                })
                .collect();
            format!(
                "{}|{}|{}|{}|{}|{}|{:?}|{:?}|{:?}",
                block.id,
                block.class,
                block.owner,
                block.name,
                block.kind,
                block.props.len(),
                ports,
                props,
                links
            )
        })
        .collect()
}

#[test]
fn the_write_is_idempotent() {
    let dir = scratch("idempotent");
    let path = dir.join("model.gmdl");
    std::fs::write(&path, WITHOUT_GEOMETRY).expect("the model is written");

    let first = genmsi(&["layout", path.to_str().expect("a path"), "--write"]);
    assert_eq!(code(&first), 0, "{}", stderr(&first));
    let text = stdout(&first);
    assert!(text.contains("status: composed"), "{text}");
    assert!(text.contains("written: yes"), "{text}");
    let after_first = std::fs::read(&path).expect("the model is there");

    let second = genmsi(&["layout", path.to_str().expect("a path"), "--write"]);
    assert_eq!(code(&second), 0, "{}", stderr(&second));
    let text = stdout(&second);
    // The second write has nothing to compose: the model is composed now.
    assert!(text.contains("status: unchanged"), "{text}");
    assert!(text.contains("written: no"), "{text}");
    let after_second = std::fs::read(&path).expect("the model is there");

    assert_eq!(
        after_second, after_first,
        "writing the same model twice changed the bytes"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_reader_never_sees_half_a_model() {
    let dir = scratch("atomic");
    let path = dir.join("model.gmdl");
    std::fs::write(&path, WITHOUT_GEOMETRY).expect("the model is written");

    let stop = Arc::new(AtomicBool::new(false));
    let broken = Arc::new(AtomicUsize::new(0));
    let reads = Arc::new(AtomicUsize::new(0));
    let watcher = {
        let path = path.clone();
        let stop = Arc::clone(&stop);
        let broken = Arc::clone(&broken);
        let reads = Arc::clone(&reads);
        thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                let Ok(bytes) = std::fs::read(&path) else {
                    continue;
                };
                reads.fetch_add(1, Ordering::Relaxed);
                let text = String::from_utf8_lossy(&bytes).to_string();
                let complete = text.trim_end().ends_with("</gmodel>")
                    && genmsi_format::parse_str(&text).is_ok();
                if !complete {
                    broken.fetch_add(1, Ordering::Relaxed);
                }
            }
        })
    };

    // The write happens again and again: the model goes back to its
    // geometry-less shape and the command composes it once more.
    for _ in 0..40 {
        let temporary = dir.join(".model.gmdl.restored");
        std::fs::write(&temporary, WITHOUT_GEOMETRY).expect("the model is written");
        std::fs::rename(&temporary, &path).expect("the model is put back");
        let output = genmsi(&["layout", path.to_str().expect("a path"), "--write"]);
        assert_eq!(code(&output), 0, "{}", stderr(&output));
    }
    stop.store(true, Ordering::Relaxed);
    watcher.join().expect("the watcher ends");

    assert!(
        reads.load(Ordering::Relaxed) > 0,
        "the watcher never read the model"
    );
    assert_eq!(
        broken.load(Ordering::Relaxed),
        0,
        "a reader saw half a model"
    );
    // The temporary file of the write does not stay behind.
    assert!(
        !dir.join(".model.gmdl.tmp").exists(),
        "the temporary file of the write stayed behind"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_model_the_engine_refuses_is_composed_and_written() {
    let dir = scratch("refused");
    let path = dir.join("model.gmdl");
    std::fs::write(&path, WITHOUT_GEOMETRY).expect("the model is written");

    // The engine refuses this model: the UNLINK has no label for the
    // unlinked transactions.
    let gpss = genmsi(&["gpss", path.to_str().expect("a path")]);
    assert_eq!(code(&gpss), 1, "the model should not compile");
    assert!(
        stderr(&gpss).contains("the operand B must not be null"),
        "{}",
        stderr(&gpss)
    );

    // The composition is a matter of geometry: it reaches the model anyway.
    let output = genmsi(&["layout", path.to_str().expect("a path"), "--write"]);
    assert_eq!(code(&output), 0, "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.contains("status: composed"), "{text}");
    assert!(text.contains("written: yes"), "{text}");

    // The write keeps the semantics of the model: the same blocks, the same
    // properties, the same ports and the same links.
    let written = std::fs::read_to_string(&path).expect("the model was written");
    assert_eq!(
        semantics(&written),
        semantics(WITHOUT_GEOMETRY),
        "the write changed the semantics of the model"
    );
    // And the model is still the model the engine refuses.
    let gpss = genmsi(&["gpss", path.to_str().expect("a path")]);
    assert_eq!(code(&gpss), 1, "the model should not compile");
    assert!(
        stderr(&gpss).contains("the operand B must not be null"),
        "{}",
        stderr(&gpss)
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_geometry_does_not_change_the_semantics() {
    let dir = scratch("semantics");
    let path = dir.join("model.gmdl");
    std::fs::write(&path, RUNNABLE).expect("the model is written");

    let gpss_before = genmsi(&["gpss", path.to_str().expect("a path")]);
    assert_eq!(code(&gpss_before), 0, "{}", stderr(&gpss_before));
    let report_before = genmsi(&[
        "simulate",
        "--json",
        "--seed",
        "42",
        path.to_str().expect("a path"),
    ]);
    assert_eq!(code(&report_before), 0, "{}", stderr(&report_before));

    let output = genmsi(&["layout", path.to_str().expect("a path"), "--write"]);
    assert_eq!(code(&output), 0, "{}", stderr(&output));
    assert!(stdout(&output).contains("written: yes"), "{}", stdout(&output));

    let gpss_after = genmsi(&["gpss", path.to_str().expect("a path")]);
    assert_eq!(code(&gpss_after), 0, "{}", stderr(&gpss_after));
    assert_eq!(
        stdout(&gpss_after),
        stdout(&gpss_before),
        "the GPSS program changed when the canvas was composed"
    );
    let report_after = genmsi(&[
        "simulate",
        "--json",
        "--seed",
        "42",
        path.to_str().expect("a path"),
    ]);
    assert_eq!(code(&report_after), 0, "{}", stderr(&report_after));
    assert_eq!(
        stdout(&report_after),
        stdout(&report_before),
        "the report changed when the canvas was composed"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_write_that_fails_leaves_the_model_alone() {
    let dir = scratch("failure");
    let path = dir.join("model.gmdl");
    std::fs::write(&path, WITHOUT_GEOMETRY).expect("the model is written");
    let before = std::fs::read(&path).expect("the model is there");
    // The temporary file of the write cannot be written: it is a directory.
    std::fs::create_dir(dir.join(".model.gmdl.tmp")).expect("the obstacle is there");

    let output = genmsi(&["layout", path.to_str().expect("a path"), "--write"]);
    assert_eq!(code(&output), 1);
    assert!(
        stderr(&output).starts_with("error: "),
        "{}",
        stderr(&output)
    );
    assert_eq!(
        std::fs::read(&path).expect("the model is there"),
        before,
        "a write that failed changed the model"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
