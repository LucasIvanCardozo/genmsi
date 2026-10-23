//! The GPSS program of a compiled model.
//!
//! The original editor compiles a model into GPSS and shows it in its
//! "Código GPSS" tab. That text is the original's own statement of what each
//! block does, and it is what this module reproduces: one statement per line,
//! with the block's name, the verb and the operands, and the statements that
//! continue a block carrying no name of their own.
//!
//! The definitions come first, as the original writes them, and the blocks
//! follow in the order the transaction walks them.
//!
//! Two things the original does are worth spelling out, because they are not
//! what the model says:
//!
//! - **The name of a block.** The original does not take the name of a
//!   statement from the model: it gives every block the name of its class
//!   (`A_Advance`, `A_Queue`), with a number when two blocks of the same
//!   class would share that name (`A_Terminate`, `A_Terminate1`). A few
//!   classes write a name of their own: the gates and the storages take the
//!   name of the module, the transfer takes the name of its input port
//!   (`Entrada1`), and the split takes the name of what it does (`Clona`).
//! - **The spelling of an expression.** A typed reference is written with a
//!   `$` where the model writes a `:` (`P:Param` becomes `P$Param`).

use std::collections::{HashMap, HashSet};

use crate::expr::{ProgramExpr, RefKind};
use crate::program::{
    AssignOp, BlockKind, Definitions, ExecBlock, FunctionDef, GateCondition, LogicOp,
    MultiTransferSelection, Program, RandomVariable, RelOp, SelectMode, SelectState,
};

/// The table the original writes for an exponential function.
///
/// The class has no table of its own: the original writes the standard
/// twenty-four point table of the exponential distribution.
const EXPONENTIAL_TABLE: &str = "0,0/0.1,0.104/0.2,0.222/0.3,0.355/0.4,0.509/0.5,0.69/0.6,0.915/0.7,1.2/0.75,1.38/0.8,1.6/0.84,1.83/0.88,2.12/0.9,2.3/0.92,2.52/0.94,2.81/0.95,2.99/0.96,3.2/0.97,3.5/0.98,3.9/0.99,4.6/0.995,5.3/0.998,6.2/0.999,7.0/0.9997,8.0";

/// Render a compiled program as GPSS.
pub fn render(program: &Program) -> String {
    let names = Names::new(program);
    let mut out = String::new();
    definitions(&mut out, &program.definitions);
    destination_functions(&mut out, program, &names);
    let order = flow_order(program);
    for (position, index) in order.iter().enumerate() {
        block_statements(&mut out, *index, program, &names);
        let following = order.get(position + 1).copied();
        connection(&mut out, *index, following, program, &names);
    }
    out
}

/// The name the original gives to every block of the program.
///
/// The names are the ones the original writes, not the ones the model
/// carries: the original gives every block the name of its class, and adds a
/// number when two blocks of the same class would take the same one.
struct Names {
    names: Vec<String>,
}

impl Names {
    /// Name every block of the program.
    fn new(program: &Program) -> Self {
        let definitions = &program.definitions;
        let mut taken: HashSet<String> = HashSet::new();
        for name in definitions
            .storages
            .keys()
            .chain(definitions.qtables.keys())
            .chain(definitions.tables.keys())
            .chain(definitions.functions.keys())
            .chain(definitions.variables.keys())
            .chain(definitions.random_variables.keys())
        {
            taken.insert(name.clone());
        }

        let mut names = vec![String::new(); program.blocks.len()];
        for block in program.blocks.iter() {
            if let BlockKind::MultiTransfer(spec) = &block.kind
                && let Some(go) = &spec.destination_function
            {
                // The function the multifurcation routes by is a name of the
                // program: no block may take it.
                taken.insert(go.clone());
            }
        }
        for (index, block) in program.blocks.iter().enumerate() {
            if block.label.is_none() {
                // A block with no input port is the start of the flow: the
                // original writes it without a name of its own.
                continue;
            }
            if block.class == "Match" {
                // The match names its statement after the label of its
                // conjugate, and that label is a name of the program.
                if let Some(label) = &block.label {
                    taken.insert(label.clone());
                }
            }
            names[index] = unique(&mut taken, base_name(program, block));
        }
        Self { names }
    }

    /// The name of a block, empty when the block has none.
    fn get(&self, index: usize) -> &str {
        self.names.get(index).map(String::as_str).unwrap_or("")
    }
}

