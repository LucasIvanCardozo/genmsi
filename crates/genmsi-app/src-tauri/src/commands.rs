//! The commands the frontend calls.
//!
//! Each command returns one engine document, built by the shared facade
//! (`genmsi-app-engine`): the shell moves the bytes, it never builds a
//! document of its own. The dev bridge's binary runs the same facade, so both
//! backends return the same document for the same model.
//!
//! The editing session lives here too: the shell keeps one model in memory
//! with its undo stack, exactly like the dev bridge keeps one in its session
//! process.
//!
//! **The run of the model does not happen on the thread of the window.**
//! Tauri runs a synchronous command on the main thread of the application,
//! and a run of the engine can take minutes: the window would not repaint,
//! the app could not be closed with education and nothing could be
//! interrupted. The run therefore goes to a thread of the blocking pool
//! (`tauri::async_runtime::spawn_blocking`), the command waits for it on the
//! async runtime, and the main thread keeps answering the events of the
//! window: the reader sees "Ejecutando…", can close the app and can ask for
//! the run to stop.
//!
//! **The cancel is a real one.** The flag the reader raises is the flag the
//! engine reads: the engine stops the run at the next event and gives the
//! partial result, so the processor is freed. The shell stops waiting for
//! the run and throws its answer away.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use genmsi_app_engine::{
    ModelFile, Session, SessionRequest, SessionResponse, answer, lint_document_bytes,
    list_models as list_model_files, model_document_bytes, run_model_documents_bytes,
    simulate_document_bytes, simulate_model_bytes, trace_document_bytes, trace_model_bytes,
};
use tauri::Manager;
use tauri::async_runtime::{Sender, channel};

/// The directory the shell offers as models.
const MODELS_DIR: &str = "fixtures/models";

/// How many models the list of recents keeps.
const RECENT_MODELS: usize = 8;

/// The name of the file the recents live in, inside the app's data
/// directory.
const RECENTS_FILE: &str = "recent-models.json";

/// The message the shell sends when the reader stops a run.
///
/// The engine reads the flag of the cancellation and stops the run at the
/// next event, so the cancel is a real one: the processor is freed and the
/// result of the run is thrown away.
const STOPPED_MESSAGE: &str = "la corrida se canceló: el motor cortó la simulación en el \
siguiente evento y la app descartó su resultado.";

/// Say what the shell did.
///
/// The shell has no console of its own, so the line goes to the log the
/// smoke check reads: it is the proof that the `invoke` path of the
/// frontend reaches the facade.
fn log(command: &str) {
    eprintln!("[genmsi-app] {command}");
}

/// The editing session the shell keeps open.
static SESSION: Mutex<Option<Session>> = Mutex::new(None);

/// The run in flight, if there is one.
static ACTIVE_RUN: Mutex<Option<ActiveRun>> = Mutex::new(None);

/// How many runs the shell has started: every run has its own number, so a
/// run that ends does not clear a newer one.
static RUN_IDS: AtomicU64 = AtomicU64::new(0);

/// The run in flight.
///
/// The shell keeps one: the app runs one model at a time, and the reader
/// stops the run that is in flight.
struct ActiveRun {
    /// The number of the run.
    id: u64,
    /// What the reader can ask of the run.
    control: RunControl,
    /// The channel that tells the waiting command how the run ended.
    end: Sender<RunEnd>,
}

/// How a run ended.
enum RunEnd {
    /// The engine answered.
    Answered(Result<String, String>),
    /// The reader asked for the run to stop: the answer is dropped.
    Stopped,
}

/// What the reader can ask of a run in flight.
///
/// This is the seam of the cancellation: the flag lives here, in the shell,
/// and the engine reads it. The shell hands the flag to the facade with the
/// run, and the engine's loop of events looks at it: the run ends with the
/// reason of the cancellation, and the shell stops waiting for it.
///
/// The flag is the same one the reader raises: the shell holds it, checks it
/// before the work begins and raises it when the reader asks for the run to
/// stop.
#[derive(Clone)]
struct RunControl(Arc<AtomicBool>);

