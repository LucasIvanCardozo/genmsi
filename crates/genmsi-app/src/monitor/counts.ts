/**
 * The figures every block of the model shows.
 *
 * A block of the model is one or several statements of the listing: a facility
 * use is five (a queue, a seize, a depart, an advance and a release), and all
 * of them count the transactions that went through the block. Adding the rows
 * of a block up would count one transaction once per statement — five times
 * for a facility use — so the figure of a block is the one of **the row that
 * counts it**, and the engine says which row that is.
 *
 * A generator has no figure: no transaction stays inside one, and the
 * generator itself is not an instance that passed through it. The generator
 * is told apart by the **block** it belongs to, not by the statement: the
 * clock is a generator of ticks, and its figure is the one of the block.
 */
import type { LiveCounts } from "./live";
import type { ReportBlockLike } from "./replay";

/** The counters the replay keeps, per line of the program. */
export interface CountState {
  /** The transactions inside every program block. */
  blocks: Record<number, number>;
  /** The entries every program block has had, since the run started. */
  totals: Record<number, number>;
}

/**
 * The figures of every block of the model, by the node of the block.
 *
 * The block of the model is the one the row belongs to (`node`); the figure
 * is the one of that row in the state of the replay.
 */
export function countsOf(
  blocks: ReportBlockLike[],
  state: CountState,
  generators: ReadonlySet<number> = new Set(),
): Map<number, LiveCounts> {
  const byNode = new Map<number, LiveCounts>();
  for (const block of blocks) {
    if (!block.counts_block || generators.has(block.node)) {
      continue;
    }
    byNode.set(block.node, {
      now: state.blocks[block.line - 1] ?? 0,
      ever: state.totals[block.line - 1] ?? 0,
    });
  }
  return byNode;
}
