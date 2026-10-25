/**
 * The replay: the state of a finished run, rebuilt by walking its trace.
 *
 * The replay is a **projection of the documents the engine already
 * published**, never a second simulation: the trace carries every event of
 * the run (`genmsi.trace/1`) and the report carries the program's blocks and
 * the final state (`genmsi.report/1`). Walking the trace rebuilds what the
 * run had at every instant:
 *
 * - the clock (the time of the event),
 * - the content of the queues (a transaction enters a queue when it enters
 *   the queue's block and leaves it when it enters the block that departs
 *   from it),
 * - the owner of the facilities (a transaction takes a facility when it
 *   leaves the seize block, and frees it when it enters the release block),
 * - the units in use of the storages,
 * - the transactions inside every block.
 *
 * No metric the engine computes is derived here: the average content and the
 * utilization are **not** rebuilt, because the report does not carry them and
 * the phase 4b review removed the column that implied otherwise. What the
 * replay shows is the *state*, which is what a monitor shows.
 *
 * The projection is deterministic: the state at event *i* is a pure function
 * of the prefix of the trace that ends at *i*. The index is computed once per
 * run and the animation reads it, so the replay of a big trace never walks
 * the whole trace again per frame.
 *
 * The module imports one thing, and it is pure: the figures of the run, the
 * arithmetic the engine uses to count them. The shapes below are the parts
 * of the engine's documents the replay reads; the generated types satisfy
 * them.
 */
import { applyChange, closeFigures, emptyFigures, type FigureChange, type Figures } from "./stats.ts";
import { FLIGHT_SECONDS } from "./balls.ts";

/** One event of the trace, as the replay reads it. */
export interface TraceEventLike {
  /** The simulated time. */
  time: number;
  /** The transaction the event belongs to. */
  xact: number;
  /** The index of the block in the program. */
  block: number;
  /** The class of the block. */
  class: string;
  /** The label of the block. */
  label: string | null;
  /** What happened. */
  event: string;
  /** The detail the event carries. */
  detail: string;
}

/** The trace of a run, as the replay reads it. */
export interface TraceDocumentLike {
  /** The time the run started. */
  start_time: number;
  /** The time the run ended. */
  end_time: number;
  /** The record limit the run used. */
  limit: number;
  /** The records the trace kept. */
  records: number;
  /** Whether the limit may have cut the trace. */
  truncated: boolean;
  /** The events of the run, in simulation order. */
  events: TraceEventLike[];
}

/** One block of the program, as the replay reads it. */
export interface ReportBlockLike {
  /** The line the report shows (the program index plus one). */
  line: number;
  /** The model's node of the block. */
  node: number;
  /** The class of the block. */
  class: string;
  /** The transactions inside the block at the end of the run. */
  current: number;
  /** The entries the block counted during the whole run. */
  entries: number;
  /**
   * Whether this row counts the transactions that entered its block.
   *
   * A report that does not say it (an engine that does not publish it yet)
   * leaves the block without a figure: no figure is better than a wrong one.
   */
  counts_block?: boolean;
}

/** The part of the report the replay reads. */
export interface ReportDocumentLike {
  /** The blocks of the program, in program order. */
  blocks: ReportBlockLike[];
  /** The entities of the run. */
  entities: {
    /** The queues of the run, with the content they ended with. */
    queues: Record<
      string,
      {
        name: string;
        content: number;
        max?: number;
        entries?: number;
        zero_entries?: number;
        total_time?: number;
        area?: number;
        retry?: number;
      }
    >;
    /** The facilities of the run. */
    facilities?: Record<
      string,
      {
        name: string;
        entries: number;
        busy_time: number;
        busy: boolean;
        retry: number;
        inter: number;
      }
    >;
    /** The storages of the run. */
    storages?: Record<
      string,
      {
        name: string;
        in_use: number;
        min_use: number;
        max_use: number;
        entries: number;
        area: number;
        retry: number;
      }
    >;
    /** The chains of the run. */
    chains?: Record<
      string,
      {
        name: string;
        content: number;
        max: number;
        entries: number;
        total_time: number;
        area: number;
      }
    >;
    /** The tables of the run, with the intervals they count. */
    tables?: Record<string, TableDefinitionLike>;
    /** The queue tables of the run, with the intervals they count. */
    qtables?: Record<string, TableDefinitionLike>;
    /** The logic switches of the run: whether they are set. */
    logic?: Record<string, boolean>;
    /** The savevalues of the run: what they hold. */
    savevalues?: Record<string, number>;
  };
}

/** How a table counts its values: the intervals of its histogram. */
export interface TableDefinitionLike {
  /** The name of the table. */
  name?: string;
  /** The first value of the first interval. */
  first: number;
  /** The width of every interval. */
  width: number;
  /** How many intervals the table has. */
  intervals: number;
  /** How many values were tabulated, as the report publishes them. */
  samples?: number;
  /** The largest value tabulated, as the report publishes it. */
  max_value?: number;
  /** The values that fell beyond the last interval. */
  overflow?: number;
  /** How many values fell in each interval, as the report publishes them. */
  frequencies?: number[];
}

/** One block of the model, as the replay reads it. */
export interface ModelBlockLike {
  /** The id of the block in the model. */
  id: number;
  /** The properties of the block, with their decoded values. */
  properties: { name: string; value: string }[];
}

/** The part of the model document the replay reads. */
export interface ModelDocumentLike {
  /** The blocks of the model. */
  blocks: ModelBlockLike[];
}

/** The state of the run at one instant of the replay. */
export interface ReplayState {
  /** The simulated clock. */
  time: number;
  /** The transactions inside every queue. */
  queues: Record<string, number>;
  /** The transaction that owns every facility (null when it is free). */
  facilities: Record<string, number | null>;
  /** The units in use of every storage. */
  storages: Record<string, number>;
  /** The transactions inside every program block. */
  blocks: Record<number, number>;
  /** The entries every program block has had, since the run started. */
  totals: Record<number, number>;
  /** The times every program block could not be entered, since the run started. */
  retries: Record<number, number>;
  /** The figures of the entities of the run, as the engine counts them. */
  figures: Figures;
}

