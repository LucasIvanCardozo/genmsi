//! The `START` of the run travels from the request to the engine.
//!
//! The `START` is the counter of terminations of the run: the original has it
//! in the panel of its simulator ("Start:", one by default) and the model
//! does not declare it, so it is a parameter of the run, like the horizon.
//! The app asks for it with the rest of the knobs of the run, and the
//! protocol carries it to the engine.

use std::path::{Path, PathBuf};

use genmsi_app_engine::{Session, SessionRequest, answer};

/// A generator of one transaction every ten units and a terminate that takes
/// one unit from the `START` of the run.
const MODEL: &str = "<gmodel versioncode=\"1\"><gclasses/><modules>
<module moduleid=\"0\" moduleclass=\"main\" owner=\"0\" caption=\"m\" name=\"m\" kind=\"C\"><links>
<link frommoduleid=\"1\" fromioid=\"2\" tomoduleid=\"2\" toioid=\"1\"/>
</links><inputs/><outputs/></module>
<module moduleid=\"1\" moduleclass=\"Generate\" owner=\"0\" caption=\"g\" name=\"g\" kind=\"A\" width=\"10\" height=\"10\" x=\"0\" y=\"0\"><inputs/><outputs><output ioid=\"2\" name=\"Salida1\" caption=\"\" mylabel=\"L1\"/></outputs><props><property name=\"Media\" hint=\"\" value=\"10\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/></props></module>
<module moduleid=\"2\" moduleclass=\"Terminate\" owner=\"0\" caption=\"t\" name=\"t\" kind=\"A\" width=\"10\" height=\"10\" x=\"0\" y=\"0\"><inputs><input ioid=\"1\" name=\"i\" caption=\"\" mylabel=\"L1\"/></inputs><outputs/><props><property name=\"Unidades\" hint=\"\" value=\"1\" type=\"TSNA\" subType=\"SNA\" visible=\"1\" unique=\"0\"/></props></module>
</modules></gmodel>";

/// The model of the test, in a directory of its own.
///
/// The directory is outside the repository and the test removes it when it
/// is done.
fn model_file(name: &str) -> (PathBuf, PathBuf) {
    let dir = std::env::temp_dir().join(format!("genmsi-app-start-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the scratch directory is created");
    let path = dir.join("start.gmdl");
    std::fs::write(&path, MODEL).expect("the model is written");
    (dir, path)
}

/// The end time of the report of a run with a `START` of `start`.
fn end_time_of(path: &Path, start: Option<u64>) -> f64 {
    let mut session = Some(Session::open(path, &path.display().to_string()).expect("opens"));
    let response = answer(
        &mut session,
        SessionRequest::SimulateCurrent {
            seed: Some(1),
            until: None,
            start,
        },
    );
    assert!(response.ok, "the request failed: {:?}", response.error);
    let document: serde_json::Value =
        serde_json::from_str(&response.document.expect("the answer carries a document"))
            .expect("the report parses");
    assert_eq!(document["end_reason"], "terminated");
    document["end_time"]
        .as_f64()
        .expect("the report says when the run ended")
}

#[test]
fn el_start_del_pedido_llega_al_motor() {
    let (dir, path) = model_file("start");

    // The default of the original: the run ends with the first transaction.
    assert_eq!(end_time_of(&path, None), 10.0);
    assert_eq!(end_time_of(&path, Some(1)), 10.0);

    // A `START` of three: the run ends with the third transaction.
    assert_eq!(end_time_of(&path, Some(3)), 30.0);

    std::fs::remove_dir_all(&dir).expect("the scratch directory is removed");
}
