/**
 * The figure every block of the model shows.
 *
 * Every block of the canvas used to show the same pair: how many
 * transactions are inside it and how many have gone through it. The pair is
 * right for a block that only receives and passes, and wrong for a block
 * that operates on an entity: a queue knows its own content, a facility
 * knows its own occupancy, a storage knows its own units.
 *
 * This module answers for ONE block: given the block (its class and its
 * properties), the live figures of the run and the report the engine
 * published, it returns the figure to draw and a short label of what it
 * means. It is a pure function — the same inputs always give the same
 * figure — and it imports nothing that draws: the canvas and the catalogue
 * read it, and the tests run it with `node --test`.
 *
 * The rule of the app holds here: **the app does not invent numbers**. Every
 * figure comes from the live figures the replay rebuilds with the engine's
 * arithmetic, or from the report the engine published. When the entity of a
 * block cannot be resolved — the model names it through a parameter, or the
 * class does not declare one — the block shows the pair of always, and no
 * number that the engine did not publish is ever drawn.
 */
import type { Figures } from "./stats.ts";

/**
 * The kind of entity a block operates on.
 *
 * `generated` and `terminated` are not entities: they are the two ends of
 * the flow, and their figure is the count the report publishes for their
 * row.
 */
export type EntityKind =
  | "generated"
  | "terminated"
  | "queue"
  | "facility"
  | "storage"
  | "table"
  | "chain"
  | "logic"
  | "value";

/** The figure a block shows. */
export type BlockFigure =
  | {
      /** The figure is about an entity. */
      type: "entity";
      /** Which kind of entity the block operates on. */
      kind: EntityKind;
      /** The name of the entity, when the figure is about one. */
      entity: string | null;
      /** The text to draw in the figure zone of the block. */
      text: string;
      /** A short label of what the figure says. */
      label: string;
    }
  | {
      /** The figure is the pair of always: inside / passed. */
      type: "counts";
      /** The transactions inside the block right now. */
      now: number;
      /** The transactions that have gone through it. */
      ever: number;
      /** The text to draw in the figure zone of the block. */
      text: string;
      /** A short label of what the figure says. */
      label: string;
    };

/** The counts a block shows today: how many are inside it, how many passed. */
export interface BlockCounts {
  /** The transactions inside the block right now. */
  now: number;
  /** The transactions that have gone through it since the run started. */
  ever: number;
}

/** The block of the model, as the figures read it. */
export interface FigureBlock {
  /** The id of the block in the model. */
  id: number;
  /** The class of the block (`Queue`, `FacilityUse`, ...). */
  class: string;
  /** The properties of the block, by name. */
  properties: Record<string, string>;
}

/** One row of the report, as the figures read it. */
export interface ReportRowLike {
  /** The line the report shows (the program index plus one). */
  line: number;
  /** The block of the model the row belongs to. */
  node: number;
  /** The class of the row. */
  class: string;
  /** The entries the row counted during the whole run. */
  entries: number;
  /** The transactions inside the block at the end of the run. */
  current: number;
  /** Whether the row counts the transactions that entered its block. */
  counts_block?: boolean;
}

/** The part of the report the figures read. */
export interface ReportFigureLike {
  /** The time the run started. */
  start_time: number;
  /** The rows of the program, in program order. */
  blocks: ReportRowLike[];
  /** The entities of the run. */
  entities: {
    /** The storages of the run, with the capacity each one declares. */
    storages?: Record<string, { capacity: number }>;
  };
}

/** What the figures of a block are read from. */
export interface FigureSource {
  /** The live figures of the run, as the replay rebuilds them. */
  figures?: Figures | null;
  /** The report the engine published. */
  report?: ReportFigureLike | null;
  /** The clock of the replay: the time the run has reached. */
  clock: number;
  /** The counts of every block, by block id. */
  counts?: Map<number, BlockCounts>;
}

/**
 * The class of every block, and the kind of entity it operates on.
 *
 * The classes that are not here operate on no entity at all: they show the
 * pair of always. `SelectF`, `SelectS` and `SelectL` are among them on
 * purpose: they *choose* an entity (the first free facility, a storage
 * according to its state) and the model does not declare which one, so the
 * engine does not publish it and the figure cannot be resolved.
 */
