//! Compilation of a `.gmdl` model into an executable program.
//!
//! The compiler is the only place that knows the block schema: it resolves
//! `[containerParameter]` placeholders, expands compound blocks into the GPSS
//! statement sequence the original generates (`FacilityUse` becomes
//! `QUEUE`/`SEIZE`/`DEPART`/`ADVANCE`/`RELEASE`, and so on), flattens container
//! links into a block graph, and pre-parses every expression.

use std::collections::{HashMap, HashSet};

use genmsi_format::{Block, GmdlModel};

use crate::error::EngineError;
use crate::expr::{BinOp, Expr, NameSource, ProgramExpr, RefKind};
use crate::rng::BLOCK_STREAM_BASE;

/// The table of the standard normal distribution the `VariableN` class uses.
///
/// The macro code of the original carries twenty-five points: the
/// accumulated probability and ten times the quantile, so that the
/// deviation of the variable is the one the class declares.
const NORMAL_TABLE: &str = ".0|-50;.00003|-40;.00135|-30;.00621|-25;.02275|-20;.06681|-15;\
.11507|-12;.15866|-10;.21186|-8;.27425|-6;.34458|-4;.44|2;.5|0;.56|2;.65542|4;.72575|6;\
.78814|8;.84134|10;.88493|12;.93319|15;.97725|20;.99379|25;.99865|30;.99997|40;1.0|50";

/// The random stream the original's multifurcation writes in the function
/// that chooses the branch (`FUNCTIONDIST MultifurcacionCH,2,{...}`).
const MULTIFURCACION_STREAM: u32 = 2;

/// How an assignment combines the new value with the old one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    /// `=`
    Set,
    /// `+`
    Add,
    /// `-`
    Sub,
}

impl AssignOp {
    /// Combine the current and new value.
    pub fn apply(self, current: f64, value: f64) -> f64 {
        match self {
            Self::Set => value,
            Self::Add => current + value,
            Self::Sub => current - value,
        }
    }

    fn parse(text: &str, node: u32, class: &str) -> Result<Self, EngineError> {
        match text.trim() {
            "" | "=" => Ok(Self::Set),
            "+" => Ok(Self::Add),
            "-" => Ok(Self::Sub),
            other => Err(EngineError::invalid_block(
                node,
                class,
                format!("unknown assignment operator {other:?}"),
            )),
        }
    }
}

/// Comparison operators of `TEST`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelOp {
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `=`
    Eq,
    /// `!=`
    Ne,
    /// `>=`
    Ge,
    /// `>`
    Gt,
}

impl RelOp {
    /// Parse the editor's representation (`=`, `!=`, `<=`, `>=`, `<`, `>`).
    pub fn parse(text: &str, node: u32, class: &str) -> Result<Self, EngineError> {
        match text.trim() {
            "<" => Ok(Self::Lt),
            "<=" => Ok(Self::Le),
            "=" | "==" => Ok(Self::Eq),
            "!=" | "<>" => Ok(Self::Ne),
            ">=" => Ok(Self::Ge),
            ">" => Ok(Self::Gt),
            other => Err(EngineError::invalid_block(
                node,
                class,
                format!("unknown comparison operator {other:?}"),
            )),
        }
    }

    /// Apply the comparison.
    pub fn test(self, left: f64, right: f64) -> bool {
        match self {
            Self::Lt => left < right,
            Self::Le => left <= right,
            Self::Eq => left == right,
            Self::Ne => left != right,
            Self::Ge => left >= right,
            Self::Gt => left > right,
        }
    }
}

/// A named entity operand (`Facility`, `Storage`, `Queue`, `Chain`).
#[derive(Debug, Clone, PartialEq)]
pub struct EntityName {
    /// Literal name or indirect parameter.
    pub source: NameSource,
}

impl EntityName {
    /// Resolve the name through a context.
    pub fn resolve(&self, ctx: &mut dyn crate::expr::Ctx) -> String {
        self.source.resolve(ctx)
    }
}

impl std::fmt::Display for EntityName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(f)
    }
}

/// Generator specification.
#[derive(Debug, Clone)]
pub struct GenerateSpec {
    /// `Cantidad`: maximum number of transactions.
    pub limit: Option<f64>,
    /// `Media`: mean interarrival.
    pub mean: ProgramExpr,
    /// `Modificador`: spread or distribution function.
    pub modifier: Option<ProgramExpr>,
    /// `OffSet`: time of the first arrival.
    pub offset: Option<ProgramExpr>,
    /// `Prioridad`: priority of generated transactions.
    pub priority: Option<ProgramExpr>,
}

/// `TERMINATE` specification.
#[derive(Debug, Clone, Copy)]
pub struct TerminateSpec {
    /// Operand A: units subtracted from the start counter.
    pub units: f64,
    /// Whether the block was synthesized from a `Clock`.
    pub synthetic: bool,
}

/// `ADVANCE` specification.
#[derive(Debug, Clone)]
pub struct AdvanceSpec {
    /// Operand A: mean time.
    pub mean: ProgramExpr,
    /// Operand B: spread or distribution function.
    pub modifier: Option<ProgramExpr>,
}

/// `ASSIGN` specification.
#[derive(Debug, Clone)]
pub struct AssignSpec {
    /// Parameter name.
    pub parameter: String,
    /// Operator.
    pub op: AssignOp,
    /// Operand.
    pub value: ProgramExpr,
}

/// `SAVEVALUE` specification.
#[derive(Debug, Clone)]
pub struct SaveValueSpec {
    /// Savevalue name.
    pub savevalue: String,
    /// Operator.
    pub op: AssignOp,
    /// Operand.
    pub value: ProgramExpr,
}

/// `Counter` specification: increment a savevalue and copy it to a parameter.
#[derive(Debug, Clone)]
pub struct CounterSpec {
    /// Savevalue incremented.
    pub savevalue: String,
    /// Parameter that receives the new value (may be empty).
    pub parameter: Option<String>,
    /// Increment.
    pub increment: ProgramExpr,
}

/// `QUEUE` / `DEPART` specification.
#[derive(Debug, Clone)]
pub struct QueueSpec {
    /// Queue name.
    pub queue: EntityName,
    /// Units to enqueue/dequeue.
    ///
    /// The original's `ARRIVE [Queue],[Cantidad]` and `DEPART [Queue],[Cantidad]`
    /// carry the units a transaction occupies in the queue: a queue counts
    /// units, not transactions.
    pub units: ProgramExpr,
}

/// Facility operand of `SEIZE` / `RELEASE`.
#[derive(Debug, Clone)]
pub struct FacilitySpec {
    /// Facility name.
    pub facility: EntityName,
}

/// `PREEMPT` specification (from `FacilityInt` and from the `Preempt` class).
#[derive(Debug, Clone)]
pub struct PreemptSpec {
    /// Facility name.
    pub facility: EntityName,
    /// How the preemption is decided when the facility is busy.
    pub priority: PreemptPriority,
    /// Parameter receiving the preempted owner's remaining time.
    pub remaining: Option<String>,
    /// Whether the owner is routed to the interrupt output.
    pub route_owner: bool,
    /// Whether the preempted owner is removed instead of going on.
    pub remove_owner: bool,
}

/// How a `PREEMPT` decides whether it may take the facility.
///
/// The original's macro code gives the mode as a `Listed` property: `Prioridad`
/// (the GPSS operand `PR`, the preempting transaction's own priority decides)
/// or `Incondicional` (the preemption happens whatever the priorities are).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreemptPriority {
    /// The preempting transaction's priority decides.
    Preempting,
    /// The preemption happens whatever the priorities are.
    Unconditional,
}

/// Storage operand of `ENTER` / `LEAVE`.
#[derive(Debug, Clone)]
pub struct StorageUnitsSpec {
    /// Storage name.
    pub storage: EntityName,
    /// Units to take or release.
    ///
    /// This is the `Cantidad` operand: the units one transaction takes on
    /// entry and gives back on exit. The capacity of the storage is the
    /// `Tamaño` its `StorageDef` declares, also in units, so a storage of
    /// `Tamaño` 1 and a `Cantidad` of 1 holds one transaction at a time.
    pub units: ProgramExpr,
}

/// `TEST` specification.
#[derive(Debug, Clone)]
pub struct TestSpec {
    /// Left operand.
    pub left: ProgramExpr,
    /// Comparison.
    pub op: RelOp,
    /// Right operand.
    pub right: ProgramExpr,
}

/// `TRANSFER` specification.
#[derive(Debug, Clone)]
pub struct TransferSpec {
    /// Probability of taking the first output, as a fraction in `[0, 1]`.
    pub probability: ProgramExpr,
    /// Whether the probability is already a fraction (function result /1000).
    pub thousandths: bool,
}

/// `MULTITRANSFER` (GenMSI `Multifurcacion`) specification.
#[derive(Debug, Clone)]
pub struct MultiTransferSpec {
    /// Branch weights, one per output.
    pub weights: Vec<ProgramExpr>,
    /// How the branch is chosen.
    pub selection: MultiTransferSelection,
    /// The distribution function the original creates to choose the branch
    /// (`MultifurcacionCH`); `None` when the class creates none.
    pub branch_function: Option<String>,
    /// The function that maps the chosen branch to its destination
    /// (`MultifurcacionGO`).
    pub destination_function: Option<String>,
}

/// How a `MULTITRANSFER` chooses the branch the transaction takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MultiTransferSelection {
    /// By the weights the model stores (`Multifurcacion`).
    Weights,
    /// By the value of a parameter of the transaction (`MultifurcacionP`).
    Parameter(String),
}

/// Logic switch operation.
///
/// The original has a third operator, the inversion, and it is carried by
/// [`LogicSpec::invert`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicOp {
    /// Set the switch.
    Set,
    /// Reset the switch.
    Reset,
}

/// `LOGIC` specification.
#[derive(Debug, Clone)]
pub struct LogicSpec {
    /// Switch name.
    pub switch: EntityName,
    /// Operation: the switch is set or reset.
    pub op: LogicOp,
    /// Whether the block inverts the switch instead of setting or resetting
    /// it.
    ///
    /// The original's `op_Swich` list has three operators — `S` (set), `R`
    /// (reset) and `I` (invert) — and the manual says of the third one that
    /// "el estado de la llave lógica especificada por el operando A es
    /// invertido". The inversion is a field of its own and not a third
    /// variant of [`LogicOp`], because the report matches `LogicOp`
    /// exhaustively and the two have to keep compiling together.
    pub invert: bool,
}

/// The state of a storage a `GATE` or a `SELECT` tests.
///
/// The original's `op_Logic_Stor` list has six operators, and the letter each
/// one writes in the program listing is its own:
///
/// | Letter | Operator     | What it asks of the storage        |
/// |--------|--------------|------------------------------------|
/// | `SF`   | `Lleno`      | the storage is full                |
/// | `SNF`  | `NoLleno`    | the storage is not full            |
/// | `SE`   | `Vacio`      | the storage is empty               |
/// | `SNE`  | `NoVacio`    | the storage is not empty           |
/// | `SV`   | `Disponible` | the storage is available (it exists) |
/// | `SNV`  | `NoDisponible` | the storage is not available     |
///
/// The truth table is measured against the original with the probes in
/// `scripts/wine-oracle/probes/gates/` (see
/// `odd/tasks/notas-comportamiento.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageOp {
    /// `SF`: the storage is full.
    Full,
    /// `SNF`: the storage is not full.
    NotFull,
    /// `SE`: the storage is empty.
    Empty,
    /// `SNE`: the storage is not empty.
    NotEmpty,
    /// `SV`: the storage is available.
    Available,
    /// `SNV`: the storage is not available.
    NotAvailable,
}

impl StorageOp {
    /// The letter the original writes in the program listing.
    pub fn letter(self) -> &'static str {
        match self {
            Self::Full => "SF",
            Self::NotFull => "SNF",
            Self::Empty => "SE",
            Self::NotEmpty => "SNE",
            Self::Available => "SV",
            Self::NotAvailable => "SNV",
        }
    }

    /// The operator the original names in the property of the class.
    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim() {
            "Lleno" | "Llena" => Some(Self::Full),
            "NoLleno" | "NoLlena" => Some(Self::NotFull),
            "Vacio" | "Vacia" => Some(Self::Empty),
            "NoVacio" | "NoVacia" => Some(Self::NotEmpty),
            "Disponible" => Some(Self::Available),
            "NoDisponible" => Some(Self::NotAvailable),
            _ => None,
        }
    }
}

/// `GATE` condition.
#[derive(Debug, Clone)]
pub enum GateCondition {
    /// Logic switch is set or reset.
    Logic {
        /// Switch name.
        switch: EntityName,
        /// Required state.
        state: bool,
    },
    /// The state of a storage.
    Storage {
        /// Storage name.
        storage: EntityName,
        /// The state the gate asks of the storage.
        op: StorageOp,
    },
    /// Facility is available / unavailable.
    Facility {
        /// Facility name.
        facility: EntityName,
        /// `true` requires available.
        available: bool,
    },
}

/// `GATE` specification.
#[derive(Debug, Clone)]
pub struct GateSpec {
    /// Tested condition.
    pub condition: GateCondition,
}