/** What one trace event changes in the state of the run. */
export interface StateChange {
  /** The queue whose content changes, and by how much. */
  queue?: { name: string; delta: number };
  /** The facility whose owner changes. */
  facility?: {
    name: string;
    /** The transaction that owns it now (null when it is free). */
    owner: number | null;
  };
  /** The storage whose units in use change, and by how much. */
  storage?: { name: string; delta: number };
  /** The program block that gains a transaction. */
  entered?: number;
  /** The program block that loses a transaction. */
  left?: number;
  /** The program block that counts an entry. */
  entered_count?: number;
  /** The program block that could not be entered. */
  retried?: number;
  /** What the change does to the figures of the entity it touches. */
  figure?: FigureChange;
}

/** One frame of the replay: a trace event and what it changes. */
export interface Frame {
  /** The simulated time of the event. */
  time: number;
  /** The transaction the event belongs to. */
  xact: number;
  /** The program block the event happened in. */
  block: number;
  /** The model's node of that block (null when the report has no such block). */
  node: number | null;
  /** The class of the block. */
  class: string;
  /** The event (`generate`, `enter`, `wait`, `leave`, `terminate`). */
  event: string;
  /** The detail the event carries. */
  detail: string;
  /** What the event changes in the state (null when it changes nothing). */
  change: StateChange | null;
}

/** One point of an entity's series: its value at one instant. */
export interface SeriesPoint {
  /** The simulated time. */
  time: number;
  /** The value of the entity (the content of a queue, the busy state of a facility). */
  value: number;
}

/** The rows of a chart: the instants of the entities and the value of each. */
export type SeriesRow = { time: number } & Record<string, number>;

/** The index of a run: what the replay reads, computed once. */
export interface ReplayIndex {
  /** The frames of the replay, in simulation order. */
  frames: Frame[];
  /** The state before the first frame. */
  initial: ReplayState;
  /** The queues the replay touches, in the order the run mentions them. */
  queues: string[];
  /** The facilities the replay touches, in the order the run mentions them. */
  facilities: string[];
  /** The storages the replay touches, in the order the run mentions them. */
  storages: string[];
  /** The program blocks that had activity, in the order the run mentions them. */
  blocks: number[];
  /** The instants each queue's content changes, with the content it takes. */
  queuePoints: Map<string, SeriesPoint[]>;
  /** The instants each facility's state changes, with the state it takes. */
  facilityPoints: Map<string, SeriesPoint[]>;
  /** Whether the trace may have been cut by its record limit. */
  truncated: boolean;
  /** How many records the trace kept. */
  records: number;
  /** The record limit the run used. */
  limit: number;
  /** The time the run started. */
  startTime: number;
  /** The time the run ended. */
  endTime: number;
}

/** The property that holds the entity of each program class. */
const ENTITY_PROPERTY: Record<string, string[]> = {
  Queue: ["Queue"],
  Depart: ["Queue"],
  Seize: ["Facility"],
  Release: ["Facility"],
  Preempt: ["Facility"],
  Return: ["Facility"],
  Enter: ["Storage"],
  Leave: ["Storage"],
  Link: ["Cadena", "Chain"],
  Unlink: ["Cadena", "Chain"],
};

/** The details of a `wait` that are not an entity's name. */
const NOT_A_NAME = new Set(["gate", "chain"]);

/** The state a run starts in: nothing has happened yet. */
export function emptyState(): ReplayState {
  return {
    time: 0,
    queues: {},
    facilities: {},
    storages: {},
    blocks: {},
    totals: {},
    retries: {},
    figures: emptyFigures(),
  };
}

/**
 * The entity's name of a block, as the model declares it.
 *
 * A literal name is taken as it is. An indirect operand (`*name`) and a
 * placeholder (`[name]`) are resolved by the engine at run time: the frontend
 * does not guess them, and the trace's own `wait` events carry the name the
 * engine resolved when it happens.
 */
function declaredName(properties: Record<string, string>, className: string): string | null {
  const candidates = ENTITY_PROPERTY[className];
  if (candidates === undefined) {
    return null;
  }
  for (const property of candidates) {
    const raw = properties[property];
    if (raw === undefined) {
      continue;
    }
    const text = raw.trim();
    // A name that is a parameter (`*caja`) or an expression (`[QCajas]`)
    // is not a name: the entity it points at is only known at run time, and
    // the trace is the one that says it.
    if (text === "" || text.startsWith("*") || text.includes("[")) {
      continue;
    }
    return text;
  }
  return null;
}

/**
 * The units a block takes, as the model declares them.
 *
 * A queue counts the units its transactions occupy, and so does a storage:
 * the original compiles the `StorageUse` as `ARRIVE [Queue],[Cantidad]`, and
 * a `Queue` class has its own `Cantidad` too. The property is a plain number
 * in every reference model; an expression is not evaluated by the frontend,
 * and one unit is used instead.
 */
function declaredUnits(properties: Record<string, string>, className: string): number {
  if (className !== "Enter" && className !== "Leave" && className !== "Queue" && className !== "Depart") {
    return 1;
  }
  const raw = properties["Cantidad"]?.trim();
  if (raw === undefined || raw === "") {
    return 1;
  }
  const value = Number(raw);
  return Number.isFinite(value) && value > 0 ? value : 1;
}

/** The properties of a block of the model, by name. */
function propertiesOf(model: ModelDocumentLike): Map<number, Record<string, string>> {
  const byBlock = new Map<number, Record<string, string>>();
  for (const block of model.blocks) {
    const properties: Record<string, string> = {};
    for (const property of block.properties) {
      properties[property.name] = property.value;
    }
    byBlock.set(block.id, properties);
  }
  return byBlock;
}

