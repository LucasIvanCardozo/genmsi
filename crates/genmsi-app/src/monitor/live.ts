/**
 * The live layer: what the picture shows at this instant.
 *
 * The replay runs at sixty frames a second and the picture has to follow it.
 * Putting that in the state of React would redraw the whole graph sixty times
 * a second, so the live layer lives in its own store and the parts of the
 * picture that do not change do not hear about it.
 *
 * Five things are live:
 *
 * - the **clock** of the replay, for the chronometer and the slider;
 * - the **counts** of every block (how many transactions are inside it now,
 *   and how many have gone through it), for the blocks themselves;
 * - the **balls** on every link, for the links themselves;
 * - the **figures** of the entities of the run (the content of the queues,
 *   the occupancy of the facilities, the units of the storages, ...), which
 *   the figure of every block is read from;
 * - the **figure** every block shows, when it shows the figure of the
 *   entity it operates on.
 *
 * Each of them is published by identity: a value that did not change keeps
 * the object it had, so the component that shows it does not redraw. The
 * clock changes on every frame, the counts change when the run reaches a new
 * record, the balls change when a transaction is on its way, and the figures
 * change when the run moves an entity.
 */
import { create } from "zustand";
import { sameFigure, type BlockFigure } from "./figures.ts";
import { emptyFigures, type Figures } from "./stats.ts";

/** What a block shows of the run. */
export interface LiveCounts {
  /** The transactions inside the block right now. */
  now: number;
  /** The transactions that have gone through it since the run started. */
  ever: number;
}

/** One ball on a link: a transaction on its way to the next block. */
export interface LiveBall {
  /** The transaction that travels, to tell one ball from another. */
  xact: number;
  /** How far along the link it is, from 0 to 1. */
  progress: number;
}

/** What the picture shows at this instant. */
export interface LivePicture {
  /** The time the run has reached, in seconds of the run. */
  clock: number;
  /** The counts of every block, by block id. */
  counts: Map<number, LiveCounts>;
  /** The balls of every link, by link id. */
  balls: Map<string, LiveBall[]>;
  /**
   * The figures of the entities of the run, as the replay rebuilds them.
   *
   * They are optional in a picture: a caller that only moves the clock
   * does not have to carry them, and the ones the store holds are kept.
   */
  figures?: Figures;
  /** The figure every block shows, by block id. */
  blockFigures?: Map<number, BlockFigure>;
}

export interface LiveState extends LivePicture {
  /** The figures of the entities of the run. */
  figures: Figures;
  /** The figure every block shows, by block id. */
  blockFigures: Map<number, BlockFigure>;
  /** Publish what the picture shows right now. */
  publish: (picture: LivePicture) => void;
  /**
   * Put the clock at an instant, without touching the rest of the picture.
   *
   * The header's slider moves the run by hand: it sets the clock to the
   * instant the reader picked and leaves the counts and the balls as they
   * are, because the window is the one that publishes them.
   */
  setClock: (clock: number) => void;
  /** Forget the picture: a new run, or a model that moved. */
  reset: () => void;
}

const EMPTY: LivePicture = {
  clock: 0,
  counts: new Map(),
  balls: new Map(),
  figures: emptyFigures(),
  blockFigures: new Map(),
};

export const useLive = create<LiveState>((set) => ({
  // The state always holds the figures; a picture may come without them (the
  // replay has not built them yet), so the initial state is built here and
  // not spread from the picture.
  clock: 0,
  counts: new Map(),
  balls: new Map(),
  figures: emptyFigures(),
  blockFigures: new Map(),
  publish: (picture) =>
    set((state) => {
      // The clock always moves, so the picture is never the same.
      const counts = reuseCounts(state.counts, picture.counts);
      const balls = reuseBalls(state.balls, picture.balls);
      const figures = reuseFigures(state.figures, picture.figures);
      const blockFigures = reuseBlockFigures(state.blockFigures, picture.blockFigures);
      return { clock: picture.clock, counts, balls, figures, blockFigures };
    }),
  setClock: (clock) => set({ clock }),
  reset: () =>
    set({
      ...EMPTY,
      counts: new Map(),
      balls: new Map(),
      figures: emptyFigures(),
      blockFigures: new Map(),
    }),
}));

/**
 * The counts, keeping the identity of the ones that did not change.
 *
 * A block's counts change when the run reaches a record of that block, and
 * not on every frame. Keeping the object of the blocks that did not move is
 * what keeps the graph from redrawing when the clock ticks.
 */
