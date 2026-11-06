/**
 * The cache of the series the charts draw.
 *
 * The rows of a chart are a **pure function** of `(points, entities, until)`:
 * only the *prefix* of the answer depends on the clock. The charts of the
 * monitor are repainted sixty times per second while the replay runs, and
 * rebuilding the series from the second zero of the run on every repaint is
 * what makes the painting expensive: the rebuild of one series of a trace of
 * ten thousand records costs more than a millisecond, which is more than the
 * whole simulation costs per frame.
 *
 * The cache builds the rows **once** and extends them: a repaint costs what
 * changed since the previous one. The three families the charts rebuild
 * today are covered: the rows of the series (`rows`), the intervals of a
 * facility (`segments`) and the distribution of the waits (`waits`).
 *
 * The cache is a **memo of a pure function**, not a piece of state of the
 * application:
 *
 * - it holds no global state and no singleton: its lifetime is decided by
 *   whoever holds it (the index of the replay holds one per run, so a new
 *   run is a new cache and needs no invalidation);
 * - calling it during a render is safe: a discarded render or a StrictMode
 *   double render asks the same question and gets the same answer, and the
 *   frontier of the cache only ever grows;
 * - it keeps the **identity** of what did not change: the same instant
 *   gives the same array, and an array whose content changed never keeps
 *   its identity. That is the discipline of the live layer (`reuseFigures`,
 *   `reuseCounts`): a component that reads an unchanged object is not
 *   repainted, and a chart that kept the identity of a changed array would
 *   show a frozen area that lies about the instant.
 *
 * The reference implementations of the series (`seriesFrom`, `segmentsOf`,
 * `histogram`, `cumulative`) stay where they are: they are the definition
 * of the answers, and the tests pin that the cache and they agree.
 */

import { countAt, cumulative, histogram } from "./series.ts";
import type { Bin, Segment, SeriesPoint, SeriesRow } from "./series.ts";
import type { WaitSample } from "./replay.ts";

/** What the waits of a queue hold, up to the instant asked for. */
export interface WaitFigure {
  /** The bins of the histogram, in order. */
  bins: Bin[];
  /** The accumulated share of each bin, in the same order. */
  shares: number[];
  /** The mean of the waits. */
  mean: number;
  /** The longest wait. */
  longest: number;
}

/**
 * The points of an entity that has none.
 *
 * The same list for every entity that has none: the cache compares the
 * identity of the lists, and an entity without points must look the same
 * every time it is asked for.
 */
const NO_POINTS: readonly SeriesPoint[] = [];

/**
 * How many answers the cache remembers per selection.
 *
 * The reader scrubs the playhead, so the same instant is asked for again
 * after another one: a few answers are enough to give it the same array.
 * The memory of the answers is bounded because the clock of a long run
 * takes a value for every frame.
 */
const MEMO_SIZE = 8;

/** The rows of one selection: the entities chosen, built up to the frontier. */
interface RowSelection {
  /** The entities of the selection, in the order of the rows. */
  entities: readonly string[];
  /** The points of each entity, in the order of the entities. */
  lists: readonly (readonly SeriesPoint[])[];
  /** The instants of the changes of the entities, plus the instant zero. */
  instants: number[];
  /** The rows built so far, one per change instant. */
  rows: SeriesRow[];
  /** How many points of each entity the built rows already consumed. */
  cursors: number[];
  /** The value of each entity at the last row built. */
  values: (number | undefined)[];
  /** How many rows are built. */
  frontier: number;
  /** The answers already given, by the instant they answer. */
  answers: Map<number, SeriesRow[]>;
}

/** The intervals of one list of points, built once. */
interface Segmentation {
  /** The instants of the points, in order. */
  times: number[];
  /** How many intervals are closed after each point. */
  closed: number[];
  /** The start of the open interval after each point. */
  openStart: number[];
  /** Whether the open interval after each point is busy. */
  openBusy: boolean[];
  /** The intervals of the whole run. */
  segments: Segment[];
  /** The answers already given, by the instant they answer. */
  answers: Map<number, Segment[]>;
}