/**
 * Build the index of a run: the frames of the replay and the series.
 *
 * The trace is walked once. Every frame carries what its event changes, so
 * the state at any instant is the fold of the frames that come before it.
 */
export function buildIndex(
  trace: TraceDocumentLike,
  report: ReportDocumentLike,
  model: ModelDocumentLike,
): ReplayIndex {
  const properties = propertiesOf(model);
  const program = report.blocks;

  // The entity of every program block, as the model declares it.
  const entities = new Map<number, string | null>();
  const units = new Map<number, number>();
  for (const block of program) {
    const declared = properties.get(block.node) ?? {};
    entities.set(block.line - 1, declaredName(declared, block.class));
    units.set(block.line - 1, declaredUnits(declared, block.class));
  }

  const frames: Frame[] = [];
  const queues: string[] = [];
  const facilities: string[] = [];
  const storages: string[] = [];
  const blocks: number[] = [];
  // The name the engine resolved when a transaction waited for an entity.
  const learned = new Map<number, string>();
  // When every transaction entered each queue, to know how long it waited
  // when it left it: the engine keeps the same map (`Queue::enqueued_at`).
  const enqueuedAt = new Map<string, Map<number, number>>();
  // And when every transaction joined each chain, by transaction: the engine
  // says which members leave the chain, and the time each of them was
  // linked is the time the chain held it.
  const linkedAt = new Map<string, Map<number, number>>();
  // Who holds each facility right now, to know whether a preemption
  // interrupts somebody.
  const holders = new Map<string, number | null>();
  // The tables of the run, with the intervals their histogram counts, so a
  // sample can be placed in the interval the engine would put it in.
  const tables = new Map<string, TableDefinitionLike>();
  for (const [name, table] of Object.entries(report.entities.tables ?? {})) {
    tables.set(name, table);
  }
  for (const [name, table] of Object.entries(report.entities.qtables ?? {})) {
    tables.set(name, table);
  }
  // The name of the entity every transaction is in, by block: a name that
  // comes from a parameter is not the same for every transaction, so it is
  // remembered per transaction.
  const names = new Map<number, Map<number, string>>();
  // The block every transaction waits in, when it waits for an entity: the
  // transaction backs up to the block it came from, and it leaves that
  // block, not the one it waits in. The entity it was waiting for belongs
  // to the block it waits in.
  const waiting = new Map<number, number>();
  const seen = {
    queue: new Set<string>(),
    facility: new Set<string>(),
    storage: new Set<string>(),
    block: new Set<number>(),
  };

  for (const event of trace.events) {
    const index = event.block;
    const block = program[index];
    const className = block?.class ?? event.class;
    const node = block?.node ?? null;
    const change: StateChange = {};

    // The block the entity of the event belongs to.
    //
    // A transaction that waits in a block that could not take it backs up
    // to the block it came from, and it leaves that block: the entity it
    // was waiting for is the one of the block it waits in.
    const waited = event.event === "leave" ? waiting.get(event.xact) : undefined;
    const entityBlock = waited ?? index;
    const entityClass = program[entityBlock]?.class ?? className;

    // The name of the entity: what the engine published in the detail of
    // the event, or what the model declares.
    //
    // The engine resolves the name of an entity that a parameter points at
    // when the transaction arrives, and publishes it in the event: the
    // trace is the only thing that knows it, and the model cannot say it.
    const published =
      (event.event === "enter" || event.event === "wait") &&
      event.detail !== "" &&
      !NOT_A_NAME.has(event.detail)
        ? event.detail
        : null;
    let entity = published;
    if (entity === null && (event.event === "leave" || event.event === "unlink")) {
      // The name the transaction had when it was in the block: a name that
      // comes from a parameter is not the same for every transaction.
      entity = recall(names, entityBlock, event.xact, null);
    }
    if (entity === null) {
      entity = entities.get(entityBlock) ?? null;
    }
    if (entity === null) {
      entity = learned.get(entityBlock) ?? null;
    }
    if (published !== null) {
      remember(names, index, event.xact, published);
      if (!learned.has(index)) {
        learned.set(index, published);
        entities.set(index, published);
      }
    }
    if (event.event === "wait") {
      // The transaction waits in this block: it will leave the block it
      // came from, and the entity it waits for is the one of this block.
      waiting.set(event.xact, index);
    }
    if (event.event === "leave" || event.event === "unlink") {
      // The transaction left the block: the name it had there is of no use
      // any more.
      forget(names, index, event.xact);
      forget(names, entityBlock, event.xact);
      waiting.delete(event.xact);
    }

    switch (entityClass) {
      case "Queue":
        // A transaction that enters a queue block is enqueued there, and it
        // takes as many places in the queue as units it carries.
        if (event.event === "enter" && entity !== null) {
          const given = units.get(entityBlock) ?? 1;
          change.queue = { name: entity, delta: given };
          change.figure = { kind: "queue-enqueue", name: entity, time: event.time, units: given };
          // The engine remembers when each transaction entered the queue:
          // that is what the wait of its departure is measured from.
          remember(enqueuedAt, entity, event.xact, event.time);
          if (!seen.queue.has(entity)) {
            seen.queue.add(entity);
            queues.push(entity);
          }
        }
        break;
      case "Depart":
        // A transaction that enters a depart block leaves the queue, and it
        // frees as many places as it took.
        if (event.event === "enter" && entity !== null) {
          const given = units.get(entityBlock) ?? 1;
          change.queue = { name: entity, delta: -given };
          const entered = recall(enqueuedAt, entity, event.xact, event.time);
          change.figure = {
            kind: "queue-dequeue",
            name: entity,
            time: event.time,
            waited: event.time - entered,
            units: given,
          };
          forget(enqueuedAt, entity, event.xact);
          if (!seen.queue.has(entity)) {
            seen.queue.add(entity);
            queues.push(entity);
          }
        }
        break;
      case "Seize":
        // A transaction that leaves a seize block took the facility: a
        // transaction that could not take it waits instead.
        if (event.event === "leave" && entity !== null) {
          change.facility = { name: entity, owner: event.xact };
          change.figure = {
            kind: "facility-seize",
            name: entity,
            time: event.time,
            xact: event.xact,
          };
          holders.set(entity, event.xact);
          if (!seen.facility.has(entity)) {
            seen.facility.add(entity);
            facilities.push(entity);
          }
        }
        break;
      case "Release":
        if (event.event === "enter" && entity !== null) {
          change.facility = { name: entity, owner: null };
          change.figure = { kind: "facility-release", name: entity, time: event.time };
          holders.set(entity, null);
          if (!seen.facility.has(entity)) {
            seen.facility.add(entity);
            facilities.push(entity);
          }
        }
        break;
      case "Preempt":
        // The preempting transaction owns the facility from now on; the one
        // it took it from takes it back when it returns.
        if (event.event === "enter" && entity !== null) {
          change.facility = { name: entity, owner: event.xact };
          // The engine counts the interruption only when there was a holder
          // to interrupt, and it restarts the clock of the facility.
          const holder = holders.get(entity) ?? null;
          change.figure = {
            kind: "facility-preempt",
            name: entity,
            time: event.time,
            xact: event.xact,
            interrupted: holder !== null && holder !== event.xact,
          };
          holders.set(entity, event.xact);
          if (!seen.facility.has(entity)) {
            seen.facility.add(entity);
            facilities.push(entity);
          }
        }
        break;
      case "Return":
        // The facility is free again: the transaction that was preempted
        // takes it back when it resumes (its seize block says so).
        if (event.event === "enter" && entity !== null) {
          change.facility = { name: entity, owner: null };
          change.figure = { kind: "facility-release", name: entity, time: event.time };
          holders.set(entity, null);
          if (!seen.facility.has(entity)) {
            seen.facility.add(entity);
            facilities.push(entity);
          }
        }
        break;
      case "Enter":
        // A transaction that leaves an enter block took the units.
        if (event.event === "leave" && entity !== null) {
          const taken = units.get(entityBlock) ?? 1;
          change.storage = { name: entity, delta: taken };
          change.figure = {
            kind: "storage-enter",
            name: entity,
            time: event.time,
            units: taken,
          };
          if (!seen.storage.has(entity)) {
            seen.storage.add(entity);
            storages.push(entity);
          }
        }
        // A transaction that waits at an enter block did not get the units.
        if (event.event === "wait" && entity !== null) {
          change.figure = { kind: "storage-retry", name: entity };
        }
        break;
      case "Leave":
        if (event.event === "enter" && entity !== null) {
          const given = units.get(entityBlock) ?? 1;
          change.storage = { name: entity, delta: -given };
          change.figure = {
            kind: "storage-leave",
            name: entity,
            time: event.time,
            units: given,
          };
          if (!seen.storage.has(entity)) {
            seen.storage.add(entity);
            storages.push(entity);
          }
        }
        break;
      case "Link":
        // A transaction that enters a link block joins the chain.
        if (event.event === "enter" && entity !== null) {
          change.figure = {
            kind: "chain-enter",
            name: entity,
            time: event.time,
            xact: event.xact,
          };
          joined(linkedAt, entity, event.xact, event.time);
        }
        break;
      default:
        break;
    }

    // The events the engine publishes about the things the report counts and
    // the trace did not carry: the members that leave a chain, the samples
    // of a table, the state of a switch and the value of a savevalue.
    if (event.event === "unlink" && entity !== null) {
      const linked: number[] = [];
      for (const member of event.detail.split(",")) {
        const xact = Number(member);
        if (!Number.isFinite(xact)) {
          continue;
        }
        linked.push(left(linkedAt, entity, xact, event.time));
        forget(linkedAt, entity, xact);
      }
      if (linked.length > 0) {
        change.figure = { kind: "chain-leave", name: entity, time: event.time, linked };
      }
    } else if (event.event === "table") {
      const sample = splitDetail(event.detail);
      if (sample !== null) {
        const definition = tables.get(sample.name);
        if (definition !== undefined) {
          const width = definition.width <= 0 ? 1 : definition.width;
          // The interval the value falls in, the engine's own arithmetic.
          const index =
            sample.value <= definition.first
              ? 0
              : Math.min(
                  Math.ceil((sample.value - definition.first) / width),
                  definition.intervals - 1,
                );
          change.figure = {
            kind: "table-sample",
            name: sample.name,
            time: event.time,
            value: sample.value,
            index: Math.max(0, index),
            intervals: definition.intervals,
          };
        }
      }
    } else if (event.event === "logic") {
      const switched = splitDetail(event.detail);
      if (switched !== null) {
        change.figure = { kind: "logic-set", name: switched.name, set: switched.value !== 0 };
      }
    } else if (event.event === "value") {
      const assigned = splitDetail(event.detail);
      if (assigned !== null) {
        change.figure = { kind: "value-set", name: assigned.name, value: assigned.value };
      }
    }

    // The transactions inside the blocks: the engine counts an entry and a
    // departure the same way the trace records them.
    if (event.event === "enter") {
      change.entered = index;
      change.entered_count = index;
      if (!seen.block.has(index)) {
        seen.block.add(index);
        blocks.push(index);
      }
    } else if (event.event === "generate") {
      // A generator counts an entry for every transaction it creates, which
      // is what the engine's report counts for its block.
      change.entered_count = index;
    } else if (event.event === "leave" || event.event === "terminate") {
      change.left = index;
      if (!seen.block.has(index)) {
        seen.block.add(index);
        blocks.push(index);
      }
    } else if (event.event === "backup") {
      // The transaction backs up to the block it came from: the engine
      // counts it there, and the replay counts it there too. The block it
      // waits in loses it, and the block the detail names gains it.
      //
      // The entities are not touched: the transaction did not enter a queue
      // and did not leave one.
      change.left = index;
      const target = targetOf(event.detail);
      if (target !== null) {
        change.entered = target;
      }
    } else if (event.event === "wait") {
      // A transaction that could not enter the block: the engine counts it
      // as a retry of the block.
      change.retried = index;
    }

    frames.push({
      time: event.time,
      xact: event.xact,
      block: index,
      node,
      class: className,
      event: event.event,
      detail: event.detail,
      change: Object.keys(change).length === 0 ? null : change,
    });
  }

  const queuePoints = new Map<string, SeriesPoint[]>();
  const facilityPoints = new Map<string, SeriesPoint[]>();
  collectPoints(frames, queuePoints, facilityPoints);

  return {
    frames,
    initial: emptyState(),
    queues,
    facilities,
    storages,
    blocks,
    queuePoints,
    facilityPoints,
    truncated: trace.truncated,
    records: trace.records,
    limit: trace.limit,
    startTime: trace.start_time,
    endTime: trace.end_time,
  };
}

