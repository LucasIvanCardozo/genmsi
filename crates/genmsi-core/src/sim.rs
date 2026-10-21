//! The discrete-event simulation engine: event chains, transactions, blocks.
//!
//! One run is a deterministic function of `(program, seed)` (decision D5).
//! Events live in a single ordered queue keyed by `(time, priority, sequence)`;
//! a transaction is only ever in one place, tracked by [`XactState`].

use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet, VecDeque};

use serde::Serialize;

use crate::error::EngineError;
use crate::expr::{Ctx, ProgramExpr, RefKind};
use crate::program::{
    BlockKind, EntityName, FunctionDef, GateCondition, GenerateSpec, LogicOp, Program,
    SelectMode, TerminateSpec, UnlinkSpec,
};
use crate::rng::Streams;
use crate::stat::Entities;
use crate::stat::Table;
use crate::xact::{Transaction, XactId, XactState};

/// Capacity used for storages referenced without a `StorageDef`.
const UNDECLARED_STORAGE_CAPACITY: i64 = 1_000_000;

/// How a run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EndReason {
    /// The termination counter reached zero.
    Terminated,
    /// The event chain ran dry while the counter was still positive.
    NoEvents,
    /// The configured simulated-time horizon was reached.
    TimeLimit,
}

impl EndReason {
    /// Stable identifier of the reason.
    ///
    /// Reported by the JSON documents (`terminated`, `no_events`,
    /// `time_limit`) and by the CLI trace summary, so consumers can rely on the
    /// spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Terminated => "terminated",
            Self::NoEvents => "no_events",
            Self::TimeLimit => "time_limit",
        }
    }
}

/// Configuration of one run.
#[derive(Debug, Clone, Copy)]
pub struct SimConfig {
    /// Base seed of every random stream.
    pub seed: u64,
    /// Safety limit: abort instead of looping forever.
    pub max_events: u64,
    /// Whether to record the structured event trace.
    pub trace: bool,
    /// Maximum number of trace records kept.
    pub trace_limit: usize,
    /// Optional simulated-time horizon: stop once the clock passes it.
    pub stop_time: Option<f64>,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            seed: 1,
            max_events: 50_000_000,
            trace: false,
            trace_limit: 100_000,
            stop_time: None,
        }
    }
}

/// Statistics of one executable statement.
#[derive(Debug, Clone, Default, Serialize)]
pub struct BlockStats {
    /// Times a transaction entered the statement.
    pub entries: u64,
    /// Transactions currently in the statement.
    pub current: u64,
    /// Failed attempts (gate retries, refused admissions).
    pub retries: u64,
}

/// One trace record.
#[derive(Debug, Clone, Serialize)]
pub struct TraceEvent {
    /// Simulation time.
    pub time: f64,
    /// Transaction id.
    pub xact: XactId,
    /// Block index in the program.
    pub block: usize,
    /// Block class.
    pub class: String,
    /// Block label.
    pub label: Option<String>,
    /// What happened (`enter`, `leave`, `wait`, `terminate`).
    pub event: &'static str,
    /// Detail (entity name, target block, ...).
    pub detail: String,
}

/// Outcome of one run.
#[derive(Debug, Clone, Serialize)]
pub struct RunResult {
    /// Program title.
    pub title: String,
    /// Seed used.
    pub seed: u64,
    /// Time the run started.
    pub start_time: f64,
    /// Time the run ended.
    pub end_time: f64,
    /// Why it ended.
    pub end_reason: EndReason,
    /// Transactions created.
    pub transactions: u64,
    /// Events processed.
    pub events: u64,
    /// Per-statement statistics.
    pub blocks: Vec<BlockStats>,
    /// Entity statistics and savevalues.
    pub entities: Entities,
    /// Structured event trace, when enabled.
    pub trace: Vec<TraceEvent>,
    /// Compiler diagnostics repeated on the result.
    pub diagnostics: Vec<String>,
    /// Names of the tables defined by the model, in report order.
    pub table_names: Vec<String>,
    /// Names of the queue tables defined by the model, in report order.
    pub qtable_names: Vec<String>,
}

impl RunResult {
    /// Elapsed simulated time.
    pub fn elapsed(&self) -> f64 {
        (self.end_time - self.start_time).max(0.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventKind {
    /// A transaction reaches a block for the first time.
    Enter,
    /// A transaction resumes inside the block it was waiting in.
    Resume,
    /// A generator tick.
    Tick,
}

#[derive(Debug, Clone, Copy)]
struct Event {
    time: f64,
    priority: i64,
    seq: u64,
    block: usize,
    xact: Option<XactId>,
    kind: EventKind,
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Event {}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time
            .total_cmp(&other.time)
            .then_with(|| other.priority.cmp(&self.priority))
            .then_with(|| self.seq.cmp(&other.seq))
    }
}

#[derive(Debug, Clone, Copy)]
struct GeneratorState {
    created: u64,
    pending: bool,
}

/// One simulation run.
pub struct Simulation<'p> {
    program: &'p Program,
    config: SimConfig,
    now: f64,
    heap: BinaryHeap<std::cmp::Reverse<Event>>,
    seq: u64,
    xacts: HashMap<XactId, Transaction>,
    next_xact: XactId,
    entities: Entities,
    block_stats: Vec<BlockStats>,
    generators: Vec<Option<GeneratorState>>,
    terminations_left: f64,
    ended: bool,
    end_reason: Option<EndReason>,
    events: u64,
    streams: Streams,
    trace: Vec<TraceEvent>,
    label_index: HashMap<String, usize>,
    passed: HashMap<String, u64>,
    transaction_groups: BTreeMap<String, Vec<XactId>>,
    numeric_groups: BTreeMap<String, Vec<f64>>,
    assemble_waiters: HashMap<usize, Vec<(u64, XactId)>>,
    match_waiters: HashMap<usize, VecDeque<XactId>>,
    gate_waiters: HashMap<usize, Vec<XactId>>,
    cancelled_resumes: HashSet<(XactId, usize)>,
    diagnostics: Vec<String>,
}

impl<'p> Simulation<'p> {
    /// Prepare a run of `program` with `config`.
    pub fn new(program: &'p Program, config: SimConfig) -> Self {
        let mut sim = Self {
            program,
            config,
            now: 0.0,
            heap: BinaryHeap::new(),
            seq: 0,
            xacts: HashMap::new(),
            next_xact: 0,
            entities: Entities::default(),
            block_stats: vec![BlockStats::default(); program.blocks.len()],
            generators: vec![None; program.blocks.len()],
            terminations_left: program.start_counter,
            ended: false,
            end_reason: None,
            events: 0,
            streams: Streams::new(config.seed),
            trace: Vec::new(),
            label_index: program.label_index(),
            passed: HashMap::new(),
            transaction_groups: BTreeMap::new(),
            numeric_groups: BTreeMap::new(),
            assemble_waiters: HashMap::new(),
            match_waiters: HashMap::new(),
            gate_waiters: HashMap::new(),
            cancelled_resumes: HashSet::new(),
            diagnostics: Vec::new(),
        };
        sim.install_definitions();
        sim
    }