impl RunControl {
    /// A run nobody has asked to stop yet.
    fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    /// Whether the reader asked for the run to stop.
    fn stopped(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    /// Ask the run to stop.
    fn stop(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    /// The flag the engine reads.
    ///
    /// The engine works on another thread, so it gets the flag itself: the
    /// one the reader raises is the one the engine looks at.
    fn flag(&self) -> Arc<AtomicBool> {
        self.0.clone()
    }
}

/// The models the app offers.
#[tauri::command]
pub fn list_models() -> Result<Vec<ModelFile>, String> {
    // The models of the repository, when the app runs from one: that list is
    // the one the development bridge uses. An installed app has no
    // repository, and the models the reader works with are the ones it opens
    // with the dialog of the system, so the list is empty.
    let directory = match crate::app_root() {
        Some(root) => root.join(MODELS_DIR),
        None => std::path::PathBuf::from(MODELS_DIR),
    };
    let models = list_model_files(&directory).unwrap_or_default();
    log(&format!("list_models: {} modelos", models.len()));
    Ok(models)
}

/// The model's graph (`genmsi.model/1`).
#[tauri::command]
pub fn load_model(path: String) -> Result<String, String> {
    log(&format!("load_model: {path}"));
    model_document_bytes(Path::new(&path)).map_err(|error| error.to_string())
}

/// The model's lint (`genmsi.lint/1`).
#[tauri::command]
pub fn lint_model(path: String) -> Result<String, String> {
    log(&format!("lint_model: {path}"));
    lint_document_bytes(Path::new(&path)).map_err(|error| error.to_string())
}

/// The run's report (`genmsi.report/1`).
///
/// The run happens on a thread of the blocking pool: the window keeps
/// painting while the engine works.
#[tauri::command]
pub async fn simulate_model(
    path: String,
    seed: Option<u64>,
    until: Option<f64>,
) -> Result<String, String> {
    log(&format!("simulate_model: {path}"));
    let path = PathBuf::from(path);
    run_off_thread(move |control| {
        simulate_document_bytes(&path, seed.unwrap_or(1), until, Some(control.flag()))
            .map_err(|error| error.to_string())
    })
    .await
}

/// The run's trace (`genmsi.trace/1`).
///
/// The run happens on a thread of the blocking pool: the window keeps
/// painting while the engine works.
#[tauri::command]
pub async fn trace_model(
    path: String,
    seed: Option<u64>,
    until: Option<f64>,
    limit: Option<usize>,
) -> Result<String, String> {
    log(&format!("trace_model: {path}"));
    let path = PathBuf::from(path);
    run_off_thread(move |control| {
        trace_document_bytes(
            &path,
            seed.unwrap_or(1),
            until,
            limit.unwrap_or(100),
            Some(control.flag()),
        )
        .map_err(|error| error.to_string())
    })
    .await
}

/// One request of the editing protocol, answered by the same facade the dev
/// bridge's session process runs.
///
/// The request and the answer are JSON: the frontend parses them the same
/// way in both backends.
///
/// The command is asynchronous: the run of the model does not happen on the
/// thread of the window. The editing operations are quick and are answered
/// right away; a run goes to a thread of the blocking pool, so the window
/// keeps painting while the engine works and the reader can stop waiting for
/// it.
#[tauri::command]
pub async fn session(request: String) -> Result<String, String> {
    let request: SessionRequest =
        serde_json::from_str(&request).map_err(|error| error.to_string())?;
    answer_request(request).await
}

/// Answer one request of the protocol.
///
/// A request that runs the model is answered off the thread of the window;
/// the rest is answered right away: they touch the session and the disk, and
/// they are quick.
async fn answer_request(request: SessionRequest) -> Result<String, String> {
    let Some(run) = RunRequest::of(&request) else {
        return answer_now(request);
    };
    // The model is copied out of the session: the run works on the copy, so
    // the session is free while the engine works and the reader can keep
    // editing the model while the run is in flight.
    let model = {
        let guard = SESSION
            .lock()
            .map_err(|_| "la sesión del modelo está ocupada".to_string())?;
        guard.as_ref().map(|session| session.model().clone())
    };
    let Some(model) = model else {
        // No model is open: the protocol says so, and it says it right away.
        return answer_now(request);
    };
    run_off_thread(move |control| {
        // The reader may have stopped the run before the work began: there
        // is no reason to run the engine then.
        if control.stopped() {
            return Err(STOPPED_MESSAGE.to_string());
        }
        let response = match run {
            RunRequest::Report { seed, until } => {
                match simulate_model_bytes(&model, seed, until, Some(control.flag())) {
                    Ok(document) => SessionResponse::document(document),
                    Err(error) => SessionResponse::error(error),
                }
            }
            RunRequest::Trace {
                seed,
                until,
                limit,
            } => match trace_model_bytes(&model, seed, until, limit, Some(control.flag())) {
                Ok(document) => SessionResponse::document(document),
                Err(error) => SessionResponse::error(error),
            },
            RunRequest::Both {
                seed,
                until,
                limit,
            } => match run_model_documents_bytes(
                &model,
                seed,
                until,
                limit,
                Some(control.flag()),
            ) {
                Ok(documents) => SessionResponse::documents(documents),
                Err(error) => SessionResponse::error(error),
            },
        };
        serde_json::to_string(&response).map_err(|error| error.to_string())
    })
    .await
}

/// Answer one request of the protocol that does not run the model.
fn answer_now(request: SessionRequest) -> Result<String, String> {
    let mut guard = SESSION
        .lock()
        .map_err(|_| "la sesión del modelo está ocupada".to_string())?;
    let response = answer(&mut guard, request);
    serde_json::to_string(&response).map_err(|error| error.to_string())
}

/// The run a request of the protocol asks for.
///
/// The three runs are the ones the facade has: the report of the model in
/// memory, its trace, and the two of one run.
enum RunRequest {
    /// The report of the model in memory.
    Report {
        /// Seed of the run.
        seed: u64,
        /// Stop the run at this simulation time.
        until: Option<f64>,
    },
    /// The trace of the model in memory.
    Trace {
        /// Seed of the run.
        seed: u64,
        /// Stop the run at this simulation time.
        until: Option<f64>,
        /// Maximum number of trace records kept.
        limit: usize,
    },
    /// The report and the trace of one run.
    Both {
        /// Seed of the run.
        seed: u64,
        /// Stop the run at this simulation time.
        until: Option<f64>,
        /// Maximum number of trace records kept.
        limit: usize,
    },
}

impl RunRequest {
    /// The run the request asks for, if it asks for one.
    ///
    /// The defaults are the ones of the protocol: the seed one and a
    /// hundred records of trace.
    fn of(request: &SessionRequest) -> Option<Self> {
        match request {
            SessionRequest::SimulateCurrent { seed, until } => Some(RunRequest::Report {
                seed: seed.unwrap_or(1),
                until: *until,
            }),
            SessionRequest::TraceCurrent {
                seed,
                until,
                limit,
            } => Some(RunRequest::Trace {
                seed: seed.unwrap_or(1),
                until: *until,
                limit: limit.unwrap_or(100),
            }),
            SessionRequest::RunCurrent {
                seed,
                until,
                limit,
            } => Some(RunRequest::Both {
                seed: seed.unwrap_or(1),
                until: *until,
                limit: limit.unwrap_or(100),
            }),
            _ => None,
        }
    }
}

/// Run the engine off the thread that asked for it, and let the reader stop
/// waiting for it.
///
/// The work goes to a thread of the blocking pool of Tauri: the thread of the
/// window keeps answering the events of the app, so the window repaints and
/// the app can be closed while the engine works.
///
/// The reader can ask for the run to stop: the shell raises the flag of the
/// cancellation, the engine stops at the next event and the shell stops
/// waiting for the run.
async fn run_off_thread<F>(work: F) -> Result<String, String>
where
    F: FnOnce(&RunControl) -> Result<String, String> + Send + 'static,
{
    let (end, mut receiver) = channel::<RunEnd>(1);
    let control = RunControl::new();
    let id = RUN_IDS.fetch_add(1, Ordering::Relaxed);
    {
        let mut guard = ACTIVE_RUN
            .lock()
            .map_err(|_| "la corrida no pudo registrarse".to_string())?;
        *guard = Some(ActiveRun {
            id,
            control: control.clone(),
            end: end.clone(),
        });
    }
    tauri::async_runtime::spawn_blocking(move || {
        let answer = work(&control);
        // The answer of a run nobody waits for is dropped: the send fails
        // when the reader stopped waiting, and the answer is thrown away
        // when it does not.
        let _ = end.try_send(RunEnd::Answered(answer));
    });
    let outcome = receiver.recv().await;
    clear_active_run(id);
    match outcome {
        Some(RunEnd::Answered(answer)) => answer,
        Some(RunEnd::Stopped) => Err(STOPPED_MESSAGE.to_string()),
        None => Err("la corrida terminó sin respuesta".to_string()),
    }
}

/// Forget the run, if it is still the one that ended.
fn clear_active_run(id: u64) {
    if let Ok(mut guard) = ACTIVE_RUN.lock()
        && guard.as_ref().is_some_and(|active| active.id == id)
    {
        *guard = None;
    }
}

/// Ask the run in flight to stop.
///
/// The flag of the cancellation goes up, so the engine stops the run at the
/// next event and the processor is freed. The command says whether there was
/// a run to stop.
#[tauri::command]
pub fn cancel_run() -> Result<bool, String> {
    let guard = ACTIVE_RUN
        .lock()
        .map_err(|_| "la corrida no pudo consultarse".to_string())?;
    let Some(active) = guard.as_ref() else {
        log("cancel_run: no hay ninguna corrida en marcha");
        return Ok(false);
    };
    active.control.stop();
    // The answer of the run is dropped: the waiting command reads this and
    // says the run was cancelled.
    let _ = active.end.try_send(RunEnd::Stopped);
    log("cancel_run: la corrida se canceló");
    Ok(true)
}

/// The file the recents live in, inside the app's data directory.
///
/// The path comes from the resolver of Tauri: the app's data directory is
/// where the shell may write without asking.
fn recents_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("no se pudo encontrar el directorio de datos: {error}"))?;
    Ok(dir.join(RECENTS_FILE))
}