function reuseCounts(
  before: Map<number, LiveCounts>,
  after: Map<number, LiveCounts>,
): Map<number, LiveCounts> {
  let changed = false;
  const counts = new Map(before);
  for (const [id, value] of after) {
    const previous = counts.get(id);
    if (previous === undefined || previous.now !== value.now || previous.ever !== value.ever) {
      counts.set(id, value);
      changed = true;
    }
  }
  if (counts.size !== after.size) {
    // A block that the run left behind (a new model, a new run) goes away.
    for (const id of counts.keys()) {
      if (!after.has(id)) {
        counts.delete(id);
        changed = true;
      }
    }
  }
  return changed ? counts : before;
}

/**
 * The balls, keeping the identity of the links that have none.
 *
 * A link without a ball keeps no entry at all, so the link that draws
 * nothing is not redrawn when another link's ball moves.
 */
function reuseBalls(
  before: Map<string, LiveBall[]>,
  after: Map<string, LiveBall[]>,
): Map<string, LiveBall[]> {
  let changed = false;
  const balls = new Map(before);
  for (const [id, value] of after) {
    const previous = balls.get(id);
    if (previous === undefined || previous.length !== value.length) {
      balls.set(id, value);
      changed = true;
      continue;
    }
    // The same number of balls: they moved, so the array is a new one.
    let moved = false;
    for (let index = 0; index < value.length; index += 1) {
      const one = value[index];
      const other = previous[index];
      if (one.xact !== other.xact || one.progress !== other.progress) {
        moved = true;
        break;
      }
    }
    if (moved) {
      balls.set(id, value);
      changed = true;
    }
  }
  for (const id of balls.keys()) {
    if (!after.has(id)) {
      balls.delete(id);
      changed = true;
    }
  }
  return changed ? balls : before;
}

/**
 * The figures of the entities of the run.
 *
 * The figures of the entities that did not move keep their identity, so a
 * component that reads one of them is not redrawn when another entity
 * moves.
 */
export function useFigures(): Figures {
  return useLive((state) => state.figures);
}

/**
 * The figure a block shows, or `undefined` when the block has none.
 *
 * The figure of a block that did not change keeps its object, so the block
 * is not redrawn when the clock ticks.
 */
export function useBlockFigure(id: number): BlockFigure | undefined {
  return useLive((state) => state.blockFigures.get(id));
}

/**
 * Whether the run has published anything.
 *
 * Before the run, the canvas shows the shapes alone: there is nothing to
 * say about the value of a block yet, and the dash of an empty figure
 * would say "no value" where the truth is "the run has not started".
 */
export function useRunHasPublished(): boolean {
  return useLive((state) => state.clock > 0);
}

/**
 * The figures of the entities, keeping the identity of the ones that did
 * not change.
 *
 * The replay rebuilds the figures of the run by copying what moved: the
 * entities that did not move keep their object, and so do the groups of
 * entities that did not move. The figures of the run keep their identity
 * when nothing moved at all.
 */
function reuseFigures(before: Figures, after: Figures | undefined): Figures {
  if (after === undefined) {
    return before;
  }
  const same =
    before.queues === after.queues &&
    before.facilities === after.facilities &&
    before.storages === after.storages &&
    before.chains === after.chains &&
    before.tables === after.tables &&
    before.logic === after.logic &&
    before.values === after.values;
  return same ? before : after;
}

/**
 * The figures of the blocks, keeping the identity of the ones that did not
 * change.
 *
 * A block whose figure did not change keeps the very same object: that is
 * what keeps the canvas from redrawing the blocks that the run did not
 * touch.
 */
function reuseBlockFigures(
  before: Map<number, BlockFigure>,
  after: Map<number, BlockFigure> | undefined,
): Map<number, BlockFigure> {
  if (after === undefined) {
    return before;
  }
  let changed = false;
  const figures = new Map(before);
  for (const [id, value] of after) {
    const previous = figures.get(id);
    if (previous === undefined || !sameFigure(previous, value)) {
      figures.set(id, value);
      changed = true;
    }
  }
  if (figures.size !== after.size) {
    // A block that the run left behind (a new model, a new run) goes away.
    for (const id of figures.keys()) {
      if (!after.has(id)) {
        figures.delete(id);
        changed = true;
      }
    }
  }
  return changed ? figures : before;
}

/** The counts of one block, or `undefined` when the block has none. */
export function useBlockCounts(id: number): LiveCounts | undefined {
  return useLive((state) => state.counts.get(id));
}

/** The balls of one link, or an empty list when the link has none. */
export function useEdgeBalls(id: string): LiveBall[] {
  return useLive((state) => state.balls.get(id)) ?? EMPTY_BALLS;
}

/** The empty list, so the links without balls share one. */
const EMPTY_BALLS: LiveBall[] = [];

// The store is visible from the browser's console while the app is being
// developed: the picture it carries is the one the reader sees.
if (import.meta.env?.DEV) {
  (globalThis as { __live?: typeof useLive }).__live = useLive;
}