/// `LINK` specification.
#[derive(Debug, Clone)]
pub struct LinkSpec {
    /// Chain name.
    pub chain: EntityName,
    /// Order of insertion.
    pub fifo: bool,
}

/// `UNLINK` specification.
#[derive(Debug, Clone)]
pub struct UnlinkSpec {
    /// Chain name.
    pub chain: EntityName,
    /// Number of transactions to unlink.
    pub count: ProgramExpr,
    /// Remove every matching transaction.
    pub all: bool,
    /// Parameter compared, when the model filters members.
    pub compare_parameter: Option<String>,
    /// Value compared against the member parameter.
    pub compare_value: Option<ProgramExpr>,
    /// Parameter of the unlinked transaction copied back into the active one.
    pub return_parameter: Option<String>,
}

/// `SPLIT` specification.
#[derive(Debug, Clone)]
pub struct SplitSpec {
    /// Total transactions after the split (original included).
    pub total: ProgramExpr,
}

/// `ASSEMBLE` / `GATHER` specification.
#[derive(Debug, Clone)]
pub struct AssembleSpec {
    /// Number of related transactions required.
    pub count: ProgramExpr,
    /// Whether all transactions continue (`GATHER`) or copies are destroyed.
    pub gather: bool,
}

/// `MATCH` specification.
#[derive(Debug, Clone)]
pub struct MatchSpec {
    /// Label of the conjugate `MATCH` block.
    pub conjugate: String,
}

/// `LOOP` specification.
#[derive(Debug, Clone)]
pub struct LoopSpec {
    /// Parameter decremented.
    pub parameter: String,
}

/// The state an entity is selected by.
///
/// The three selections of the original that pick an entity of a class —a
/// logic switch, a storage and an entity whose value is compared— say in
/// their `Listed` operator which state they want.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectState {
    /// A logic switch that is set (`LS`).
    LogicSet,
    /// A logic switch that is not set (`LR`).
    LogicReset,
    /// A storage that is full (`SF`).
    StorageFull,
    /// A storage that is not full (`SNF`).
    StorageNotFull,
    /// A storage that is empty (`SE`).
    StorageEmpty,
    /// A storage that is not empty (`SNE`).
    StorageNotEmpty,
    /// A storage that is in the "available" state (`SV`).
    ///
    /// The state is the one the manual describes: "El storage tiene que
    /// estar disponible para que sea seleccionado"
    /// (`VISUALSIS.HLP`).  A storage starts available, and only the
    /// statements `SAVAIL` and `SUNAVAIL` change it; the library of
    /// classes of the original has none of them, so every storage of a
    /// model of the editor is available.
    StorageAvailable,
    /// A storage that is not in the "available" state (`SNV`).
    StorageNotAvailable,
    /// An entity whose value equals the compared one (`E`).
    EntityEqual,
}

/// Selection mode of `SELECT`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectMode {
    /// Smallest metric.
    Min,
    /// Largest metric.
    Max,
    /// First entity not in use.
    FirstFree,
}

/// The entity a `SELECT` walks.
///
/// The original's `entitySNAclass` list has 38 classes, and each one names an
/// attribute of an entity: the content of a queue, the units in use of a
/// storage, the busy flag of a facility, the average time of a chain... The
/// engine walks the queues, the storages, the facilities and the logic
/// switches; the other families (the chains, the groups, the labels, the
/// parameters, the savevalues, the variables, the functions and the random
/// seeds) are named and written, but the engine does not walk them yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectEntity {
    /// A queue (`Q`, `QA`, `QC`, `QM`, `QT`, `QX`, `QZ`).
    Queue,
    /// A storage and the units in use in it (`S`, `SA`, `SC`, `SM`, `SR`,
    /// `ST`, `SV`).
    Storage,
    /// A storage and the units left in it (`R`).
    StorageRemaining,
    /// A facility (`F`, `FC`, `FT`, `FV`, `FR`).
    Facility,
    /// A logic switch (`LS`, `LR`).
    LogicSwitch,
    /// A class of a family the engine does not walk yet.
    Other,
}

/// `SELECT` / `SELECTF` specification.
#[derive(Debug, Clone)]
pub struct SelectSpec {
    /// Entity the selection walks.
    pub entity: SelectEntity,
    /// Class of the entity, as the original's `entitySNAclass` list names it
    /// (`Q`, `S`, `R`, `F`, ...): the statement carries it as its last
    /// operand.
    pub class: String,
    /// Parameter receiving the chosen entity number.
    pub parameter: String,
    /// First entity number.
    pub low: ProgramExpr,
    /// Last entity number.
    pub high: ProgramExpr,
    /// Selection mode.
    pub mode: SelectMode,
    /// The state the entity is selected by, when the class names one.
    ///
    /// The `SelectL`, `SelectS` and `SelectSNA` classes of the original pick
    /// the entity of a class whose state is the one their operator names.
    pub state: Option<SelectState>,
    /// Value the entity is compared against (`SelectSNA`).
    pub compare_value: Option<ProgramExpr>,
    /// Whether a selection that finds nothing takes the second output.
    ///
    /// The benches of the original show the transactions of a `SelectL`, a
    /// `SelectS` and a `SelectSNA` going on by the first output when there is
    /// nothing to select; the `Select` and the `SelectF` classes of the
    /// editor take their second output instead.
    pub missing_to_failure: bool,
}

/// `TABULATE` specification.
#[derive(Debug, Clone)]
pub struct TabulateSpec {
    /// Table name.
    pub table: String,
    /// Optional weight (accepted, currently recorded once per observation).
    pub weight: Option<ProgramExpr>,
}

/// Transaction-group membership (`JOIN`).
#[derive(Debug, Clone)]
pub struct JoinSpec {
    /// Group name.
    pub group: String,
    /// Value added to a numeric group; `None` adds the transaction itself.
    pub number: Option<ProgramExpr>,
}

/// `EXAMINE` specification.
///
/// The `ExamineNumber` class asks whether a *number* belongs to a numeric
/// group; the `Examine` class asks whether the *transaction* belongs to a
/// group of transactions.
#[derive(Debug, Clone)]
pub struct ExamineSpec {
    /// Numeric group name.
    pub group: String,
    /// Value tested for membership.
    pub number: ProgramExpr,
    /// Whether the transaction itself is the one examined.
    ///
    /// The `Examine` class of the original examines the transaction: the
    /// number of the `ExamineNumber` class is not part of it.
    pub transaction: bool,
}

/// `REMOVE` specification.
#[derive(Debug, Clone)]
pub struct RemoveSpec {
    /// Transaction group name.
    pub group: String,
    /// Maximum number removed.
    pub count: ProgramExpr,
    /// Remove all matches.
    pub all: bool,
    /// Parameter compared.
    pub parameter: Option<String>,
    /// Value compared.
    pub value: Option<ProgramExpr>,
    /// Whether the transaction itself is the one removed.
    ///
    /// The `RemoveMe` class of the original takes the transaction out of its
    /// group; the `RemoveTrans` class takes the members that match a
    /// comparison.
    pub me: bool,
    /// The number of the numeric mode (`REMOVE E <grupo>,,<número>`).
    ///
    /// The `RemoveNumber` class of the original removes a number from a
    /// numeric group; the manual says so, and the original's own report
    /// shows the group empty and with no numeric group row at all.
    pub number: Option<ProgramExpr>,
}

/// `SCAN` specification.
#[derive(Debug, Clone)]
pub struct ScanSpec {
    /// Transaction group name.
    pub group: String,
    /// Comparison mode: `MIN` selects the smallest `compare_parameter`.
    pub minimum: bool,
    /// Parameter compared (`None` means "compare a value operand").
    pub compare_parameter: Option<String>,
    /// Value compared when no parameter is given.
    pub compare_value: Option<ProgramExpr>,
    /// Parameter copied from the found member.
    pub from_parameter: Option<String>,
    /// Parameter receiving the copied value.
    pub to_parameter: Option<String>,
}

/// `ALTER` specification.
///
/// The macro of the original says it in its own words: the transactions of a
/// group are **compared** (`CompOp`, `CompParam`, `CompValue`) and the ones
/// that match are **altered** (`AltParam`, `AltValor`).
#[derive(Debug, Clone)]
pub struct AlterSpec {
    /// Transaction group name.
    pub group: String,
    /// Parameter altered in the member.
    pub altered_parameter: Option<String>,
    /// New value.
    pub altered_value: Option<ProgramExpr>,
    /// Parameter compared in the member.
    pub compared_parameter: Option<String>,
    /// Value the parameter is compared against.
    pub compared_value: Option<ProgramExpr>,
    /// How the two are compared.
    pub comparison: RelOp,
    /// Number of members altered.
    pub count: ProgramExpr,
}

/// Everything a block does at run time.
#[derive(Debug, Clone)]
pub enum BlockKind {
    /// Create transactions.
    Generate(GenerateSpec),
    /// Remove a transaction and decrement the start counter.
    Terminate(TerminateSpec),
    /// Stay in the model for a delay.
    Advance(AdvanceSpec),
    /// Assign a parameter.
    Assign(AssignSpec),
    /// Modify a savevalue.
    SaveValue(SaveValueSpec),
    /// Increment a savevalue and copy it to a parameter.
    Counter(CounterSpec),
    /// Enter a queue.
    Queue(QueueSpec),
    /// Leave a queue.
    Depart(QueueSpec),
    /// Seize a facility, waiting if busy.
    Seize(FacilitySpec),
    /// Release a facility.
    Release(FacilitySpec),
    /// Preempt a facility.
    Preempt(PreemptSpec),
    /// Return a preempted facility.
    Return(FacilitySpec),
    /// Enter a storage, waiting for units.
    Enter(StorageUnitsSpec),
    /// Release storage units.
    Leave(StorageUnitsSpec),
    /// Branch on a condition.
    Test(TestSpec),
    /// Probabilistic branch.
    Transfer(TransferSpec),
    /// Multi-way probabilistic branch.
    MultiTransfer(MultiTransferSpec),
    /// Conditional block, optionally routing to a failure output.
    Gate(GateSpec),
    /// Set or reset a logic switch.
    Logic(LogicSpec),
    /// Put the transaction in a user chain.
    Link(LinkSpec),
    /// Remove transactions from a user chain.
    Unlink(UnlinkSpec),
    /// Split into copies.
    Split(SplitSpec),
    /// Wait for related transactions.
    Assemble(AssembleSpec),
    /// Wait for a conjugate match.
    Match(MatchSpec),
    /// Decrement a parameter and loop.
    Loop(LoopSpec),
    /// Choose among numbered entities.
    Select(SelectSpec),
    /// Record a value into a table.
    Tabulate(TabulateSpec),
    /// Change the transaction priority.
    Priority(ProgramExpr),
    /// Add to a group.
    Join(JoinSpec),
    /// Test numeric group membership.
    Examine(ExamineSpec),
    /// Remove members from a transaction group.
    Remove(RemoveSpec),
    /// Search a transaction group.
    Scan(ScanSpec),
    /// Alter members of a transaction group.
    Alter(AlterSpec),
    /// Change a logic switch at load time (no runtime effect).
    InitialLogic(LogicSpec),
    /// Place the transaction at the end of the current events chain.
    Buffer,
    /// A block with no simulation semantics (`Note`, containers).
    NoOp,
}

/// One executable statement.
#[derive(Debug, Clone)]
pub struct ExecBlock {
    /// `moduleid` of the module this statement came from.
    pub module_id: u32,
    /// Block class name.
    pub class: String,
    /// Module name.
    pub name: String,
    /// GPSS label (`LOC`), when the module has one.
    pub label: Option<String>,
    /// What the block does.
    pub kind: BlockKind,
    /// Target block per output port, in port order.
    pub next: Vec<Option<usize>>,
    /// Output port names, parallel to `next`.
    pub port_names: Vec<String>,
    /// Random stream used by the block's own draws.
    pub stream: u32,
}

impl ExecBlock {
    /// Target of the output port with `name`, falling back to the first port.
    pub fn next_named(&self, name: &str) -> Option<usize> {
        self.port_names
            .iter()
            .position(|port| port == name)
            .and_then(|index| self.next.get(index).copied().flatten())
            .or_else(|| self.next.first().copied().flatten())
    }

    /// Target of the block's main output: the port named `Salida1`, otherwise
    /// the first port.
    ///
    /// The editor does not always declare `Salida1` first: `FacilityInt` writes
    /// its failure output (`SalidaAlt`) before it, and leaves that port
    /// unlinked, so "first port" is not the main output.
    pub fn next_default(&self) -> Option<usize> {
        self.port_names
            .iter()
            .position(|port| port == "Salida1")
            .and_then(|index| self.next.get(index).copied().flatten())
            .or_else(|| self.next.first().copied().flatten())
    }

    /// Target of the output port at `index`.
    pub fn next_port(&self, index: usize) -> Option<usize> {
        self.next.get(index).copied().flatten()
    }

    /// Whether the failure output (second port) is connected.
    pub fn has_failure_port(&self) -> bool {
        self.next.get(1).is_some_and(Option::is_some)
    }
}

