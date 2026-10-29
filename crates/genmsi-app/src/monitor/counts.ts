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
 *
 * A **container** has no figure of its own either: the engine compiles it
 * inline, so the report has no row for it. Its figures are computed from
 * the blocks it holds and from the link its entry door carries
 * (`containerCountsOf`), with the same rule: the figure of a block is the
 * one of the row that counts it, and the sum of the figures of the blocks
 * of a container counts every entity once, because an entity is in one
 * place only.
 */
import type { LiveCounts } from "./live";
import type { ReportBlockLike } from "./replay";
import type { ModelBlock, ModelEdge } from "../types";

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

/**
 * The model as the figures of the containers need it.
 *
 * The containers are not rows of the report — the engine compiles them
 * inline — so their figures are not read: they are computed from the blocks
 * they hold and from the link their entry door carries.
 */
export interface CountedModel {
  /** The blocks of the model, in the order of the model. */
  blocks: readonly ModelBlock[];
  /** The links of the model, in the order of the model. */
  edges: readonly ModelEdge[];
}

/**
 * The figures of every block of the model, the containers included.
 *
 * The figures of the blocks that are rows of the report are read from it,
 * and the figures of the containers are computed on top of them.
 *
 * @param blocks the rows of the report
 * @param state the counters of the replay
 * @param model the model the containers belong to
 * @param crossings the entities that entered each container
 * @param generators the blocks that have no figure of their own
 */
export function countsWithContainers(
  blocks: ReportBlockLike[],
  state: CountState,
  model: CountedModel,
  crossings: ReadonlyMap<number, number> = new Map(),
  generators: ReadonlySet<number> = new Set(),
): Map<number, LiveCounts> {
  const counts = countsOf(blocks, state, generators);
  for (const [id, figure] of containerCountsOf(model, counts)) {
    counts.set(id, figure);
  }
  // The entities that entered the container are the ones that crossed its
  // border: the count of the moves says it exactly, where the total of the
  // block the entry door leads to does not (a queue has no total of its own
  // in the replay).
  for (const [id, crossed] of crossings) {
    const figure = counts.get(id);
    if (figure !== undefined) {
      counts.set(id, { now: figure.now, ever: crossed });
    }
  }
  return counts;
}

/**
 * The figures of the containers of the model, by the id of the container.
 *
 * A container has no row of the report: the engine compiles it inline, so
 * its figures are computed.
 *
 * - The entities **inside** the container are the ones of the blocks it
 *   holds: an entity is in one place only, so the sum of the figures of
 *   its blocks is exact. The figure of a block is the one of the row that
 *   counts it (`countsOf`), never the sum of the rows of a block that
 *   expands into several, so the sum is made over the figures and not
 *   over the rows.
 * - The entities that **passed** through the container are the ones that
 *   crossed its border, and they come from the trace of the run
 *   (`crossingsUntil`): a container has as many entry doors as the reader
 *   gave it, and every one of them counts. A container nobody entered has
 *   passed nothing, and its total is zero.
 */
export function containerCountsOf(
  model: CountedModel,
  counts: ReadonlyMap<number, LiveCounts>,
): Map<number, LiveCounts> {
  const children = new Map<number, ModelBlock[]>();
  for (const block of model.blocks) {
    if (block.id === 0) {
      continue;
    }
    const siblings = children.get(block.container);
    if (siblings === undefined) {
      children.set(block.container, [block]);
    } else {
      siblings.push(block);
    }
  }
  const memo = new Map<number, LiveCounts>();
  const figures = new Map<number, LiveCounts>();
  for (const block of model.blocks) {
    if (block.kind !== "C") {
      continue;
    }
    figures.set(
      block.id,
      figureOfContainer(block, counts, children, memo, new Set()),
    );
  }
  return figures;
}

/**
 * The entities that entered every container of the model.
 *
 * A container is entered by a move that comes from outside and goes in: the
 * count of those moves is the number of entities that went through the
 * container. The moves of the run are the ones the trace records, and the
 * app has them already.
 *
 * The crossings are **events**, not counts: they are told once, when the run
 * is known, and counted later, up to the time the replay has reached. The
 * count grows with the replay and says what happened so far, and the count
 * of a run of a hundred thousand moves does not cost a hundred thousand
 * walks every frame.
 *
 * @param model the model the containers belong to
 * @param transitions the moves of the run, in the order they happened
 */