/// The name a class gives to its statement.
///
/// Most classes take the name of the module, with the `A_` the original
/// puts in front of it. The classes whose statements come from a macro of the
/// original write a name of their own.
fn base_name(program: &Program, block: &ExecBlock) -> String {
    match block.class.as_str() {
        // The transfer names its statement after the input port of its macro.
        "Transfer" => "Entrada1".to_string(),
        // The split names its statement after what it does.
        "Split" => "Clona".to_string(),
        // These macros write the name of their module.
        "GateF" | "GateL" | "GateS" | "GateM" | "Gate" => block.name.clone(),
        // The multifurcations name their statement after the module too: the
        // macro code of the original writes `='moduleName'`, without the `A_`
        // the other macros put in front.
        "Multifurcacion" | "MultifurcacionP" => block.name.clone(),
        _ if is_storage_macro(program, block) => block.name.clone(),
        _ => format!("A_{}", block.name),
    }
}

/// Whether the block belongs to the macro of the original that stores.
///
/// The macro of `StorageUse` writes the name of its module (`StorageUse`,
/// `StorageUse1`, ...); the macros of `FacilityUse` and `FacilityInt` write
/// the name of their class. A module that says more than one thing and
/// enters a storage is that macro.
fn is_storage_macro(program: &Program, block: &ExecBlock) -> bool {
    let mut statements = 0;
    let mut enters = false;
    for other in program
        .blocks
        .iter()
        .filter(|other| other.module_id == block.module_id)
    {
        statements += 1;
        if matches!(other.kind, BlockKind::Enter(_)) {
            enters = true;
        }
    }
    statements > 1 && enters
}

/// A name that no other block of the program takes.
fn unique(taken: &mut HashSet<String>, base: String) -> String {
    if taken.insert(base.clone()) {
        return base;
    }
    for index in 1.. {
        let candidate = format!("{base}{index}");
        if taken.insert(candidate.clone()) {
            return candidate;
        }
    }
    base
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
        }
    }

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
///
/// The columns are the ones the original uses: the name in fifteen
/// characters, the verb in fifteen more, and the operands after that.
fn line(out: &mut String, name: &str, verb: &str, operands: &str) {
    if operands.is_empty() {
        out.push_str(&format!("{name:<15} {verb:<15}  \n"));
    } else {
        out.push_str(&format!("{name:<15} {verb:<15} {operands}  \n"));
    }
}

/// The definitions: the storages, the tables, the functions and the initial
/// values.
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

    let mut tables: Vec<_> = defs.tables.iter().collect();
    tables.sort_by(|a, b| a.0.cmp(b.0));
    for (name, table) in tables {
        line(
            out,
            name,
            "TABLE",
            &format!(
                "{},{},{},{}",
                sna(&table.attribute),
                table.first,
                table.width,
                table.intervals
            ),
        );
    }

    let mut functions: Vec<_> = defs.functions.iter().collect();
    functions.sort_by(|a, b| a.0.cmp(b.0));
    for (name, function) in functions {
        function_definition(out, name, function);
    }

    let mut variables: Vec<_> = defs.variables.iter().collect();
    variables.sort_by(|a, b| a.0.cmp(b.0));
    for (name, expression) in variables {
        line(out, name, "VARIABLE", &sna(expression));
    }

    let mut randoms: Vec<_> = defs.random_variables.iter().collect();
    randoms.sort_by(|a, b| a.0.cmp(b.0));
    for (name, variable) in randoms {
        line(out, name, "VARIABLE", &random_variable(variable));
    }

    for (name, value) in &defs.initial_savevalues {
        line(out, "", "INITIAL", &format!("X{name},{value}"));
    }
    for (name, _) in &defs.initial_logic {
        line(out, "", "INITIAL", &format!("LS{name}"));
    }
}