/**
 * Apply one frame to the state.
 *
 * The state is a new object: the view draws what comes back and never what
 * it kept.
 */
export function applyFrame(state: ReplayState, frame: Frame): ReplayState {
  const change = frame.change;
  if (change === null) {
    return state.time === frame.time ? state : { ...state, time: frame.time };
  }
  // Only the records that change are copied: the replay of a big trace
  // applies thousands of frames, and copying what did not move is the cost.
  let queues = state.queues;
  let facilities = state.facilities;
  let storages = state.storages;
  let blocks = state.blocks;
  let totals = state.totals;
  let retries = state.retries;
  if (change.queue !== undefined) {
    const content = Math.max(0, (queues[change.queue.name] ?? 0) + change.queue.delta);
    queues = { ...queues, [change.queue.name]: content };
  }
  if (change.facility !== undefined) {
    facilities = { ...facilities, [change.facility.name]: change.facility.owner };
  }
  if (change.storage !== undefined) {
    const units = Math.max(0, (storages[change.storage.name] ?? 0) + change.storage.delta);
    storages = { ...storages, [change.storage.name]: units };
  }
  if (change.entered !== undefined) {
    blocks = { ...blocks, [change.entered]: (blocks[change.entered] ?? 0) + 1 };
  }
  if (change.left !== undefined) {
    blocks = { ...blocks, [change.left]: Math.max(0, (blocks[change.left] ?? 0) - 1) };
  }
  if (change.entered_count !== undefined) {
    totals = { ...totals, [change.entered_count]: (totals[change.entered_count] ?? 0) + 1 };
  }
  if (change.retried !== undefined) {
    retries = { ...retries, [change.retried]: (retries[change.retried] ?? 0) + 1 };
  }
  // The figures of the entities: the engine's own arithmetic, applied to the
  // change the frame carries.
  let figures = state.figures;
  if (change.figure !== undefined) {
    figures = applyChange(figures, change.figure);
  }
  return { time: frame.time, queues, facilities, storages, blocks, totals, retries, figures };
}

