/**
 * The figures of the run, as the engine computes them.
 *
 * The engine publishes the report of a finished run: the content a queue had
 * at the end, the area it covered, the time its transactions waited. The app
 * shows those numbers when the run is over, but a reader watching the
 * reproduction wants to see them *growing*.
 *
 * This module rebuilds them, in the browser, while the run is replayed. The
 * arithmetic is the engine's own — the same right-hand-side integrals over
 * the same instants, in `crates/genmsi-core/src/stat.rs`:
 *
 * - the **queue** (`enqueue`, `dequeue`, `finish`): the area is the content
 *   times the time; the total time is the sum of the waits;
 * - the **facility** (`start_busy`, `stop_busy`): the busy time is the time
 *   it was held;
 * - the **storage** (`try_enter`, `leave`, `finish`): the area is the units
 *   in use times the time;
 * - the **chain** (`link`, `unlink`, `finish`): the area is the size times
 *   the time, and the total time is the sum of the links.
 *
 * Because the events and the formulas are the engine's, the numbers coincide
 * with the report's at the end of the run. The replay's fidelity check
 * proves it on every run: the live figures are compared with the report the
 * engine published.
 */
/** What a queue has seen since the run started. */
export interface QueueFigure {
  /** The transactions waiting in it right now. */
  content: number;
  /** The most transactions it ever held. */
  max: number;
  /** How many transactions entered it. */
  entries: number;
  /** How many of them waited no time at all. */
  zeroEntries: number;
  /** The time its transactions waited, added up. */
  totalTime: number;
  /** Its content times the time, added up. */
  area: number;
  /** When its content last changed. */
  lastChange: number;
}

/** What a facility has seen since the run started. */
export interface FacilityFigure {
  /** Whether it is held right now. */
  busy: boolean;
  /** The transaction that holds it, if any. */
  owner: number | null;
  /** How many times it was taken. */
  entries: number;
  /** The time it was held, added up. */
  busyTime: number;
  /** When it was taken for the last time. */
  seizedAt: number;
  /** How many times a transaction could not take it. */
  retry: number;
  /** How many times its holder was interrupted. */
  inter: number;
}

/** What a storage has seen since the run started. */
export interface StorageFigure {
  /** The units in use right now. */
  inUse: number;
  /** The fewest units it ever held. */
  minUse: number;
  /** The most units it ever held. */
  maxUse: number;
  /** How many times units were taken. */
  entries: number;
  /** Its units times the time, added up. */
  area: number;
  /** How many times the units did not fit. */
  retry: number;
  /** When its content last changed. */
  lastChange: number;
}

/** What a chain has seen since the run started. */
export interface ChainFigure {
  /** The transactions in it right now. */
  content: number;
  /** The most transactions it ever held. */
  max: number;
  /** How many transactions joined it. */
  entries: number;
  /** The time its transactions were linked, added up. */
  totalTime: number;
  /** Its size times the time, added up. */
  area: number;
  /** When its size last changed. */
  lastChange: number;
}

/** What a table has tabulated since the run started. */
export interface TableFigure {
  /** How many values were tabulated. */
  samples: number;
  /** The largest value tabulated. */
  maxValue: number;
  /** The sum of the values tabulated, as the engine's table keeps it. */
  sum: number;
  /** The sum of the squares of the values tabulated. */
  sumSquares: number;
  /** The values that fell beyond the last interval. */
  overflow: number;
  /** How many values fell in each interval. */
  frequencies: number[];
}

/** The figures of every entity of the run. */
export interface Figures {
  /** The queues, by name. */
  queues: Record<string, QueueFigure>;
  /** The facilities, by name. */
  facilities: Record<string, FacilityFigure>;
  /** The storages, by name. */
  storages: Record<string, StorageFigure>;
  /** The chains, by name. */
  chains: Record<string, ChainFigure>;
  /** The tables, by name. */
  tables: Record<string, TableFigure>;
  /** The logic switches, by name: whether they are set. */
  logic: Record<string, boolean>;
  /** The savevalues, by name: what they hold. */
  values: Record<string, number>;
}