/// The function that maps the branch of a multifurcation to its destination.
///
/// The macro code of the original writes `MULTITRANSFER MultifurcacionGO,
/// FN:MultifurcacionCH,{...}`: the function takes the number the branch
/// function chose and gives back the name of the block the transaction goes
/// to.
fn destination_functions(out: &mut String, program: &Program, names: &Names) {
    for block in &program.blocks {
        let BlockKind::MultiTransfer(spec) = &block.kind else {
            continue;
        };
        let Some(go) = &spec.destination_function else {
            continue;
        };
        let argument = match &spec.selection {
            MultiTransferSelection::Weights => match &spec.branch_function {
                Some(branch) => format!("FN${branch}"),
                None => continue,
            },
            MultiTransferSelection::Parameter(parameter) => format!("P${parameter}"),
        };
        line(
            out,
            go,
            "FUNCTION",
            &format!("{argument},D{}", block.next.len()),
        );
        let points: Vec<String> = block
            .next
            .iter()
            .enumerate()
            .map(|(index, target)| {
                let name = target
                    .map(|target| names.get(target).to_string())
                    .unwrap_or_default();
                format!("{},{}", index + 1, name)
            })
            .collect();
        out.push_str(&points.join("/"));
        out.push('\n');
    }
}

/// A function: the statement, and the table of its points below it.
fn function_definition(out: &mut String, name: &str, function: &FunctionDef) {
    match function {
        FunctionDef::Continuous { argument, points } => {
            function_table(out, name, &sna(argument), "C", points)
        }
        FunctionDef::Discrete { argument, points } => {
            function_table(out, name, &sna(argument), "D", points)
        }
        FunctionDef::Evaluate { argument, points } => {
            function_table(out, name, &sna(argument), "E", points)
        }
        FunctionDef::Exponential { stream } => {
            line(out, name, "FUNCTION", &format!("RN{stream},C24"));
            out.push_str(EXPONENTIAL_TABLE);
            out.push('\n');
        }
        FunctionDef::Distribution { stream, cumulative } => {
            line(
                out,
                name,
                "FUNCTION",
                &format!("RN{stream},D{}", cumulative.len()),
            );
            // The original writes the distribution as the running sum of the
            // probabilities and the number each of them selects.
            let points: Vec<String> = cumulative
                .iter()
                .enumerate()
                .map(|(index, bound)| format!("{bound},{}", index + 1))
                .collect();
            out.push_str(&points.join("/"));
            out.push('\n');
        }
    }
}

/// A function with a table of its own.
fn function_table(
    out: &mut String,
    name: &str,
    argument: &str,
    kind: &str,
    points: &[(ProgramExpr, ProgramExpr)],
) {
    line(
        out,
        name,
        "FUNCTION",
        &format!("{argument},{kind}{}", points.len()),
    );
    let points: Vec<String> = points
        .iter()
        .map(|(x, y)| format!("{},{}", sna(x), sna(y)))
        .collect();
    out.push_str(&points.join("/"));
    out.push('\n');
}

/// The expression the original writes for a random variable.
fn random_variable(variable: &RandomVariable) -> String {
    let min = sna(&variable.min);
    let max = sna(&variable.max);
    let stream = variable.stream;
    format!("((RN{stream}#1000)+RN{stream})@({min}+({max})-({min}))+({min})")
}

/// An expression as the original writes it.
///
/// A typed reference is written with a `$` where the model writes a `:`.
fn sna(expr: &ProgramExpr) -> String {
    expr.source().replace(':', "$")
}

/// The connection of a block, when the flow does not fall through.
///
/// The original writes the program's blocks in the order the transaction
/// visits them, so a transaction goes to the next block by itself. The
/// connections that break that order are written as a `TRANSFER` with no name
/// of its own.
fn connection(
    out: &mut String,
    index: usize,
    following: Option<usize>,
    program: &Program,
    names: &Names,
) {
    let block = &program.blocks[index];
    if matches!(
        block.kind,
        BlockKind::Transfer(_) | BlockKind::MultiTransfer(_)
    ) {
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
        let name = names.get(*destination);
        if name.is_empty() {
            continue;
        }
        operands.push(',');
        operands.push_str(name);
    }
    if operands.is_empty() {
        return;
    }
    line(out, "", "TRANSFER", &operands);
}