/// The models opened lately, the most recent first.
///
/// A file that is not there, or that cannot be read, is a list without
/// recents: the app opens models anyway.
fn read_recents(file: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(file) else {
        return Vec::new();
    };
    let Ok(models) = serde_json::from_str::<Vec<String>>(&text) else {
        return Vec::new();
    };
    models
}

/// Write the recents, creating the app's data directory if it is not there.
fn write_recents(file: &Path, models: &[String]) -> Result<(), String> {
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|error| format!("no se pudo crear {}: {error}", dir.display()))?;
    }
    let text = serde_json::to_string_pretty(models).map_err(|error| error.to_string())?;
    std::fs::write(file, text)
        .map_err(|error| format!("no se pudo escribir {}: {error}", file.display()))
}

/// The models opened lately, the most recent first.
#[tauri::command]
pub fn recent_models(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let file = recents_file(&app)?;
    log("recent_models: los modelos recientes");
    Ok(read_recents(&file))
}

/// Remember a model as one of the last opened ones.
///
/// The list is short and the last one is the first: the model the reader
/// comes back to is the one he opened last.
#[tauri::command]
pub fn remember_model(app: tauri::AppHandle, path: String) -> Result<Vec<String>, String> {
    let file = recents_file(&app)?;
    log(&format!("remember_model: {path}"));
    let mut models = read_recents(&file);
    models.retain(|model| model != &path);
    models.insert(0, path);
    models.truncate(RECENT_MODELS);
    write_recents(&file, &models)?;
    Ok(models)
}