/**
 * One change of the run, in the terms the engine's own methods use.
 *
 * The replay knows what happened to which entity and says it in this
 * vocabulary; the figures are accumulated from it. The wait of a dequeue is
 * the time the transaction waited, which only the replay knows: it is the one
 * that remembers when the transaction entered the queue.
 */
export type FigureChange =
  | { kind: "queue-enqueue"; name: string; time: number; units: number }
  | { kind: "queue-dequeue"; name: string; time: number; waited: number; units: number }
  | { kind: "facility-seize"; name: string; time: number; xact: number }
  | { kind: "facility-release"; name: string; time: number }
  | {
      kind: "facility-preempt";
      name: string;
      time: number;
      xact: number;
      /** Whether the facility had a holder to interrupt. */
      interrupted: boolean;
    }
  | { kind: "facility-retry"; name: string }
  | { kind: "storage-enter"; name: string; time: number; units: number }
  | { kind: "storage-leave"; name: string; time: number; units: number }
  | { kind: "storage-retry"; name: string }
  | { kind: "chain-enter"; name: string; time: number; xact: number }
  | {
      kind: "chain-leave";
      name: string;
      time: number;
      /** When each member that left the chain joined it. */
      linked: number[];
    }
  | {
      kind: "table-sample";
      name: string;
      time: number;
      /** The value that was tabulated. */
      value: number;
      /** The interval the value fell in. */
      index: number;
      /** How many intervals the table has. */
      intervals: number;
    }
  | { kind: "logic-set"; name: string; set: boolean }
  | { kind: "value-set"; name: string; value: number };

/** A queue that nothing has happened to yet. */
function emptyQueue(now: number): QueueFigure {
  return {
    content: 0,
    max: 0,
    entries: 0,
    zeroEntries: 0,
    totalTime: 0,
    area: 0,
    lastChange: now,
  };
}

/** A facility that nothing has happened to yet. */
function emptyFacility(now: number): FacilityFigure {
  return {
    busy: false,
    owner: null,
    entries: 0,
    busyTime: 0,
    seizedAt: now,
    retry: 0,
    inter: 0,
  };
}

/** A storage that nothing has happened to yet. */
function emptyStorage(now: number): StorageFigure {
  return {
    inUse: 0,
    minUse: 0,
    maxUse: 0,
    entries: 0,
    area: 0,
    retry: 0,
    lastChange: now,
  };
}

/** A chain that nothing has happened to yet. */
function emptyChain(now: number): ChainFigure {
  return {
    content: 0,
    max: 0,
    entries: 0,
    totalTime: 0,
    area: 0,
    lastChange: now,
  };
}

/**
 * The figures of a run that has not started yet.
 *
 * The integrals of the engine are measured from the instant the run
 * starts, and every entity that appears in the run is measured from the
 * instant it first appears.
 */
export function emptyFigures(): Figures {
  return {
    queues: {},
    facilities: {},
    storages: {},
    chains: {},
    tables: {},
    logic: {},
    values: {},
  };
}

/** The time between two instants, never negative: the engine's `.max(0.0)`. */
function elapsed(from: number, to: number): number {
  return Math.max(0, to - from);
}

/**
 * Apply one change of the run to the figures.
 *
 * The formulas are the engine's, one by one: `Queue::enqueue` and
 * `Queue::dequeue`, `Facility::start_busy` and `Facility::stop_busy`,
 * `Storage::try_enter` and `Storage::leave`, and the chain's link and
 * unlink, all in `crates/genmsi-core/src/stat.rs`.
 *
 * The figures that did not change keep their identity, and the ones that did
 * are new objects: the reader of a long run applies thousands of changes,
 * and copying what did not move is the cost.
 */