    fn install_definitions(&mut self) {
        let program = self.program;
        for (name, capacity) in &program.definitions.storages {
            self.entities.storage(name, *capacity, 0.0);
        }
        for (name, value) in &program.definitions.initial_savevalues {
            self.entities.savevalues.insert(name.clone(), *value);
        }
        for (name, state) in &program.definitions.initial_logic {
            self.entities.logic.insert(name.clone(), *state);
        }
        for (name, def) in &program.definitions.tables {
            self.entities.tables.insert(
                name.clone(),
                Table::new(name.clone(), def.first, def.width, def.intervals),
            );
        }
        for (name, def) in &program.definitions.qtables {
            self.entities.qtables.insert(
                name.clone(),
                Table::new(name.clone(), def.first, def.width, def.intervals),
            );
        }
        // Schedule every generator's first arrival: the offset when given,
        // otherwise the mean interarrival (GPSS places the first transaction at
        // time A, not at time zero).
        let generators: Vec<(usize, GenerateSpec)> = program
            .blocks
            .iter()
            .enumerate()
            .filter_map(|(index, block)| match &block.kind {
                BlockKind::Generate(spec) => Some((index, spec.clone())),
                _ => None,
            })
            .collect();
        for (index, spec) in generators {
            let first = {
                let mut ctx = CtxView {
                    sim: self,
                    xact: 0,
                    depth: 0,
                };
                match &spec.offset {
                    Some(offset) => offset.eval(&mut ctx),
                    None => spec.mean.eval(&mut ctx),
                }
            };
            self.generators[index] = Some(GeneratorState {
                created: 0,
                pending: true,
            });
            self.block_stats[index].current = 1;
            self.push_event(first.max(0.0), 0, index, None, EventKind::Tick);
        }
    }

    fn push_event(
        &mut self,
        time: f64,
        priority: i64,
        block: usize,
        xact: Option<XactId>,
        kind: EventKind,
    ) {
        self.seq += 1;
        let seq = self.seq;
        self.heap.push(std::cmp::Reverse(Event {
            time,
            priority,
            seq,
            block,
            xact,
            kind,
        }));
    }

    /// Run until the termination counter reaches zero (or the model deadlocks).
    pub fn run(mut self) -> Result<RunResult, EngineError> {
        while !self.ended {
            let Some(std::cmp::Reverse(event)) = self.heap.pop() else {
                self.finish(EndReason::NoEvents);
                break;
            };
            self.events += 1;
            if self.events > self.config.max_events {
                let mut hottest: Vec<(u64, usize)> = self
                    .block_stats
                    .iter()
                    .enumerate()
                    .map(|(index, stats)| (stats.entries, index))
                    .collect();
                hottest.sort_unstable_by(|left, right| right.cmp(left));
                let detail: Vec<String> = hottest
                    .iter()
                    .take(5)
                    .map(|(entries, index)| {
                        let block = &self.program.blocks[*index];
                        format!(
                            "{}#{} {} ({} entries)",
                            block.label.clone().unwrap_or_default(),
                            block.module_id,
                            block.class,
                            entries
                        )
                    })
                    .collect();
                return Err(EngineError::Runtime(format!(
                    "event limit ({}) exceeded at time {}; the model has a zero-delay loop; hottest blocks: {}",
                    self.config.max_events,
                    self.now,
                    detail.join(", ")
                )));
            }
            if event.time > self.now {
                self.now = event.time;
            }
            if let Some(limit) = self.config.stop_time
                && self.now > limit
            {
                self.finish(EndReason::TimeLimit);
                break;
            }
            match event.kind {
                EventKind::Tick => self.tick(event.block)?,
                EventKind::Enter => {
                    if let Some(xact) = event.xact {
                        self.execute(xact, event.block, false)?;
                    }
                }
                EventKind::Resume => {
                    if let Some(xact) = event.xact {
                        if self.cancelled_resumes.remove(&(xact, event.block)) {
                            continue;
                        }
                        self.execute(xact, event.block, true)?;
                    }
                }
            }
        }
        let end_time = self.now;
        self.entities.finish(end_time);
        let diagnostics: Vec<String> = self
            .program
            .diagnostics
            .iter()
            .chain(self.diagnostics.iter())
            .cloned()
            .collect();
        let mut table_names: Vec<String> =
            self.program.definitions.tables.keys().cloned().collect();
        table_names.sort();
        let mut qtable_names: Vec<String> =
            self.program.definitions.qtables.keys().cloned().collect();
        qtable_names.sort();
        Ok(RunResult {
            title: self.program.title.clone(),
            seed: self.config.seed,
            start_time: 0.0,
            end_time,
            end_reason: self.end_reason.unwrap_or(EndReason::NoEvents),
            transactions: self.next_xact,
            events: self.events,
            blocks: self.block_stats,
            entities: self.entities,
            trace: self.trace,
            diagnostics,
            table_names,
            qtable_names,
        })
    }

    fn finish(&mut self, reason: EndReason) {
        if !self.ended {
            self.ended = true;
            self.end_reason = Some(reason);
        }
    }

    /// Handle a generator tick.
    fn tick(&mut self, block_index: usize) -> Result<(), EngineError> {
        let block = self.program.blocks[block_index].clone();
        let BlockKind::Generate(spec) = &block.kind else {
            return Ok(());
        };
        let spec = spec.clone();
        let Some(state) = self.generators[block_index] else {
            return Ok(());
        };
        if !state.pending {
            return Ok(());
        }
        if spec.limit.is_some_and(|limit| state.created as f64 >= limit) {
            self.generators[block_index] = Some(GeneratorState {
                pending: false,
                ..state
            });
            self.block_stats[block_index].current = 0;
            return Ok(());
        }

        self.block_stats[block_index].entries += 1;
        let priority = {
            let mut ctx = CtxView {
                sim: self,
                xact: 0,
                depth: 0,
            };
            spec.priority
                .as_ref()
                .map(|expr| expr.eval(&mut ctx) as i64)
                .unwrap_or(0)
        };
        let xact = self.new_transaction(priority, self.now);
        if self.config.trace {
            self.record_trace(xact, block_index, "generate", String::new());
        }
        match block.next_default() {
            Some(target) => self.enter_block(xact, target),
            None => self.remove_transaction(xact),
        }

        let created = state.created + 1;
        if spec.limit.is_some_and(|limit| created as f64 >= limit) {
            self.generators[block_index] = Some(GeneratorState {
                created,
                pending: false,
            });
            self.block_stats[block_index].current = 0;
            return Ok(());
        }
        let (mean, modifier) = {
            let mut ctx = CtxView {
                sim: self,
                xact,
                depth: 0,
            };
            evaluate_delay(&spec.mean, spec.modifier.as_ref(), &mut ctx)
        };
        let delay = apply_modifier(mean, modifier, &mut self.streams, block.stream);
        self.generators[block_index] = Some(GeneratorState {
            created,
            pending: true,
        });
        self.push_event(
            self.now + delay.max(0.0),
            0,
            block_index,
            None,
            EventKind::Tick,
        );
        Ok(())
    }

    fn new_transaction(&mut self, priority: i64, now: f64) -> XactId {
        self.next_xact += 1;
        let id = self.next_xact;
        self.xacts.insert(id, Transaction::new(id, priority, now));
        id
    }

    fn remove_transaction(&mut self, id: XactId) {
        if let Some(transaction) = self.xacts.get_mut(&id) {
            transaction.transition(XactState::Terminated);
        }
        self.xacts.remove(&id);
    }

    fn enter_block(&mut self, xact: XactId, block: usize) {
        if let Some(transaction) = self.xacts.get_mut(&xact) {
            transaction.current_block = Some(block);
            transaction.transition(XactState::Ready);
        }
        self.push_event(self.now, self.priority_of(xact), block, Some(xact), EventKind::Enter);
    }

