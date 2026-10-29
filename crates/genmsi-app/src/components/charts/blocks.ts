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
 */
import { countsOf } from "../../monitor/counts.ts";
import type { LiveCounts } from "../../monitor/live.ts";
import type { ReplayState } from "../../monitor/replay.ts";
import type { ModelBody, ReportDocument } from "../../types";

/** The figures of one block of the model. */
export interface BlockRow {
  /** The name of the block, as the model names it. */
  name: string;
  /** The class of the block. */
  class: string;
  /** How many transactions went through it. */
  entries: number;
  /** How many transactions are inside it right now. */
  current: number;
  /** How many times a transaction could not enter it. */
  retries: number;
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
 * The figures of every block of the model.
 *
 * The transactions that went through a block are the ones the row that
 * counts it holds; the retries are the ones of all of its rows.
 */
export function blockRows(
  report: ReportDocument,
  model: ModelBody | null,
  state: ReplayState | null,
): BlockRow[] {
  const counts: Map<number, LiveCounts> =
    state === null
      ? new Map()
      : countsOf(report.blocks, state);

  // The retries of every program row, and the rows that count their block.
  const retries = new Map<number, number>();
  const counting = new Set<number>();
  for (const row of report.blocks) {
    const line = row.line - 1;
    const live = state?.retries[line] ?? 0;
    const published = row.retries ?? 0;
    retries.set(row.node, (retries.get(row.node) ?? 0) + (state === null ? published : live));
    if (row.counts_block === true) {
      counting.add(row.node);
    }
  }

  const nodes = new Set<number>([
    ...report.blocks.map((row) => row.node),
    ...counts.keys(),
  ]);
  const rows: BlockRow[] = [];
  for (const node of nodes) {
    // Las cifras son las del instante: hasta ahora el bloque no vio
    // ninguna transacción si el reloj todavía no llegó a él.
    const count = counts.get(node);
    const entries = count?.ever ?? 0;
    const current = count?.now ?? 0;
    rows.push({
      name: nameOf(model, node, `bloque ${node}`),
      class: classOf(model, node),
      entries,
      current,
      retries: retries.get(node) ?? 0,
      live: count !== undefined,
    });
  }
  return rows.sort((a, b) => b.entries - a.entries);
}