/// The statements of one block: the first carries the name, the rest do not.
fn block_statements(out: &mut String, index: usize, program: &Program, names: &Names) {
    let block = &program.blocks[index];
    let name = names.get(index).to_string();
    let mut first = true;
    let mut statement = |out: &mut String, verb: &str, operands: &str| {
        let column = if first { name.as_str() } else { "" };
        first = false;
        line(out, column, verb, operands);
    };

    match &block.kind {
        BlockKind::Generate(spec) => {
            let operands = operands([
                sna(&spec.mean),
                spec.modifier.as_ref().map(sna).unwrap_or_default(),
                spec.offset.as_ref().map(sna).unwrap_or_default(),
                spec.priority.as_ref().map(sna).unwrap_or_default(),
                spec.limit
                    .map(|limit| limit.to_string())
                    .unwrap_or_default(),
            ]);
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
            let operands = operands([
                sna(&spec.mean),
                spec.modifier.as_ref().map(sna).unwrap_or_default(),
            ]);
            statement(out, "ADVANCE", &operands);
        }
        BlockKind::Examine(spec) => {
            // The `Examine` class examines the transaction itself: the number
            // of the `ExamineNumber` class is not part of it.
            let operands = if spec.transaction {
                format!("{},,", spec.group)
            } else {
                format!("{},{},", spec.group, sna(&spec.number))
            };
            statement(out, "EXAMINE", &operands);
        }
        BlockKind::Remove(spec) => {
            if spec.me {
                // The class takes the transaction out of its group: the
                // `REMOVE` without the comparison of the `RemoveTrans` class.
                statement(out, "REMOVE", &spec.group);
            } else {
                // The original writes: the group, the number of transactions,
                // the parameter it compares and its value.
                let operands = format!(
                    "E {},{},{},{},{}",
                    spec.group,
                    sna(&spec.count),
                    "",
                    spec.parameter.clone().unwrap_or_default(),
                    spec.value.as_ref().map(sna).unwrap_or_default(),
                );
                statement(out, "REMOVE", &operands);
            }
        }
        BlockKind::Assign(spec) => {
            if block.class == "Mark" {
                // The `Mark` class writes the hour in a parameter: the
                // original gives the statement a name of its own.
                statement(out, "MARK", &spec.parameter);
            } else {
                let operator = match spec.op {
                    AssignOp::Set => "",
                    AssignOp::Add => "+",
                    AssignOp::Sub => "-",
                };
                let operands = format!("{}{},{}", spec.parameter, operator, sna(&spec.value));
                statement(out, "ASSIGN", &operands);
            }
        }
        BlockKind::SaveValue(spec) => {
            let operator = match spec.op {
                AssignOp::Set => "",
                AssignOp::Add => "+",
                AssignOp::Sub => "-",
            };
            let operands = format!("{}{},{}", spec.savevalue, operator, sna(&spec.value));
            statement(out, "SAVEVALUE", &operands);
        }
        BlockKind::Counter(spec) => {
            // The counter increments a savevalue and copies it to a parameter.
            let operands = format!("{}+,{}", spec.savevalue, sna(&spec.increment));
            statement(out, "SAVEVALUE", &operands);
            if let Some(parameter) = &spec.parameter {
                statement(out, "ASSIGN", &format!("{parameter},X${}", spec.savevalue));
            }
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
        BlockKind::Preempt(spec) => {
            // The original writes four operands: the facility, the priority
            // of the preempting transaction, whether the owner is routed to
            // the interrupt output, and the parameter that receives the
            // owner's remaining time.
            let priority = match spec.priority {
                crate::program::PreemptPriority::Preempting => "PR",
                crate::program::PreemptPriority::Unconditional => "",
            };
            let mut parts = vec![spec.facility.to_string(), priority.to_string()];
            parts.push(if spec.route_owner {
                "1".into()
            } else {
                String::new()
            });
            parts.push(spec.remaining.clone().unwrap_or_default());
            statement(out, "PREEMPT", &parts.join(","));
        }
        BlockKind::Return(spec) => {
            statement(out, "RETURN", &spec.facility.to_string());
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
        BlockKind::Test(spec) => {
            let operator = match spec.op {
                RelOp::Lt => "L",
                RelOp::Le => "LE",
                RelOp::Eq => "E",
                RelOp::Ne => "NE",
                RelOp::Ge => "GE",
                RelOp::Gt => "G",
            };
            let operands = format!("{operator} {},{}", sna(&spec.left), sna(&spec.right));
            statement(out, "TEST", &operands);
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
                    operands.push_str(names.get(*index));
                }
            }
            statement(out, "TRANSFER", &operands);
        }
        BlockKind::MultiTransfer(spec) => {
            // The original writes the transfer that sends the transaction to
            // the function that maps the branch to its destination.
            let operands = match &spec.destination_function {
                Some(go) => format!(",FN${go}"),
                None => {
                    let mut operands = String::new();
                    for (index, weight) in spec.weights.iter().enumerate() {
                        if index > 0 {
                            operands.push(',');
                        }
                        operands.push_str(&weight.to_string());
                    }
                    operands
                }
            };
            statement(out, "TRANSFER", &operands);
        }
        BlockKind::Gate(spec) => {
            let operands = match &spec.condition {
                GateCondition::Logic { switch, state } => {
                    if block.class == "Gate" {
                        // The `GateM` class evaluates the label of the
                        // transaction: `M` when the transaction must match
                        // and `NM` when it must not.
                        format!("{} {switch}", if *state { "M" } else { "NM" })
                    } else {
                        format!("{} {switch}", if *state { "LS" } else { "L" })
                    }
                }
                GateCondition::Storage { storage, empty } => {
                    format!("{} {storage}", if *empty { "S" } else { "SNF" })
                }
                GateCondition::Facility {
                    facility,
                    available,
                } => format!("{} {facility}", if *available { "NU" } else { "U" }),
            };
            statement(out, "GATE", &operands);
        }
        BlockKind::Logic(spec) => {
            let operator = match spec.op {
                LogicOp::Set => "S",
                LogicOp::Reset => "R",
            };
            statement(out, "LOGIC", &format!("{operator} {}", spec.switch));
        }
        BlockKind::Link(spec) => {
            // The link writes the block it feeds as its last operand.
            let mode = if spec.fifo { "FIFO" } else { "LIFO" };
            let mut parts = vec![spec.chain.to_string(), mode.to_string()];
            if let Some(target) = block.next.first().copied().flatten() {
                parts.push(names.get(target).to_string());
            }
            statement(out, "LINK", &parts.join(","));
        }
        BlockKind::Unlink(spec) => {
            // The unlink writes the block that receives the transactions it
            // unchains, the number of transactions it takes, and the
            // comparison it makes.
            let operands = format!(
                "E {},{},{},{},{}",
                spec.chain,
                alternate_destination(block, names),
                sna(&spec.count),
                spec.compare_parameter.clone().unwrap_or_default(),
                spec.compare_value.as_ref().map(sna).unwrap_or_default(),
            );
            statement(out, "UNLINK", &operands);
        }
        BlockKind::Split(spec) => {
            statement(out, "SPLIT", &sna(&spec.total));
        }
        BlockKind::Assemble(spec) => {
            let verb = if spec.gather { "GATHER" } else { "ASSEMBLE" };
            statement(out, verb, &sna(&spec.count));
        }
        BlockKind::Match(spec) => {
            // The macro of the match writes the transfer to the conjugate
            // first, and the match statement itself after it.
            line(out, &name, "TRANSFER", &format!(",{}", spec.conjugate));
            let label = block.label.clone().unwrap_or_default();
            line(out, &label, "MATCH", &spec.conjugate);
        }
        BlockKind::Loop(spec) => {
            statement(out, "LOOP", &format!("{},", spec.parameter));
        }
        BlockKind::Select(spec) => {
            let class = match spec.kind {
                RefKind::Queue => "Q",
                RefKind::Storage => "S",
                RefKind::Facility => "R",
                RefKind::LogicSwitch => "LS",
                _ => "",
            };
            let operands = match spec.state {
                // The logic switches: `LS` picks one that is set and `LR` one
                // that is not.
                Some(SelectState::LogicSet) => format!(
                    "LS {},{},{}",
                    spec.parameter,
                    sna(&spec.low),
                    sna(&spec.high)
                ),
                Some(SelectState::LogicReset) => format!(
                    "LR {},{},{}",
                    spec.parameter,
                    sna(&spec.low),
                    sna(&spec.high)
                ),
                // The storages: `SF` a full one and `SNF` one that is not.
                Some(SelectState::StorageFull) => format!(
                    "SF {},{},{}",
                    spec.parameter,
                    sna(&spec.low),
                    sna(&spec.high)
                ),
                Some(SelectState::StorageNotFull) => format!(
                    "SNF {},{},{}",
                    spec.parameter,
                    sna(&spec.low),
                    sna(&spec.high)
                ),
                // The entity of a class whose value equals the compared one.
                Some(SelectState::EntityEqual) => format!(
                    "E {},{},{},{},{},",
                    spec.parameter,
                    sna(&spec.low),
                    sna(&spec.high),
                    spec.compare_value.as_ref().map(sna).unwrap_or_default(),
                    class
                ),
                None => match spec.mode {
                    SelectMode::FirstFree => format!(
                        "NU {},{},{}",
                        spec.parameter,
                        sna(&spec.low),
                        sna(&spec.high)
                    ),
                    SelectMode::Max => format!(
                        "MAX {},{},{},,{}",
                        spec.parameter,
                        sna(&spec.low),
                        sna(&spec.high),
                        class
                    ),
                    SelectMode::Min => format!(
                        "MIN {},{},{},,{}",
                        spec.parameter,
                        sna(&spec.low),
                        sna(&spec.high),
                        class
                    ),
                },
            };
            statement(out, "SELECT", &operands);
        }
        BlockKind::Tabulate(spec) => {
            let operands = match &spec.weight {
                Some(weight) => format!("{},{}", spec.table, sna(weight)),
                None => spec.table.clone(),
            };
            statement(out, "TABULATE", &operands);
        }
        BlockKind::Join(spec) => {
            let operands = match &spec.number {
                Some(number) => format!("{},{}", spec.group, sna(number)),
                None => spec.group.clone(),
            };
            statement(out, "JOIN", &operands);
        }
        BlockKind::Scan(spec) => {
            let operands = format!(
                "E {},{},{},{},{}",
                spec.group,
                spec.compare_parameter.clone().unwrap_or_default(),
                spec.compare_value.as_ref().map(sna).unwrap_or_default(),
                spec.from_parameter.clone().unwrap_or_default(),
                spec.to_parameter.clone().unwrap_or_default(),
            );
            statement(out, "SCAN", &operands);
        }
        BlockKind::Alter(spec) => {
            // The original writes: the group, the number of transactions, the
            // parameter it alters and its value, and the parameter it
            // compares and its value.
            let operands = format!(
                "E {},{},{},{},{},{}",
                spec.group,
                sna(&spec.count),
                spec.altered_parameter.clone().unwrap_or_default(),
                spec.altered_value.as_ref().map(sna).unwrap_or_default(),
                spec.compared_parameter.clone().unwrap_or_default(),
                spec.compared_value.as_ref().map(sna).unwrap_or_default(),
            );
            statement(out, "ALTER", &operands);
        }
        BlockKind::Priority(value) => {
            statement(out, "PRIORITY", &sna(value));
        }
        BlockKind::Buffer => {
            statement(out, "BUFFER", "");
        }
        BlockKind::InitialLogic(spec) => {
            let operator = match spec.op {
                LogicOp::Set => "S",
                LogicOp::Reset => "R",
            };
            statement(out, "LOGIC", &format!("{operator} {}", spec.switch));
        }
        BlockKind::NoOp => {}
    }
}