/** The waits of one list of samples, built once. */
interface Waiting {
  /** The instants of the departures, in order. */
  times: number[];
  /** The wait of each departure, in order. */
  waits: number[];
  /** The sum of the waits up to each departure. */
  sums: number[];
  /** The longest wait up to each departure. */
  longest: number[];
  /** The waits of the prefix the clock reached. */
  values: number[];
  /** How many departures the prefix holds. */
  frontier: number;
  /** The width the histogram was built with. */
  width: number;
  /** The bins of the histogram of the prefix. */
  bins: Bin[];
  /** The accumulated share of each bin. */
  shares: number[];
  /** How many departures the bins hold. */
  binned: number;
  /** The answers already given, by the instant they answer. */
  answers: Map<number, WaitFigure | null>;
}

/**
 * The cache of the series the charts draw.
 *
 * The cache lives by itself: whoever holds it decides how long it lives.
 */
export class SeriesCache {
  /** The rows of each selection, by the map of points and the entities. */
  private readonly selections = new WeakMap<
    ReadonlyMap<string, readonly SeriesPoint[]>,
    Map<string, RowSelection>
  >();

  /** The intervals of each list of points. */
  private readonly segmentations = new WeakMap<readonly SeriesPoint[], Segmentation>();

  /** The waits of each list of samples. */
  private readonly waiting = new WeakMap<readonly WaitSample[], Waiting>();

  /**
   * The rows of the selected entities, up to the instant asked for.
   *
   * The rows are the instants of the entities and the value each one had
   * there: the instant zero, the instants of the changes, and the instant
   * asked for. The last row is the instant of the reader, so the area of
   * the chart reaches the cursor; the rows before it are the ones that
   * changed, and they are built once and extended.
   *
   * @param points the points of every entity of the run
   * @param entities the entities of the chart, in the order of the rows
   * @param until the instant the reader is at
   */
  rows(
    points: ReadonlyMap<string, readonly SeriesPoint[]>,
    entities: readonly string[],
    until: number,
  ): SeriesRow[] {
    const selection = this.selectionOf(points, entities);
    const remembered = selection.answers.get(until);
    if (remembered !== undefined) {
      return remembered;
    }
    // The change instants at or before the clock: the rows of the answer
    // are the ones of the run that already happened.
    const count = countAt(selection.instants, until);
    // The series always starts at zero: the row of the instant zero is
    // built even when the reader is before it (or the clock is unknown).
    const zero = selection.instants.indexOf(0);
    const before = !(until >= 0);
    this.extend(selection, before ? Math.max(count, zero + 1) : count);
    const answer = selection.rows.slice(0, count);
    if (before && zero >= count) {
      answer.push(selection.rows[zero]);
    }
    if (Number.isFinite(until) && until > 0) {
      // The row of the instant of the reader is not a change instant: it is
      // built on demand, with the values of the last change before it.
      const isChange = count > 0 && selection.instants[count - 1] === until;
      if (!isChange) {
        const last = selection.rows[count - 1];
        const row: SeriesRow = { time: until };
        for (const name of selection.entities) {
          row[name] = last[name] ?? 0;
        }
        answer.push(row);
      }
    }
    remember(selection.answers, until, answer);
    return answer;
  }

  /**
   * The intervals of an entity's state, up to the instant asked for.
   *
   * The intervals are the gaps between the changes of the entity, and they
   * stop at the instant asked for: the chart shows what has happened so
   * far. A smaller instant gives the matching prefix, and never the
   * intervals that lie beyond it.
   *
   * @param points the changes of the entity
   * @param endTime the instant the reader is at
   */
  segments(points: readonly SeriesPoint[], endTime: number): Segment[] {
    const segmentation = this.segmentationOf(points);
    const remembered = segmentation.answers.get(endTime);
    if (remembered !== undefined) {
      return remembered;
    }
    const last = countAt(segmentation.times, endTime) - 1;
    const closed = last < 0 ? 0 : segmentation.closed[last];
    const start = last < 0 ? 0 : segmentation.openStart[last];
    const busy = last < 0 ? false : segmentation.openBusy[last];
    // The intervals already closed are the ones of the state of the entity:
    // they are shared with the segmentation and never built again.
    const answer = segmentation.segments.slice(0, closed);
    if (endTime > start) {
      answer.push({ start, end: endTime, busy });
    }
    remember(segmentation.answers, endTime, answer);
    return answer;
  }