    fn resume_block(&mut self, xact: XactId, block: usize) {
        if let Some(transaction) = self.xacts.get_mut(&xact) {
            transaction.current_block = Some(block);
            transaction.transition(XactState::Ready);
        }
        self.push_event(self.now, self.priority_of(xact), block, Some(xact), EventKind::Resume);
    }

    fn priority_of(&self, xact: XactId) -> i64 {
        self.xacts.get(&xact).map(|xact| xact.priority).unwrap_or(0)
    }

    fn record_trace(
        &mut self,
        xact: XactId,
        block: usize,
        event: &'static str,
        detail: String,
    ) {
        if !self.config.trace || self.trace.len() >= self.config.trace_limit {
            return;
        }
        let reference = &self.program.blocks[block];
        self.trace.push(TraceEvent {
            time: self.now,
            xact,
            block,
            class: reference.class.clone(),
            label: reference.label.clone(),
            event,
            detail,
        });
    }

    /// Execute (or resume) one block for one transaction.
    fn execute(
        &mut self,
        xact: XactId,
        block_index: usize,
        resume: bool,
    ) -> Result<(), EngineError> {
        if !self.xacts.contains_key(&xact) {
            return Ok(());
        }
        let block = self.program.blocks[block_index].clone();
        if !resume {
            if let Some(transaction) = self.xacts.get_mut(&xact) {
                transaction.current_block = Some(block_index);
                transaction.entered_at = self.now;
            }
            self.block_stats[block_index].entries += 1;
            self.block_stats[block_index].current += 1;
            if let Some(label) = &block.label {
                *self.passed.entry(label.clone()).or_insert(0) += 1;
            }
            if self.config.trace {
                self.record_trace(xact, block_index, "enter", String::new());
            }
        }

        match block.kind.clone() {
            BlockKind::NoOp | BlockKind::Buffer | BlockKind::InitialLogic(_) => {
                self.leave(xact, block_index, block.next_default());
            }
            BlockKind::Generate(_) => {
                self.leave(xact, block_index, block.next_default());
            }
            BlockKind::Advance(spec) => {
                if resume {
                    if let Some(transaction) = self.xacts.get_mut(&xact) {
                        transaction.scheduled_at = None;
                    }
                    self.leave(xact, block_index, block.next_default());
                } else {
                    let (mean, modifier) = {
                        let mut ctx = CtxView {
                            sim: self,
                            xact,
                            depth: 0,
                        };
                        evaluate_delay(&spec.mean, spec.modifier.as_ref(), &mut ctx)
                    };
                    let delay = apply_modifier(mean, modifier, &mut self.streams, block.stream);
                    if delay <= 0.0 {
                        self.leave(xact, block_index, block.next_default());
                    } else {
                        if let Some(transaction) = self.xacts.get_mut(&xact) {
                            transaction.transition(XactState::Scheduled);
                            transaction.scheduled_at = Some(self.now + delay);
                        }
                        self.push_event(
                            self.now + delay,
                            self.priority_of(xact),
                            block_index,
                            Some(xact),
                            EventKind::Resume,
                        );
                    }
                }
            }
            BlockKind::Assign(spec) => {
                let value = {
                    let mut ctx = CtxView {
                        sim: self,
                        xact,
                        depth: 0,
                    };
                    spec.value.eval(&mut ctx)
                };
                if let Some(transaction) = self.xacts.get_mut(&xact) {
                    let current = transaction.parameter(&spec.parameter);
                    transaction.set_parameter(&spec.parameter, spec.op.apply(current, value));
                }
                self.leave(xact, block_index, block.next_default());
            }
            BlockKind::SaveValue(spec) => {
                let value = {
                    let mut ctx = CtxView {
                        sim: self,
                        xact,
                        depth: 0,
                    };
                    spec.value.eval(&mut ctx)
                };
                let cell = self.entities.savevalue(&spec.savevalue);
                *cell = spec.op.apply(*cell, value);
                self.leave(xact, block_index, block.next_default());
            }
            BlockKind::Counter(spec) => {
                let increment = {
                    let mut ctx = CtxView {
                        sim: self,
                        xact,
                        depth: 0,
                    };
                    spec.increment.eval(&mut ctx)
                };
                let cell = self.entities.savevalue(&spec.savevalue);
                *cell += increment;
                let new_value = *cell;
                if let Some(parameter) = &spec.parameter
                    && let Some(transaction) = self.xacts.get_mut(&xact)
                {
                    transaction.set_parameter(parameter, new_value);
                }
                self.leave(xact, block_index, block.next_default());
            }
            BlockKind::Queue(spec) => {
                let name = self.resolve_name(&spec.queue, xact);
                if spec.units > 0.0 {
                    self.entities.queue(&name, self.now).enqueue(xact, self.now);
                }
                self.leave(xact, block_index, block.next_default());
            }
            BlockKind::Depart(spec) => {
                let name = self.resolve_name(&spec.queue, xact);
                let waited = self.entities.queue(&name, self.now).dequeue(xact, self.now);
                self.record_queue_tables(&name, waited);
                self.leave(xact, block_index, block.next_default());
            }
            BlockKind::Seize(spec) => {
                let name = self.resolve_name(&spec.facility, xact);
                let acquired = {
                    let facility = self.entities.facility(&name, self.now);
                    if facility.available && !facility.busy {
                        facility.start_busy(self.now);
                        facility.owner = Some(xact);
                        facility.entries += 1;
                        true
                    } else {
                        facility.waiters.push(xact);
                        false
                    }
                };
                if acquired {
                    self.leave(xact, block_index, block.next_default());
                } else {
                    let mut waiters = {
                        let facility = self.entities.facility(&name, self.now);
                        std::mem::take(&mut facility.waiters)
                    };
                    waiters.sort_by_key(|waiter| (std::cmp::Reverse(self.priority_of(*waiter)), *waiter));
                    self.entities.facility(&name, self.now).waiters = waiters;
                    if let Some(transaction) = self.xacts.get_mut(&xact) {
                        transaction.transition(XactState::Waiting);
                    }
                    self.block_stats[block_index].retries += 1;
                    if self.config.trace {
                        self.record_trace(xact, block_index, "wait", name);
                    }
                }
            }
            BlockKind::Release(spec) => {
                let name = self.resolve_name(&spec.facility, xact);
                self.release_facility(&name);
                self.leave(xact, block_index, block.next_default());
            }
            BlockKind::Preempt(spec) => {
                let name = self.resolve_name(&spec.facility, xact);
                let previous = {
                    let facility = self.entities.facility(&name, self.now);
                    let previous = facility.owner.filter(|owner| *owner != xact);
                    if previous.is_some() {
                        facility.inter += 1;
                    }
                    previous
                };
                if let Some(previous) = previous {
                    let remaining = self.suspend_owner(previous, spec.remaining.as_deref());
                    if let Some(facility) = self.entities.facilities.get_mut(&name) {
                        facility.preempted = Some(crate::stat::Preempted {
                            xact: previous,
                            remaining,
                        });
                    }
                }
                let facility = self.entities.facility(&name, self.now);
                facility.start_busy(self.now);
                facility.owner = Some(xact);
                facility.entries += 1;
                self.leave(xact, block_index, block.next_default());
            }
            BlockKind::Return(spec) => {
                let name = self.resolve_name(&spec.facility, xact);
                let preempted = {
                    let facility = self.entities.facility(&name, self.now);
                    facility.stop_busy(self.now);
                    facility.preempted.take()
                };
                if let Some(previous) = preempted {
                    self.resume_preempted(previous.xact, previous.remaining);
                }
                self.release_waiters(&name);
                self.retry_gates();
                self.leave(xact, block_index, block.next_default());
            }
            BlockKind::Enter(spec) => {
                let name = self.resolve_name(&spec.storage, xact);
                let units = self.eval_units(&spec.units, xact);
                let admitted = self
                    .entities
                    .storage(&name, UNDECLARED_STORAGE_CAPACITY, self.now)
                    .try_enter(units, self.now);
                if admitted {
                    self.release_storage_waiters(&name);
                    self.retry_gates();
                    self.leave(xact, block_index, block.next_default());
                } else {
                    let mut waiters = {
                        let storage = self.entities.storage(
                            &name,
                            UNDECLARED_STORAGE_CAPACITY,
                            self.now,
                        );
                        storage.waiters.push(xact);
                        std::mem::take(&mut storage.waiters)
                    };
                    waiters.sort_by_key(|waiter| (std::cmp::Reverse(self.priority_of(*waiter)), *waiter));
                    self.entities
                        .storage(&name, UNDECLARED_STORAGE_CAPACITY, self.now)
                        .waiters = waiters;
                    if let Some(transaction) = self.xacts.get_mut(&xact) {
                        transaction.transition(XactState::Waiting);
                    }
                    self.block_stats[block_index].retries += 1;
                    if self.config.trace {
                        self.record_trace(xact, block_index, "wait", name);
                    }
                }
            }
            BlockKind::Leave(spec) => {
                let name = self.resolve_name(&spec.storage, xact);
                let units = self.eval_units(&spec.units, xact);
                self.entities
                    .storage(&name, UNDECLARED_STORAGE_CAPACITY, self.now)
                    .leave(units, self.now);
                self.release_storage_waiters(&name);
                self.retry_gates();
                self.leave(xact, block_index, block.next_default());
            }
            BlockKind::Test(spec) => {
                let (left, right) = {
                    let mut ctx = CtxView {
                        sim: self,
                        xact,
                        depth: 0,
                    };
                    (spec.left.eval(&mut ctx), spec.right.eval(&mut ctx))
                };
                let target = if spec.op.test(left, right) {
                    block.next_named("Salida1")
                } else {
                    block.next_named("Salida2")
                };
                self.leave(xact, block_index, target);
            }
            BlockKind::Transfer(spec) => {
                let mut probability = {
                    let mut ctx = CtxView {
                        sim: self,
                        xact,
                        depth: 0,
                    };
                    spec.probability.eval(&mut ctx)
                };
                if spec.thousandths {
                    probability /= 1000.0;
                }
                let probability = probability.clamp(0.0, 1.0);
                let draw = self.streams.uniform(block.stream);
                let target = if draw < probability {
                    block.next_named("Salida1")
                } else {
                    block.next_named("Salida2")
                };
                self.leave(xact, block_index, target);
            }
            BlockKind::MultiTransfer(spec) => {
                let weights: Vec<f64> = {
                    let mut ctx = CtxView {
                        sim: self,
                        xact,
                        depth: 0,
                    };
                    spec.weights.iter().map(|weight| weight.eval(&mut ctx)).collect()
                };
                let total: f64 = weights.iter().sum();
                let draw = self.streams.uniform(block.stream) * total;
                let mut acc = 0.0;
                let mut chosen = block.next.len().saturating_sub(1);
                for (index, weight) in weights.iter().enumerate() {
                    acc += weight;
                    if draw < acc {
                        chosen = index;
                        break;
                    }
                }
                let target = block.next_port(chosen).or_else(|| block.next_default());
                self.leave(xact, block_index, target);
            }
            BlockKind::Gate(spec) => {
                let pass = self.gate_condition(&spec.condition, xact);
                if pass {
                    self.leave(xact, block_index, block.next_named("Salida1"));
                } else if block.has_failure_port() {
                    self.leave(xact, block_index, block.next_named("Salida2"));
                } else {
                    self.block_stats[block_index].retries += 1;
                    self.gate_waiters.entry(block_index).or_default().push(xact);
                    if let Some(transaction) = self.xacts.get_mut(&xact) {
                        transaction.transition(XactState::Waiting);
                    }
                    if self.config.trace {
                        self.record_trace(xact, block_index, "wait", "gate".to_string());
                    }
                }
            }
            BlockKind::Logic(spec) => {
                let name = self.resolve_name(&spec.switch, xact);
                *self.entities.logic(&name) = spec.op == LogicOp::Set;
                self.retry_gates();
                self.leave(xact, block_index, block.next_default());
            }
            BlockKind::Link(spec) => {
                let name = self.resolve_name(&spec.chain, xact);
                {
                    let chain = self.entities.chain(&name, self.now);
                    chain.area += chain.content as f64 * (self.now - chain.last_change).max(0.0);
                    chain.last_change = self.now;
                    if spec.fifo {
                        chain.members.push(xact);
                    } else {
                        chain.members.insert(0, xact);
                    }
                    chain.content += 1;
                    chain.max = chain.max.max(chain.content);
                    chain.entries += 1;
                    chain.linked_at.insert(xact, self.now);
                }
                if let Some(transaction) = self.xacts.get_mut(&xact) {
                    transaction.chain = Some(name);
                    transaction.transition(XactState::Waiting);
                }
                if self.config.trace {
                    self.record_trace(xact, block_index, "wait", "chain".to_string());
                }
            }
            BlockKind::Unlink(spec) => {
                let name = self.resolve_name(&spec.chain, xact);
                let (count, wanted) = {
                    let mut ctx = CtxView {
                        sim: self,
                        xact,
                        depth: 0,
                    };
                    // `Modo = Todos` (`ALL` in the generated GPSS) releases every
                    // matching member, whatever `Cantidad` says.
                    let count = if spec.all {
                        usize::MAX
                    } else {
                        spec.count.eval(&mut ctx).round().max(1.0) as usize
                    };
                    let wanted = spec.compare_value.as_ref().map(|expr| expr.eval(&mut ctx));
                    (count, wanted)
                };
                let removed = self.unlink(&name, count, &spec, wanted);
                if removed.is_empty() {
                    let target = if block.has_failure_port() {
                        block.next_named("Salida2")
                    } else {
                        block.next_named("Salida1")
                    };
                    self.leave(xact, block_index, target);
                } else {
                    let target = block
                        .next_named("SalidaAlt")
                        .or_else(|| block.next_named("Salida1"));
                    for removed_xact in removed {
                        if let Some(transaction) = self.xacts.get_mut(&removed_xact) {
                            transaction.chain = None;
                            transaction.transition(XactState::Ready);
                        }
                        match target {
                            Some(target) => self.enter_block(removed_xact, target),
                            None => self.remove_transaction(removed_xact),
                        }
                    }
                    self.leave(xact, block_index, block.next_named("Salida1"));
                }
            }
            BlockKind::Split(spec) => {
                let total = {
                    let mut ctx = CtxView {
                        sim: self,
                        xact,
                        depth: 0,
                    };
                    spec.total.eval(&mut ctx).round().max(1.0) as usize
                };
                let key = self
                    .xacts
                    .get(&xact)
                    .and_then(|transaction| transaction.assembly)
                    .unwrap_or(xact);
                if let Some(transaction) = self.xacts.get_mut(&xact) {
                    transaction.assembly = Some(key);
                }
                let copy_target = block
                    .next_named("Salida2")
                    .or_else(|| block.next_default());
                // GPSS SPLIT A creates A copies; the original continues to its own
                // output. Models rely on this: `SPLIT P$n` followed by
                // `ASSEMBLE 1+P$n` (the copies plus the original).
                for _ in 0..total {
                    let priority = self.priority_of(xact);
                    let copy = self.new_transaction(priority, self.now);
                    let (parameters, mark) = self
                        .xacts
                        .get(&xact)
                        .map(|original| (original.parameters.clone(), original.mark))
                        .unwrap_or_default();
                    if let Some(clone) = self.xacts.get_mut(&copy) {
                        clone.parameters = parameters;
                        clone.assembly = Some(key);
                        clone.mark = mark;
                    }
                    match copy_target {
                        Some(target) => self.enter_block(copy, target),
                        None => self.remove_transaction(copy),
                    }
                }
                self.leave(xact, block_index, block.next_named("Salida1"));
            }
            BlockKind::Assemble(spec) => {
                let count = {
                    let mut ctx = CtxView {
                        sim: self,
                        xact,
                        depth: 0,
                    };
                    spec.count.eval(&mut ctx).round().max(1.0) as usize
                };
                let key = self
                    .xacts
                    .get(&xact)
                    .and_then(|transaction| transaction.assembly)
                    .unwrap_or(xact);
                let mut ready = Vec::new();
                {
                    let waiters = self.assemble_waiters.entry(block_index).or_default();
                    waiters.push((key, xact));
                    let matching = waiters.iter().filter(|(candidate, _)| *candidate == key).count();
                    if matching >= count {
                        let mut remaining = Vec::new();
                        let mut gathered = Vec::new();
                        for (candidate, id) in std::mem::take(waiters) {
                            if candidate == key && gathered.len() < count {
                                gathered.push(id);
                            } else {
                                remaining.push((candidate, id));
                            }
                        }
                        waiters.extend(remaining);
                        ready = gathered;
                    }
                }
                if ready.is_empty() {
                    if let Some(transaction) = self.xacts.get_mut(&xact) {
                        transaction.transition(XactState::Waiting);
                    }
                } else {
                    let target = block.next_default();
                    for (index, id) in ready.into_iter().enumerate() {
                        if spec.gather || index == 0 {
                            if let Some(transaction) = self.xacts.get_mut(&id) {
                                transaction.transition(XactState::Ready);
                            }
                            match target {
                                Some(target) => self.enter_block(id, target),
                                None => self.remove_transaction(id),
                            }
                        } else {
                            self.remove_transaction(id);
                        }
                    }
                }
            }
            BlockKind::Match(spec) => {
                let Some(conjugate) = self.label_index.get(&spec.conjugate).copied() else {
                    self.leave(xact, block_index, block.next_default());
                    return Ok(());
                };
                // Pair with the oldest transaction waiting at the conjugate
                // block. Every waiter is kept: a block can hold any number of
                // them, and pairing them first-in first-out is what keeps the
                // two flows in step.
                let partner = self
                    .match_waiters
                    .get_mut(&conjugate)
                    .and_then(VecDeque::pop_front)
                    .filter(|partner| self.xacts.contains_key(partner));
                match partner {
                    Some(partner) => {
                        if let Some(transaction) = self.xacts.get_mut(&partner) {
                            transaction.transition(XactState::Ready);
                        }
                        // `leave` decrements the conjugate block's CURRENT count:
                        // the partner is leaving that block, not this one.
                        let partner_target = self.program.blocks[conjugate].next_default();
                        self.leave(partner, conjugate, partner_target);
                        self.leave(xact, block_index, block.next_default());
                    }
                    None => {
                        self.match_waiters
                            .entry(block_index)
                            .or_default()
                            .push_back(xact);
                        if let Some(transaction) = self.xacts.get_mut(&xact) {
                            transaction.transition(XactState::Waiting);
                        }
                    }
                }
            }
            BlockKind::Loop(spec) => {
                let remaining = if let Some(transaction) = self.xacts.get_mut(&xact) {
                    let value = transaction.parameter(&spec.parameter) - 1.0;
                    transaction.set_parameter(&spec.parameter, value);
                    value
                } else {
                    0.0
                };
                let target = if remaining > 0.0 {
                    block.next_named("Salida2")
                } else {
                    block.next_named("Salida1")
                };
                self.leave(xact, block_index, target);
            }
            BlockKind::Select(spec) => {
                let (low, high) = {
                    let mut ctx = CtxView {
                        sim: self,
                        xact,
                        depth: 0,
                    };
                    (
                        spec.low.eval(&mut ctx).round() as i64,
                        spec.high.eval(&mut ctx).round() as i64,
                    )
                };
                let chosen = self.select(&spec.kind, low.min(high), low.max(high), spec.mode);
                match chosen {
                    Some(number) => {
                        if let Some(transaction) = self.xacts.get_mut(&xact) {
                            transaction.set_parameter(&spec.parameter, number as f64);
                        }
                        self.leave(xact, block_index, block.next_named("Salida1"));
                    }
                    None => {
                        let target = if block.has_failure_port() {
                            block.next_named("Salida2")
                        } else {
                            block.next_named("Salida1")
                        };
                        self.leave(xact, block_index, target);
                    }
                }
            }
            BlockKind::Tabulate(spec) => {
                if let Some(definition) = self.program.definitions.tables.get(&spec.table).cloned() {
                    let value = {
                        let mut ctx = CtxView {
                            sim: self,
                            xact,
                            depth: 0,
                        };
                        definition.attribute.eval(&mut ctx)
                    };
                    if let Some(table) = self.entities.tables.get_mut(&spec.table) {
                        table.tabulate(value);
                    }
                } else {
                    self.diagnostics
                        .push(format!("TABULATE {} has no TABLE definition", spec.table));
                }
                self.leave(xact, block_index, block.next_default());
            }
            BlockKind::Priority(expr) => {
                let priority = {
                    let mut ctx = CtxView {
                        sim: self,
                        xact,
                        depth: 0,
                    };
                    expr.eval(&mut ctx).round() as i64
                };
                if let Some(transaction) = self.xacts.get_mut(&xact) {
                    transaction.priority = priority;
                }
                self.leave(xact, block_index, block.next_default());
            }
            BlockKind::Join(spec) => {
                match &spec.number {
                    Some(number) => {
                        let value = {
                            let mut ctx = CtxView {
                                sim: self,
                                xact,
                                depth: 0,
                            };
                            number.eval(&mut ctx)
                        };
                        self.numeric_groups
                            .entry(spec.group.clone())
                            .or_default()
                            .push(value);
                    }
                    None => {
                        self.transaction_groups
                            .entry(spec.group.clone())
                            .or_default()
                            .push(xact);
                    }
                }
                self.leave(xact, block_index, block.next_default());
            }
            BlockKind::Examine(spec) => {
                let number = {
                    let mut ctx = CtxView {
                        sim: self,
                        xact,
                        depth: 0,
                    };
                    spec.number.eval(&mut ctx)
                };
                let present = self
                    .numeric_groups
                    .get(&spec.group)
                    .is_some_and(|members| {
                        members
                            .iter()
                            .any(|value| (value - number).abs() < 1e-9)
                    });
                let target = if present {
                    block.next_named("Salida1")
                } else {
                    block.next_named("Salida2")
                };
                self.leave(xact, block_index, target);
            }
            BlockKind::Remove(spec) => {
                let (count, wanted) = {
                    let mut ctx = CtxView {
                        sim: self,
                        xact,
                        depth: 0,
                    };
                    let count = if spec.all {
                        usize::MAX
                    } else {
                        spec.count.eval(&mut ctx).round().max(1.0) as usize
                    };
                    let wanted = spec.value.as_ref().map(|expr| expr.eval(&mut ctx));
                    (count, wanted)
                };
                let removed = self.remove_from_group(
                    &spec.group,
                    count,
                    spec.parameter.as_deref(),
                    wanted,
                );
                // The removed transactions are *detached* from the group, not
                // resumed: they keep waiting where they are (normally inside a
                // LINK block) until an UNLINK releases them. Resuming them here
                // would run them through this block's own output.
                if removed.is_empty() && block.has_failure_port() {
                    let target = block.next_named("Salida2");
                    self.leave(xact, block_index, target);
                } else {
                    let target = block.next_named("Salida1");
                    self.leave(xact, block_index, target);
                }
            }
            BlockKind::Scan(spec) => {
                let compare = {
                    let mut ctx = CtxView {
                        sim: self,
                        xact,
                        depth: 0,
                    };
                    spec.compare_value.as_ref().map(|expr| expr.eval(&mut ctx))
                };
                let found = self.scan_group(&spec, compare);
                match found {
                    Some((from_value, to_parameter)) => {
                        if let Some(parameter) = to_parameter
                            && let Some(transaction) = self.xacts.get_mut(&xact)
                        {
                            transaction.set_parameter(&parameter, from_value);
                        }
                        self.leave(xact, block_index, block.next_named("Salida1"));
                    }
                    None => {
                        let target = block.next_named("Salida2");
                        self.leave(xact, block_index, target);
                    }
                }
            }
            BlockKind::Alter(spec) => {
                let (count, value) = {
                    let mut ctx = CtxView {
                        sim: self,
                        xact,
                        depth: 0,
                    };
                    let count = spec.count.eval(&mut ctx).round().max(1.0) as usize;
                    let value = spec.value.as_ref().map(|expr| expr.eval(&mut ctx));
                    (count, value)
                };
                let changed = self.alter_group(&spec.group, count, spec.parameter.as_deref(), value);
                let target = if changed {
                    block.next_named("Salida1")
                } else {
                    block.next_named("Salida2")
                };
                self.leave(xact, block_index, target);
            }
            BlockKind::Terminate(spec) => self.terminate(xact, block_index, spec),
        }
        Ok(())
    }