#[cfg(test)]
mod tests {
    use super::*;
    use genmsi_app_engine::run_documents_bytes;
    use std::sync::mpsc as std_mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    /// A model that keeps the engine busy for a while.
    ///
    /// A generator that makes a transaction every hundredth of a second, and
    /// a clock of half an hour: the model has no `Terminate` operand, so the
    /// run lasts until the clock ends it. The engine has hundreds of
    /// thousands of events to work on, and the flag has time to be raised in
    /// the middle of them.
    const BUSY_MODEL: &str = r#"<gmodel versioncode="1"><gclasses/><modules>
<module moduleid="0" moduleclass="main" owner="0" caption="m" name="m" kind="C"><links>
<link frommoduleid="1" fromioid="2" tomoduleid="2" toioid="1"/>
</links><inputs/><outputs/></module>
<module moduleid="1" moduleclass="Generate" owner="0" caption="Generate" name="Generate" kind="A" width="10" height="10" x="0" y="0"><inputs/><outputs><output ioid="2" name="Salida1" caption="" mylabel="T1"/></outputs><props><property name="Media" hint="" value="0.01" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="2" moduleclass="Terminate" owner="0" caption="Terminate" name="Terminate" kind="A" width="10" height="10" x="0" y="0"><inputs><input ioid="1" name="Entrada1" caption="" mylabel="T1"/></inputs><outputs/><props><property name="Unidades" hint="" value="0" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
<module moduleid="3" moduleclass="Clock" owner="0" caption="Clock" name="Clock" kind="A" width="10" height="10" x="0" y="0"><props><property name="Frecuencia" hint="" value="1800" type="TSNA" subType="SNA" visible="1" unique="0"/></props></module>
</modules></gmodel>"#;