/**
 * The state after the frames that come up to `index` (the frame itself
 * included).
 *
 * The state is a pure function of the prefix of the trace: the same prefix
 * always gives the same state.
 */
/**
 * Remember something about a transaction in an entity.
 *
 * The engine keeps a map per entity — `Queue::enqueued_at` and
 * `Chain::linked_at` — and so does the replay: the wait of a transaction is
 * the time between its entry and its departure.
 */
function remember<K, V>(
  map: Map<K, Map<number, V>>,
  key: K,
  xact: number,
  value: V,
): void {
  const entries = map.get(key);
  if (entries === undefined) {
    map.set(key, new Map([[xact, value]]));
    return;
  }
  entries.set(xact, value);
}

/**
 * What is known about a transaction in an entity.
 *
 * When it is not known, the instant itself is returned: the wait is then no
 * time at all, which is what the engine counts when a transaction leaves
 * something it never entered.
 */
function recall<K, V>(
  map: Map<K, Map<number, V>>,
  key: K,
  xact: number,
  fallback: V,
): V {
  return map.get(key)?.get(xact) ?? fallback;
}

/** Forget that a transaction entered an entity: it left it. */
function forget<K, V>(map: Map<K, Map<number, V>>, key: K, xact: number): void {
  map.get(key)?.delete(xact);
}

/**
 * Split a detail of the form "{name} {value}".
 *
 * The name of an entity may hold spaces and a number does not, so the two
 * are separated by the last space. The number may be written with the comma
 * as its decimal point, which is how the report prints them.
 */
function splitDetail(detail: string): { name: string; value: number } | null {
  const cut = detail.lastIndexOf(" ");
  if (cut <= 0) {
    return null;
  }
  const written = detail.slice(cut + 1);
  const value = Number(written.replace(",", "."));
  if (!Number.isFinite(value)) {
    return null;
  }
  return { name: detail.slice(0, cut), value };
}

/**
 * Remember that a transaction joined a chain.
 *
 * The engine keeps the members of a chain and the time each of them joined,
 * and so does the replay.
 */
function joined(map: Map<string, Map<number, number>>, entity: string, xact: number, time: number): void {
  const members = map.get(entity);
  if (members === undefined) {
    map.set(entity, new Map([[xact, time]]));
    return;
  }
  members.set(xact, time);
}

/**
 * When the member that left a chain joined it.
 *
 * When the chain is not known, the instant itself is returned: the member
 * was there for no time at all.
 */
function left(map: Map<string, Map<number, number>>, entity: string, xact: number, fallback: number): number {
  return map.get(entity)?.get(xact) ?? fallback;
}

export function stateAt(index: ReplayIndex, upTo: number): ReplayState {
  const last = Math.min(upTo, index.frames.length - 1);
  let state = index.initial;
  for (let position = 0; position <= last; position += 1) {
    state = applyFrame(state, index.frames[position]);
  }
  return state;
}