export function applyChange(figures: Figures, change: FigureChange): Figures {
  switch (change.kind) {
    case "queue-enqueue": {
      const before = figures.queues[change.name] ?? emptyQueue(change.time);
      const queue: QueueFigure = {
        ...before,
        area: before.area + before.content * elapsed(before.lastChange, change.time),
        lastChange: change.time,
        content: before.content + change.units,
        max: Math.max(before.max, before.content + change.units),
        entries: before.entries + 1,
      };
      return { ...figures, queues: { ...figures.queues, [change.name]: queue } };
    }
    case "queue-dequeue": {
      const before = figures.queues[change.name] ?? emptyQueue(change.time);
      const content = Math.max(0, before.content - change.units);
      const queue: QueueFigure = {
        ...before,
        area: before.area + before.content * elapsed(before.lastChange, change.time),
        lastChange: change.time,
        content,
        totalTime: before.totalTime + change.waited,
        zeroEntries: before.zeroEntries + (change.waited === 0 ? 1 : 0),
      };
      return { ...figures, queues: { ...figures.queues, [change.name]: queue } };
    }
    case "facility-seize": {
      const before = figures.facilities[change.name] ?? emptyFacility(change.time);
      const facility: FacilityFigure = {
        ...before,
        busy: true,
        owner: change.xact,
        entries: before.entries + 1,
        seizedAt: change.time,
      };
      return { ...figures, facilities: { ...figures.facilities, [change.name]: facility } };
    }
    case "facility-release": {
      const before = figures.facilities[change.name] ?? emptyFacility(change.time);
      const facility: FacilityFigure = {
        ...before,
        busyTime: before.busy ? before.busyTime + elapsed(before.seizedAt, change.time) : before.busyTime,
        busy: false,
        owner: null,
      };
      return { ...figures, facilities: { ...figures.facilities, [change.name]: facility } };
    }
    case "facility-preempt": {
      const before = figures.facilities[change.name] ?? emptyFacility(change.time);
      // The engine restarts the clock of the facility when it is taken by
      // force, and counts the interruption only when there was a holder.
      const facility: FacilityFigure = {
        ...before,
        busy: true,
        owner: change.xact,
        entries: before.entries + 1,
        seizedAt: change.time,
        inter: before.inter + (change.interrupted ? 1 : 0),
      };
      return { ...figures, facilities: { ...figures.facilities, [change.name]: facility } };
    }
    case "facility-retry": {
      const before = figures.facilities[change.name] ?? emptyFacility(0);
      const facility: FacilityFigure = { ...before, retry: before.retry + 1 };
      return { ...figures, facilities: { ...figures.facilities, [change.name]: facility } };
    }
    case "storage-enter": {
      const before = figures.storages[change.name] ?? emptyStorage(change.time);
      const inUse = before.inUse + change.units;
      const storage: StorageFigure = {
        ...before,
        area: before.area + before.inUse * elapsed(before.lastChange, change.time),
        lastChange: change.time,
        inUse,
        maxUse: Math.max(before.maxUse, inUse),
        entries: before.entries + 1,
      };
      return { ...figures, storages: { ...figures.storages, [change.name]: storage } };
    }
    case "storage-leave": {
      const before = figures.storages[change.name] ?? emptyStorage(change.time);
      const inUse = Math.max(0, before.inUse - change.units);
      const storage: StorageFigure = {
        ...before,
        area: before.area + before.inUse * elapsed(before.lastChange, change.time),
        lastChange: change.time,
        inUse,
        minUse: Math.min(before.minUse, inUse),
      };
      return { ...figures, storages: { ...figures.storages, [change.name]: storage } };
    }
    case "storage-retry": {
      const before = figures.storages[change.name] ?? emptyStorage(0);
      const storage: StorageFigure = { ...before, retry: before.retry + 1 };
      return { ...figures, storages: { ...figures.storages, [change.name]: storage } };
    }
    case "chain-enter": {
      const before = figures.chains[change.name] ?? emptyChain(change.time);
      const chain: ChainFigure = {
        ...before,
        area: before.area + before.content * elapsed(before.lastChange, change.time),
        lastChange: change.time,
        content: before.content + 1,
        max: Math.max(before.max, before.content + 1),
        entries: before.entries + 1,
      };
      return { ...figures, chains: { ...figures.chains, [change.name]: chain } };
    }
    case "chain-leave": {
      const before = figures.chains[change.name] ?? emptyChain(change.time);
      // The members that left the chain: the engine took them out and said
      // which ones, and the time each of them was linked is the time the
      // chain held it.
      const left = change.linked.length;
      const waited = change.linked.reduce(
        (sum, linked) => sum + elapsed(linked, change.time),
        0,
      );
      const chain: ChainFigure = {
        ...before,
        area: before.area + before.content * elapsed(before.lastChange, change.time),
        lastChange: change.time,
        content: Math.max(0, before.content - left),
        totalTime: before.totalTime + waited,
      };
      return { ...figures, chains: { ...figures.chains, [change.name]: chain } };
    }
    case "table-sample": {
      const before = figures.tables[change.name];
      const frequencies = before === undefined ? [] : before.frequencies.slice();
      while (frequencies.length < change.intervals) {
        frequencies.push(0);
      }
      frequencies[change.index] = (frequencies[change.index] ?? 0) + 1;
      const table: TableFigure = {
        samples: (before?.samples ?? 0) + 1,
        maxValue: Math.max(before?.maxValue ?? 0, change.value),
        sum: (before?.sum ?? 0) + change.value,
        sumSquares: (before?.sumSquares ?? 0) + change.value * change.value,
        // The last interval of a table is the class of the values that do
        // not fit in the others: the engine counts them apart.
        overflow:
          (before?.overflow ?? 0) +
          (change.intervals > 1 && change.index === change.intervals - 1 ? 1 : 0),
        frequencies,
      };
      return { ...figures, tables: { ...figures.tables, [change.name]: table } };
    }
    case "logic-set": {
      return { ...figures, logic: { ...figures.logic, [change.name]: change.set } };
    }
    case "value-set": {
      return { ...figures, values: { ...figures.values, [change.name]: change.value } };
    }
    default:
      return figures;
  }
}

