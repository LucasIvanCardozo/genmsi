//! The GPSS program of a compiled model.
//!
//! The original editor compiles a model into GPSS and shows it in its
//! "Código GPSS" tab. That text is the original's own statement of what each
//! block does, and it is what this module reproduces: one statement per line,
//! with the block's label, the verb and the operands, and the statements that
//! continue a block carrying no name of their own.
//!
//! The definitions come first, as the original writes them, and the blocks
//! follow in the order of the model.

use crate::program::{BlockKind, Definitions, ExecBlock, Program};

/// Render a compiled program as GPSS.
pub fn render(program: &Program) -> String {
    let mut out = String::new();
    definitions(&mut out, &program.definitions);
    for block in &program.blocks {
        block_statements(&mut out, block, program);
    }
    out
}

/// Write one statement: the name, the verb and the operands.
fn line(out: &mut String, name: &str, verb: &str, operands: &str) {
    if operands.is_empty() {
        out.push_str(&format!("{name:<16} {verb}\n"));
    } else {
        out.push_str(&format!("{name:<16} {verb:<16} {operands}\n"));
    }
}

/// The definitions: the storages, the tables and the queue tables.
fn definitions(out: &mut String, defs: &Definitions) {
    let mut storages: Vec<_> = defs.storages.iter().collect();
    storages.sort_by(|a, b| a.0.cmp(b.0));
    for (name, capacity) in storages {
        line(out, name, "STORAGE", &capacity.to_string());
    }

    let mut qtables: Vec<_> = defs.qtables.iter().collect();
    qtables.sort_by(|a, b| a.0.cmp(b.0));
    for (name, table) in qtables {
        line(
            out,
            name,
            "QTABLE",
            &format!(
                "{},{},{},{}",
                table.queue, table.first, table.width, table.intervals
            ),
        );
    }
}

/// The name the original gives to a destination: the label of the block, and
/// its name when the block has no label.
fn destination(program: &Program, index: usize) -> String {
    let Some(target) = program.blocks.get(index) else {
        return String::new();
    };
    match &target.label {
        Some(label) if !label.is_empty() => label.clone(),
        _ => target.name.clone(),
    }
}

/// The probability of a `TRANSFER`, in the form the original writes it: in
/// thousandths when the block says so.
fn probability(spec: &crate::program::TransferSpec) -> String {
    let source = spec.probability.source();
    if let Ok(value) = source.parse::<f64>() {
        // The original writes the property as it is in the model, in
        // thousandths: 850 means eighty-five per cent.
        let per_thousand = if spec.thousandths { value } else { value * 1000.0 };
        return (per_thousand).round().to_string();
    }
    source.to_string()
}

/// The statements of one block: the first carries the label, the rest do not.
fn block_statements(out: &mut String, block: &ExecBlock, program: &Program) {
    let name = block.label.clone().unwrap_or_default();
    let mut first = true;
    let mut statement = |out: &mut String, verb: &str, operands: &str| {
        let column = if first { name.as_str() } else { "" };
        first = false;
        line(out, column, verb, operands);
    };

    match &block.kind {
        BlockKind::Generate(spec) => {
            let mut operands = spec.mean.to_string();
            if let Some(modifier) = &spec.modifier {
                operands.push(',');
                operands.push_str(&modifier.to_string());
            }
            statement(out, "GENERATE", &operands);
        }
        BlockKind::Terminate(spec) => {
            let operands = if spec.units == 0.0 {
                String::new()
            } else {
                spec.units.to_string()
            };
            statement(out, "TERMINATE", &operands);
        }
        BlockKind::Advance(spec) => {
            let mut operands = spec.mean.to_string();
            if let Some(modifier) = &spec.modifier {
                operands.push(',');
                operands.push_str(&modifier.to_string());
            }
            statement(out, "ADVANCE", &operands);
        }
        BlockKind::Assign(spec) => {
            let operands = format!("{},{}", spec.parameter, spec.value);
            statement(out, "ASSIGN", &operands);
        }
        BlockKind::Queue(spec) => {
            statement(out, "QUEUE", &spec.queue.to_string());
        }
        BlockKind::Depart(spec) => {
            statement(out, "DEPART", &spec.queue.to_string());
        }
        BlockKind::Seize(spec) => {
            statement(out, "SEIZE", &spec.facility.to_string());
        }
        BlockKind::Release(spec) => {
            statement(out, "RELEASE", &spec.facility.to_string());
        }
        BlockKind::Enter(spec) => {
            let operands = if spec.units.source().parse::<f64>() == Ok(1.0) {
                spec.storage.to_string()
            } else {
                format!("{},{}", spec.storage, spec.units)
            };
            statement(out, "ENTER", &operands);
        }
        BlockKind::Leave(spec) => {
            let operands = if spec.units.source().parse::<f64>() == Ok(1.0) {
                spec.storage.to_string()
            } else {
                format!("{},{}", spec.storage, spec.units)
            };
            statement(out, "LEAVE", &operands);
        }
        BlockKind::Priority(value) => {
            statement(out, "PRIORITY", &value.to_string());
        }
        BlockKind::Transfer(spec) => {
            let mut operands = probability(spec);
            for index in block.next.iter().flatten() {
                let name = destination(program, *index);
                if !name.is_empty() {
                    operands.push(',');
                    operands.push_str(&name);
                }
            }
            statement(out, "TRANSFER", &operands);
        }
        BlockKind::MultiTransfer(spec) => {
            let mut operands = String::new();
            for (index, weight) in spec.weights.iter().enumerate() {
                if index > 0 {
                    operands.push(',');
                }
                operands.push_str(&weight.to_string());
            }
            statement(out, "TRANSFER", &operands);
        }
        _ => {
            // The remaining blocks join this list as their operand rendering
            // is written; the verb is already known.
            let verb = block.class.to_uppercase();
            statement(out, &verb, "");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn reference_model() -> Program {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/models/TP2__problema1.gmdl");
        let model = genmsi_format::load(&path).expect("the reference model loads");
        crate::compile_model(&model).expect("the reference model compiles")
    }

    /// The statements with their padding removed, so a test can look for a
    /// statement without counting the spaces between the columns.
    fn statements(gpss: &str) -> Vec<String> {
        gpss
            .lines()
            .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
            .filter(|line| !line.is_empty())
            .collect()
    }

    #[test]
    fn writes_the_statements_the_original_writes() {
        let lines = statements(&render(&reference_model()));
        for expected in [
            "GENERATE 60,30",
            "QUEUE centro",
            "ADVANCE 10,5",
            "QTABLE centro,0,600,8",
            "SEIZE Recepcion",
            "RELEASE Recepcion",
        ] {
            assert!(
                lines.iter().any(|line| line.ends_with(expected)),
                "no statement ends with {expected}: {lines:#?}"
            );
        }
    }

    #[test]
    fn writes_the_definitions_before_the_blocks() {
        let gpss = render(&reference_model());
        let storage = gpss.find("STORAGE").expect("a storage");
        let queue = gpss.find("QUEUE ").expect("a queue");
        assert!(storage < queue, "the definitions come first:\n{gpss}");
    }

    #[test]
    fn the_probability_is_written_in_thousandths() {
        let lines = statements(&render(&reference_model()));
        let transfer = lines
            .iter()
            .find(|line| line.contains("TRANSFER") && line.contains("850"))
            .expect("a transfer with its probability");
        assert!(
            transfer.starts_with("Entrada1 TRANSFER 850,"),
            "{transfer}"
        );
    }
}