  /**
   * The histogram of the waits of a queue, up to the instant asked for.
   *
   * The width of the intervals is a constant of the queue: it is chosen
   * once, from the whole trace, so that two instants of the same queue are
   * comparable. A width that changed under the reader would re-bin the
   * histogram while the run advances.
   *
   * @param samples the departures of the queue
   * @param width the width of the intervals of the histogram
   * @param until the instant the reader is at
   */
  waits(samples: readonly WaitSample[], width: number, until: number): WaitFigure | null {
    const state = this.waitingOf(samples, width);
    const remembered = state.answers.get(until);
    if (remembered !== undefined) {
      return remembered;
    }
    const count = countAt(state.times, until);
    if (count === 0) {
      // The queue has not let a transaction go yet: there is nothing to
      // draw.
      remember(state.answers, until, null);
      return null;
    }
    // The waits of the prefix: they are kept, and the prefix only grows
    // when the clock grows.
    if (count > state.frontier) {
      for (let index = state.frontier; index < count; index += 1) {
        state.values.push(state.waits[index]);
      }
    } else {
      state.values.length = count;
    }
    state.frontier = count;
    if (state.binned !== count) {
      state.bins = histogram(state.values, width);
      state.shares = cumulative(state.bins);
      state.binned = count;
    }
    const figure: WaitFigure = {
      bins: state.bins,
      shares: state.shares,
      mean: state.sums[count - 1] / count,
      longest: state.longest[count - 1],
    };
    remember(state.answers, until, figure);
    return figure;
  }

  /**
   * The rows of a selection, built once.
   *
   * The selection is the map of points and the entities: the map holds the
   * run, and the entities are the ones the chart shows. A map whose points
   * changed under the cache is another run, and its rows are built again.
   */
  private selectionOf(
    points: ReadonlyMap<string, readonly SeriesPoint[]>,
    entities: readonly string[],
  ): RowSelection {
    const key = JSON.stringify(entities);
    let byEntities = this.selections.get(points);
    if (byEntities === undefined) {
      byEntities = new Map<string, RowSelection>();
      this.selections.set(points, byEntities);
    }
    const lists = entities.map((name) => points.get(name) ?? NO_POINTS);
    const found = byEntities.get(key);
    if (found !== undefined && sameLists(found.lists, lists)) {
      return found;
    }
    const selection = buildSelection(entities, lists);
    byEntities.set(key, selection);
    return selection;
  }

  /**
   * Build the rows of the change instants that are missing.
   *
   * The rows are built once and kept: a row is never built again, and the
   * row objects are shared by the answers that hold them.
   */
  private extend(selection: RowSelection, target: number): void {
    while (selection.frontier < target) {
      const time = selection.instants[selection.frontier];
      for (let index = 0; index < selection.entities.length; index += 1) {
        const list = selection.lists[index];
        let cursor = selection.cursors[index];
        while (cursor < list.length && list[cursor].time <= time) {
          selection.values[index] = list[cursor].value;
          cursor += 1;
        }
        selection.cursors[index] = cursor;
      }
      const row: SeriesRow = { time };
      for (let index = 0; index < selection.entities.length; index += 1) {
        row[selection.entities[index]] = selection.values[index] ?? 0;
      }
      selection.rows.push(row);
      selection.frontier += 1;
    }
  }

  /** The intervals of a list of points, built once. */
  private segmentationOf(points: readonly SeriesPoint[]): Segmentation {
    const found = this.segmentations.get(points);
    if (found !== undefined) {
      return found;
    }
    const segmentation = buildSegmentation(points);
    this.segmentations.set(points, segmentation);
    return segmentation;
  }