    fn eval_units(&mut self, expr: &ProgramExpr, xact: XactId) -> i64 {
        let mut ctx = CtxView {
            sim: self,
            xact,
            depth: 0,
        };
        expr.eval(&mut ctx).round().max(1.0) as i64
    }

    fn leave(&mut self, xact: XactId, from: usize, target: Option<usize>) {
        if self.block_stats[from].current > 0 {
            self.block_stats[from].current -= 1;
        }
        match target {
            Some(target) => {
                if self.config.trace {
                    self.record_trace(xact, from, "leave", format!("-> {target}"));
                }
                self.enter_block(xact, target);
            }
            None => {
                if self.config.trace {
                    self.record_trace(xact, from, "terminate", "no output".to_string());
                }
                self.remove_transaction(xact);
            }
        }
    }

    fn terminate(&mut self, xact: XactId, block_index: usize, spec: TerminateSpec) {
        if self.block_stats[block_index].current > 0 {
            self.block_stats[block_index].current -= 1;
        }
        if spec.units > 0.0 {
            self.terminations_left -= spec.units;
            if self.terminations_left <= 0.0 {
                self.finish(EndReason::Terminated);
            }
        }
        if self.config.trace {
            self.record_trace(xact, block_index, "terminate", String::new());
        }
        self.remove_transaction(xact);
    }

