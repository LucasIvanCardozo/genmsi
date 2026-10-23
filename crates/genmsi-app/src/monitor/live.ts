/**
 * The live layer: what the picture shows at this instant.
 *
 * The replay runs at sixty frames a second and the picture has to follow it.
 * Putting that in the state of React would redraw the whole graph sixty times
 * a second, so the live layer lives in its own store and the parts of the
 * picture that do not change do not hear about it.
 *
 * Three things are live:
 *
 * - the **clock** of the replay, for the chronometer and the slider;
 * - the **counts** of every block (how many transactions are inside it now,
 *   and how many have gone through it), for the blocks themselves;
 * - the **balls** on every link, for the links themselves.
 *
 * Each of them is published by identity: a value that did not change keeps
 * the object it had, so the component that shows it does not redraw. The
 * clock changes on every frame, the counts change when the run reaches a new
 * record, and the balls change when a transaction is on its way.
 */
import { create } from "zustand";

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
}

export interface LiveState extends LivePicture {
  /** Publish what the picture shows right now. */
  publish: (picture: LivePicture) => void;
  /** Forget the picture: a new run, or a model that moved. */
  reset: () => void;
}

const EMPTY: LivePicture = {
  clock: 0,
  counts: new Map(),
  balls: new Map(),
};

export const useLive = create<LiveState>((set) => ({
  ...EMPTY,
  publish: (picture) =>
    set((state) => {
      // The clock always moves, so the picture is never the same.
      const counts = reuseCounts(state.counts, picture.counts);
      const balls = reuseBalls(state.balls, picture.balls);
      return { clock: picture.clock, counts, balls };
    }),
  reset: () => set({ ...EMPTY, counts: new Map(), balls: new Map() }),
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
