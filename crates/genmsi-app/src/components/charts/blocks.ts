/**
 * The figures of the blocks of the model.
 *
 * A block of the model is one or several statements of the listing: a
 * facility use is five (a queue, a seize, a depart, an advance and a
 * release), and all of them count the transactions that went through the
 * block. The figure of a block is the one of **the row that counts it**,
 * and the engine says which row that is — so a transaction is not counted
 * five times.
 *
 * The retries are the other way round: each row counts its own, and the
 * block adds the ones of its rows.
 *
 * The counts and the retries are the ones of the instant the clock marks:
 * they come from the live layer, which publishes them with the clock of
 * the frame the replay reached.
 */
import type { LiveCounts } from "../../monitor/live.ts";
import type { ModelBody, ReportDocument } from "../../types";

/** The figures of one block of the model. */
export interface BlockRow {
  /** The name of the block, as the model names it. */
  name: string;
  /** The class of the block. */
  class: string;
  /** The GPSS label of the block, when it declares one. */
  label: string | null;
  /** How many transactions went through it. */
  entries: number;
  /** How many transactions are inside it right now. */
  current: number;
  /** How many times a transaction could not enter it. */
  retries: number;
  /**
   * How many transactions the engine counted in it during the whole run,
   * or null when the report does not know the block.
   */
  published: number | null;
  /**
   * Whether the block generates the transactions of the model.
   *
   * A generator does not receive the transactions: it creates them, so its
   * figure counts the transactions it created.
   */
  generator: boolean;
  /** Whether the figures are the ones of the instant. */
  live: boolean;
}

/** The name a reader knows a block by: its caption, or its name. */
function nameOf(
  model: ModelBody | null,
  node: number,
  fallback: string,
): string {
  const block = model?.blocks.find((candidate) => candidate.id === node);
  if (block === undefined) {
    return fallback;
  }
  const caption = block.caption.trim();
  const name = block.name.trim();
  return caption !== "" ? caption : name !== "" ? name : fallback;
}

/** The class of a block of the model. */
function classOf(model: ModelBody | null, node: number): string {
  return model?.blocks.find((candidate) => candidate.id === node)?.class ?? "";
}

/**
 * The name a reader knows a block by.
 *
 * The model can call two blocks the same, so the name of the card carries
 * the class of the block: two rows of the ranking that say the same thing
 * are two different blocks of the model.
 */
export function blockNameOf(row: BlockRow): string {
  if (row.generator) {
    // El generador ya se llama así en la mayoría de los modelos: sólo se
    // dice cuando el nombre del modelo no lo dice.
    return row.name.toLowerCase().includes("generador")
      ? row.name
      : `${row.name} · generador`;
  }
  if (row.class !== "") {
    return `${row.name} · ${row.class}`;
  }
  // Sin clase, la etiqueta del GPSS es lo que distingue dos bloques que
  // el modelo llama igual.
  return row.label === null ? row.name : `${row.name} · ${row.label}`;
}

/**
 * What the figure of a block counts, when it is not the transactions that
 * went through it.
 *
 * A generator creates the transactions: its figure is the number of
 * transactions it created, and the card says so.
 */
export function blockNoteOf(row: BlockRow): string | undefined {
  return row.generator ? "las transacciones que creó" : undefined;
}

/**
 * The figures of every block of the model.
 *
 * The transactions that went through a block are the ones the row that
 * counts it holds; the retries are the ones of all of its rows.
 *
 * @param report the report of the run
 * @param model the model of the run, for the names of the blocks
 * @param counts the counts of every block at the instant, by node
 * @param retries the times every program block could not be entered
 */
export function blockRows(
  report: ReportDocument,
  model: ModelBody | null,
  counts: ReadonlyMap<number, LiveCounts>,
  retries: Readonly<Record<number, number>>,
): BlockRow[] {
  // The retries of every program row, added up by the block they belong
  // to: each row counts its own, and the block adds the ones of its rows.
  const perNode = new Map<number, number>();
  // The rows that count the block: the engine says which row that is, and
  // its count is the one the engine published for the whole run.
  const counting = new Map<
    number,
    { entries: number; label: string | null }
  >();
  for (const row of report.blocks) {
    const line = row.line - 1;
    perNode.set(row.node, (perNode.get(row.node) ?? 0) + (retries[line] ?? 0));
    if (row.counts_block) {
      counting.set(row.node, { entries: row.entries, label: row.label });
    }
  }

  const nodes = new Set<number>(report.blocks.map((row) => row.node));
  const rows: BlockRow[] = [];
  for (const node of nodes) {
    // Las cifras son las del instante: hasta ahora el bloque no vio
    // ninguna transacción si el reloj todavía no llegó a él.
    const count = counts.get(node);
    const entries = count?.ever ?? 0;
    const current = count?.now ?? 0;
    const published = counting.get(node);
    const className = classOf(model, node);
    rows.push({
      name: nameOf(model, node, `bloque ${node}`),
      class: className,
      label: published?.label ?? null,
      entries,
      current,
      retries: perNode.get(node) ?? 0,
      published: published?.entries ?? null,
      generator: className === "Generate",
      live: count !== undefined,
    });
  }
  return rows.sort((a, b) => b.entries - a.entries);
}