    fn resolve_name(&mut self, entity: &EntityName, xact: XactId) -> String {
        let mut ctx = CtxView {
            sim: self,
            xact,
            depth: 0,
        };
        entity.resolve(&mut ctx)
    }

    fn gate_condition(&mut self, condition: &GateCondition, xact: XactId) -> bool {
        match condition {
            GateCondition::Logic { switch, state } => {
                let name = self.resolve_name(switch, xact);
                *self.entities.logic(&name) == *state
            }
            GateCondition::Storage { storage, empty } => {
                let name = self.resolve_name(storage, xact);
                self.entities
                    .storage(&name, UNDECLARED_STORAGE_CAPACITY, self.now)
                    .is_empty()
                    == *empty
            }
            GateCondition::Facility {
                facility,
                available,
            } => {
                let name = self.resolve_name(facility, xact);
                let facility = self.entities.facility(&name, self.now);
                (facility.available && !facility.busy) == *available
            }
        }
    }

    /// Retry every transaction blocked in a GATE block, in priority order.
    fn retry_gates(&mut self) {
        let waiters: Vec<(usize, XactId)> = self
            .gate_waiters
            .iter()
            .flat_map(|(block, ids)| ids.iter().map(move |id| (*block, *id)))
            .collect();
        self.gate_waiters.clear();
        for (block, xact) in waiters {
            if self.xacts.contains_key(&xact) {
                self.resume_block(xact, block);
            }
        }
    }