/// Table definition.
#[derive(Debug, Clone)]
pub struct TableDef {
    /// Recorded expression.
    pub attribute: ProgramExpr,
    /// Upper bound of the first interval.
    pub first: f64,
    /// Interval width.
    pub width: f64,
    /// Number of intervals.
    pub intervals: usize,
}

/// Queue table definition.
#[derive(Debug, Clone)]
pub struct QTableDef {
    /// Queue whose waiting times are recorded.
    pub queue: String,
    /// Upper bound of the first interval.
    pub first: f64,
    /// Interval width.
    pub width: f64,
    /// Number of intervals.
    pub intervals: usize,
}

/// Function definition.
#[derive(Debug, Clone)]
pub enum FunctionDef {
    /// Piecewise linear interpolation.
    Continuous {
        /// Argument.
        argument: ProgramExpr,
        /// Points `(x, y)`.
        points: Vec<(ProgramExpr, ProgramExpr)>,
    },
    /// Step function.
    Discrete {
        /// Argument.
        argument: ProgramExpr,
        /// Points `(x, y)`.
        points: Vec<(ProgramExpr, ProgramExpr)>,
    },
    /// Step function whose result is an expression.
    Evaluate {
        /// Argument.
        argument: ProgramExpr,
        /// Points `(x, y expression)`.
        points: Vec<(ProgramExpr, ProgramExpr)>,
    },
    /// Exponential with mean 1 (`FunctionExpon`).
    Exponential {
        /// Random stream.
        stream: u32,
    },
    /// Discrete distribution over indices 1..N (`FunctionDist`).
    Distribution {
        /// Random stream.
        stream: u32,
        /// Cumulative bounds.
        cumulative: Vec<f64>,
    },
}

/// Random variable (`VariableA`): uniform integer in `[min, max]`.
#[derive(Debug, Clone)]
pub struct RandomVariable {
    /// Minimum.
    pub min: ProgramExpr,
    /// Maximum.
    pub max: ProgramExpr,
    /// Random stream.
    pub stream: u32,
}

/// Everything defined outside the block flow.
#[derive(Debug, Clone, Default)]
pub struct Definitions {
    /// `Function` / `FunctionExpon` / `FunctionDist` definitions.
    pub functions: HashMap<String, FunctionDef>,
    /// `Variable` definitions.
    pub variables: HashMap<String, ProgramExpr>,
    /// `VariableA` definitions.
    pub random_variables: HashMap<String, RandomVariable>,
    /// `StorageDef` capacities, in units.
    pub storages: HashMap<String, i64>,
    /// The names of the `StorageDef` definitions, in the order the model
    /// declares them.
    ///
    /// The original writes the storages of its report in the order the
    /// program finishes with them, and the ones the program never touches
    /// keep the order of the declaration (the fifty storages of the model
    /// `Problema 1__semaforo-1` go from `1` to `50`, the order the model
    /// declares them).
    pub storage_order: Vec<String>,
    /// `Table` definitions.
    pub tables: HashMap<String, TableDef>,
    /// The names of the `Table` definitions, in the order the model declares
    /// them.
    ///
    /// The original writes the tables of its listing in the order the model
    /// declares them, and the tables of its report the other way round: the
    /// last one declared is the first one of the report
    /// (`/tmp/all-oracle/` and the probe `tablas_tres`).
    pub table_order: Vec<String>,
    /// `QTable` definitions.
    pub qtables: HashMap<String, QTableDef>,
    /// `InitiaSV` initial values.
    pub initial_savevalues: Vec<(String, f64)>,
    /// `InitialLS` initial switch states.
    pub initial_logic: Vec<(String, bool)>,
}

/// A compiled, runnable model.
#[derive(Debug, Clone)]
pub struct Program {
    /// Model name (file stem or main block name).
    pub title: String,
    /// Executable statements.
    pub blocks: Vec<ExecBlock>,
    /// Definitions.
    pub definitions: Definitions,
    /// Non-fatal observations made while compiling.
    pub diagnostics: Vec<String>,
    /// Number of distinct entities declared, for report headers.
    pub declared_blocks: usize,
}

impl Program {
    /// Block index by module id (first statement of the module).
    pub fn entry_index(&self, module_id: u32) -> Option<usize> {
        self.blocks
            .iter()
            .position(|block| block.module_id == module_id)
    }

    /// Labels of blocks, used by `MATCH` and diagnostics.
    pub fn label_index(&self) -> HashMap<String, usize> {
        let mut map = HashMap::new();
        for (index, block) in self.blocks.iter().enumerate() {
            if let Some(label) = &block.label {
                map.entry(label.clone()).or_insert(index);
            }
        }
        map
    }
}

/// Placeholder fallback policy.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Fallback {
    /// SNA context: an unresolved placeholder becomes `0`.
    Zero,
    /// Name context: an unresolved placeholder keeps its text.
    Literal,
}

/// The block classes the compiler understands.
///
/// The editor's catalogue (`genmsi_format::catalogue`) must stay inside this
/// list: a class the compiler does not know would create a block that
/// simulates nothing, and the user would only find out when the model runs.
pub fn known_classes() -> &'static [&'static str] {
    &[
        "Advance",
        "Alter",
        "Assemble",
        "Assign",
        "Buffer",
        "Clock",
        "Counter",
        "Depart",
        "Enter",
        "Examine",
        "ExamineNumber",
        "FacilityInt",
        "FacilityUse",
        "Function",
        "FunctionDist",
        "FunctionExpon",
        "GateF",
        "GateL",
        "GateM",
        "GateS",
        "Gather",
        "Generate",
        "InitiaSV",
        "InitialLS",
        "JoinMe",
        "JoinNumber",
        "Leave",
        "Link",
        "Logic",
        "Loop",
        "Mark",
        "Match",
        "MFunction",
        "MFunctionDist",
        "Multifurcacion",
        "MultifurcacionP",
        "Note",
        "Preempt",
        "Priority",
        "QTable",
        "Queue",
        "Release",
        "RemoveMe",
        "RemoveNumber",
        "RemoveTrans",
        "Return",
        "SaveValue",
        "Scan",
        "Seize",
        "Select",
        "SelectF",
        "SelectL",
        "SelectS",
        "SelectSNA",
        "Split",
        "StorageDef",
        "StorageUse",
        "Table",
        "Tabulate",
        "Terminate",
        "Test",
        "Transfer",
        "Unlink",
        "Variable",
        "VariableA",
        "VariableN",
    ]
}

/// Compile a model into a runnable program.
pub fn compile(model: &GmdlModel, title: impl Into<String>) -> Result<Program, EngineError> {
    genmsi_format::validate_runnable(model).map_err(EngineError::from)?;
    let mut compiler = Compiler {
        model,
        diagnostics: Vec::new(),
    };
    compiler.compile(title.into())
}

struct Compiler<'a> {
    model: &'a GmdlModel,
    diagnostics: Vec<String>,
}