const ENTITY_KIND: Record<string, EntityKind> = {
  Queue: "queue",
  Depart: "queue",
  Seize: "facility",
  Release: "facility",
  Return: "facility",
  Preempt: "facility",
  FacilityUse: "facility",
  FacilityInt: "facility",
  Enter: "storage",
  Leave: "storage",
  StorageUse: "storage",
  Tabulate: "table",
  Link: "chain",
  Unlink: "chain",
  Logic: "logic",
  InitialLS: "logic",
  SaveValue: "value",
  InitiaSV: "value",
  Counter: "value",
};

/** The property each kind of entity is named by. */
const ENTITY_PROPERTY: Record<EntityKind, string[]> = {
  generated: [],
  terminated: [],
  queue: ["Queue"],
  facility: ["Facility"],
  storage: ["Storage"],
  table: ["NombreTabla"],
  chain: ["Cadena"],
  logic: ["LogicSwich"],
  value: ["SaveValue"],
};

/** The label each kind of figure carries. */
const LABELS: Record<EntityKind, string> = {
  generated: "flechas generadas",
  terminated: "transacciones terminadas",
  queue: "contenido de la cola",
  facility: "% de ocupación del recurso",
  storage: "unidades / capacidad",
  table: "nº de muestras",
  chain: "tamaño de la cadena",
  logic: "valor del interruptor",
  value: "valor que guarda",
};

/** The label of the pair of always. */
const COUNTS_LABEL = "dentro / pasadas";

/**
 * The entity a block operates on, as the model declares it.
 *
 * A literal name is taken as it is. An indirect operand (`*caja`) and a
 * placeholder (`[QCajas]`) are resolved by the engine at run time: the
 * frontend does not guess them, so the block shows the pair of always.
 * This is the rule the replay follows to learn the entity of a block
 * (`declaredName` in `replay.ts`), kept here because the module answers for
 * the block itself and not for the events of the run.
 */
function declaredName(block: FigureBlock, kind: EntityKind): string | null {
  for (const property of ENTITY_PROPERTY[kind]) {
    const raw = block.properties[property];
    if (raw === undefined) {
      continue;
    }
    const text = raw.trim();
    if (text === "" || text.startsWith("*") || text.includes("[")) {
      continue;
    }
    return text;
  }
  return null;
}

/** The pair of always, for a block that operates on no entity. */
function countsFigure(counts: BlockCounts | undefined): BlockFigure {
  if (counts === undefined) {
    return { type: "counts", now: 0, ever: 0, text: "", label: COUNTS_LABEL };
  }
  // A block the run has not reached yet shows nothing: the space of the
  // figure is reserved anyway, so the label does not move when it appears.
  return {
    type: "counts",
    now: counts.now,
    ever: counts.ever,
    text: counts.ever > 0 ? `${counts.now}/${counts.ever}` : "",
    label: COUNTS_LABEL,
  };
}

/** The figure of an entity, with the label its kind carries. */
function entityFigure(
  kind: EntityKind,
  entity: string | null,
  text: string,
): BlockFigure {
  return { type: "entity", kind, entity, text, label: LABELS[kind] };
}

/**
 * The row of the report that counts a block.
 *
 * A block of the model can be several rows of the listing (a facility use
 * is five), and the engine marks the row that counts the block. The rows of
 * a generator and of a terminate are one each, and they carry the count the
 * engine published for them.
 */
function countingRow(
  block: FigureBlock,
  report: ReportFigureLike | null | undefined,
): ReportRowLike | null {
  if (report === null || report === undefined) {
    return null;
  }
  const rows = report.blocks.filter(
    (row) => row.node === block.id && row.class === block.class,
  );
  if (rows.length === 0) {
    return null;
  }
  return rows.find((row) => row.counts_block === true) ?? rows[0];
}

/**
 * The occupancy of a facility, as the engine computes it.
 *
 * The engine's `utilization` is the busy time over the elapsed time of the
 * run. The busy time of the live figure is the time the facility was held
 * and released, plus the time it has been held since: the facility that is
 * busy right now has not closed its interval yet.
 */
function occupancy(
  busyTime: number,
  busy: boolean,
  seizedAt: number,
  clock: number,
  start: number,
): number {
  const held = busyTime + (busy ? Math.max(0, clock - seizedAt) : 0);
  const elapsed = clock - start;
  if (elapsed <= 0) {
    return 0;
  }
  return held / elapsed;
}

/**
 * The figure a block of the model shows.
 *
 * The block shows the figure of the entity it operates on, and the pair of
 * always when it operates on none, or when the entity cannot be resolved.
 */