/**
 * The state after the frames from `from` (excluded) to `to` (included),
 * starting from `state`.
 *
 * The animation walks forward, so the state is carried from one frame to the
 * next instead of being rebuilt.
 */
export function advance(state: ReplayState, index: ReplayIndex, from: number, to: number): ReplayState {
  let current = state;
  const last = Math.min(to, index.frames.length - 1);
  for (let position = Math.max(from + 1, 0); position <= last; position += 1) {
    current = applyFrame(current, index.frames[position]);
  }
  return current;
}

/**
 * The speeds the monitor offers, as multiples of the real time.
 *
 * A speed is absolute: `1x` is one simulated second per second of wall
 * clock, whatever the length of the run. Multiplying a fixed number of
 * seconds for the whole run instead made the same label mean a different
 * thing in every model — 300x in a one-hour run, 2400x in an eight-hour one —
 * and left the reader without a way to know the scale.
 */
export const SPEEDS = [1, 2, 5, 10, 25, 50, 100, 250, 500, 1000] as const;

/**
 * A transaction moving from one block of the model to another.
 *
 * The trace states the move itself: the `leave` record of a block carries the
 * block it goes to (`-> 12`), and the transaction's id says which instance
 * moved. A transition is therefore read from the trace, never inferred.
 *
 * The transition also carries the **flight**: the time the transaction spent
 * in the block it left. The ball that crosses the link travels exactly that
 * time, so it leaves the block when the transaction enters it and arrives
 * when the transaction leaves it.
 */
export interface Transition {
  /** The transaction that moved. */
  xact: number;
  /** The model block the instance left. */
  from: number;
  /** The model block the instance went to. */
  to: number;
  /** The time of the run at which the move happened. */
  time: number;
  /** The seconds the transaction spent in the block it came from. */
  flight: number;
}

/**
 * The moves of the run, as the model shows them.
 *
 * The trace's blocks are the program's statements, and one block of the model
 * is several of them (a facility use is a queue, a seize, a depart, an advance
 * and a release). A move that stays inside the same block of the model is not
 * a move a reader can see, so it is left out: the ball travels between the
 * rectangles the canvas draws.
 *
 * The moves are grouped by the instant they happen, so the animation can find
 * the ones of a moment without walking the whole list.
 */
export interface TransitionIndex {
  /** The moves, ordered by the time they happened. */
  moves: Transition[];
  /** The times the moves happened, sorted (the same order as `moves`). */
  times: number[];
}

/**
 * The moves of the run, read from the trace.
 *
 * `nodeOf` maps a program block to the block of the model it belongs to.
 */
export function buildTransitions(
  trace: TraceDocumentLike,
  nodeOf: (block: number) => number | null,
): TransitionIndex {
  const moves: Transition[] = [];
  // The block of the model every transaction is in, and the instant it
  // entered it. The flight of a move is the difference between the two: the
  // time the transaction spent in the block it left. A transaction that is
  // between blocks has no block of the model (null).
  const inside = new Map<number, number | null>();
  const enteredAt = new Map<number, number>();
  for (const event of trace.events) {
    if (event.event === "leave") {
      const target = targetOf(event.detail);
      const from = nodeOf(event.block);
      if (target !== null && from !== null) {
        const to = nodeOf(target);
        // A move that stays inside the same block of the model is invisible.
        if (to !== null && to !== from) {
          const entered = enteredAt.get(event.xact);
          // The flight of a move is the time the transaction spent in the
          // block it came from, and the ball arrives at the instant of the
          // move. A transaction **born** in that block has no time there at
          // all: it appears and leaves in the same instant. Its ball leaves
          // the block the whole flight before the instance reaches the next
          // one, so every ball crosses in the same time and the illusion
          // holds: when the ball arrives, the instance passes.
          const flight =
            entered === undefined
              ? FLIGHT_SECONDS
              : Math.max(0, event.time - entered);
          moves.push({ xact: event.xact, from, to, time: event.time, flight });
          // The transaction leaves the block of the model it was in: the
          // block it enters next is another one, and the flight of the move
          // that takes it out is measured from that entry.
          inside.set(event.xact, null);
        }
      }
      continue;
    }
    if (event.event === "enter") {
      const node = nodeOf(event.block);
      // The transaction enters a block of the model: the ball of the move
      // that brought it here leaves now. A block of the model may be
      // several blocks of the program, and the transaction stays in it
      // until it leaves the block of the model.
      if (node !== null && inside.get(event.xact) !== node) {
        enteredAt.set(event.xact, event.time);
        inside.set(event.xact, node);
      }
      continue;
    }
    if (event.event === "backup") {
      // The transaction backs up to the block it came from: the engine
      // counts it there, and the ball of the move goes back to the block of
      // the model it is counted in. It is not a move: the transaction does
      // not go anywhere, it is counted in another block.
      const target = targetOf(event.detail);
      if (target !== null) {
        const node = nodeOf(target);
        if (node !== null) {
          inside.set(event.xact, node);
        }
      }
      continue;
    }
    if (event.event === "generate" || event.event === "terminate") {
      inside.delete(event.xact);
      enteredAt.delete(event.xact);
    }
  }
  const times = moves.map((move) => move.time);
  return { moves, times };
}

/**
 * The block a `leave` record goes to, or null when it does not say.
 */
function targetOf(detail: string): number | null {
  const text = detail.trim();
  if (!text.startsWith("->")) {
    return null;
  }
  const target = text.slice(2).trim();
  const value = Number.parseInt(target, 10);
  return Number.isNaN(value) ? null : value;
}