    fn release_facility(&mut self, name: &str) {
        {
            let facility = self.entities.facility(name, self.now);
            facility.stop_busy(self.now);
        }
        self.release_waiters(name);
        self.retry_gates();
    }

    fn release_waiters(&mut self, name: &str) {
        let waiters = {
            let facility = self.entities.facility(name, self.now);
            std::mem::take(&mut facility.waiters)
        };
        if waiters.is_empty() {
            return;
        }
        let mut sorted: Vec<XactId> = waiters
            .into_iter()
            .filter(|candidate| self.xacts.contains_key(candidate))
            .collect();
        sorted.sort_by_key(|xact| (std::cmp::Reverse(self.priority_of(*xact)), *xact));
        // Retry only the head of the pending chain; the rest stay queued for the
        // next release. Resuming one waiter and dropping the others would strand
        // every transaction behind it in the SEIZE block forever.
        let resumed = if sorted.is_empty() {
            None
        } else {
            Some(sorted.remove(0))
        };
        if !sorted.is_empty() {
            let facility = self.entities.facility(name, self.now);
            sorted.append(&mut facility.waiters);
            facility.waiters = sorted;
        }
        if let Some(xact) = resumed
            && let Some(block) = self
                .xacts
                .get(&xact)
                .and_then(|transaction| transaction.current_block)
        {
            self.resume_block(xact, block);
        }
    }

    fn release_storage_waiters(&mut self, name: &str) {
        let waiters = {
            let storage = self
                .entities
                .storage(name, UNDECLARED_STORAGE_CAPACITY, self.now);
            std::mem::take(&mut storage.waiters)
        };
        for xact in waiters {
            let Some(block) = self
                .xacts
                .get(&xact)
                .and_then(|transaction| transaction.current_block)
            else {
                continue;
            };
            self.resume_block(xact, block);
        }
    }

    fn suspend_owner(&mut self, owner: XactId, remaining_parameter: Option<&str>) -> f64 {
        let current = self
            .xacts
            .get(&owner)
            .and_then(|transaction| transaction.current_block);
        let mut remaining = 0.0;
        if let Some(block) = current
            && matches!(self.program.blocks[block].kind, BlockKind::Advance(_))
        {
            // Cancel the pending resume instead of draining the event queue; the
            // scheduled time is kept on the transaction.
            if let Some(scheduled) = self
                .xacts
                .get(&owner)
                .and_then(|transaction| transaction.scheduled_at)
            {
                remaining = (scheduled - self.now).max(0.0);
                self.cancelled_resumes.insert((owner, block));
                if self.block_stats[block].current > 0 {
                    self.block_stats[block].current -= 1;
                }
            }
        }
        if let Some(transaction) = self.xacts.get_mut(&owner) {
            transaction.scheduled_at = None;
        }
        if let Some(parameter) = remaining_parameter
            && let Some(transaction) = self.xacts.get_mut(&owner)
        {
            transaction.set_parameter(parameter, remaining);
        }
        if let Some(transaction) = self.xacts.get_mut(&owner) {
            transaction.transition(XactState::Waiting);
        }
        remaining
    }