impl Compiler<'_> {
    fn compile(&mut self, title: String) -> Result<Program, EngineError> {
        let definitions = self.collect_definitions()?;
        let mut blocks = Vec::new();
        let mut entry_index: HashMap<u32, usize> = HashMap::new();
        let mut pending_links: Vec<(u32, Vec<(String, usize)>)> = Vec::new();

        for module in self.model.activity_blocks() {
            let Some(first) = self.expand(module, &mut blocks)? else {
                continue;
            };
            entry_index.insert(module.id, first);
            let ports: Vec<(String, usize)> = module
                .outputs
                .iter()
                .map(|port| (port.name.clone(), port.ioid as usize))
                .collect();
            // Remember which statements belong to the module so the last one can
            // carry the module's real outputs.
            pending_links.push((module.id, ports));
        }

        // Wire the primitives: each module's last statement gets the module outputs.
        let mut module_last: HashMap<u32, usize> = HashMap::new();
        for (index, block) in blocks.iter().enumerate() {
            module_last.insert(block.module_id, index);
        }
        for (index, block) in blocks.iter_mut().enumerate() {
            if module_last.get(&block.module_id) == Some(&index) {
                continue;
            }
            // Intermediate statement of a compound block: single output to the next.
            let next_index = index + 1;
            block.next = vec![Some(next_index)];
            block.port_names = vec!["Salida1".to_string()];
        }

        // Resolve module outputs through the container hierarchy.
        for (module_id, ports) in &pending_links {
            let Some(&last) = module_last.get(module_id) else {
                continue;
            };
            let mut targets = Vec::new();
            for (name, ioid) in ports {
                let target = self.resolve_target(*module_id, *ioid as u32, &entry_index)?;
                targets.push((name.clone(), target));
            }
            let block = &mut blocks[last];
            block.port_names = targets.iter().map(|(name, _)| name.clone()).collect();
            block.next = targets.into_iter().map(|(_, target)| target).collect();
        }

        // Check MATCH conjugates against the labels the program actually has.
        let label_map: HashMap<String, usize> = blocks
            .iter()
            .enumerate()
            .filter_map(|(index, block)| block.label.clone().map(|label| (label, index)))
            .collect();
        self.check_match_targets(&blocks, &label_map)?;
        self.check_references(&blocks, &definitions)?;
        self.report_undeclared_storages(&blocks, &definitions);
        self.report_examine_without_destination(&blocks);

        Ok(Program {
            title,
            declared_blocks: blocks.len(),
            blocks,
            definitions,
            diagnostics: std::mem::take(&mut self.diagnostics),
        })
    }

    /// Expand one module into one or more GPSS statements.
    fn expand(
        &mut self,
        module: &Block,
        blocks: &mut Vec<ExecBlock>,
    ) -> Result<Option<usize>, EngineError> {
        let class = module.class.as_str();
        let node = module.id;
        let label = module.label().map(str::to_string);
        let stream = BLOCK_STREAM_BASE + node;
        macro_rules! push {
            ($kind:expr, $label:expr, $class:expr $(,)?) => {{
                let index = blocks.len();
                blocks.push(ExecBlock {
                    module_id: node,
                    class: $class.to_string(),
                    name: module.name.clone(),
                    label: $label,
                    kind: $kind,
                    next: Vec::new(),
                    port_names: Vec::new(),
                    stream,
                });
                index
            }};
        }

        match class {
            // Definitions and non-runtime modules.
            "Function" | "FunctionDist" | "FunctionExpon" | "MFunction" | "MFunctionDist"
            | "Variable" | "VariableA" | "VariableN" | "Table" | "QTable" | "StorageDef"
            | "InitiaSV" | "InitialLS" | "Note" => Ok(None),
            "Clock" => {
                let frequency = self.require_number(module, "Frecuencia")?;
                let generate = push!(
                    BlockKind::Generate(GenerateSpec {
                        limit: None,
                        mean: self.number_expr(frequency),
                        modifier: None,
                        offset: None,
                        priority: None,
                    }),
                    None,
                    "Generate",
                );
                let terminate = push!(
                    BlockKind::Terminate(TerminateSpec {
                        units: 1.0,
                        synthetic: true,
                    }),
                    None,
                    "Terminate",
                );
                blocks[generate].next = vec![Some(terminate)];
                blocks[generate].port_names = vec!["Salida1".to_string()];
                Ok(Some(generate))
            }
            "Generate" => {
                // The mean the model does not set is empty, not zero: the
                // original writes `GENERATE ,,,1` for a generator whose
                // `Media` is empty (the `TP10` report, row 52). The value of
                // the mean is still zero, so the engine behaves the same.
                let mean = self.sna(module, "Media")?.unwrap_or_else(|| self.empty_expr());
                let spec = GenerateSpec {
                    limit: self.number(module, "Cantidad"),
                    mean,
                    modifier: self.sna(module, "Modificador")?,
                    offset: self.sna(module, "OffSet")?,
                    priority: self.sna(module, "Prioridad")?,
                };
                Ok(Some(push!(BlockKind::Generate(spec), label, class)))
            }
            "Terminate" => {
                let units = self.number(module, "Unidades").unwrap_or(0.0);
                Ok(Some(push!(
                    BlockKind::Terminate(TerminateSpec {
                        units,
                        synthetic: false,
                    }),
                    label,
                    class,
                )))
            }
            "Advance" => {
                let mean = self.sna(module, "Media")?.ok_or_else(|| {
                    EngineError::invalid_block(node, class, "Media is required")
                })?;
                Ok(Some(push!(
                    BlockKind::Advance(AdvanceSpec {
                        mean,
                        modifier: self.sna(module, "Modificador")?,
                    }),
                    label,
                    class,
                )))
            }
            "Assign" => {
                let parameter = self.require_name(module, "Parametro")?;
                let value = self.sna(module, "Valor")?.ok_or_else(|| {
                    EngineError::invalid_block(node, class, "Valor is required")
                })?;
                let op = AssignOp::parse(
                    &module.prop_value("Operador").unwrap_or_default(),
                    node,
                    class,
                )?;
                Ok(Some(push!(
                    BlockKind::Assign(AssignSpec {
                        parameter,
                        op,
                        value,
                    }),
                    label,
                    class,
                )))
            }
            "SaveValue" => {
                let savevalue = self.require_name(module, "SaveValue")?;
                let value = self.sna(module, "Valor")?.ok_or_else(|| {
                    EngineError::invalid_block(node, class, "Valor is required")
                })?;
                let op = AssignOp::parse(
                    &module.prop_value("Op").unwrap_or_default(),
                    node,
                    class,
                )?;
                Ok(Some(push!(
                    BlockKind::SaveValue(SaveValueSpec {
                        savevalue,
                        op,
                        value,
                    }),
                    label,
                    class,
                )))
            }
            "Counter" => {
                let savevalue = self.require_name(module, "SaveValue")?;
                let parameter = self.name(module, "Param");
                let increment = self
                    .sna(module, "Incremento")?
                    .unwrap_or_else(|| self.number_expr(1.0));
                Ok(Some(push!(
                    BlockKind::Counter(CounterSpec {
                        savevalue,
                        parameter,
                        increment,
                    }),
                    label,
                    class,
                )))
            }
            "Queue" => {
                let queue = self.entity(module, "Queue")?;
                let units = self
                    .sna(module, "Cantidad")?
                    .unwrap_or_else(|| self.number_expr(1.0));
                Ok(Some(push!(
                    BlockKind::Queue(QueueSpec { queue, units }),
                    label,
                    class,
                )))
            }
            "Depart" => {
                let queue = self.entity(module, "Queue")?;
                let units = self
                    .sna(module, "Cantidad")?
                    .unwrap_or_else(|| self.number_expr(1.0));
                Ok(Some(push!(
                    BlockKind::Depart(QueueSpec { queue, units }),
                    label,
                    class,
                )))
            }
            "FacilityUse" => {
                let facility = self.entity(module, "Facility")?;
                let mean = self.sna(module, "Media")?.ok_or_else(|| {
                    EngineError::invalid_block(node, class, "Media is required")
                })?;
                let modifier = self.sna(module, "Modificador")?;
                let queue = self.optional_entity(module, "Queue")?;
                let first;
                if let Some(queue) = &queue {
                    first = Some(push!(
                        BlockKind::Queue(QueueSpec {
                            queue: queue.clone(),
                            units: self.number_expr(1.0),
                        }),
                        label.clone(),
                        "Queue",
                    ));
                    blocks.push(ExecBlock {
                        module_id: node,
                        class: "Seize".into(),
                        name: module.name.clone(),
                        label: None,
                        kind: BlockKind::Seize(FacilitySpec {
                            facility: facility.clone(),
                        }),
                        next: Vec::new(),
                        port_names: Vec::new(),
                        stream,
                    });
                    blocks.push(ExecBlock {
                        module_id: node,
                        class: "Depart".into(),
                        name: module.name.clone(),
                        label: None,
                        kind: BlockKind::Depart(QueueSpec {
                            queue: queue.clone(),
                            units: self.number_expr(1.0),
                        }),
                        next: Vec::new(),
                        port_names: Vec::new(),
                        stream,
                    });
                } else {
                    first = Some(push!(
                        BlockKind::Seize(FacilitySpec {
                            facility: facility.clone(),
                        }),
                        label.clone(),
                        "Seize",
                    ));
                }
                blocks.push(ExecBlock {
                    module_id: node,
                    class: "Advance".into(),
                    name: module.name.clone(),
                    label: None,
                    kind: BlockKind::Advance(AdvanceSpec { mean, modifier }),
                    next: Vec::new(),
                    port_names: Vec::new(),
                    stream,
                });
                blocks.push(ExecBlock {
                    module_id: node,
                    class: "Release".into(),
                    name: module.name.clone(),
                    label: None,
                    kind: BlockKind::Release(FacilitySpec { facility }),
                    next: Vec::new(),
                    port_names: Vec::new(),
                    stream,
                });
                Ok(first)
            }
            "Seize" => {
                let facility = self.entity(module, "Facility")?;
                Ok(Some(push!(
                    BlockKind::Seize(FacilitySpec { facility }),
                    label,
                    class,
                )))
            }
            "Release" => {
                let facility = self.entity(module, "Facility")?;
                Ok(Some(push!(
                    BlockKind::Release(FacilitySpec { facility }),
                    label,
                    class,
                )))
            }
            // The `Preempt` class interrupts a facility: one statement, the
            // `PREEMPT` of the original (`PREEMPT Recurso,PR,,Remanente`).
            "Preempt" => {
                let facility = self.entity(module, "Facility")?;
                let remaining = self.name(module, "Remanente");
                let priority = self.preempt_priority(module);
                let remove_owner = self.remove_owner(module);
                let route_owner = module
                    .outputs
                    .iter()
                    .any(|port| port.name == "SalidaAlt" && port.label().is_some());
                Ok(Some(push!(
                    BlockKind::Preempt(PreemptSpec {
                        facility,
                        priority,
                        remaining,
                        route_owner,
                        remove_owner,
                    }),
                    label,
                    class,
                )))
            }
            // The `Return` class gives the facility back: the original's macro
            // code is a `RELEASE`, not the GPSS `RETURN` of the `FacilityInt`
            // class.
            "Return" => {
                let facility = self.entity(module, "Facility")?;
                Ok(Some(push!(
                    BlockKind::Release(FacilitySpec { facility }),
                    label,
                    "Release",
                )))
            }
            "FacilityInt" => {
                let facility = self.entity(module, "Facility")?;
                let mean = self.sna(module, "Media")?.ok_or_else(|| {
                    EngineError::invalid_block(node, class, "Media is required")
                })?;
                let modifier = self.sna(module, "Modificador")?;
                let queue = self.optional_entity(module, "Queue")?;
                let remaining = self.name(module, "Remanente");
                let priority = self.preempt_priority(module);
                let remove_owner = self.remove_owner(module);
                let route_owner = module
                    .outputs
                    .iter()
                    .any(|port| port.name == "SalidaAlt" && port.label().is_some());
                let first;
                if let Some(queue) = &queue {
                    first = Some(push!(
                        BlockKind::Queue(QueueSpec {
                            queue: queue.clone(),
                            units: self.number_expr(1.0),
                        }),
                        label.clone(),
                        "Queue",
                    ));
                    blocks.push(ExecBlock {
                        module_id: node,
                        class: "Preempt".into(),
                        name: module.name.clone(),
                        label: None,
                        kind: BlockKind::Preempt(PreemptSpec {
                            facility: facility.clone(),
                            priority,
                            remaining: remaining.clone(),
                            route_owner,
                            remove_owner,
                        }),
                        next: Vec::new(),
                        port_names: Vec::new(),
                        stream,
                    });
                    blocks.push(ExecBlock {
                        module_id: node,
                        class: "Depart".into(),
                        name: module.name.clone(),
                        label: None,
                        kind: BlockKind::Depart(QueueSpec {
                            queue: queue.clone(),
                            units: self.number_expr(1.0),
                        }),
                        next: Vec::new(),
                        port_names: Vec::new(),
                        stream,
                    });
                } else {
                    first = Some(push!(
                        BlockKind::Preempt(PreemptSpec {
                            facility: facility.clone(),
                            priority,
                            remaining: remaining.clone(),
                            route_owner,
                            remove_owner,
                        }),
                        label.clone(),
                        "Preempt",
                    ));
                }
                blocks.push(ExecBlock {
                    module_id: node,
                    class: "Advance".into(),
                    name: module.name.clone(),
                    label: None,
                    kind: BlockKind::Advance(AdvanceSpec { mean, modifier }),
                    next: Vec::new(),
                    port_names: Vec::new(),
                    stream,
                });
                blocks.push(ExecBlock {
                    module_id: node,
                    class: "Return".into(),
                    name: module.name.clone(),
                    label: None,
                    kind: BlockKind::Return(FacilitySpec { facility }),
                    next: Vec::new(),
                    port_names: Vec::new(),
                    stream,
                });
                Ok(first)
            }
            "StorageUse" => {
                let storage = self.entity(module, "Storage")?;
                let mean = self.sna(module, "Media")?.ok_or_else(|| {
                    EngineError::invalid_block(node, class, "Media is required")
                })?;
                let modifier = self.sna(module, "Modificador")?;
                let queue = self.optional_entity(module, "Queue")?;
                let units = self
                    .sna(module, "Cantidad")?
                    .unwrap_or_else(|| self.number_expr(1.0));
                let first;
                if let Some(queue) = &queue {
                    first = Some(push!(
                        BlockKind::Queue(QueueSpec {
                            queue: queue.clone(),
                            units: units.clone(),
                        }),
                        label.clone(),
                        "Queue",
                    ));
                    blocks.push(ExecBlock {
                        module_id: node,
                        class: "Enter".into(),
                        name: module.name.clone(),
                        label: None,
                        kind: BlockKind::Enter(StorageUnitsSpec {
                            storage: storage.clone(),
                            units: units.clone(),
                        }),
                        next: Vec::new(),
                        port_names: Vec::new(),
                        stream,
                    });
                    blocks.push(ExecBlock {
                        module_id: node,
                        class: "Depart".into(),
                        name: module.name.clone(),
                        label: None,
                        kind: BlockKind::Depart(QueueSpec {
                            queue: queue.clone(),
                            units: units.clone(),
                        }),
                        next: Vec::new(),
                        port_names: Vec::new(),
                        stream,
                    });
                } else {
                    first = Some(push!(
                        BlockKind::Enter(StorageUnitsSpec {
                            storage: storage.clone(),
                            units: units.clone(),
                        }),
                        label.clone(),
                        "Enter",
                    ));
                }
                blocks.push(ExecBlock {
                    module_id: node,
                    class: "Advance".into(),
                    name: module.name.clone(),
                    label: None,
                    kind: BlockKind::Advance(AdvanceSpec { mean, modifier }),
                    next: Vec::new(),
                    port_names: Vec::new(),
                    stream,
                });
                blocks.push(ExecBlock {
                    module_id: node,
                    class: "Leave".into(),
                    name: module.name.clone(),
                    label: None,
                    kind: BlockKind::Leave(StorageUnitsSpec { storage, units }),
                    next: Vec::new(),
                    port_names: Vec::new(),
                    stream,
                });
                Ok(first)
            }
            "Enter" => {
                let storage = self.entity(module, "Storage")?;
                let units = self
                    .sna(module, "Cantidad")?
                    .unwrap_or_else(|| self.number_expr(1.0));
                Ok(Some(push!(
                    BlockKind::Enter(StorageUnitsSpec { storage, units }),
                    label,
                    class,
                )))
            }
            "Leave" => {
                let storage = self.entity(module, "Storage")?;
                let units = self
                    .sna(module, "Cantidad")?
                    .unwrap_or_else(|| self.number_expr(1.0));
                Ok(Some(push!(
                    BlockKind::Leave(StorageUnitsSpec { storage, units }),
                    label,
                    class,
                )))
            }
            "Test" => {
                let left = self.sna(module, "1_Valor")?.ok_or_else(|| {
                    EngineError::invalid_block(node, class, "operand A is required")
                })?;
                let right = self.sna(module, "3_Valor")?.ok_or_else(|| {
                    EngineError::invalid_block(node, class, "operand B is required")
                })?;
                let op = RelOp::parse(
                    &module.prop_value("2_OP").unwrap_or_default(),
                    node,
                    class,
                )?;
                Ok(Some(push!(
                    BlockKind::Test(TestSpec { left, op, right }),
                    label,
                    class,
                )))
            }
            "Transfer" => {
                let text = module.prop_value("Prob").unwrap_or_default();
                let (probability, thousandths) = self.probability(module, &text)?;
                Ok(Some(push!(
                    BlockKind::Transfer(TransferSpec {
                        probability,
                        thousandths,
                    }),
                    label,
                    class,
                )))
            }
            "Multifurcacion" | "MultifurcacionP" => {
                let mut weights = Vec::new();
                for index in 1..=module.outputs.len().max(1) {
                    let name = format!("Prob{index}");
                    if let Some(value) = module.prop_value(&name) {
                        weights.push(self.number_expr(self.percent_to_fraction(&value)?));
                    }
                }
                // The `Multifurcacion` class splits the traffic with the
                // probabilities it stores; the `MultifurcacionP` class routes
                // by the value of a parameter of the transaction.
                let selection = if class == "MultifurcacionP" {
                    let parameter = self.name(module, "Parametro").ok_or_else(|| {
                        EngineError::invalid_block(node, class, "Parametro is required")
                    })?;
                    MultiTransferSelection::Parameter(parameter)
                } else {
                    if weights.is_empty() {
                        return Err(EngineError::invalid_block(
                            node,
                            class,
                            "no probabilities defined",
                        ));
                    }
                    MultiTransferSelection::Weights
                };
                Ok(Some(push!(
                    BlockKind::MultiTransfer(MultiTransferSpec {
                        weights,
                        selection,
                        branch_function: self.name(module, "MultifurcacionCH"),
                        destination_function: self.name(module, "MultifurcacionGO"),
                    }),
                    label,
                    class,
                )))
            }
            "GateM" => {
                // The gate waits for a label: the transaction must match the
                // label the class names, or the opposite when the operator is
                // `NotMaching`. The label travels in the `Logic` condition:
                // the condition of a gate is the thing it evaluates, and the
                // engine tells the two apart by the class of the statement.
                let rotulo = self.entity(module, "Rotulo")?;
                let matching = !module
                    .prop_value("op")
                    .is_some_and(|op| op.trim().eq_ignore_ascii_case("notmaching"));
                Ok(Some(push!(
                    BlockKind::Gate(GateSpec {
                        condition: GateCondition::Logic {
                            switch: rotulo,
                            state: matching,
                        },
                    }),
                    label,
                    "Gate",
                )))
            }
            "GateL" => {
                let switch = self.entity(module, "LogicSwich")?;
                let state = module
                    .prop_value("op")
                    .is_some_and(|op| op.eq_ignore_ascii_case("set"));
                Ok(Some(push!(
                    BlockKind::Gate(GateSpec {
                        condition: GateCondition::Logic { switch, state },
                    }),
                    label,
                    class,
                )))
            }
            "GateS" => {
                let storage = self.entity(module, "Storage")?;
                // The class has six operators (`op_Logic_Stor`); the default of
                // the class is `NoLleno`, the one that opens the gate when the
                // storage still has room.
                let op = module
                    .prop_value("op")
                    .and_then(|op| StorageOp::from_name(&op))
                    .unwrap_or(StorageOp::NotFull);
                Ok(Some(push!(
                    BlockKind::Gate(GateSpec {
                        condition: GateCondition::Storage { storage, op },
                    }),
                    label,
                    class,
                )))
            }
            "GateF" => {
                let facility = self.entity(module, "Facility")?;
                let available = module
                    .prop_value("op")
                    .is_none_or(|op| op.eq_ignore_ascii_case("libre") || op.eq_ignore_ascii_case("nu"));
                Ok(Some(push!(
                    BlockKind::Gate(GateSpec {
                        condition: GateCondition::Facility {
                            facility,
                            available,
                        },
                    }),
                    label,
                    class,
                )))
            }
            "Logic" => {
                let switch = self.entity(module, "LogicSwich")?;
                // The original's `op_Swich` list has three operators: `Set`,
                // `Reset` and `Invertir`. The third one inverts the state of
                // the switch, and it used to be read as a reset, because the
                // compiler only knew about the first two.
                let operator = module.prop_value("Op").unwrap_or_default();
                let operator = operator.trim();
                let invert = operator.eq_ignore_ascii_case("invertir");
                let op = if !invert && operator.eq_ignore_ascii_case("set") {
                    LogicOp::Set
                } else {
                    LogicOp::Reset
                };
                Ok(Some(push!(
                    BlockKind::Logic(LogicSpec { switch, op, invert }),
                    label,
                    class,
                )))
            }
            "Link" => {
                let chain = self.entity(module, "Cadena")?;
                let fifo = module
                    .prop_value("Modo")
                    .is_none_or(|mode| !mode.eq_ignore_ascii_case("LIFO"));
                Ok(Some(push!(
                    BlockKind::Link(LinkSpec { chain, fifo }),
                    label,
                    class,
                )))
            }
            "Unlink" => {
                let chain = self.entity(module, "Cadena")?;
                let count = self
                    .sna(module, "Cantidad")?
                    .unwrap_or_else(|| self.number_expr(1.0));
                let all = module
                    .prop_value("Modo")
                    .is_some_and(|mode| mode.eq_ignore_ascii_case("Todos"));
                let compare_parameter = self.name(module, "CompParam");
                let compare_value = self.sna(module, "CompValue")?;
                Ok(Some(push!(
                    BlockKind::Unlink(UnlinkSpec {
                        chain,
                        count,
                        all,
                        compare_parameter,
                        compare_value,
                        return_parameter: None,
                    }),
                    label,
                    class,
                )))
            }
            "Split" => {
                let total = self
                    .sna(module, "Cantidad")?
                    .ok_or_else(|| EngineError::invalid_block(node, class, "Cantidad is required"))?;
                Ok(Some(push!(
                    BlockKind::Split(SplitSpec { total }),
                    label,
                    class,
                )))
            }
            "Assemble" | "Gather" => {
                let count = self
                    .sna(module, "Cantidad")?
                    .ok_or_else(|| EngineError::invalid_block(node, class, "Cantidad is required"))?;
                Ok(Some(push!(
                    BlockKind::Assemble(AssembleSpec {
                        count,
                        gather: class == "Gather",
                    }),
                    label,
                    class,
                )))
            }
            "Mark" => {
                // The class writes the hour in a parameter: it is the
                // assignment of the clock, and the original writes it as a
                // `MARK` statement of its own.
                let parameter = self.require_name(module, "Parametro")?;
                Ok(Some(push!(
                    BlockKind::Assign(AssignSpec {
                        parameter,
                        op: AssignOp::Set,
                        value: self.clock_expr(),
                    }),
                    label,
                    class,
                )))
            }
            "Match" => {
                let own = module
                    .prop_value("RotuloPropio")
                    .or_else(|| module.label().map(str::to_string))
                    .unwrap_or_else(|| format!("MATCH{node}"));
                let conjugate = module.prop_value("RotuloMatch").unwrap_or_default();
                // The original expands the class into two statements: a
                // `TRANSFER` that sends the transaction to the conjugate and
                // the `MATCH` itself. The report lists the two, and the
                // entries are in the transfer.
                let transfer = push!(
                    BlockKind::Transfer(TransferSpec {
                        probability: self.unconditional_expr(),
                        thousandths: false,
                    }),
                    label,
                    "MatchTransfer",
                );
                let matching = push!(
                    BlockKind::Match(MatchSpec { conjugate }),
                    Some(own),
                    class
                );
                blocks[transfer].next = vec![Some(matching)];
                blocks[transfer].port_names = vec!["Salida1".to_string()];
                Ok(Some(transfer))
            }
            "Loop" => {
                let parameter = self.require_name(module, "Parametro")?;
                Ok(Some(push!(
                    BlockKind::Loop(LoopSpec { parameter }),
                    label,
                    class,
                )))
            }
            "Priority" => {
                let priority = self.sna(module, "Prioridad")?.ok_or_else(|| {
                    EngineError::invalid_block(node, class, "Prioridad is required")
                })?;
                Ok(Some(push!(BlockKind::Priority(priority), label, class)))
            }
            "Select" | "SelectF" | "SelectL" | "SelectS" | "SelectSNA" => {
                let mode = if class == "SelectF" {
                    SelectMode::FirstFree
                } else if class == "Select"
                    && module
                        .prop_value("op")
                        .is_some_and(|op| op.eq_ignore_ascii_case("Maximo"))
                {
                    SelectMode::Max
                } else {
                    SelectMode::Min
                };
                let (entity, entity_class) = match class {
                    // The three classes that pick an entity of a kind say
                    // which one they pick.
                    "SelectF" => (SelectEntity::Facility, "F".to_string()),
                    "SelectS" => (SelectEntity::Storage, "S".to_string()),
                    "SelectL" => (SelectEntity::LogicSwitch, "LS".to_string()),
                    // The `Select` and the `SelectSNA` classes take it from
                    // the `ClaseSNA` property, whose options are the 38
                    // classes of the original's `entitySNAclass` list.
                    _ => {
                        let letter = module
                            .prop_value("ClaseSNA")
                            .unwrap_or_else(|| "Q".to_string());
                        let letter = letter.trim().to_string();
                        let entity = select_entity(&letter).ok_or_else(|| {
                            EngineError::invalid_block(
                                node,
                                class,
                                format!("unsupported SELECT entity class {letter:?}"),
                            )
                        })?;
                        (entity, letter)
                    }
                };
                let parameter = self.require_name(module, "Param")?;
                let low = self
                    .sna(module, "minimo")?
                    .unwrap_or_else(|| self.number_expr(1.0));
                let high = self
                    .sna(module, "maximo")?
                    .unwrap_or_else(|| self.number_expr(1.0));
                let op = module.prop_value("op").unwrap_or_default();
                let state = match class {
                    // `op_Logic_Swich`: `Set` picks a switch that is set and
                    // `Reset` one that is not.
                    "SelectL" => Some(if op.trim().eq_ignore_ascii_case("reset") {
                        SelectState::LogicReset
                    } else {
                        SelectState::LogicSet
                    }),
                    // `op_Logic_Stor`: the state of the storage.
                    "SelectS" => Some(storage_state(&op)),
                    // `op_Relac`: the value of the entity is compared with the
                    // one the class names.
                    "SelectSNA" => Some(SelectState::EntityEqual),
                    _ => None,
                };
                let compare_value = if class == "SelectSNA" {
                    self.sna(module, "valorComp")?
                } else {
                    None
                };
                Ok(Some(push!(
                    BlockKind::Select(SelectSpec {
                        entity,
                        class: entity_class,
                        parameter,
                        low,
                        high,
                        mode,
                        state,
                        compare_value,
                        // The classes that pick an entity of the original go on
                        // by their first output when there is nothing to pick.
                        missing_to_failure: matches!(class, "Select" | "SelectF"),
                    }),
                    label,
                    // The three selections of the original are `SELECT`
                    // statements: the class says which one they are.
                    match class {
                        "SelectL" | "SelectS" | "SelectSNA" => "Select",
                        other => other,
                    },
                )))
            }
            "Tabulate" => {
                let table = self.require_name(module, "NombreTabla")?;
                Ok(Some(push!(
                    BlockKind::Tabulate(TabulateSpec {
                        table,
                        weight: self.sna(module, "Peso")?,
                    }),
                    label,
                    class,
                )))
            }
            "JoinNumber" | "JoinMe" => {
                let group = self.require_name(module, "Grupo")?;
                Ok(Some(push!(
                    BlockKind::Join(JoinSpec {
                        group,
                        number: self.sna(module, "Numero")?,
                    }),
                    label,
                    class,
                )))
            }
            "ExamineNumber" => {
                let group = self.require_name(module, "Grupo")?;
                let number = self.sna(module, "Numero")?.ok_or_else(|| {
                    EngineError::invalid_block(node, class, "Numero is required")
                })?;
                Ok(Some(push!(
                    BlockKind::Examine(ExamineSpec {
                        group,
                        number,
                        transaction: false,
                    }),
                    label,
                    class,
                )))
            }
            "Examine" => {
                // The class asks whether the *transaction* belongs to a group
                // of transactions; the number of the `ExamineNumber` class is
                // not part of it.
                let group = self.require_name(module, "Grupo")?;
                Ok(Some(push!(
                    BlockKind::Examine(ExamineSpec {
                        group,
                        number: self.empty_expr(),
                        transaction: true,
                    }),
                    label,
                    class,
                )))
            }
            "RemoveMe" => {
                // The class takes the transaction out of its group.
                let group = self.require_name(module, "Grupo")?;
                Ok(Some(push!(
                    BlockKind::Remove(RemoveSpec {
                        group,
                        // The class removes the transaction itself: the number
                        // of transactions the `RemoveTrans` class removes is
                        // not part of it.
                        count: self.empty_expr(),
                        all: false,
                        parameter: None,
                        value: None,
                        me: true,
                        number: None,
                    }),
                    label,
                    "Remove",
                )))
            }
            "RemoveNumber" => {
                // The `REMOVE` of the original in numeric mode: the manual
                // says that the statement works on a numeric group when the
                // operand C (the number) is there, and that the transaction
                // goes to the alternative destination (the operand F, the
                // failure output of the class, "No existe el Nº") when the
                // number is not a member of the group.
                //
                // The statement the original writes is
                // `REMOVE E <grupo>,,<número>` (and the destination as the
                // seventh operand when the failure output is connected):
                // `crates/genmsi-core/src/gpss.rs` cannot write it yet (see
                // `odd/tasks/notas-w3-clases.md`).
                let group = self.require_name(module, "Grupo")?;
                let number = self
                    .sna(module, "Numero")?
                    .unwrap_or_else(|| self.number_expr(0.0));
                Ok(Some(push!(
                    BlockKind::Remove(RemoveSpec {
                        group,
                        // The limit of the removal is empty in numeric mode.
                        count: self.empty_expr(),
                        all: false,
                        parameter: None,
                        value: None,
                        me: false,
                        number: Some(number),
                    }),
                    label,
                    // The statement is a `REMOVE`: the report names it by
                    // the verb, as it does with the `RemoveMe` class.
                    "Remove",
                )))
            }
            "RemoveTrans" => {
                let group = self.require_name(module, "Grupo")?;
                let count = self
                    .sna(module, "Cantidad")?
                    .unwrap_or_else(|| self.number_expr(1.0));
                let all = module
                    .prop_value("Modo")
                    .is_some_and(|mode| mode.eq_ignore_ascii_case("Todos"));
                Ok(Some(push!(
                    BlockKind::Remove(RemoveSpec {
                        group,
                        count,
                        all,
                        parameter: self.name(module, "CompParam"),
                        value: self.sna(module, "CompValue")?,
                        me: false,
                        number: None,
                    }),
                    label,
                    class,
                )))
            }
            "Scan" => {
                let group = self.require_name(module, "Grupo")?;
                let minimum = module
                    .prop_value("CompOp")
                    .is_some_and(|op| op.eq_ignore_ascii_case("Minimo"));
                Ok(Some(push!(
                    BlockKind::Scan(ScanSpec {
                        group,
                        minimum,
                        compare_parameter: self.name(module, "CompParam"),
                        compare_value: self.sna(module, "CompValue")?,
                        from_parameter: self.name(module, "DeParam"),
                        to_parameter: self.name(module, "EnParam"),
                    }),
                    label,
                    class,
                )))
            }
            "Alter" => {
                let group = self.require_name(module, "Grupo")?;
                let comparison = RelOp::parse(
                    &module.prop_value("CompOp").unwrap_or_default(),
                    node,
                    class,
                )?;
                Ok(Some(push!(
                    BlockKind::Alter(AlterSpec {
                        group,
                        altered_parameter: self.name(module, "AltParam"),
                        altered_value: self.sna(module, "AltValor")?,
                        compared_parameter: self.name(module, "CompParam"),
                        compared_value: self.sna(module, "CompValue")?,
                        comparison,
                        count: self
                            .sna(module, "Cantidad")?
                            .unwrap_or_else(|| self.number_expr(1.0)),
                    }),
                    label,
                    class,
                )))
            }
            "Buffer" => Ok(Some(push!(BlockKind::Buffer, label, class))),
            other => Err(EngineError::unsupported(
                node,
                class,
                format!("block class {other:?} is not implemented by the engine"),
            )),
        }
    }

    fn check_match_targets(
        &self,
        blocks: &[ExecBlock],
        labels: &HashMap<String, usize>,
    ) -> Result<(), EngineError> {
        for block in blocks {
            let BlockKind::Match(spec) = &block.kind else {
                continue;
            };
            if !spec.conjugate.is_empty() && !labels.contains_key(&spec.conjugate) {
                return Err(EngineError::invalid_block(
                    block.module_id,
                    "Match",
                    format!(
                        "MATCH {} refers to unknown label {:?}",
                        block.name, spec.conjugate
                    ),
                ));
            }
        }
        Ok(())
    }

    /// Verify that every statically named function and variable exists.
    fn check_references(
        &self,
        blocks: &[ExecBlock],
        definitions: &Definitions,
    ) -> Result<(), EngineError> {
        let mut missing: Vec<(u32, String)> = Vec::new();
        let mut check = |node: u32, expr: &ProgramExpr| {
            collect_references(expr.ast(), &mut |kind, name| {
                let exists = match kind {
                    RefKind::Function => definitions.functions.contains_key(name),
                    RefKind::Variable => {
                        definitions.variables.contains_key(name)
                            || definitions.random_variables.contains_key(name)
                    }
                    _ => true,
                };
                if !exists {
                    missing.push((node, format!("{kind}:{name}")));
                }
            });
        };
        for block in blocks {
            for expr in block_expressions(&block.kind) {
                check(block.module_id, expr);
            }
        }
        for def in definitions.variables.values() {
            check(0, def);
        }
        for table in definitions.tables.values() {
            check(0, &table.attribute);
        }
        for function in definitions.functions.values() {
            match function {
                FunctionDef::Continuous { argument, points }
                | FunctionDef::Discrete { argument, points }
                | FunctionDef::Evaluate { argument, points } => {
                    check(0, argument);
                    for (x, y) in points {
                        check(0, x);
                        check(0, y);
                    }
                }
                FunctionDef::Exponential { .. } | FunctionDef::Distribution { .. } => {}
            }
        }
        if let Some((node, name)) = missing.first() {
            return Err(EngineError::invalid_block(
                *node,
                "expression",
                format!("{name} is referenced but never defined"),
            ));
        }
        Ok(())
    }

    /// Report the storages the model uses that nothing declares.
    ///
    /// The original's `StorageUse` says it plainly: "El Storage debe estar
    /// declarado (esta clase no lo declara)", and it adds "Se debe utilizar
    /// otro módulo que defina la capacidad del Storage". A storage without a
    /// declaration has no capacity to offer, so the engine never invents one:
    /// the run stops when a transaction reaches it, exactly as the original
    /// does ("Referencia a una entidad storage inexistente").
    ///
    /// The model still compiles: the original opens a model that uses an
    /// undeclared storage, and the simulation is what stops.
    ///
    /// The format reads the same finding from the model; the engine takes its
    /// wording when it has one, so the two do not say the same thing twice.
    fn report_undeclared_storages(&mut self, blocks: &[ExecBlock], definitions: &Definitions) {
        let findings = genmsi_format::undeclared_entities(self.model);
        let mut reported: HashSet<(u32, String)> = HashSet::new();
        for block in blocks {
            let storage = match &block.kind {
                BlockKind::Enter(spec) | BlockKind::Leave(spec) => Some(&spec.storage),
                BlockKind::Gate(spec) => match &spec.condition {
                    GateCondition::Storage { storage, .. } => Some(storage),
                    _ => None,
                },
                _ => None,
            };
            let Some(storage) = storage else {
                continue;
            };
            // An indirect name (`*Param`) is resolved while the model runs,
            // and the engine refuses it then.
            let NameSource::Literal(name) = &storage.source else {
                continue;
            };
            if definitions.storages.contains_key(name) {
                continue;
            }
            // One finding per module: a `StorageUse` becomes an `ENTER` and a
            // `LEAVE` of the same module.
            if !reported.insert((block.module_id, name.clone())) {
                continue;
            }
            let finding = findings
                .iter()
                .find(|entity| {
                    entity.kind.eq_ignore_ascii_case("Storage")
                        && entity.block == block.module_id
                        && entity.name == *name
                })
                .map(|entity| entity.message())
                .unwrap_or_else(|| {
                    format!(
                        "the model uses the storage '{name}' in the {} block {}, and no \
                         StorageDef declares it",
                        block.class, block.module_id
                    )
                });
            self.diagnostics.push(format!(
                "{finding} The storage has no capacity: the run stops when a transaction \
                 reaches it."
            ));
        }
    }

    /// Report the examinations that have no destination of their own.
    ///
    /// The third operand of the `EXAMINE` is the block the transaction goes
    /// to when the group does not hold it, and the manual of the original
    /// says of it: "Número de bloque. Destino alternativo para la
    /// transacción activa si no es encontrado ningún miembro. Requerido"
    /// (`VISUALSIS.HLP`).  The engine of the original refuses a model whose
    /// `EXAMINE` has no such operand ("Error en linea 1 -> Faltan
    /// parámetros"), so the engine of this repository says it out loud
    /// instead of compiling a model the original would not run.
    fn report_examine_without_destination(&mut self, blocks: &[ExecBlock]) {
        for block in blocks {
            if !matches!(block.kind, BlockKind::Examine(_)) {
                continue;
            }
            let destination = block
                .port_names
                .iter()
                .position(|name| name == "Salida2")
                .and_then(|index| block.next.get(index).copied().flatten());
            if destination.is_some() {
                continue;
            }
            self.diagnostics.push(format!(
                "the model examines a group in the {} block {} without a destination of \
                 its own: the original refuses the model (\"Faltan parámetros\"). The \
                 third operand of the EXAMINE is the block the transaction goes to when \
                 the group does not hold it.",
                block.class, block.module_id
            ));
        }
    }

    fn collect_definitions(&mut self) -> Result<Definitions, EngineError> {
        let mut definitions = Definitions::default();
        for module in self.model.activity_blocks() {
            let node = module.id;
            let class = module.class.as_str();
            match class {
                "StorageDef" => {
                    let name = self.require_name(module, "Storage")?;
                    // The `Tamaño` is the capacity of the storage, in units:
                    // how many units it holds at a time.
                    let capacity = self.number(module, "Tamaño").unwrap_or(0.0) as i64;
                    if !definitions.storage_order.contains(&name) {
                        definitions.storage_order.push(name.clone());
                    }
                    definitions.storages.insert(name, capacity.max(0));
                }
                "InitiaSV" => {
                    let name = self.require_name(module, "SaveValue")?;
                    let value = self.number(module, "Valor").unwrap_or(0.0);
                    definitions.initial_savevalues.push((name, value));
                }
                "InitialLS" => {
                    let switch = self.require_name(module, "LogicSwich")?;
                    // The editor stores the initial value in `Valor`; older files
                    // and hand-written fixtures use `Op`.
                    let state = module
                        .prop_value("Valor")
                        .or_else(|| module.prop_value("Op"))
                        .is_some_and(|op| op.eq_ignore_ascii_case("Set"));
                    definitions.initial_logic.push((switch, state));
                }
                "Variable" => {
                    let name = self.require_name(module, "NombreVar")?;
                    let expression = self
                        .sna(module, "Expresion")?
                        .ok_or_else(|| {
                            EngineError::invalid_block(node, class, "Expresion is required")
                        })?;
                    definitions.variables.insert(name, expression);
                }
                "VariableA" => {
                    let name = self.require_name(module, "NombreVar")?;
                    let min = self
                        .sna(module, "Minimo")?
                        .unwrap_or_else(|| self.number_expr(0.0));
                    let max = self
                        .sna(module, "Maximo")?
                        .unwrap_or_else(|| self.number_expr(0.0));
                    let stream = self.number(module, "Seed").unwrap_or(0.0) as u32;
                    definitions
                        .random_variables
                        .insert(name, RandomVariable { min, max, stream });
                }
                "VariableN" => {
                    // The class creates its own function (the standard normal
                    // distribution, drawn from the seed it names) and the
                    // variable that reads it.
                    let function = self.require_name(module, "NombreFunc")?;
                    let variable = self.require_name(module, "NombreVar")?;
                    let seed = self.number(module, "Seed").unwrap_or(90.0) as u32;
                    let argument = ProgramExpr::parse(&format!("RN{seed}")).map_err(|err| {
                        EngineError::invalid_block(node, class, format!("{err}"))
                    })?;
                    let points = self.parse_points(module, NORMAL_TABLE)?;
                    definitions.functions.insert(
                        function.clone(),
                        FunctionDef::Continuous { argument, points },
                    );
                    let expression = self.normal_variable(module, &function)?;
                    definitions.variables.insert(variable, expression);
                }
                "Multifurcacion" => {
                    // The macro code of the original creates the function that
                    // chooses the branch: a distribution over the shares the
                    // class declares.
                    let Some(name) = self.name(module, "MultifurcacionCH") else {
                        continue;
                    };
                    let mut weights = Vec::new();
                    for index in 1..=5 {
                        let Some(value) = module.prop_value(&format!("Prob{index}")) else {
                            continue;
                        };
                        weights.push(self.percent_to_fraction(&value)?);
                    }
                    if weights.is_empty() {
                        continue;
                    }
                    let total: f64 = weights.iter().sum();
                    let mut cumulative = Vec::with_capacity(weights.len());
                    let mut running = 0.0;
                    for weight in &weights {
                        running += weight / total;
                        // The original writes the running sum with the
                        // decimals it can show; the sums of twentieths are
                        // rounded so that the text says `0.6` and not
                        // `0.6000000000000001`.
                        cumulative.push((running * 1e10).round() / 1e10);
                    }
                    if let Some(last) = cumulative.last_mut() {
                        *last = 1.0;
                    }
                    definitions.functions.insert(
                        name,
                        FunctionDef::Distribution {
                            stream: MULTIFURCACION_STREAM,
                            cumulative,
                        },
                    );
                }
                "Function" => {
                    let name = self.require_name(module, "NombreFunc")?;
                    let argument = self
                        .sna(module, "Atributo")?
                        .unwrap_or_else(|| self.number_expr(0.0));
                    let text = module
                        .prop_value("Tabla")
                        .ok_or_else(|| EngineError::invalid_block(node, class, "Tabla is required"))?;
                    let points = self.parse_points(module, &text)?;
                    let kind = module.prop_value("Tipo").unwrap_or_default();
                    let def = if kind.eq_ignore_ascii_case("Continua") {
                        FunctionDef::Continuous { argument, points }
                    } else if kind.eq_ignore_ascii_case("Eval") {
                        FunctionDef::Evaluate { argument, points }
                    } else {
                        FunctionDef::Discrete { argument, points }
                    };
                    definitions.functions.insert(name, def);
                }
                "FunctionExpon" => {
                    let name = self.require_name(module, "NombreFunc")?;
                    let stream = self.number(module, "Seed").unwrap_or(1.0) as u32;
                    definitions
                        .functions
                        .insert(name, FunctionDef::Exponential { stream: stream.max(1) });
                }
                "FunctionDist" => {
                    let name = self.require_name(module, "NombreDist")?;
                    let text = module.prop_value("Distribucion").ok_or_else(|| {
                        EngineError::invalid_block(node, class, "Distribucion is required")
                    })?;
                    let weights = self.parse_percent_list(module, &text)?;
                    let stream = self.number(module, "Semilla").unwrap_or(1.0) as u32;
                    definitions.functions.insert(
                        name,
                        FunctionDef::Distribution {
                            stream: stream.max(1),
                            cumulative: cumulative_bounds(&weights),
                        },
                    );
                }
                "MFunction" => {
                    // The class defines several functions over the same
                    // input attribute: one row of the matrix per function,
                    // and the row holds the value of the function for each
                    // input (the first, the second, ...).
                    let names = self.parse_name_list(module, "NombresFunc")?;
                    let argument = self
                        .sna(module, "Atributo")?
                        .unwrap_or_else(|| self.number_expr(0.0));
                    let rows = self.parse_matrix(module, "Matriz")?;
                    for (index, name) in names.iter().enumerate() {
                        let Some(row) = rows.get(index) else {
                            continue;
                        };
                        let mut points = Vec::with_capacity(row.len());
                        for (position, value) in row.iter().enumerate() {
                            points.push((
                                self.number_expr(position as f64 + 1.0),
                                self.resolve_sna_text(module, value)?,
                            ));
                        }
                        if points.is_empty() {
                            continue;
                        }
                        definitions.functions.insert(
                            name.clone(),
                            FunctionDef::Evaluate {
                                argument: argument.clone(),
                                points,
                            },
                        );
                    }
                }
                "MFunctionDist" => {
                    // The class defines several distribution functions: one
                    // row of the matrix of results per function, with the
                    // probability of each result.
                    let names = self.parse_name_list(module, "NombresDist")?;
                    let rows = self.parse_percent_matrix(module, "Resultados")?;
                    let stream = self.number(module, "Semilla").unwrap_or(1.0) as u32;
                    for (index, name) in names.iter().enumerate() {
                        let Some(weights) = rows.get(index) else {
                            continue;
                        };
                        if weights.is_empty() {
                            continue;
                        }
                        definitions.functions.insert(
                            name.clone(),
                            FunctionDef::Distribution {
                                stream: stream.max(1),
                                cumulative: cumulative_bounds(weights),
                            },
                        );
                    }
                }
                "Table" => {
                    let name = self.require_name(module, "NombreTabla")?;
                    let attribute = self
                        .sna(module, "Atributo")?
                        .ok_or_else(|| {
                            EngineError::invalid_block(node, class, "Atributo is required")
                        })?;
                    let first = self.number(module, "PrimerIntervalo").unwrap_or(0.0);
                    let width = self.number(module, "TamFrecuencia").unwrap_or(1.0);
                    let intervals = self.number(module, "Cantidad").unwrap_or(1.0) as usize;
                    definitions.tables.insert(
                        name.clone(),
                        TableDef {
                            attribute,
                            first,
                            width,
                            intervals: intervals.max(1),
                        },
                    );
                    if !definitions.table_order.contains(&name) {
                        // The order the model declares the tables in is the
                        // order the original writes them in.
                        definitions.table_order.push(name);
                    }
                }
                "QTable" => {
                    let name = self.require_name(module, "NombreTabla")?;
                    let queue = self.require_name(module, "Queue")?;
                    let first = self.number(module, "PrimerIntervalo").unwrap_or(0.0);
                    let width = self.number(module, "TamFrecuencia").unwrap_or(1.0);
                    let intervals = self.number(module, "Cantidad").unwrap_or(1.0) as usize;
                    definitions.qtables.insert(
                        name,
                        QTableDef {
                            queue,
                            first,
                            width,
                            intervals: intervals.max(1),
                        },
                    );
                }
                _ => {}
            }
        }
        Ok(definitions)
    }

    /// The expression of the `VariableN` class.
    ///
    /// The macro code of the original writes `[Media]+([Desvio]#FN:NORMAL)/10`:
    /// the `#` is the multiplication of the original's language (its
    /// expression editor offers no `*` key) and the table of the function
    /// holds ten times the quantile, so that the deviation of the variable
    /// is the one the class declares. The text keeps the spelling of the
    /// original, whose `#` is the multiplication.
    fn normal_variable(&self, module: &Block, function: &str) -> Result<ProgramExpr, EngineError> {
        let media = self
            .sna(module, "Media")?
            .unwrap_or_else(|| self.number_expr(0.0));
        let desvio = self
            .sna(module, "Desvio")?
            .unwrap_or_else(|| self.number_expr(0.0));
        let source = format!("{}+({}#FN:{})/10", media.source(), desvio.source(), function);
        let tree = Expr::Bin(
            BinOp::Add,
            Box::new(media.ast().clone()),
            Box::new(Expr::Bin(
                BinOp::Div,
                Box::new(Expr::Bin(
                    BinOp::Mul,
                    Box::new(desvio.ast().clone()),
                    Box::new(Expr::Ref {
                        kind: RefKind::Function,
                        name: NameSource::Literal(function.to_string()),
                    }),
                )),
                Box::new(Expr::Num(10.0)),
            )),
        );
        Ok(ProgramExpr::new(source, tree))
    }

    /// The names of a list property (`{FuncA;FuncB}`).
    ///
    /// The `MFunction` and the `MFunctionDist` classes declare the names of
    /// the functions they define in a list: one name per function.
    fn parse_name_list(
        &self,
        module: &Block,
        prop: &str,
    ) -> Result<Vec<String>, EngineError> {
        let text = module.prop_value(prop).ok_or_else(|| {
            EngineError::invalid_block(module.id, &module.class, format!("{prop} is required"))
        })?;
        Ok(strip_braces(text.trim())
            .split(';')
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty())
            .collect())
    }

    /// The rows of a matrix property (`{{11;12;13};{21;22;23}}`).
    ///
    /// The `MFunction` class declares the values of every function it
    /// defines in a matrix: one row per function.
    fn parse_matrix(
        &self,
        module: &Block,
        prop: &str,
    ) -> Result<Vec<Vec<String>>, EngineError> {
        let text = module.prop_value(prop).ok_or_else(|| {
            EngineError::invalid_block(module.id, &module.class, format!("{prop} is required"))
        })?;
        Ok(split_outside_braces(strip_braces(text.trim()))
            .iter()
            .map(|row| {
                strip_braces(row.trim())
                    .split(';')
                    .map(|cell| cell.trim().to_string())
                    .filter(|cell| !cell.is_empty())
                    .collect()
            })
            .filter(|row: &Vec<String>| !row.is_empty())
            .collect())
    }

    /// The rows of a matrix of percentages (`{{20%;30%;50%};{25%;35%;40%}}`).
    ///
    /// The `MFunctionDist` class declares the probability of every result of
    /// every function it defines in a matrix: one row per function.
    fn parse_percent_matrix(
        &self,
        module: &Block,
        prop: &str,
    ) -> Result<Vec<Vec<f64>>, EngineError> {
        let rows = self.parse_matrix(module, prop)?;
        let mut weights = Vec::with_capacity(rows.len());
        for row in rows {
            let mut values = Vec::with_capacity(row.len());
            for entry in row {
                let value = entry.trim_end_matches('%').trim();
                let number = value.parse::<f64>().map_err(|_| {
                    EngineError::invalid_block(
                        module.id,
                        &module.class,
                        format!("invalid percentage {entry:?}"),
                    )
                })?;
                values.push(number.max(0.0));
            }
            weights.push(values);
        }
        Ok(weights)
    }

    fn parse_points(
        &self,
        module: &Block,
        text: &str,
    ) -> Result<Vec<(ProgramExpr, ProgramExpr)>, EngineError> {
        let inner = text.trim();
        let inner = inner
            .strip_prefix('{')
            .and_then(|rest| rest.strip_suffix('}'))
            .unwrap_or(inner);
        let mut points = Vec::new();
        for pair in inner.split(';') {
            let pair = pair.trim();
            if pair.is_empty() {
                continue;
            }
            let mut parts = pair.split('|');
            let x = parts.next().unwrap_or_default();
            let y = parts.next().unwrap_or_default();
            points.push((
                self.resolve_sna_text(module, x)?,
                self.resolve_sna_text(module, y)?,
            ));
        }
        if points.is_empty() {
            return Err(EngineError::invalid_block(
                module.id,
                &module.class,
                "function table is empty",
            ));
        }
        Ok(points)
    }

    fn parse_percent_list(
        &self,
        module: &Block,
        text: &str,
    ) -> Result<Vec<f64>, EngineError> {
        let inner = text.trim();
        let inner = inner
            .strip_prefix('{')
            .and_then(|rest| rest.strip_suffix('}'))
            .unwrap_or(inner);
        let mut weights = Vec::new();
        for entry in inner.split(';') {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }
            let value = entry.trim_end_matches('%').trim();
            let resolved = self.resolve_placeholders(module, value, Fallback::Zero)?;
            let number = resolved.trim().parse::<f64>().map_err(|_| {
                EngineError::invalid_block(
                    module.id,
                    &module.class,
                    format!("invalid percentage {entry:?}"),
                )
            })?;
            weights.push(number.max(0.0));
        }
        if weights.is_empty() {
            return Err(EngineError::invalid_block(
                module.id,
                &module.class,
                "distribution is empty",
            ));
        }
        Ok(weights)
    }

    fn probability(
        &self,
        module: &Block,
        text: &str,
    ) -> Result<(ProgramExpr, bool), EngineError> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Ok((self.number_expr(1.0), false));
        }
        if let Some(percent) = trimmed.strip_suffix('%') {
            let value = percent.trim().parse::<f64>().map_err(|_| {
                EngineError::invalid_block(
                    module.id,
                    &module.class,
                    format!("invalid percentage {text:?}"),
                )
            })?;
            return Ok((self.number_expr(value / 100.0), false));
        }
        if let Ok(value) = trimmed.parse::<f64>() {
            return Ok((self.number_expr(value / 1000.0), false));
        }
        let expr = self.resolve_sna_text(module, trimmed)?;
        Ok((expr, true))
    }

    /// How the `PREEMPT` of a class decides whether it may take the facility.
    ///
    /// The original's macro code gives the mode as a `Listed` property: the
    /// `Prioridad` mode writes the GPSS operand `PR` (the preempting
    /// transaction's own priority decides) and the `Incondicional` mode
    /// writes nothing (the preemption happens whatever the priorities are).
    fn preempt_priority(&self, module: &Block) -> PreemptPriority {
        match module.prop_value("ModoInterrupcion") {
            Some(mode) if mode.trim().eq_ignore_ascii_case("incondicional") => {
                PreemptPriority::Unconditional
            }
            _ => PreemptPriority::Preempting,
        }
    }

    /// Whether the preempted owner is removed instead of going on.
    ///
    /// The original's `ModoArrebato` says whether the owner of the
    /// interrupted facility is removed (`Remover`) or kept (`NoRemover`).
    fn remove_owner(&self, module: &Block) -> bool {
        module
            .prop_value("ModoArrebato")
            .is_some_and(|mode| mode.trim().eq_ignore_ascii_case("remover"))
    }

    fn percent_to_fraction(&self, text: &str) -> Result<f64, EngineError> {        let trimmed = text.trim();
        let value = trimmed.trim_end_matches('%').trim();
        let number = value.parse::<f64>().map_err(|_| EngineError::Expression {
            text: text.to_string(),
            message: "invalid percentage".to_string(),
        })?;
        Ok(number / 100.0)
    }

    /// Resolve every output port of `module` to an executable statement index.
    ///
    /// Containers are transparent pass-throughs with two port kinds:
    ///
    /// - an **entry** port appears as a link starting *at* the container
    ///   (`(container, ioid) -> (child, ioid)`), so entering continues inside;
    /// - an **exit** port appears as a link ending *at* the container
    ///   (`(child, ioid) -> (container, ioid)`), so leaving continues in the
    ///   container's parent, where a link starts at `(container, ioid)`.
    ///
    /// Nested containers are followed until an activity block is reached.
    fn resolve_target(
        &self,
        module_id: u32,
        ioid: u32,
        entry_index: &HashMap<u32, usize>,
    ) -> Result<Option<usize>, EngineError> {
        self.resolve_in_owner(module_id, ioid, entry_index, 0)
    }

    fn resolve_in_owner(
        &self,
        module_id: u32,
        ioid: u32,
        entry_index: &HashMap<u32, usize>,
        depth: usize,
    ) -> Result<Option<usize>, EngineError> {
        if depth > 64 {
            return Err(EngineError::Graph {
                message: format!(
                    "container cycle while resolving module {module_id} port {ioid}"
                ),
            });
        }
        let module = self.model.block(module_id).ok_or_else(|| EngineError::Graph {
            message: format!("link points at missing module {module_id}"),
        })?;
        let owner = self.model.block(module.owner).ok_or_else(|| EngineError::Graph {
            message: format!("module {module_id} has missing owner {}", module.owner),
        })?;
        let Some(link) = owner
            .links
            .iter()
            .find(|link| link.from_module == module_id && link.from_ioid == ioid)
        else {
            return Ok(None);
        };
        self.follow(link.to_module, link.to_ioid, entry_index, depth)
    }

    /// Continue from `(module, ioid)` after a link already reached it.
    fn follow(
        &self,
        module_id: u32,
        ioid: u32,
        entry_index: &HashMap<u32, usize>,
        depth: usize,
    ) -> Result<Option<usize>, EngineError> {
        let module = self.model.block(module_id).ok_or_else(|| EngineError::Graph {
            message: format!("link points at missing module {module_id}"),
        })?;
        if !module.is_container() {
            return Ok(entry_index.get(&module_id).copied());
        }
        // Exiting the container: the parent owns the continuation.
        if module.outputs.iter().any(|port| port.ioid == ioid) {
            return self.resolve_in_owner(module_id, ioid, entry_index, depth + 1);
        }
        // Entering the container: an internal link may start at the port.
        if let Some(inner) = module
            .links
            .iter()
            .find(|link| link.from_module == module_id && link.from_ioid == ioid)
        {
            let (next_module, next_ioid) = (inner.to_module, inner.to_ioid);
            return self.follow(next_module, next_ioid, entry_index, depth + 1);
        }
        // The editor also allows an entry port without an explicit internal link;
        // the flow then starts at the container's first child (lowest module id).
        let first_child = self
            .model
            .blocks
            .iter()
            .filter(|block| block.owner == module_id && block.id != module_id)
            .min_by_key(|block| block.id);
        match first_child {
            Some(child) if !child.is_container() => {
                Ok(entry_index.get(&child.id).copied())
            }
            Some(child) => self.follow(child.id, ioid, entry_index, depth + 1),
            None => Ok(None),
        }
    }

    fn number_expr(&self, value: f64) -> ProgramExpr {
        ProgramExpr::parse(&format!("{value}")).expect("formatted numbers always parse")
    }

    /// The expression of the model clock, written the way the original writes
    /// the `MARK` statement: without a text of its own, because the statement
    /// carries the hour in its name.
    fn clock_expr(&self) -> ProgramExpr {
        ProgramExpr::new(
            String::new(),
            Expr::System(crate::expr::SystemSymbol::Clock),
        )
    }

    /// An expression with no text: the operand the original leaves empty.
    fn empty_expr(&self) -> ProgramExpr {
        ProgramExpr::new(String::new(), Expr::Num(0.0))
    }

    /// An expression that always holds, written with no text.
    ///
    /// The original writes the probability of an unconditional transfer with
    /// an empty field (`TRANSFER  ,A_Enter`).
    fn unconditional_expr(&self) -> ProgramExpr {
        ProgramExpr::new(String::new(), Expr::Num(1.0))
    }

    fn number(&self, module: &Block, name: &str) -> Option<f64> {
        let text = module.prop_value(name)?;
        let resolved = self
            .resolve_placeholders(module, &text, Fallback::Zero)
            .ok()?;
        resolved.trim().parse::<f64>().ok()
    }

    fn require_number(&self, module: &Block, name: &str) -> Result<f64, EngineError> {
        self.number(module, name).ok_or_else(|| {
            EngineError::invalid_block(
                module.id,
                &module.class,
                format!("{name} must be a number"),
            )
        })
    }

    fn name(&self, module: &Block, prop: &str) -> Option<String> {
        let text = module.prop_value(prop)?;
        let resolved = self
            .resolve_placeholders(module, &text, Fallback::Literal)
            .ok()?;
        let trimmed = resolved.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    }

    fn require_name(&self, module: &Block, prop: &str) -> Result<String, EngineError> {
        self.name(module, prop).ok_or_else(|| {
            EngineError::invalid_block(
                module.id,
                &module.class,
                format!("{prop} is required"),
            )
        })
    }

    fn entity(&self, module: &Block, prop: &str) -> Result<EntityName, EngineError> {
        let text = module.prop_value(prop).ok_or_else(|| {
            EngineError::invalid_block(module.id, &module.class, format!("{prop} is required"))
        })?;
        self.entity_name(module, &text)
    }

    /// Like [`Self::entity`], but an empty or missing property means "no entity".
    fn optional_entity(
        &self,
        module: &Block,
        prop: &str,
    ) -> Result<Option<EntityName>, EngineError> {
        let Some(text) = module.prop_value(prop) else {
            return Ok(None);
        };
        let entity = self.entity_name(module, &text)?;
        let empty = match &entity.source {
            NameSource::Literal(name) => name.is_empty(),
            NameSource::Indirect(_) => false,
        };
        Ok((!empty).then_some(entity))
    }

    /// Build an entity operand, honouring the GPSS `*parameter` indirect form.
    fn entity_name(&self, module: &Block, text: &str) -> Result<EntityName, EngineError> {
        let resolved = self.resolve_placeholders(module, text, Fallback::Literal)?;
        let trimmed = resolved.trim();
        let source = match trimmed.strip_prefix('*') {
            Some(indirect) => NameSource::Indirect(indirect.trim().to_string()),
            None => NameSource::Literal(trimmed.to_string()),
        };
        Ok(EntityName { source })
    }

    fn sna(&self, module: &Block, prop: &str) -> Result<Option<ProgramExpr>, EngineError> {
        let Some(text) = module.prop_value(prop) else {
            return Ok(None);
        };
        if text.trim().is_empty() {
            return Ok(None);
        }
        self.resolve_sna_text(module, &text).map(Some)
    }

    fn resolve_sna_text(
        &self,
        module: &Block,
        text: &str,
    ) -> Result<ProgramExpr, EngineError> {
        let resolved = self.resolve_placeholders(module, text, Fallback::Zero)?;
        ProgramExpr::parse(&resolved).map_err(|err| match err {
            EngineError::Expression { message, .. } => EngineError::invalid_block(
                module.id,
                &module.class,
                format!("{message} in {text:?}"),
            ),
            other => other,
        })
    }

    /// Replace `[name]` placeholders with the nearest container's property value.
    fn resolve_placeholders(
        &self,
        module: &Block,
        text: &str,
        fallback: Fallback,
    ) -> Result<String, EngineError> {
        let mut current = text.to_string();
        for _ in 0..8 {
            if !current.contains('[') {
                return Ok(current);
            }
            let mut out = String::with_capacity(current.len());
            let mut rest = current.as_str();
            while let Some(start) = rest.find('[') {
                out.push_str(&rest[..start]);
                let tail = &rest[start + 1..];
                let Some(end) = tail.find(']') else {
                    out.push_str(&rest[start..]);
                    rest = "";
                    break;
                };
                let name = &tail[..end];
                match self.lookup_container_prop(module, name.trim()) {
                    Some(value) => out.push_str(&value),
                    None => match fallback {
                        Fallback::Zero => out.push('0'),
                        Fallback::Literal => out.push_str(name),
                    },
                }
                rest = &tail[end + 1..];
            }
            out.push_str(rest);
            current = out;
        }
        Err(EngineError::invalid_block(
            module.id,
            &module.class,
            format!("placeholder nesting is too deep in {text:?}"),
        ))
    }

    /// Resolve a `[name]` placeholder against the enclosing containers.
    ///
    /// The nearest container that defines a usable value wins. A container
    /// whose value is empty or is the placeholder itself (`media = [media]`,
    /// which the editor writes when a container inherits its parent's setting)
    /// does not shadow the outer scope.
    fn lookup_container_prop(&self, module: &Block, name: &str) -> Option<String> {
        let mut owner = module.owner;
        for _ in 0..64 {
            let container = self.model.block(owner)?;
            if let Some(prop) = container.prop(name)
                && prop.is_used()
            {
                let value = prop.value();
                let trimmed = value.trim();
                if trimmed != format!("[{name}]") {
                    return Some(value);
                }
            }
            if container.is_root() {
                return None;
            }
            owner = container.owner;
        }
        None
    }
}