/**
 * The clock of a replay that advances by `elapsed` seconds of wall clock.
 *
 * The clock **accumulates**: it is the time the run has reached, and it only
 * moves forward by what the wall clock spent. Anchoring it at the time of the
 * current frame instead (the obvious way to write it) makes the replay stall
 * whenever the step is smaller than the gap between two frames: the clock
 * restarts from the frame's time on every paint, so it never crosses the next
 * frame and the playhead stays where it was. A model whose events are ten
 * seconds apart would then only move at the speeds whose step exceeds ten
 * seconds.
 *
 * The returned clock is the time the run has reached, which may be past the
 * last frame: the playhead is the last frame at or before it.
 */
export function advanceClock(clock: number, elapsed: number, rate: number): number {
  return clock + elapsed * rate;
}

/**
 * The frame the clock is at: the last one that happened at or before it.
 */
export function frameAt(index: ReplayIndex, time: number): number {
  const frames = index.frames;
  if (frames.length === 0) {
    return -1;
  }
  let low = 0;
  let high = frames.length - 1;
  while (low < high) {
    const middle = Math.ceil((low + high) / 2);
    if (frames[middle].time <= time) {
      low = middle;
    } else {
      high = middle - 1;
    }
  }
  return low;
}

/**
 * The instants each entity's value changes, with the value it takes.
 *
 * The series is the **exact** history of the entity, not a sample of it: a
 * queue that fills and empties between two samples of the trace would be
 * invisible in a sample, and a facility that is free for a moment would look
 * busy. The series is built once, when the index is built, by folding the
 * frames; the chart reads it.
 */
function collectPoints(
  frames: Frame[],
  queuePoints: Map<string, SeriesPoint[]>,
  facilityPoints: Map<string, SeriesPoint[]>,
): void {
  let state = emptyState();
  for (const frame of frames) {
    state = applyFrame(state, frame);
    const change = frame.change;
    if (change === null) {
      continue;
    }
    if (change.queue !== undefined) {
      const list = queuePoints.get(change.queue.name) ?? [];
      list.push({ time: frame.time, value: state.queues[change.queue.name] ?? 0 });
      queuePoints.set(change.queue.name, list);
    }
    if (change.facility !== undefined) {
      const list = facilityPoints.get(change.facility.name) ?? [];
      list.push({
        time: frame.time,
        value: state.facilities[change.facility.name] === null ? 0 : 1,
      });
      facilityPoints.set(change.facility.name, list);
    }
  }
}

/**
 * The rows of a chart: the instants of the entities, with the value of each.
 *
 * The rows are the union of the instants of the chosen entities, and the
 * value of an entity is the one it had at that instant.
 */
export function mergeSeries(
  entities: string[],
  points: Map<string, SeriesPoint[]>,
  until = Number.POSITIVE_INFINITY,
): SeriesRow[] {
  const times = new Set<number>();
  for (const name of entities) {
    for (const point of points.get(name) ?? []) {
      if (point.time <= until) {
        times.add(point.time);
      }
    }
  }
  const sorted = [...times].sort((a, b) => a - b);
  const cursors = new Map<string, number>();
  const values = new Map<string, number>();
  const rows: SeriesRow[] = [];
  for (const time of sorted) {
    const row: SeriesRow = { time };
    for (const name of entities) {
      const list = points.get(name) ?? [];
      let cursor = cursors.get(name) ?? 0;
      while (cursor < list.length && list[cursor].time <= time) {
        values.set(name, list[cursor].value);
        cursor += 1;
      }
      cursors.set(name, cursor);
      row[name] = values.get(name) ?? 0;
    }
    rows.push(row);
  }
  return rows;
}

/**
 * What the projection says about itself.
 *
 * The replay rebuilds the state of the run; the report publishes the final
 * state. When the two agree, the projection is faithful; when they do not,
 * the reason is shown instead of hidden.
 */
export interface Fidelity {
  /** Whether the derived content of every queue is the one the report publishes. */
  queues: boolean;
  /** Whether the derived transactions of every block are the ones the report publishes. */
  blocks: boolean;
  /** The queues whose derived content differs from the report's. */
  queueMismatches: string[];
  /** The blocks whose derived count differs from the report's. */
  blockMismatches: number[];
  /** Whether the derived entries of every block are the ones the report publishes. */
  entries: boolean;
  /** The blocks whose derived entries differ from the report's. */
  entryMismatches: number[];
  /** The generator blocks, which the engine counts apart. */
  generators: number;
  /** Whether the live figures of every entity are the ones the report publishes. */
  figures: boolean;
  /** The entities whose live figures differ from the report's. */
  figureMismatches: string[];
  /** Whether the live samples of every table are the ones the report publishes. */
  tables: boolean;
  /** The tables whose live samples differ from the report's. */
  tableMismatches: string[];
  /** Whether the live switches and values are the ones the report publishes. */
  switches: boolean;
  /** The switches and values that differ from the report's. */
  switchMismatches: string[];
}

/**
 * Whether two numbers are the same number.
 *
 * The figures are computed with the engine's arithmetic, in the same order,
 * so they come out the same; a hair of tolerance keeps a rounding apart
 * from a mistake.
 */
function sameNumber(derived: number, published: number | undefined): boolean {
  if (published === undefined) {
    return true;
  }
  const scale = Math.max(1, Math.abs(published));
  return Math.abs(derived - published) <= scale * 1e-9;
}

/**
 * Whether two histograms are the same.
 *
 * The live histogram is rebuilt from the samples the trace carries, with the
 * engine's own arithmetic: the two must be the same numbers, interval by
 * interval.
 */
function sameFrequencies(
  derived: number[] | undefined,
  published: number[] | undefined,
): boolean {
  if (derived === undefined || published === undefined) {
    return true;
  }
  if (derived.length !== published.length) {
    return false;
  }
  return derived.every((count, index) => count === published[index]);
}

/**
 * Compare the state the replay ends in with the state the report publishes.
 */