    fn resume_preempted(&mut self, owner: XactId, remaining: f64) {
        let Some(block) = self
            .xacts
            .get(&owner)
            .and_then(|transaction| transaction.current_block)
        else {
            return;
        };
        if remaining > 0.0 && matches!(self.program.blocks[block].kind, BlockKind::Advance(_)) {
            self.block_stats[block].current += 1;
            if let Some(transaction) = self.xacts.get_mut(&owner) {
                transaction.transition(XactState::Scheduled);
                transaction.scheduled_at = Some(self.now + remaining);
            }
            self.push_event(
                self.now + remaining,
                self.priority_of(owner),
                block,
                Some(owner),
                EventKind::Resume,
            );
        } else {
            self.resume_block(owner, block);
        }
    }

    fn unlink(
        &mut self,
        chain_name: &str,
        count: usize,
        spec: &UnlinkSpec,
        wanted: Option<f64>,
    ) -> Vec<XactId> {
        let members = {
            let chain = self.entities.chain(chain_name, self.now);
            chain.members.clone()
        };
        let mut removed = Vec::new();
        for id in members {
            if removed.len() >= count {
                break;
            }
            let matches = match (&spec.compare_parameter, wanted) {
                (Some(parameter), Some(wanted)) => self.xacts.get(&id).is_some_and(|transaction| {
                    (transaction.parameter(parameter) - wanted).abs() < 1e-9
                }),
                _ => true,
            };
            if matches {
                removed.push(id);
            }
        }
        if removed.is_empty() {
            return removed;
        }
        let removed_set: HashSet<XactId> = removed.iter().copied().collect();
        let chain = self.entities.chain(chain_name, self.now);
        chain.members.retain(|id| !removed_set.contains(id));
        chain.area += chain.content as f64 * (self.now - chain.last_change).max(0.0);
        chain.last_change = self.now;
        let now = self.now;
        for id in &removed {
            chain.content = chain.content.saturating_sub(1);
            if let Some(linked) = chain.linked_at.remove(id) {
                chain.total_time += (now - linked).max(0.0);
            }
        }
        removed
    }

    fn select(&mut self, kind: &RefKind, low: i64, high: i64, mode: SelectMode) -> Option<i64> {
        let mut best: Option<(i64, f64)> = None;
        for number in low..=high {
            let name = crate::error::numeric_name(number as f64);
            let (metric, free) = match kind {
                RefKind::Queue => {
                    let queue = self.entities.queue(&name, self.now);
                    (queue.content as f64, queue.content == 0)
                }
                RefKind::Storage => {
                    let storage =
                        self.entities
                            .storage(&name, UNDECLARED_STORAGE_CAPACITY, self.now);
                    (storage.in_use as f64, storage.remaining() > 0)
                }
                RefKind::Facility => {
                    let facility = self.entities.facility(&name, self.now);
                    ((facility.busy as i64) as f64, facility.available && !facility.busy)
                }
                _ => (0.0, true),
            };
            match mode {
                // GPSS SELECT with the "not in use" mode: the first entity that is
                // free, not the first entity with the smallest metric.
                SelectMode::FirstFree => {
                    if free {
                        return Some(number);
                    }
                }
                SelectMode::Min => {
                    if best.is_none_or(|(_, best_metric)| metric < best_metric) {
                        best = Some((number, metric));
                    }
                }
                SelectMode::Max => {
                    if best.is_none_or(|(_, best_metric)| metric > best_metric) {
                        best = Some((number, metric));
                    }
                }
            }
        }
        best.map(|(number, _)| number)
    }

    fn remove_from_group(
        &mut self,
        group: &str,
        count: usize,
        parameter: Option<&str>,
        wanted: Option<f64>,
    ) -> Vec<XactId> {
        let members = self.transaction_groups.get(group).cloned().unwrap_or_default();
        let mut removed = Vec::new();
        let mut kept = Vec::new();
        for id in members {
            let matches = match (parameter, wanted) {
                (Some(parameter), Some(wanted)) => self.xacts.get(&id).is_some_and(|transaction| {
                    (transaction.parameter(parameter) - wanted).abs() < 1e-9
                }),
                _ => true,
            };
            if matches && removed.len() < count {
                removed.push(id);
            } else {
                kept.push(id);
            }
        }
        if let Some(slot) = self.transaction_groups.get_mut(group) {
            *slot = kept;
        }
        removed
    }

    fn scan_group(
        &mut self,
        spec: &crate::program::ScanSpec,
        compare: Option<f64>,
    ) -> Option<(f64, Option<String>)> {
        let members = self.transaction_groups.get(&spec.group).cloned().unwrap_or_default();
        let mut best: Option<(f64, f64)> = None;
        for id in members {
            let Some(transaction) = self.xacts.get(&id) else {
                continue;
            };
            let from_value = spec
                .from_parameter
                .as_deref()
                .map(|parameter| transaction.parameter(parameter))
                .unwrap_or(0.0);
            let candidate = match &spec.compare_parameter {
                Some(parameter) => transaction.parameter(parameter),
                None => from_value,
            };
            if spec.minimum {
                if best.is_none_or(|(_, best_metric)| candidate < best_metric) {
                    best = Some((from_value, candidate));
                }
            } else {
                if let Some(wanted) = compare
                    && (candidate - wanted).abs() > 1e-9
                {
                    continue;
                }
                return Some((from_value, spec.to_parameter.clone()));
            }
        }
        best.map(|(from_value, _)| (from_value, spec.to_parameter.clone()))
    }

    fn alter_group(
        &mut self,
        group: &str,
        count: usize,
        parameter: Option<&str>,
        value: Option<f64>,
    ) -> bool {
        let members = self.transaction_groups.get(group).cloned().unwrap_or_default();
        let mut changed = false;
        for id in members.into_iter().take(count) {
            if let (Some(parameter), Some(value), Some(transaction)) =
                (parameter, value, self.xacts.get_mut(&id))
            {
                transaction.set_parameter(parameter, value);
                changed = true;
            }
        }
        changed
    }