  /** The waits of a list of samples, built once for the width asked for. */
  private waitingOf(samples: readonly WaitSample[], width: number): Waiting {
    let state = this.waiting.get(samples);
    if (state === undefined) {
      state = buildWaiting(samples);
      this.waiting.set(samples, state);
    }
    if (state.width !== width) {
      // The width of the intervals changed under the reader: the histogram
      // of the queue is built again, with the new width.
      state.width = width;
      state.values = [];
      state.frontier = 0;
      state.bins = [];
      state.shares = [];
      state.binned = -1;
      state.answers.clear();
    }
    return state;
  }
}

/**
 * Keep an answer, so that the same instant gives the same array.
 *
 * The memory of the answers is bounded: the clock of a long run takes a
 * value for every frame, and the answers of the frames nobody asks for
 * again are of no use.
 */
function remember<T>(answers: Map<number, T>, until: number, answer: T): void {
  answers.set(until, answer);
  while (answers.size > MEMO_SIZE) {
    const oldest = answers.keys().next().value;
    if (oldest === undefined) {
      break;
    }
    answers.delete(oldest);
  }
}

/** Whether two lists of points are the same lists. */
function sameLists(
  before: readonly (readonly SeriesPoint[])[],
  after: readonly (readonly SeriesPoint[])[],
): boolean {
  if (before.length !== after.length) {
    return false;
  }
  return before.every((list, index) => list === after[index]);
}

/** The rows of a selection, with the instants of its changes. */
function buildSelection(
  entities: readonly string[],
  lists: readonly (readonly SeriesPoint[])[],
): RowSelection {
  // The instants of the changes of the entities, plus the instant zero:
  // the series of a chart starts at zero, whatever the first change is.
  const instants = new Set<number>([0]);
  for (const list of lists) {
    for (const point of list) {
      instants.add(point.time);
    }
  }
  return {
    entities: [...entities],
    lists: [...lists],
    instants: [...instants].sort((a, b) => a - b),
    rows: [],
    cursors: entities.map(() => 0),
    values: entities.map(() => undefined),
    frontier: 0,
    answers: new Map<number, SeriesRow[]>(),
  };
}

/**
 * The intervals of a list of points, and what every prefix of them closes.
 *
 * The points arrive in the order of the run, but they are sorted anyway:
 * the reference sorts them, and the cache must answer the same.
 */
function buildSegmentation(points: readonly SeriesPoint[]): Segmentation {
  const sorted = [...points].sort((a, b) => a.time - b.time);
  const times: number[] = [];
  const closed: number[] = [];
  const openStart: number[] = [];
  const openBusy: boolean[] = [];
  const segments: Segment[] = [];
  let start = 0;
  let busy = false;
  for (const point of sorted) {
    if (point.time > start) {
      segments.push({ start, end: point.time, busy });
    }
    closed.push(segments.length);
    start = point.time;
    busy = point.value > 0;
    openStart.push(start);
    openBusy.push(busy);
    times.push(point.time);
  }
  return {
    times,
    closed,
    openStart,
    openBusy,
    segments,
    answers: new Map<number, Segment[]>(),
  };
}

/** The waits of a list of samples, and what every prefix of them holds. */
function buildWaiting(samples: readonly WaitSample[]): Waiting {
  const sorted = [...samples].sort((a, b) => a.time - b.time);
  const times: number[] = [];
  const waits: number[] = [];
  const sums: number[] = [];
  const longest: number[] = [];
  let sum = 0;
  let far = 0;
  for (const sample of sorted) {
    times.push(sample.time);
    waits.push(sample.wait);
    sum += sample.wait;
    far = sample.wait > far ? sample.wait : far;
    sums.push(sum);
    longest.push(far);
  }
  return {
    times,
    waits,
    sums,
    longest,
    values: [],
    frontier: 0,
    width: Number.NaN,
    bins: [],
    shares: [],
    binned: -1,
    answers: new Map<number, WaitFigure | null>(),
  };
}