export interface Crossings {
  /** The time of each crossing, in the order they happened. */
  readonly times: readonly number[];
  /** The container each crossing entered, parallel to `times`. */
  readonly containers: readonly number[];
}

/**
 * The crossings of the borders of the model, as the events that made them.
 *
 * A move that comes from outside a container and goes in crosses its
 * border: that is an entity that went through it. A move that leaves is
 * not a crossing: the container counts the entities that entered it, and an
 * entity that goes in and out went through it once.
 *
 * @param model the model the containers belong to
 * @param transitions the moves of the run, in the order they happened
 */
export function crossingsEvents(
  model: CountedModel,
  transitions: readonly { from: number; to: number; time: number }[],
): Crossings {
  const owner = new Map<number, number>();
  for (const block of model.blocks) {
    owner.set(block.id, block.container);
  }
  const times: number[] = [];
  const containers: number[] = [];
  for (const move of transitions) {
    const from = chainOf(owner, move.from);
    const to = chainOf(owner, move.to);
    // The move enters the containers the destination is inside of and the
    // origin is not: those are the borders it crosses coming in.
    for (const container of to) {
      if (!from.has(container)) {
        times.push(move.time);
        containers.push(container);
      }
    }
  }
  return { times, containers };
}

/**
 * The entities that entered each container, up to a time of the run.
 *
 * The count is the one of the crossings that happened **up to** `until`, so
 * it grows with the replay and says what happened so far.
 *
 * @param events the crossings of the model
 * @param until the time of the run the count stops at
 */
export function crossingsUntil(
  events: Crossings,
  until: number,
): Map<number, number> {
  const crossings = new Map<number, number>();
  // The crossings happen in the order of the moves, so the ones that are
  // already made are the first ones: a binary search finds where the count
  // stops.
  let low = 0;
  let high = events.times.length;
  while (low < high) {
    const middle = (low + high) >>> 1;
    if (events.times[middle] <= until) {
      low = middle + 1;
    } else {
      high = middle;
    }
  }
  for (let index = 0; index < low; index += 1) {
    const container = events.containers[index];
    crossings.set(container, (crossings.get(container) ?? 0) + 1);
  }
  return crossings;
}

/**
 * The containers a block is inside of, from the block itself up.
 *
 * The root of the model is not a container of the picture: the walk stops
 * before it.
 */
function chainOf(owner: ReadonlyMap<number, number>, block: number): Set<number> {
  const chain = new Set<number>();
  let current = owner.get(block);
  while (current !== undefined && current !== 0) {
    if (chain.has(current)) {
      break;
    }
    chain.add(current);
    current = owner.get(current);
  }
  return chain;
}

/**
 * The figures of one container.
 *
 * The walk of the containers can go deep —a container inside a container—,
 * so the figures already computed are kept and the containers that reach
 * themselves (a cycle of doors) stop the walk instead of looping for ever.
 *
 * Only the entities **inside** are computed here: the entities that passed
 * through the container are the ones that crossed its border, and they come
 * from the trace of the run (`crossingsUntil`). A container has as many
 * entry doors as the reader gave it, and every one of them counts.
 */
function figureOfContainer(
  container: ModelBlock,
  counts: ReadonlyMap<number, LiveCounts>,
  children: ReadonlyMap<number, ModelBlock[]>,
  memo: Map<number, LiveCounts>,
  visiting: Set<number>,
): LiveCounts {
  const known = memo.get(container.id);
  if (known !== undefined) {
    return known;
  }
  if (visiting.has(container.id)) {
    return { now: 0, ever: 0 };
  }
  visiting.add(container.id);
  // The entities inside: the sum of the figures of the blocks the
  // container holds.
  let now = 0;
  for (const child of children.get(container.id) ?? []) {
    now +=
      child.kind === "C"
        ? figureOfContainer(child, counts, children, memo, visiting).now
        : counts.get(child.id)?.now ?? 0;
  }
  visiting.delete(container.id);
  const figure: LiveCounts = { now, ever: 0 };
  memo.set(container.id, figure);
  return figure;
}