    /// The model the tests run.
    ///
    /// It is the model of the repository: the tests compare the answer of
    /// the shell with the answer of the protocol, and both have to run the
    /// same model.
    fn reference() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../fixtures/models/TP6__Clase 6_1_PuertaBanco.gmdl")
    }

    /// The name the shell gives to the model of the tests.
    fn reference_name() -> String {
        reference().display().to_string()
    }

    /// The lock the tests that touch the shell's globals take.
    ///
    /// The run in flight and the session are globals of the shell: the
    /// tests that use them run one after the other.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    /// Take the lock of the globals.
    ///
    /// A test that fails does not lock the others out: the lock only makes
    /// the tests run one after the other.
    fn globals() -> std::sync::MutexGuard<'static, ()> {
        TEST_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The request of a run that gives the report and the trace.
    fn run_request() -> SessionRequest {
        SessionRequest::RunCurrent {
            seed: Some(1),
            until: None,
            limit: Some(100),
        }
    }

    #[test]
    fn the_run_works_on_a_thread_that_is_not_the_one_that_asked_for_it() {
        let _globals = globals();
        let caller = thread::current().id();
        let (sender, receiver) = std_mpsc::channel();
        let answer = tauri::async_runtime::block_on(run_off_thread(move |_| {
            sender
                .send(thread::current().id())
                .expect("the test receives the thread");
            thread::sleep(Duration::from_millis(50));
            Ok("done".to_string())
        }));
        let worker = receiver.recv().expect("the worker sends its thread");
        assert_ne!(
            caller, worker,
            "the run happened on the thread that asked for it"
        );
        assert_eq!(answer.expect("the run answered"), "done");
    }

    #[test]
    fn the_reader_can_stop_waiting_for_a_run() {
        let _globals = globals();
        let (started, wait) = std_mpsc::channel();
        let stopper = thread::spawn(move || {
            wait.recv().expect("the run starts");
            assert!(
                cancel_run().expect("the shell answers"),
                "there was a run to stop"
            );
        });
        let answer = tauri::async_runtime::block_on(run_off_thread(move |_| {
            started.send(()).ok();
            // The work of the test stands for the run of the engine: the
            // reader does not wait for it.
            thread::sleep(Duration::from_secs(2));
            Ok("the engine answered".to_string())
        }));
        stopper.join().expect("the reader finishes");
        assert_eq!(
            answer.expect_err("the run was stopped"),
            STOPPED_MESSAGE,
            "the shell did not say the run was stopped"
        );
    }

    #[test]
    fn the_run_off_the_thread_gives_the_documents_the_protocol_gives() {
        let _globals = globals();
        let path = reference();
        {
            let mut guard = SESSION.lock().expect("the session is free");
            *guard = Some(Session::open(&path, &reference_name()).expect("the model opens"));
        }
        // What the protocol answers, in the session process of the dev
        // bridge: the bytes the frontend knows.
        let mut session = Some(Session::open(&path, &reference_name()).expect("the model opens"));
        let expected = serde_json::to_string(&answer(&mut session, run_request()))
            .expect("the answer serializes");
        // What the shell answers, with the run off the thread of the window.
        let answered =
            tauri::async_runtime::block_on(answer_request(run_request())).expect("the run answers");
        assert_eq!(
            answered, expected,
            "the run off the thread gave other documents"
        );
    }

    #[test]
    fn a_stopped_run_leaves_no_run_in_flight() {
        let _globals = globals();
        let (started, wait) = std_mpsc::channel();
        let stopper = thread::spawn(move || {
            wait.recv().expect("the run starts");
            cancel_run().expect("the shell answers");
        });
        let answer = tauri::async_runtime::block_on(run_off_thread(move |_| {
            started.send(()).ok();
            thread::sleep(Duration::from_millis(200));
            Ok("the engine answered".to_string())
        }));
        stopper.join().expect("the reader finishes");
        assert!(answer.is_err(), "the run was stopped");
        assert!(
            !cancel_run().expect("the shell answers"),
            "the stopped run is still in flight"
        );
    }

    /// The busy model of the tests, in a directory of its own.
    ///
    /// The directory is outside the repository and the test removes it when
    /// it is done.
    fn busy_model(name: &str) -> (PathBuf, PathBuf) {
        let dir = std::env::temp_dir().join(format!("genmsi-app-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("the scratch directory is created");
        let path = dir.join("pesado.gmdl");
        std::fs::write(&path, BUSY_MODEL).expect("the model is written");
        (dir, path)
    }

    #[test]
    fn the_simulate_command_gives_the_report_of_the_facade() {
        let _globals = globals();
        let path = reference();
        let command = tauri::async_runtime::block_on(simulate_model(
            path.display().to_string(),
            Some(1),
            None,
        ))
        .expect("the command answers");
        let facade = simulate_document_bytes(&path, 1, None, None).expect("the facade answers");
        assert_eq!(command, facade, "the command gave another report");
    }

    #[test]
    fn the_trace_command_gives_the_trace_of_the_facade() {
        let _globals = globals();
        let path = reference();
        let command = tauri::async_runtime::block_on(trace_model(
            path.display().to_string(),
            Some(1),
            None,
            Some(100),
        ))
        .expect("the command answers");
        let facade =
            trace_document_bytes(&path, 1, None, 100, None).expect("the facade answers");
        assert_eq!(command, facade, "the command gave another trace");
    }

    #[test]
    fn the_flag_of_the_reader_kills_the_run_of_the_engine() {
        let _globals = globals();
        let (dir, path) = busy_model("cancel");

        // The whole run: how long the engine takes when nobody stops it.
        let whole_start = Instant::now();
        let whole = run_documents_bytes(&path, 1, None, 100, None).expect("the engine runs");
        let whole_elapsed = whole_start.elapsed();
        let whole_report: serde_json::Value =
            serde_json::from_str(&whole.report).expect("the report parses");
        assert_eq!(whole_report["end_reason"], "terminated");

        // The same run, with the reader raising the flag in the middle of it.
        let (started, wait) = std_mpsc::channel();
        let (sender, receiver) = std_mpsc::channel();
        let stopper = thread::spawn(move || {
            wait.recv().expect("the run starts");
            thread::sleep(Duration::from_millis(20));
            assert!(
                cancel_run().expect("the shell answers"),
                "there was a run to stop"
            );
        });
        let model_path = path.clone();
        let answer_of_the_shell = tauri::async_runtime::block_on(run_off_thread(move |control| {
            started.send(()).ok();
            let engine_start = Instant::now();
            let documents = run_documents_bytes(&model_path, 1, None, 100, Some(control.flag()))
                .map_err(|error| error.to_string())?;
            sender
                .send((documents, engine_start.elapsed()))
                .expect("the test receives the answer");
            Ok("the engine answered".to_string())
        }));
        stopper.join().expect("the reader finishes");
        let (documents, engine_elapsed) = receiver.recv().expect("the engine answers");

        // The engine stopped: the run ends with the reason of the
        // cancellation, and it ends much earlier than the whole run.
        let report: serde_json::Value =
            serde_json::from_str(&documents.report).expect("the report parses");
        assert_eq!(
            report["end_reason"], "cancelled",
            "the engine did not stop for the reason of the cancellation"
        );
        assert!(
            whole_elapsed > Duration::from_millis(50),
            "the whole run is too short to be cancelled in the middle: {whole_elapsed:?}"
        );
        assert!(
            engine_elapsed * 2 < whole_elapsed,
            "the engine did not stop much earlier: {engine_elapsed:?} against {whole_elapsed:?}"
        );
        assert!(
            report["transactions"].as_u64() < whole_report["transactions"].as_u64(),
            "the cancelled run holds the whole run: {:?}",
            report["transactions"]
        );

        // The shell stopped waiting for the run: the answer of the run is
        // thrown away.
        assert_eq!(
            answer_of_the_shell.expect_err("the run was stopped"),
            STOPPED_MESSAGE,
            "the shell did not say the run was stopped"
        );

        std::fs::remove_dir_all(&dir).expect("the scratch directory is removed");
    }
}