export function figureOf(block: FigureBlock, source: FigureSource): BlockFigure {
  const counts = source.counts?.get(block.id);

  if (block.class === "Generate" || block.class === "Terminate") {
    const row = countingRow(block, source.report);
    if (row === null) {
      return countsFigure(counts);
    }
    const kind: EntityKind = block.class === "Generate" ? "generated" : "terminated";
    return entityFigure(kind, null, String(row.entries));
  }

  const kind = ENTITY_KIND[block.class];
  if (kind === undefined) {
    return countsFigure(counts);
  }
  const name = declaredName(block, kind);
  if (name === null) {
    return countsFigure(counts);
  }
  const figures = source.figures;
  if (figures === null || figures === undefined) {
    return countsFigure(counts);
  }

  switch (kind) {
    case "queue": {
      const queue = figures.queues[name];
      if (queue === undefined) {
        return countsFigure(counts);
      }
      return entityFigure(kind, name, String(queue.content));
    }
    case "facility": {
      const facility = figures.facilities[name];
      if (facility === undefined) {
        return countsFigure(counts);
      }
      const start = source.report?.start_time ?? 0;
      const ratio = occupancy(
        facility.busyTime,
        facility.busy,
        facility.seizedAt,
        source.clock,
        start,
      );
      return entityFigure(kind, name, `${Math.round(ratio * 100)}%`);
    }
    case "storage": {
      const storage = figures.storages[name];
      if (storage === undefined) {
        return countsFigure(counts);
      }
      // The capacity is what the report publishes: without it the figure
      // would be half of the one the block owes, and the block shows the
      // pair of always instead.
      const capacity = source.report?.entities.storages?.[name]?.capacity;
      if (capacity === undefined) {
        return countsFigure(counts);
      }
      return entityFigure(kind, name, `${storage.inUse}/${capacity}`);
    }
    case "table": {
      const table = figures.tables[name];
      if (table === undefined) {
        return countsFigure(counts);
      }
      return entityFigure(kind, name, String(table.samples));
    }
    case "chain": {
      const chain = figures.chains[name];
      if (chain === undefined) {
        return countsFigure(counts);
      }
      return entityFigure(kind, name, String(chain.content));
    }
    case "logic": {
      // A switch that is reset is a figure too: the absence of the switch
      // in the figures is what says that the run has not reached it.
      const set = figures.logic[name];
      if (set === undefined) {
        return countsFigure(counts);
      }
      return entityFigure(kind, name, set ? "1" : "0");
    }
    case "value": {
      const value = figures.values[name];
      if (value === undefined) {
        return countsFigure(counts);
      }
      return entityFigure(kind, name, String(value));
    }
    default:
      return countsFigure(counts);
  }
}

/**
 * The figures of every block of the model, by block id.
 *
 * The map is built from scratch: the live layer is the one that keeps the
 * identity of the figures that did not change, so that a block whose figure
 * did not move is not redrawn.
 */
export function figuresOf(
  blocks: readonly FigureBlock[],
  source: FigureSource,
): Map<number, BlockFigure> {
  const figures = new Map<number, BlockFigure>();
  for (const block of blocks) {
    figures.set(block.id, figureOf(block, source));
  }
  return figures;
}

/**
 * Whether two figures say the same thing.
 *
 * The live layer keeps the object of the figure that did not change, and
 * this is what tells it that the figure did not change.
 */
export function sameFigure(left: BlockFigure, right: BlockFigure): boolean {
  if (left.type !== right.type) {
    return false;
  }
  if (left.type === "counts" && right.type === "counts") {
    return left.now === right.now && left.ever === right.ever && left.text === right.text;
  }
  if (left.type === "entity" && right.type === "entity") {
    return (
      left.kind === right.kind &&
      left.entity === right.entity &&
      left.text === right.text &&
      left.label === right.label
    );
  }
  return false;
}

/** The properties of a block of the model, by name. */
export function propertiesByName(
  properties: readonly { name: string; value: string }[],
): Record<string, string> {
  const byName: Record<string, string> = {};
  for (const property of properties) {
    byName[property.name] = property.value;
  }
  return byName;
}

/** The block of the model, as the figures read it. */
export function figureBlockOf(block: {
  id: number;
  class: string;
  properties: readonly { name: string; value: string }[];
}): FigureBlock {
  return {
    id: block.id,
    class: block.class,
    properties: propertiesByName(block.properties),
  };
}
