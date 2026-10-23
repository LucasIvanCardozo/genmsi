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

use std::collections::HashMap;

use crate::program::{BlockKind, Definitions, ExecBlock, Program};

/// Render a compiled program as GPSS.
pub fn render(program: &Program) -> String {
    let mut out = String::new();
    definitions(&mut out, &program.definitions);
    let order = flow_order(program);
    for (position, index) in order.iter().enumerate() {
        let block = &program.blocks[*index];
        block_statements(&mut out, block, program);
        let following = order.get(position + 1).copied();
        connection(&mut out, block, following, program);
    }
    out
}

/// The blocks in the order a transaction visits them.
///
/// The original writes the program's blocks in the order the transaction
/// walks them, so a transaction goes to the next block by itself and only the
/// connections that break that order are written as a `TRANSFER`. The order
/// is the one a walk of the graph gives, with the order of the model to break
/// the ties, and the statements of a compound block stay together.
fn flow_order(program: &Program) -> Vec<usize> {
    // The modules, in the order of the model, with the range of blocks each
    // one owns.
    let mut modules: Vec<(u32, Vec<usize>)> = Vec::new();
    for (index, block) in program.blocks.iter().enumerate() {
        match modules.last_mut() {
            Some((id, blocks)) if *id == block.module_id => blocks.push(index),
            _ => modules.push((block.module_id, vec![index])),
        }
    }

    // The walk: from a module to the ones its last statement leads to.
    let module_of: HashMap<usize, usize> = modules
        .iter()
        .enumerate()
        .flat_map(|(position, (_, blocks))| blocks.iter().map(move |index| (*index, position)))
        .collect();
    let mut successors: Vec<Vec<usize>> = vec![Vec::new(); modules.len()];
    for (position, (_, blocks)) in modules.iter().enumerate() {
        let Some(last) = blocks.last() else {
            continue;
        };
        let block = &program.blocks[*last];
        for target in block.next.iter().flatten() {
            let Some(destination) = module_of.get(target).copied() else {
                continue;
            };
            if destination != position && !successors[position].contains(&destination) {
                successors[position].push(destination);
            }
        }    }

    // The walk itself: a module goes when every module that leads to it has
    // already gone, so the order is the one the transaction follows.
    let mut predecessors = vec![0usize; modules.len()];
    for successors_of in &successors {
        for successor in successors_of {
            predecessors[*successor] += 1;
        }
    }
    let mut pending: Vec<usize> = (0..modules.len()).collect();
    let mut visited = vec![false; modules.len()];
    let mut order = Vec::with_capacity(program.blocks.len());
    let mut follows: Vec<usize> = Vec::new();
    while !pending.is_empty() {
        // The module that continues the walk goes first; when the walk is
        // broken, the order of the model breaks the tie.
        let available = |candidate: usize| predecessors[candidate] == 0;
        let chosen = pending
            .iter()
            .position(|candidate| available(*candidate) && follows.contains(candidate))
            .or_else(|| pending.iter().position(|candidate| available(*candidate)));
        let Some(position) = chosen else {
            // A cycle: the remaining modules keep the order of the model.
            break;
        };
        let position = pending.remove(position);
        visited[position] = true;
        order.extend(modules[position].1.iter().copied());
        follows = successors[position].clone();
        for successor in &successors[position] {
            predecessors[*successor] -= 1;
            if !visited[*successor] && !pending.contains(successor) {
                pending.push(*successor);
            }
        }
    }
    for (position, (_, blocks)) in modules.iter().enumerate() {
        if !visited[position] {
            order.extend(blocks.iter().copied());
        }
    }
    order
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

/// The connection of a block, when the flow does not fall through.
///
/// The original writes the program's blocks in the order the transaction
/// visits them, so a transaction goes to the next block by itself. The
/// connections that break that order are written as a `TRANSFER` with no name
/// of its own.
fn connection(out: &mut String, block: &ExecBlock, following: Option<usize>, program: &Program) {
    if matches!(block.kind, BlockKind::Transfer(_) | BlockKind::MultiTransfer(_)) {
        // The block's own statement already says where the transaction goes.
        return;
    }
    let Some(target) = block.next.first().copied().flatten() else {
        return;
    };
    if following == Some(target) {
        // The flow falls through to the next block.
        return;
    }
    let mut operands = String::new();
    for destination in block.next.iter().flatten() {
        let name = destination_label(program, *destination);
        if name.is_empty() {
            continue;
        }
        operands.push(',');
        operands.push_str(&name);
    }
    if operands.is_empty() {
        return;
    }
    line(out, "", "TRANSFER", &operands);
}

/// The name the original gives to a destination: the label of the block, and
/// its name when the block has no label.
fn destination_label(program: &Program, index: usize) -> String {
    let Some(target) = program.blocks.get(index) else {
        return String::new();
    };
    match &target.label {
        Some(label) if !label.is_empty() => label.clone(),
        _ => target.name.clone(),
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
        BlockKind::Preempt(spec) => {
            // The original writes four operands: the facility, the priority
            // of the preempting transaction, whether the owner is routed to
            // the interrupt output, and the parameter that receives the
            // owner's remaining time.
            let mut parts = vec![spec.facility.to_string(), String::new()];
            parts.push(if spec.route_owner { "1".into() } else { String::new() });
            parts.push(spec.remaining.clone().unwrap_or_default());
            statement(out, "PREEMPT", &parts.join(","));
        }
        BlockKind::Return(spec) => {
            statement(out, "RETURN", &spec.facility.to_string());
        }
        BlockKind::Priority(value) => {
            statement(out, "PRIORITY", &value.to_string());
        }
        BlockKind::Transfer(spec) => {
            // The original writes the destinations in the reverse of the port
            // order, and leaves an empty field for a port with no link. The
            // order of the text is not the order the engine follows: the
            // probability goes to the first port (see
            // `scripts/wine-oracle/probes/`).
            let mut operands = probability(spec);
            for target in block.next.iter().rev() {
                operands.push(',');
                if let Some(index) = target {
                    operands.push_str(&destination(program, *index));
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