    fn record_queue_tables(&mut self, queue: &str, waited: f64) {
        let program = self.program;
        let names: Vec<String> = program
            .definitions
            .qtables
            .iter()
            .filter(|(_, definition)| definition.queue == queue)
            .map(|(name, _)| name.clone())
            .collect();
        for name in names {
            if let Some(table) = self.entities.qtables.get_mut(&name) {
                table.tabulate(waited);
            }
        }
    }
}

/// Evaluate the mean and optional modifier of a delay expression.
///
/// Returns `(mean, modifier)` where `modifier` is `Some((value, is_function))`.
fn evaluate_delay(
    mean: &ProgramExpr,
    modifier: Option<&ProgramExpr>,
    ctx: &mut CtxView<'_, '_>,
) -> (f64, Option<(f64, bool)>) {
    let mean = mean.eval(ctx);
    let modifier = modifier.map(|expr| {
        let is_function = matches!(
            expr.ast(),
            crate::expr::Expr::Ref {
                kind: RefKind::Function,
                ..
            }
        );
        (expr.eval(ctx), is_function)
    });
    (mean, modifier)
}

/// Turn a delay mean and modifier into a sampled delay.
///
/// A numeric modifier is a symmetric spread (`uniform(mean - B, mean + B)`);
/// a function modifier is a distribution evaluated to a multiplier (the GPSS
/// idiom `GENERATE FN$Mean,FN$Expon`), so the result is `mean * f`.
fn apply_modifier(
    mean: f64,
    modifier: Option<(f64, bool)>,
    streams: &mut Streams,
    stream: u32,
) -> f64 {
    match modifier {
        Some((value, true)) => (mean * value).max(0.0),
        Some((value, false)) => {
            let spread = value.abs();
            let draw = streams.uniform(stream);
            (mean - spread + 2.0 * spread * draw).max(0.0)
        }
        None => mean.max(0.0),
    }
}

/// Expression context bound to one transaction.
struct CtxView<'a, 'p> {
    sim: &'a mut Simulation<'p>,
    xact: XactId,
    depth: u32,
}

impl Ctx for CtxView<'_, '_> {
    fn parameter(&mut self, name: &str) -> f64 {
        self.sim
            .xacts
            .get(&self.xact)
            .map(|transaction| transaction.parameter(name))
            .unwrap_or(0.0)
    }

    fn reference(&mut self, kind: RefKind, name: &str) -> f64 {
        if self.depth > 64 {
            return 0.0;
        }
        match kind {
            RefKind::Parameter => self.parameter(name),
            RefKind::SaveValue => *self.sim.entities.savevalue(name),
            RefKind::Queue => self.sim.entities.queue(name, self.sim.now).content as f64,
            RefKind::Storage => {
                self.sim
                    .entities
                    .storage(name, UNDECLARED_STORAGE_CAPACITY, self.sim.now)
                    .in_use as f64
            }
            RefKind::Facility => {
                let facility = self.sim.entities.facility(name, self.sim.now);
                (facility.available && !facility.busy) as i64 as f64
            }
            RefKind::Chain => self.sim.entities.chain(name, self.sim.now).content as f64,
            RefKind::LogicSwitch => *self.sim.entities.logic(name) as i64 as f64,
            RefKind::Passed => *self.sim.passed.get(name).unwrap_or(&0) as f64,
            RefKind::Waiting => self
                .sim
                .label_index
                .get(name)
                .map(|index| self.sim.block_stats[*index].current as f64)
                .unwrap_or(0.0),
            RefKind::BooleanVariable | RefKind::Matrix => 0.0,
            RefKind::Variable => {
                if let Some(expr) = self.sim.program.definitions.variables.get(name).cloned() {
                    let mut nested = CtxView {
                        sim: self.sim,
                        xact: self.xact,
                        depth: self.depth + 1,
                    };
                    return expr.eval(&mut nested);
                }
                if let Some(definition) = self
                    .sim
                    .program
                    .definitions
                    .random_variables
                    .get(name)
                    .cloned()
                {
                    let low = {
                        let mut nested = CtxView {
                            sim: self.sim,
                            xact: self.xact,
                            depth: self.depth + 1,
                        };
                        definition.min.eval(&mut nested)
                    };
                    let high = {
                        let mut nested = CtxView {
                            sim: self.sim,
                            xact: self.xact,
                            depth: self.depth + 1,
                        };
                        definition.max.eval(&mut nested)
                    };
                    let low = low.round() as i64;
                    let high = high.round() as i64;
                    return self
                        .sim
                        .streams
                        .uniform_integer(definition.stream, low.min(high), low.max(high))
                        as f64;
                }
                0.0
            }
            RefKind::Function => {
                let Some(definition) = self
                    .sim
                    .program
                    .definitions
                    .functions
                    .get(name)
                    .cloned()
                else {
                    return 0.0;
                };
                self.eval_function(&definition)
            }
        }
    }

    fn clock(&mut self) -> f64 {
        self.sim.now
    }

    fn absolute_clock(&mut self) -> f64 {
        self.sim.now
    }

    fn termination_counter(&mut self) -> f64 {
        self.sim.terminations_left
    }

    fn random(&mut self, stream: u32) -> f64 {
        self.sim.streams.uniform(stream)
    }
}

impl CtxView<'_, '_> {
    fn arg(&mut self, expr: &ProgramExpr) -> f64 {
        let mut nested = CtxView {
            sim: self.sim,
            xact: self.xact,
            depth: self.depth + 1,
        };
        expr.eval(&mut nested)
    }

    fn eval_function(&mut self, def: &FunctionDef) -> f64 {
        match def {
            FunctionDef::Exponential { stream } => self.sim.streams.exponential(*stream, 1.0),
            FunctionDef::Distribution { stream, cumulative } => {
                (self.sim.streams.cumulative_index(*stream, cumulative) + 1) as f64
            }
            FunctionDef::Continuous { argument, points } => {
                let x = self.arg(argument);
                let table: Vec<(f64, f64)> = points
                    .iter()
                    .map(|(px, py)| (self.arg(px), self.arg(py)))
                    .collect();
                interpolate(&table, x)
            }
            FunctionDef::Discrete { argument, points } => {
                let x = self.arg(argument);
                let table: Vec<(f64, f64)> = points
                    .iter()
                    .map(|(px, py)| (self.arg(px), self.arg(py)))
                    .collect();
                step(&table, x)
            }
            FunctionDef::Evaluate { argument, points } => {
                let x = self.arg(argument);
                let mut chosen: Option<&ProgramExpr> = None;
                for (px, py) in points {
                    if self.arg(px) <= x + 1e-12 {
                        chosen = Some(py);
                    }
                }
                chosen.map(|expr| self.arg(expr)).unwrap_or(0.0)
            }
        }
    }
}

/// Linear interpolation over an ordered table.
fn interpolate(table: &[(f64, f64)], x: f64) -> f64 {
    if table.is_empty() {
        return 0.0;
    }
    if x <= table[0].0 {
        return table[0].1;
    }
    for window in table.windows(2) {
        let (x0, y0) = window[0];
        let (x1, y1) = window[1];
        if x <= x1 {
            if (x1 - x0).abs() < 1e-12 {
                return y1;
            }
            let ratio = (x - x0) / (x1 - x0);
            return y0 + ratio * (y1 - y0);
        }
    }
    table.last().map(|(_, y)| *y).unwrap_or(0.0)
}

/// Step function over an ordered table.
fn step(table: &[(f64, f64)], x: f64) -> f64 {
    let mut value = table.first().map(|(_, y)| *y).unwrap_or(0.0);
    for (px, py) in table {
        if *px <= x + 1e-12 {
            value = *py;
        } else {
            break;
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolate_is_linear_and_clamped() {
        let table = [(10.0, 100.0), (20.0, 300.0)];
        assert_eq!(interpolate(&table, 5.0), 100.0);
        assert_eq!(interpolate(&table, 15.0), 200.0);
        assert_eq!(interpolate(&table, 25.0), 300.0);
        assert_eq!(interpolate(&[], 1.0), 0.0);
    }

    #[test]
    fn step_picks_the_last_lower_bound() {
        let table = [(1.0, 10.0), (2.0, 20.0), (3.0, 30.0)];
        assert_eq!(step(&table, 0.5), 10.0);
        assert_eq!(step(&table, 2.0), 20.0);
        assert_eq!(step(&table, 99.0), 30.0);
    }
}