/// The text of a value the original writes between braces.
fn strip_braces(text: &str) -> &str {
    let trimmed = text.trim();
    trimmed
        .strip_prefix('{')
        .and_then(|rest| rest.strip_suffix('}'))
        .unwrap_or(trimmed)
}

/// Split a list on `;`, leaving the separators inside braces alone.
///
/// The matrix of the original holds a list of lists (`{{11;12};{21;22}}`), and
/// the separators of the inner lists are not the separators of the outer one.
fn split_outside_braces(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    for character in text.chars() {
        match character {
            '{' => {
                depth += 1;
                current.push(character);
            }
            '}' => {
                depth = depth.saturating_sub(1);
                current.push(character);
            }
            ';' if depth == 0 => {
                parts.push(std::mem::take(&mut current));
            }
            _ => current.push(character),
        }
    }
    parts.push(current);
    parts
}

/// The running sum of the weights of a distribution.
///
/// The original writes the table of a `D` function as the running sum of the
/// probabilities and the number each of them selects (`0.2,1/0.5,2/1,3`).
fn cumulative_bounds(weights: &[f64]) -> Vec<f64> {
    let total: f64 = weights.iter().sum();
    let mut cumulative = Vec::with_capacity(weights.len());
    let mut running = 0.0;
    for weight in weights {
        running += weight / total;
        cumulative.push(running);
    }
    if let Some(last) = cumulative.last_mut() {
        *last = 1.0;
    }
    cumulative
}

