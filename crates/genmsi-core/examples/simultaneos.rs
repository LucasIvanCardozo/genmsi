//! Measure the order of the events of the same instant: what the original
//! does, and what this engine does.
//!
//! The original (GenMSI 1.2020.07.28, the Delphi desktop simulator) keeps
//! its simulated time in whole units, so many events share an instant and
//! the order inside the instant decides who takes a facility, who is
//! preempted and the order of the queue. This engine keeps the time in
//! `f64`, and the tie-break of its event queue is the other way round.
//!
//! The example reads the original's own reports of the probes of this work
//! unit and runs the same probes here, and prints the two readings side by
//! side:
//!
//! 1. two transactions that arrive in the same instant to a free facility:
//!    which one takes it;
//! 2. the transaction that was already waiting against the one that arrives
//!    at the very instant the facility is released;
//! 3. the order in which a facility's chain of delay is served;
//! 4. which transaction a `PREEMPT` displaces when two of them arrive in
//!    the same instant;
//! 5. two transactions that resume from an advance in the same instant: the
//!    one that was scheduled last goes first.
//!
//! Every candidate of a probe writes the clock into its own parameter with
//! a `MARK` and tabulates it into its own table with a `TABULATE`, so the
//! histogram of the table is the moment in which that candidate took the
//! facility.
//!
//! Usage:
//!
//! ```text
//! cargo run --release -p genmsi-core --example simultaneos -- [options]
//!
//!   --probes <dir>   the probe models and the original's reports of them
//!                    (default /tmp/simultaneos-probes)
//!   --seed <n>       the seed of our runs (default 7)
//! ```

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use genmsi_core::{SimConfig, simulate};

// ---------------------------------------------------------------------------
// The text of the original's report.
// ---------------------------------------------------------------------------

/// One table of a report: its name and the values its histogram holds.
///
/// The histogram has one row per class and the count of the row is the
/// number of values that fell in it. The classes of a table of width one
/// are the values themselves, so the sample can be read back exactly.
fn tables(report: &str) -> Vec<(String, Vec<f64>)> {
    let mut tables = Vec::new();
    let mut current: Option<(String, Vec<f64>)> = None;
    let mut inside = false;
    for line in report.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("TABLAS") || trimmed.starts_with("QTABLES") {
            current = None;
            inside = false;
            continue;
        }
        // The header of a table: its name and the statement of its class.
        let mut fields = trimmed.split_whitespace();
        if let (Some(name), Some("TABLE")) = (fields.next(), fields.next()) {
            if let Some(table) = current.take() {
                tables.push(table);
            }
            current = Some((name.to_string(), Vec::new()));
            inside = false;
            continue;
        }
        let Some((_, values)) = current.as_mut() else {
            continue;
        };
        if trimmed.starts_with("INTERVALO") {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        let fields: Vec<&str> = trimmed.split_whitespace().collect();
        if fields.len() < 4 || fields[1] != "-" {
            continue;
        }
        let (Ok(low), Ok(high), Ok(frequency)) = (
            fields[0].parse::<f64>(),
            fields[2].parse::<f64>(),
            fields[3].parse::<usize>(),
        ) else {
            continue;
        };
        let value = if (low - high).abs() < f64::EPSILON {
            low
        } else {
            (low + high) / 2.0
        };
        values.extend(std::iter::repeat_n(value, frequency));
    }
    if let Some(table) = current {
        tables.push(table);
    }
    tables
}

/// One row of the block table of a report.
struct BlockRow {
    /// The label of the block (`LOC`).
    label: String,
    /// The class of the block (`BLOCK_TYPE`).
    class: String,
    /// How many transactions entered the block (`ENTRY COUNT`).
    entries: u64,
    /// How many transactions are in the block (`CURRENT COUNT`).
    current: u64,
    /// How many transactions wait at the block (`RETRY`).
    retry: u64,
}