export function checkFidelity(index: ReplayIndex, report: ReportDocumentLike): Fidelity {
  const final = stateAt(index, index.frames.length - 1);
  const queueMismatches: string[] = [];
  for (const [name, queue] of Object.entries(report.entities.queues)) {
    const derived = final.queues[name] ?? 0;
    if (derived !== queue.content) {
      queueMismatches.push(name);
    }
  }
  const blockMismatches: number[] = [];
  const entryMismatches: number[] = [];
  let generators = 0;
  for (const block of report.blocks) {
    // A generator block holds the generator itself in the engine's count, not
    // a transaction: no transaction ever stays inside one.
    if (block.class === "Generate") {
      generators += 1;
      continue;
    }
    const derived = final.blocks[block.line - 1] ?? 0;
    if (derived !== block.current) {
      blockMismatches.push(block.line - 1);
    }
    // The entries the run counted for the block, rebuilt from the trace: the
    // engine's own total has to come out of the walk.
    const entries = final.totals[block.line - 1] ?? 0;
    if (entries !== block.entries) {
      entryMismatches.push(block.line - 1);
    }
  }
  // The live figures, closed at the end of the run, against the report the
  // engine published. This is the proof that the figures the app shows while
  // the run is replayed are the engine's own numbers.
  const figureMismatches: string[] = [];
  const closed = closeFigures(final.figures, index.endTime);
  for (const [name, queue] of Object.entries(report.entities.queues)) {
    const derived = closed.queues[name];
    if (derived === undefined) {
      // The engine knows an entity the projection does not: that is a
      // difference too, and the reader must not see a number that is not
      // the run's.
      if ((queue.entries ?? 0) > 0 || queue.content !== 0) {
        figureMismatches.push(`cola ${name}`);
      }
      continue;
    }
    if (
      derived.content !== queue.content ||
      !sameNumber(derived.max, queue.max) ||
      !sameNumber(derived.entries, queue.entries) ||
      !sameNumber(derived.zeroEntries, queue.zero_entries) ||
      !sameNumber(derived.totalTime, queue.total_time) ||
      !sameNumber(derived.area, queue.area)
    ) {
      figureMismatches.push(`cola ${name}`);
    }
  }
  for (const [name, facility] of Object.entries(report.entities.facilities ?? {})) {
    const derived = closed.facilities[name];
    if (derived === undefined) {
      if (facility.entries > 0 || facility.busy_time > 0) {
        figureMismatches.push(`facility ${name}`);
      }
      continue;
    }
    if (
      derived.busy !== facility.busy ||
      !sameNumber(derived.entries, facility.entries) ||
      !sameNumber(derived.busyTime, facility.busy_time) ||
      !sameNumber(derived.inter, facility.inter)
    ) {
      figureMismatches.push(`facility ${name}`);
    }
  }
  for (const [name, storage] of Object.entries(report.entities.storages ?? {})) {
    const derived = closed.storages[name];
    if (derived === undefined) {
      if (storage.entries > 0 || storage.area > 0) {
        figureMismatches.push(`almacen ${name}`);
      }
      continue;
    }
    if (
      derived.inUse !== storage.in_use ||
      !sameNumber(derived.minUse, storage.min_use) ||
      !sameNumber(derived.maxUse, storage.max_use) ||
      !sameNumber(derived.entries, storage.entries) ||
      !sameNumber(derived.area, storage.area)
    ) {
      figureMismatches.push(`almacen ${name}`);
    }
  }
  for (const [name, chain] of Object.entries(report.entities.chains ?? {})) {
    const derived = closed.chains[name];
    if (derived === undefined) {
      if (chain.entries > 0) {
        figureMismatches.push(`cadena ${name}`);
      }
      continue;
    }
    if (
      derived.content !== chain.content ||
      !sameNumber(derived.max, chain.max) ||
      !sameNumber(derived.entries, chain.entries) ||
      !sameNumber(derived.totalTime, chain.total_time) ||
      !sameNumber(derived.area, chain.area)
    ) {
      figureMismatches.push(`cadena ${name}`);
    }
  }
  // The samples of the tables, the histogram they make, the state of the
  // switches and the value of the savevalues.
  const tableMismatches: string[] = [];
  for (const [name, table] of Object.entries(report.entities.tables ?? {})) {
    const derived = closed.tables[name];
    if (derived === undefined) {
      if ((table.samples ?? 0) > 0) {
        tableMismatches.push(`tabla ${name}`);
      }
      continue;
    }
    if (
      derived.samples !== table.samples ||
      !sameNumber(derived.maxValue, table.max_value) ||
      !sameFrequencies(derived.frequencies, table.frequencies)
    ) {
      tableMismatches.push(`tabla ${name}`);
    }
  }
  for (const [name, table] of Object.entries(report.entities.qtables ?? {})) {
    const derived = closed.tables[name];
    if (derived === undefined) {
      if ((table.samples ?? 0) > 0) {
        tableMismatches.push(`qtabla ${name}`);
      }
      continue;
    }
    if (
      derived.samples !== table.samples ||
      !sameNumber(derived.maxValue, table.max_value) ||
      !sameFrequencies(derived.frequencies, table.frequencies)
    ) {
      tableMismatches.push(`qtabla ${name}`);
    }
  }
  const switchMismatches: string[] = [];
  for (const [name, state] of Object.entries(report.entities.logic ?? {})) {
    const derived = closed.logic[name];
    if (derived !== undefined && derived !== state) {
      switchMismatches.push(`interruptor ${name}`);
    }
  }
  for (const [name, value] of Object.entries(report.entities.savevalues ?? {})) {
    const derived = closed.values[name];
    if (derived !== undefined && !sameNumber(derived, value)) {
      switchMismatches.push(`valor ${name}`);
    }
  }
  return {
    queues: queueMismatches.length === 0,
    blocks: blockMismatches.length === 0,
    entries: entryMismatches.length === 0,
    queueMismatches,
    blockMismatches,
    entryMismatches,
    figures: figureMismatches.length === 0,
    figureMismatches,
    tables: tableMismatches.length === 0,
    tableMismatches,
    switches: switchMismatches.length === 0,
    switchMismatches,    generators,
  };
}