/// The name of the block that receives the transactions an alternate output
/// sends.
fn alternate_destination(block: &ExecBlock, names: &Names) -> String {
    block
        .port_names
        .iter()
        .position(|port| port.starts_with("SalidaAlt"))
        .and_then(|index| block.next.get(index).copied().flatten())
        .map(|target| names.get(target).to_string())
        .unwrap_or_default()
}

/// The operands of a statement, without the empty ones at the end.
fn operands<I: IntoIterator<Item = String>>(parts: I) -> String {
    let parts: Vec<String> = parts.into_iter().collect();
    let mut end = parts.len();
    while end > 0 && parts[end - 1].is_empty() {
        end -= 1;
    }
    parts[..end].join(",")
}

/// The probability of a `TRANSFER`, in the form the original writes it: in
/// thousandths when the block says so.
fn probability(spec: &crate::program::TransferSpec) -> String {
    let source = spec.probability.source();
    if let Ok(value) = source.parse::<f64>() {
        // The original writes the property as it is in the model, in
        // thousandths: 850 means eighty-five per cent.
        let per_thousand = if spec.thousandths {
            value
        } else {
            value * 1000.0
        };
        return (per_thousand).round().to_string();
    }
    source.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn reference_model() -> Program {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/models/TP2__problema1.gmdl");
        let model = genmsi_format::load(&path).expect("the reference model loads");
        crate::compile_model(&model).expect("the reference model compiles")
    }

    /// The statements with their padding removed, so a test can look for a
    /// statement without counting the spaces between the columns.
    fn statements(gpss: &str) -> Vec<String> {
        gpss.lines()
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
        assert!(transfer.starts_with("Entrada1 TRANSFER 850,"), "{transfer}");
    }
}