/// The blocks of a report.
///
/// The columns of the report are not separated when the label of the block
/// is long: the `RETRY` count and the label come glued (`0A_PreemptaCinco`).
fn blocks(report: &str) -> Vec<BlockRow> {
    let mut rows = Vec::new();
    for line in report.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 6 {
            continue;
        }
        let (Ok(entries), Ok(current)) = (fields[1].parse::<u64>(), fields[2].parse::<u64>())
        else {
            continue;
        };
        // The retry count and the label of the block can come glued.
        let digits: String = fields[3].chars().take_while(|c| c.is_ascii_digit()).collect();
        let retry: u64 = digits.parse().unwrap_or(0);
        let (label, class) = if digits.len() < fields[3].len() {
            (
                fields[3][digits.len()..].to_string(),
                fields[4].to_string(),
            )
        } else {
            (fields[4].to_string(), fields[5].to_string())
        };
        if !class.chars().next().is_some_and(|first| first.is_ascii_alphabetic()) {
            continue;
        }
        rows.push(BlockRow {
            label,
            class,
            entries,
            retry,
            current,
        });
    }
    rows
}

/// The row of a facility in a report.
fn facility(report: &str) -> Option<Vec<String>> {
    let mut inside = false;
    for line in report.lines() {
        if line.starts_with("FACILITY") {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        if !line.trim().is_empty() {
            return Some(line.split_whitespace().map(str::to_string).collect());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// The runs of this engine.
// ---------------------------------------------------------------------------

/// Run a probe with this engine and return its report.
fn run_probe(path: &Path, seed: u64) -> Result<String, String> {
    let model = genmsi_format::load(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let (program, result) = simulate(
        &model,
        SimConfig {
            seed,
            stop_time: Some(3600.0),
            trace: false,
            ..SimConfig::default()
        },
    )
    .map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(genmsi_core::render(&result, &program))
}

// ---------------------------------------------------------------------------
// The probes.
// ---------------------------------------------------------------------------

/// One probe: what it asks, and the tables whose readings answer it.
struct Probe {
    /// The name of the model.
    name: &'static str,
    /// What the probe asks.
    question: &'static str,
    /// The tables whose readings answer it, in the order they are read.
    tables: &'static [&'static str],
    /// The blocks whose counts answer it (label and class).
    blocks: &'static [(&'static str, &'static str)],
}

/// The probes of this work unit, in the order they are read.
const PROBES: &[Probe] = &[
    Probe {
        name: "dosnuevos",
        question: "dos transacciones que llegan en el mismo instante a una facilidad libre: ¿cuál entra primero?",
        tables: &["TablaUno", "TablaDos"],
        blocks: &[],
    },
    Probe {
        name: "dosnuevosInvertido",
        question: "lo mismo, con las dos cadenas al revés en el modelo: ¿gana la que va después?",
        tables: &["TablaUno", "TablaDos"],
        blocks: &[],
    },
    Probe {
        name: "esperando",
        question: "la que ya esperaba contra la que llega en el instante de la liberación: ¿quién toma la facilidad?",
        tables: &["TablaEspera", "TablaNuevo"],
        blocks: &[],
    },
    Probe {
        name: "prioridadEspera",
        question: "lo mismo, con la que llega nueva de mayor prioridad: ¿quién toma la facilidad?",
        tables: &["TablaEspera", "TablaNuevo"],
        blocks: &[],
    },
    Probe {
        name: "fifo",
        question: "tres transacciones esperando, de la misma prioridad: ¿en qué orden salen?",
        tables: &["TablaUno", "TablaDos", "TablaTres"],
        blocks: &[],
    },
    Probe {
        name: "prioridadesCola",
        question: "tres transacciones esperando, de distinta prioridad: ¿en qué orden salen?",
        tables: &["TablaUno", "TablaDos", "TablaTres"],
        blocks: &[],
    },
    Probe {
        name: "avances",
        question: "dos transacciones que reanudan un avance en el mismo instante: ¿cuál va primero?",
        tables: &["TablaUno", "TablaDos"],
        blocks: &[],
    },
    Probe {
        name: "preempta",
        question: "dos transacciones de distinta prioridad que preemptan en el mismo instante: ¿a quién se le quita?",
        tables: &["TablaCinco", "TablaDiez"],
        blocks: &[
            ("A_PreemptaCinco", "PREEMPT"),
            ("A_PreemptaDiez", "PREEMPT"),
        ],
    },
    Probe {
        name: "preemptaInvertida",
        question: "lo mismo, con las dos cadenas al revés en el modelo: ¿a quién se le quita?",
        tables: &["TablaCinco", "TablaDiez"],
        blocks: &[
            ("A_PreemptaCinco", "PREEMPT"),
            ("A_PreemptaDiez", "PREEMPT"),
        ],
    },
    Probe {
        name: "preemptaTarde",
        question: "los dos preemptan en instantes distintos: ¿a quién se le quita?",
        tables: &["TablaCinco", "TablaDiez"],
        blocks: &[
            ("A_PreemptaCinco", "PREEMPT"),
            ("A_PreemptaDiez", "PREEMPT"),
        ],
    },
];

// ---------------------------------------------------------------------------
// The options and the report.
// ---------------------------------------------------------------------------

/// The options of the example.
struct Options {
    probes: PathBuf,
    seed: u64,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            probes: PathBuf::from("/tmp/simultaneos-probes"),
            seed: 7,
        }
    }
}

fn parse_options() -> Options {
    let mut options = Options::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().unwrap_or_default();
        match arg.as_str() {
            "--probes" => options.probes = PathBuf::from(value()),
            "--seed" => options.seed = value().parse().unwrap_or(options.seed),
            other => eprintln!("unknown option: {other}"),
        }
    }
    options
}

/// The header of a section.
fn section(out: &mut String, title: &str) {
    let _ = writeln!(out, "\n{title}");
    let _ = writeln!(out, "{}", "-".repeat(title.len()));
}

/// The readings of one table of a report: the values of its histogram.
fn readings(report: &str, table: &str) -> Option<Vec<f64>> {
    tables(report)
        .into_iter()
        .find(|(name, _)| name == table)
        .map(|(_, values)| values)
}

/// A reading, written the way the original writes its numbers.
fn reading(values: &[f64]) -> String {
    if values.is_empty() {
        return "(nada)".to_string();
    }
    values
        .iter()
        .map(|value| format!("{value:.0}"))
        .collect::<Vec<String>>()
        .join(", ")
}

fn main() {
    let options = parse_options();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "================================================================================"
    );
    let _ = writeln!(
        out,
        " El orden de los eventos del mismo instante: el original contra este motor"
    );
    let _ = writeln!(
        out,
        "================================================================================"
    );
    let _ = writeln!(out, " las sondas: {}", options.probes.display());
    let _ = writeln!(
        out,
        " los reportes del original: {}/oracle",
        options.probes.display()
    );
    let _ = writeln!(out, " nuestra semilla: {}", options.seed);
    let _ = writeln!(out);
    let _ = writeln!(out, " Las preguntas, textuales:");
    let _ = writeln!(
        out,
        "  1. ¿Quién gana un SEIZE: la que estaba esperando, la que llega nueva o la de mayor"
    );
    let _ = writeln!(out, "     prioridad? Y con dos que llegan nuevas a la vez, ¿cuál entra primero?");
    let _ = writeln!(
        out,
        "  2. ¿A quién preempta: dos transacciones de distinta prioridad que compiten por el"
    );
    let _ = writeln!(out, "     mismo recurso en el mismo instante, a cuál se le quita?");
    let _ = writeln!(
        out,
        "  3. El FIFO de la cola: el orden en que salen las transacciones de una cola cuando se"
    );
    let _ = writeln!(out, "     vacía en un mismo instante.");

    let mut missing = Vec::new();
    for probe in PROBES {
        section(&mut out, &format!("{} — {}", probe.name.to_uppercase(), probe.question));
        let model = options.probes.join(format!("{}.gmdl", probe.name));
        let original_path = options
            .probes
            .join("oracle")
            .join(format!("{}.report.txt", probe.name));
        let original = std::fs::read_to_string(&original_path).ok();
        let ours = run_probe(&model, options.seed).ok();
        if original.is_none() || ours.is_none() {
            missing.push(probe.name.to_string());
            let _ = writeln!(
                out,
                "   (falta la medida: {} y {})",
                original_path.display(),
                model.display()
            );
            continue;
        }
        let original = original.unwrap_or_default();
        let ours = ours.unwrap_or_default();
        let _ = writeln!(out, "   tabla             el original      este motor");
        for table in probe.tables {
            let original_values = readings(&original, table).unwrap_or_default();
            let our_values = readings(&ours, table).unwrap_or_default();
            let _ = writeln!(
                out,
                "   {table:<16}  {:<15}  {}",
                reading(&original_values),
                reading(&our_values)
            );
        }
        if let Some(row) = facility(&original) {
            let _ = writeln!(
                out,
                "   facility (el original): {}",
                row.join(" ")
            );
            if let Some(row) = facility(&ours) {
                let _ = writeln!(
                    out,
                    "   facility (este motor): {}",
                    row.join(" ")
                );
            }
        }
        for (label, class) in probe.blocks {
            let original_row = blocks(&original)
                .into_iter()
                .find(|row| row.label == *label && row.class == *class);
            let our_row = blocks(&ours)
                .into_iter()
                .find(|row| row.label == *label && row.class == *class);
            let read = |row: Option<BlockRow>| match row {
                Some(row) => format!(
                    "entradas={} en el bloque={} esperando={}",
                    row.entries, row.current, row.retry
                ),
                None => "(el bloque no existe)".to_string(),
            };
            let _ = writeln!(
                out,
                "   {label:<18} el original: {:<26} este motor: {}",
                read(original_row),
                read(our_row)
            );
        }
    }

    section(&mut out, "EL VEREDICTO");
    let _ = writeln!(
        out,
        "   El original ordena los eventos de un mismo instante por prioridad descendente y, entre"
    );
    let _ = writeln!(
        out,
        "   los de la misma prioridad, al revés: el último programado se procesa primero."
    );
    let _ = writeln!(
        out,
        "   La cadena de demora de una facilidad se sirve por prioridad descendente y, entre los de"
    );
    let _ = writeln!(
        out,
        "   la misma prioridad, por orden de llegada (FIFO)."
    );
    let _ = writeln!(
        out,
        "   Este motor ordena los eventos de un mismo instante por prioridad descendente y, entre"
    );
    let _ = writeln!(
        out,
        "   los de la misma prioridad, en el orden en que se programaron (FIFO): al revés que el"
    );
    let _ = writeln!(out, "   original.");
    section(&mut out, "DÓNDE SE APLICARÍA LA REGLA");
    let _ = writeln!(
        out,
        "   El desempate de los eventos de un mismo instante está en"
    );
    let _ = writeln!(
        out,
        "   `crates/genmsi-core/src/sim.rs`: el campo `seq` de `struct Event` y el"
    );
    let _ = writeln!(
        out,
        "   `impl Ord for Event` (la comparación es `(time, priority, seq)`), y el"
    );
    let _ = writeln!(
        out,
        "   número de secuencia en `Simulation::push_event` (que hoy invierte el orden"
    );
    let _ = writeln!(
        out,
        "   sólo para los eventos de la hora en que termina la corrida)."
    );
    let _ = writeln!(
        out,
        "   La cadena de demora de una facilidad está en el mismo archivo:"
    );
    let _ = writeln!(
        out,
        "   `Simulation::release_waiters` y el bloque `BlockKind::Seize`, que ordenan a"
    );
    let _ = writeln!(
        out,
        "   los que esperan por prioridad y, a igualdad, por orden de llegada: eso ya"
    );
    let _ = writeln!(out, "   coincide con el original.");
    let _ = writeln!(
        out,
        "   Y un segundo cambio, que no es evidente: en `Simulation::install_definitions`"
    );
    let _ = writeln!(
        out,
        "   la llegada pendiente de un generador se programa con prioridad 0; en el"
    );
    let _ = writeln!(
        out,
        "   original lleva la prioridad de la transacción, y sin eso una llegada"
    );
    let _ = writeln!(
        out,
        "   pendiente de prioridad 0 se cuela por delante de los eventos de las"
    );
    let _ = writeln!(out, "   transacciones de mayor prioridad.");
    if !missing.is_empty() {
        let _ = writeln!(
            out,
            "   (faltan las medidas de: {})",
            missing.join(", ")
        );
    }

    print!("{out}");
}