/**
 * Close the integrals that are still open at `now`.
 *
 * This is the engine's `Entities::finish`: the queues, the storages and the
 * chains close their area, and the facilities that are still held close
 * their busy time. The total time of a queue and of a chain is not closed
 * here: the engine adds it when the transaction leaves.
 */
export function closeFigures(figures: Figures, now: number): Figures {
  const queues: Record<string, QueueFigure> = {};
  for (const [name, queue] of Object.entries(figures.queues)) {
    queues[name] = {
      ...queue,
      area: queue.area + queue.content * elapsed(queue.lastChange, now),
      lastChange: now,
    };
  }
  const storages: Record<string, StorageFigure> = {};
  for (const [name, storage] of Object.entries(figures.storages)) {
    storages[name] = {
      ...storage,
      area: storage.area + storage.inUse * elapsed(storage.lastChange, now),
      lastChange: now,
    };
  }
  const chains: Record<string, ChainFigure> = {};
  for (const [name, chain] of Object.entries(figures.chains)) {
    chains[name] = {
      ...chain,
      area: chain.area + chain.content * elapsed(chain.lastChange, now),
      lastChange: now,
    };
  }
  const facilities: Record<string, FacilityFigure> = {};
  for (const [name, facility] of Object.entries(figures.facilities)) {
    facilities[name] = facility.busy
      ? { ...facility, busyTime: facility.busyTime + elapsed(facility.seizedAt, now) }
      : facility;
  }
  return { queues, facilities, storages, chains, tables: figures.tables, logic: figures.logic, values: figures.values };
}