/// The entity a class of the original's `entitySNAclass` list belongs to.
///
/// The list has 38 classes, and each one names an attribute of an entity: the
/// content of a queue, the units in use of a storage, the busy flag of a
/// facility, the size of a chain... The engine walks the queues, the
/// storages, the facilities and the logic switches; the other families (the
/// chains, the groups, the labels, the parameters, the savevalues, the
/// variables, the functions and the random seeds) are named and written, but
/// the engine does not walk them yet.
fn select_entity(class: &str) -> Option<SelectEntity> {
    Some(match class {
        // The queues.
        "Q" | "QA" | "QC" | "QM" | "QT" | "QX" | "QZ" => SelectEntity::Queue,
        // The storages: `S` is the units in use and `R` the units left.
        "S" | "SA" | "SC" | "SM" | "SR" | "ST" | "SV" => SelectEntity::Storage,
        "R" => SelectEntity::StorageRemaining,
        // The facilities.
        "F" | "FC" | "FT" | "FV" | "FR" => SelectEntity::Facility,
        // The logic switches.
        "LS" | "LR" => SelectEntity::LogicSwitch,
        // The chains, the groups, the labels, the parameters, the savevalues,
        // the variables, the functions and the random seeds.
        "CH" | "CC" | "CA" | "CM" | "CT" | "GN" | "GT" | "W" | "N" | "MP" | "P" | "X" | "BV"
        | "V" | "FN" | "RN" => SelectEntity::Other,
        _ => return None,
    })
}

