/**
 * The series the charts draw: the value of an entity over the time of the run.
 *
 * The series is the **exact** history of an entity, not a sample of it: a
 * queue that fills and empties between two samples of the trace would be
 * invisible in a sample, and a facility that is free for a moment would look
 * busy. The series is built once, when the index of the replay is built, by
 * folding the frames; the chart reads it.
 *
 * **The series always starts at zero.** The axis of a chart whose `x` is the
 * time of the run is the time of the run, from 0 to its end: the first
 * change of an entity may come much later, and a chart that started there
 * would hide the time before it. The first row of the series is the instant
 * zero, with the value the entity had when the run started.
 */

import { number } from "../format.ts";

/** One point of an entity's series: its value at one instant. */
export interface SeriesPoint {
  /** The simulated time. */
  time: number;
  /** The value of the entity (the content of a queue, the busy state of a facility). */
  value: number;
}

/** The rows of a chart: the instants of the entities and the value of each. */
export type SeriesRow = { time: number } & Record<string, number>;

/**
 * The rows of a chart, from the start of the run to the instant asked for.
 *
 * The rows are the union of the instants of the chosen entities, plus the
 * instant zero and the instant asked for: a chart shows the entity as it was
 * from the beginning of the run, and its area reaches the cursor.
 *
 * The value of an entity is the one it had at that instant: the last change
 * at or before it.
 */
export function seriesFrom(
  entities: string[],
  points: Map<string, SeriesPoint[]>,
  until = Number.POSITIVE_INFINITY,
): SeriesRow[] {
  const times = new Set<number>([0]);
  for (const name of entities) {
    for (const point of points.get(name) ?? []) {
      if (point.time <= until) {
        times.add(point.time);
      }
    }
  }
  if (Number.isFinite(until) && until > 0) {
    times.add(until);
  }
  const sorted = [...times].sort((a, b) => a - b);
  const cursors = new Map<string, number>();
  const values = new Map<string, number>();
  const rows: SeriesRow[] = [];
  for (const time of sorted) {
    const row: SeriesRow = { time };
    for (const name of entities) {
      const list = points.get(name) ?? [];
      let cursor = cursors.get(name) ?? 0;
      while (cursor < list.length && list[cursor].time <= time) {
        values.set(name, list[cursor].value);
        cursor += 1;
      }
      cursors.set(name, cursor);
      row[name] = values.get(name) ?? 0;
    }
    rows.push(row);
  }
  return rows;
}

/** One interval of an entity's state: the entity held it, or did not. */
export interface Segment {
  /** When the interval starts. */
  start: number;
  /** When the interval ends. */
  end: number;
  /** Whether the entity was busy during the interval. */
  busy: boolean;
}

/**
 * The intervals of an entity's state, from the start of the run to the
 * instant asked for.
 *
 * The intervals are the gaps between the changes of the entity: the state it
 * had from the start of the run is the first interval, and the state it had
 * at the end of the run is the last one.
 *
 * The intervals **stop at the instant asked for**: the chart shows what has
 * happened so far, and the reader sees the run advance as the clock moves.
 */
export function segmentsOf(points: SeriesPoint[], endTime: number): Segment[] {
  const sorted = [...points]
    .filter((point) => point.time <= endTime)
    .sort((a, b) => a.time - b.time);
  const segments: Segment[] = [];
  let start = 0;
  let busy = false;
  for (const point of sorted) {
    if (point.time > start) {
      segments.push({ start, end: point.time, busy });
    }
    start = point.time;
    busy = point.value > 0;
  }
  if (endTime > start) {
    segments.push({ start, end: endTime, busy });
  }
  return segments;
}

/** One interval of a histogram: the values it holds and how many there are. */
export interface Bin {
  /** The lower bound of the interval. */
  low: number;
  /** The upper bound of the interval. */
  high: number;
  /** How many values fell in the interval. */
  count: number;
}

/**
 * The histogram of a list of values.
 *
 * The interval a value falls in is the one it belongs to: the interval of
 * `width` that starts at `first + index * width` and holds the values from
 * its start (included) to its end (excluded). The bins are the ones the
 * values reach: a list of values that all fit in the first interval has one
 * bin.
 *
 * The histogram of a **table of the model** is not built here: the engine
 * publishes its frequencies, and the chart reads them as they are.
 */
export function histogram(values: number[], width: number, first = 0): Bin[] {
  if (width <= 0) {
    return [];
  }
  const counts: number[] = [];
  for (const value of values) {
    const index = Math.max(0, Math.floor((value - first) / width));
    // The bins between the first and this one are empty, and they are part
    // of the histogram: a hole in the array would be a hole in the chart.
    while (counts.length <= index) {
      counts.push(0);
    }
    counts[index] += 1;
  }
  return counts.map((count, index) => ({
    low: first + index * width,
    high: first + (index + 1) * width,
    count,
  }));
}

/**
 * The running share of the samples of a histogram, in percent.
 *
 * The share is of the samples the bins hold: the last value is a hundred.
 */
export function cumulative(bins: Bin[]): number[] {
  const total = bins.reduce((sum, bin) => sum + bin.count, 0);
  if (total <= 0) {
    return bins.map(() => 0);
  }
  let running = 0;
  return bins.map((bin) => {
    running += bin.count;
    return (running / total) * 100;
  });
}

/**
 * How many of the instants are at or before the given one.
 *
 * The instants are sorted, so the count is a search: the number of
 * transactions the run had created at an instant is the number of the
 * instants of their creation that are not after it.
 */
export function countAt(times: number[], until: number): number {
  let low = 0;
  let high = times.length - 1;
  let found = -1;
  while (low <= high) {
    const middle = Math.floor((low + high) / 2);
    if (times[middle] <= until) {
      found = middle;
      low = middle + 1;
    } else {
      high = middle - 1;
    }
  }
  return found + 1;
}

export interface HistogramRow {
  x: number;
  label: string;
  frequency: number;
  accumulated: number;
  overflowClass: boolean;
}

/** Compute the histogram rows and labels for table intervals. */
export function histogramRows(
  table: { first: number; width: number; intervals: number },
  frequencies: readonly number[],
): HistogramRow[] {
  const total = frequencies.reduce((sum, count) => sum + count, 0);
  let running = 0;
  return frequencies.map((count, index) => {
    running += count;
    const low = index === 0 ? 0 : table.first + (index - 1) * table.width;
    const high = table.first + index * table.width;
    const isOverflow = index === frequencies.length - 1 && table.intervals > 1;
    return {
      x: isOverflow ? low + table.width / 2 : (low + high) / 2,
      label: isOverflow ? `> ${number(low)}` : `${number(low)} - ${number(high)}`,
      frequency: count,
      accumulated: total > 0 ? (running / total) * 100 : 0,
      overflowClass: isOverflow,
    };
  });
}
