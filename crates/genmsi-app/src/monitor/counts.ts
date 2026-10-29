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
 *   came in by its entry door: the block the internal link of that door
 *   points to is where the flow enters the container, and its total of
 *   entries is the total that went through the container. A container
 *   without an entry door —or with a door that carries no link— has
 *   passed nothing, and its total is zero.
 */
export function containerCountsOf(
  model: CountedModel,
  counts: ReadonlyMap<number, LiveCounts>,
): Map<number, LiveCounts> {
  const byId = new Map<number, ModelBlock>();
  const children = new Map<number, ModelBlock[]>();
  for (const block of model.blocks) {
    byId.set(block.id, block);
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
      figureOfContainer(block, model, counts, byId, children, memo, new Set()),
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
 * The count is the one of the moves that happened **up to** `until`, so it
 * grows with the replay and says what happened so far.
 *
 * @param model the model the containers belong to
 * @param transitions the moves of the run, in the order they happened
 * @param until the time of the run the count stops at
 */
export function crossingsOf(
  model: CountedModel,
  transitions: readonly { from: number; to: number; time: number }[],
  until: number,
): Map<number, number> {
  const owner = new Map<number, number>();
  for (const block of model.blocks) {
    owner.set(block.id, block.container);
  }
  const crossings = new Map<number, number>();
  for (const move of transitions) {
    if (move.time > until) {
      continue;
    }
    const from = chainOf(owner, move.from);
    const to = chainOf(owner, move.to);
    // The move enters the containers the destination is inside of and the
    // origin is not: those are the borders it crosses coming in.
    for (const container of to) {
      if (!from.has(container)) {
        crossings.set(container, (crossings.get(container) ?? 0) + 1);
      }
    }
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
 */
function figureOfContainer(
  container: ModelBlock,
  model: CountedModel,
  counts: ReadonlyMap<number, LiveCounts>,
  byId: ReadonlyMap<number, ModelBlock>,
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
        ? figureOfContainer(child, model, counts, byId, children, memo, visiting).now
        : counts.get(child.id)?.now ?? 0;
  }
  // The entities that passed: the total of the block the entry door leads
  // to.
  let ever = 0;
  const entry = entryTargetOf(container, model, byId);
  if (entry !== undefined) {
    ever =
      entry.kind === "C"
        ? figureOfContainer(entry, model, counts, byId, children, memo, visiting).ever
        : counts.get(entry.id)?.ever ?? 0;
  }
  visiting.delete(container.id);
  const figure: LiveCounts = { now, ever };
  memo.set(container.id, figure);
  return figure;
}

/**
 * The block the flow enters the container by.
 *
 * The entry door of a container is one of its input ports, and the link
 * that leaves that port inside the container is the one that says which
 * block the flow enters by. A container with several entry doors takes
 * the first one that carries a link.
 */
function entryTargetOf(
  container: ModelBlock,
  model: CountedModel,
  byId: ReadonlyMap<number, ModelBlock>,
): ModelBlock | undefined {
  for (const port of container.inputs) {
    const link = model.edges.find(
      (edge) => edge.from_module === container.id && edge.from_ioid === port.ioid,
    );
    if (link === undefined) {
      continue;
    }
    const target = byId.get(link.to_module);
    if (target !== undefined) {
      return target;
    }
  }
  return undefined;
}