/// The state a storage is selected by.
///
/// The `op_Logic_Stor` list of the original offers six states, and the GPSS
/// operator of each is the one its code names: `SF` a full storage, `SNF` one
/// that is not full, `SE` an empty one, `SNE` one that is not empty, `SV` one
/// that is available and `SNV` one that is not.
fn storage_state(op: &str) -> SelectState {
    match op.trim() {
        "Lleno" | "Llena" => SelectState::StorageFull,
        "Vacio" | "Vacia" => SelectState::StorageEmpty,
        "NoVacio" | "NoVacia" => SelectState::StorageNotEmpty,
        "Disponible" => SelectState::StorageAvailable,
        "NoDisponible" => SelectState::StorageNotAvailable,
        _ => SelectState::StorageNotFull,
    }
}

/// All expressions a block holds, for reference checking.
fn block_expressions(kind: &BlockKind) -> Vec<&ProgramExpr> {
    match kind {
        BlockKind::Generate(spec) => {
            let mut out = vec![&spec.mean];
            out.extend(spec.modifier.iter());
            out.extend(spec.offset.iter());
            out.extend(spec.priority.iter());
            out
        }
        BlockKind::Advance(spec) => {
            let mut out = vec![&spec.mean];
            out.extend(spec.modifier.iter());
            out
        }
        BlockKind::Assign(spec) => vec![&spec.value],
        BlockKind::SaveValue(spec) => vec![&spec.value],
        BlockKind::Counter(spec) => vec![&spec.increment],
        BlockKind::Enter(spec) | BlockKind::Leave(spec) => vec![&spec.units],
        BlockKind::Test(spec) => vec![&spec.left, &spec.right],
        BlockKind::Transfer(spec) => vec![&spec.probability],
        BlockKind::MultiTransfer(spec) => spec.weights.iter().collect(),
        BlockKind::Unlink(spec) => {
            let mut out = vec![&spec.count];
            out.extend(spec.compare_value.iter());
            out
        }
        BlockKind::Split(spec) => vec![&spec.total],
        BlockKind::Assemble(spec) => vec![&spec.count],
        BlockKind::Select(spec) => vec![&spec.low, &spec.high],
        BlockKind::Tabulate(spec) => spec.weight.iter().collect(),
        BlockKind::Priority(expr) => vec![expr],
        BlockKind::Join(spec) => spec.number.iter().collect(),
        BlockKind::Examine(spec) => vec![&spec.number],
        BlockKind::Remove(spec) => {
            let mut out = vec![&spec.count];
            out.extend(spec.value.iter());
            out
        }
        BlockKind::Scan(spec) => spec.compare_value.iter().collect(),
        BlockKind::Alter(spec) => {
            let mut out = vec![&spec.count];
            out.extend(spec.altered_value.iter());
            out.extend(spec.compared_value.iter());
            out
        }
        _ => Vec::new(),
    }
}

/// Walk an expression tree collecting statically named references.
fn collect_references(expr: &Expr, visit: &mut impl FnMut(RefKind, &str)) {
    match expr {
        Expr::Ref {
            kind,
            name: NameSource::Literal(name),
        } => visit(*kind, name),
        Expr::Ref { .. } => {}
        Expr::Neg(inner) => collect_references(inner, visit),
        Expr::Bin(_, left, right) => {
            collect_references(left, visit);
            collect_references(right, visit);
        }
        _ => {}
    }
}
