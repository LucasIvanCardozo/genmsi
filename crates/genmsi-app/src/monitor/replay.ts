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
 * This module has no imports on purpose: it is the pure part of the monitor,
 * and the tests run it without a bundler. The shapes below are the parts of
 * the engine's documents the replay reads; the generated types satisfy them.
 */

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
}

/** The part of the report the replay reads. */
export interface ReportDocumentLike {
  /** The blocks of the program, in program order. */
  blocks: ReportBlockLike[];
  /** The entities of the run. */
  entities: {
    /** The queues of the run, with the content they ended with. */
    queues: Record<string, { name: string; content: number }>;
  };
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
const ENTITY_PROPERTY: Record<string, string> = {
  Queue: "Queue",
  Depart: "Queue",
  Seize: "Facility",
  Release: "Facility",
  Preempt: "Facility",
  Return: "Facility",
  Enter: "Storage",
  Leave: "Storage",
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
  const property = ENTITY_PROPERTY[className];
  if (property === undefined) {
    return null;
  }
  const raw = properties[property];
  if (raw === undefined) {
    return null;
  }
  const text = raw.trim();
  if (text === "" || text.startsWith("*") || text.includes("[")) {
    return null;
  }
  return text;
}

/**
 * The units a block takes, as the model declares them.
 *
 * Only the storages count units; a queue counts the transactions it holds.
 * The property is a plain number in every reference model; an expression is
 * not evaluated by the frontend, and one unit is used instead.
 */
function declaredUnits(properties: Record<string, string>, className: string): number {
  if (className !== "Enter" && className !== "Leave") {
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

    // The name of the entity: what the model declares, or what the engine
    // resolved when the transaction waited for it.
    let entity = entities.get(index) ?? null;
    if (entity === null && event.event === "wait" && !NOT_A_NAME.has(event.detail)) {
      entity = event.detail;
      learned.set(index, entity);
      entities.set(index, entity);
    }
    if (entity === null) {
      entity = learned.get(index) ?? null;
    }

    switch (className) {
      case "Queue":
        // A transaction that enters a queue block is enqueued there.
        if (event.event === "enter" && entity !== null) {
          change.queue = { name: entity, delta: 1 };
          if (!seen.queue.has(entity)) {
            seen.queue.add(entity);
            queues.push(entity);
          }
        }
        break;
      case "Depart":
        // A transaction that enters a depart block leaves the queue.
        if (event.event === "enter" && entity !== null) {
          change.queue = { name: entity, delta: -1 };
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
          if (!seen.facility.has(entity)) {
            seen.facility.add(entity);
            facilities.push(entity);
          }
        }
        break;
      case "Release":
        if (event.event === "enter" && entity !== null) {
          change.facility = { name: entity, owner: null };
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
          if (!seen.facility.has(entity)) {
            seen.facility.add(entity);
            facilities.push(entity);
          }
        }
        break;
      case "Enter":
        // A transaction that leaves an enter block took the units.
        if (event.event === "leave" && entity !== null) {
          change.storage = { name: entity, delta: units.get(index) ?? 1 };
          if (!seen.storage.has(entity)) {
            seen.storage.add(entity);
            storages.push(entity);
          }
        }
        break;
      case "Leave":
        if (event.event === "enter" && entity !== null) {
          change.storage = { name: entity, delta: -(units.get(index) ?? 1) };
          if (!seen.storage.has(entity)) {
            seen.storage.add(entity);
            storages.push(entity);
          }
        }
        break;
      default:
        break;
    }

    // The transactions inside the blocks: the engine counts an entry and a
    // departure the same way the trace records them.
    if (event.event === "enter") {
      change.entered = index;
      if (!seen.block.has(index)) {
        seen.block.add(index);
        blocks.push(index);
      }
    } else if (event.event === "leave" || event.event === "terminate") {
      change.left = index;
      if (!seen.block.has(index)) {
        seen.block.add(index);
        blocks.push(index);
      }
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
  return { time: frame.time, queues, facilities, storages, blocks };
}

/**
 * The state after the frames that come up to `index` (the frame itself
 * included).
 *
 * The state is a pure function of the prefix of the trace: the same prefix
 * always gives the same state.
 */
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
): SeriesRow[] {
  const times = new Set<number>();
  for (const name of entities) {
    for (const point of points.get(name) ?? []) {
      times.add(point.time);
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
  /** The generator blocks, which the engine counts apart. */
  generators: number;
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
  }
  return {
    queues: queueMismatches.length === 0,
    blocks: blockMismatches.length === 0,
    queueMismatches,
    blockMismatches,
    generators,
  };
}
