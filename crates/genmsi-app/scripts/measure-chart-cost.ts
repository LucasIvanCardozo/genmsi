// Measure the cost of one paint of the charts section.
//
// The charts section is painted sixty times per second while the replay
// runs, and every paint rebuilds the three families the charts draw from the
// start of the run: the series of the queues, the segments of the facilities
// and the distribution of the waits. The rebuild walks the whole history of
// the run, so a paint costs what the run did so far and not what changed
// since the previous paint.
//
// This script measures that cost with the engine's own data (the model, the
// report and the trace of a real run) and compares it with the cost of the
// cache of the series (`src/monitor/seriesCache.ts`), which the index of the
// replay carries as `index.series`. The two paths are compared row by row:
// when they disagree, the script fails.
//
//     node crates/genmsi-app/scripts/measure-chart-cost.ts
//
// The binaries it reads are the debug ones: build them first
// (`cargo build -p genmsi-cli -p genmsi-app-engine`).
//
// How the cost is measured: the sweep of clocks is painted several times
// and the **least** measurement of every paint is kept. The machine that
// runs the script is shared, and the least disturbed measurement is the
// closest to the cost of the work itself. The cache is measured with a
// fresh instance on every pass, so every pass pays the same cold start.
import { deepStrictEqual } from "node:assert";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildIndex, type ReplayIndex, type WaitSample } from "../src/monitor/replay.ts";
import {
  cumulative,
  histogram,
  segmentsOf,
  seriesFrom,
  waitWidth,
  type Bin,
  type Segment,
  type SeriesPoint,
  type SeriesRow,
} from "../src/monitor/series.ts";

const repoRoot = fileURLToPath(new URL("../../..", import.meta.url));
const cliBin = `${repoRoot}target/debug/genmsi`;
const facadeBin = `${repoRoot}target/debug/genmsi-app-engine`;
const model = "fixtures/models/TP4__supermercado.gmdl";

/** How many paints per second the charts section does by default. */
const PAINTS_PER_SECOND = 60;

/** How many clocks the sweep spreads over the length of the run. */
const SPREAD_CLOCKS = 60;

/** How many consecutive clocks of the end of the run the sweep paints. */
const TAIL_CLOCKS = 30;

/** How many times the whole sweep is painted, to get the least disturbed cost. */
const PASSES = 5;

/** How many times the rebuild of one clock is repeated, for the same reason. */
const REPEATS = 3;

/**
 * The module of the cache of the series.
 *
 * It lands in its own unit, so the script cannot import it statically: a
 * static import of a module that is not there yet breaks the whole script,
 * and the "before" measurement is worth having on its own.
 */
const CACHE_MODULE = "../src/monitor/seriesCache.ts";

/** The figure of the waits of a queue, as the cache returns it. */
interface WaitFigure {
  /** The intervals of the histogram and how many samples fell in each. */
  bins: Bin[];
  /** The running share of the samples of the histogram, in percent. */
  shares: number[];
  /** The mean wait of the samples that left the queue. */
  mean: number;
  /** The longest wait of the samples that left the queue. */
  longest: number;
}

/** The constructor of the cache of the series. */
type CacheConstructor = new () => SeriesCacheLike;

/**
 * The cache of the series, as this script uses it.
 *
 * It is declared here and not imported so that the script works with the
 * cache and without it: the module may not be there yet.
 */
interface SeriesCacheLike {
  rows(
    points: ReadonlyMap<string, readonly SeriesPoint[]>,
    entities: readonly string[],
    until: number,
  ): SeriesRow[];
  segments(points: readonly SeriesPoint[], endTime: number): Segment[];
  waits(samples: readonly WaitSample[], width: number, until: number): WaitFigure | null;
}

function bytes(bin: string, args: string[]): unknown {
  return JSON.parse(
    execFileSync(bin, args, { cwd: repoRoot, maxBuffer: 512 * 1024 * 1024 }).toString("utf8"),
  );
}

/**
 * The longest wait of a list of waits.
 *
 * The chart of the waits computes it the same way (`WaitDistribution.tsx`),
 * and the width of its intervals comes from it.
 */
function longestOf(waits: number[]): number {
  let longest = 0;
  for (const wait of waits) {
    if (wait > longest) {
      longest = wait;
    }
  }
  return longest;
}

/**
 * The waits of a queue at an instant, as the chart computes them.
 *
 * This is the reference of the waits family: the samples that left the queue
 * up to the instant, the histogram of their waits with the width chosen once
 * for the queue, the running share of the samples and the mean and the
 * longest wait of the samples.
 */
function referenceWaits(
  samples: readonly WaitSample[],
  width: number,
  clock: number,
): WaitFigure | null {
  const waits: number[] = [];
  for (const sample of samples) {
    if (sample.time <= clock) {
      waits.push(sample.wait);
    }
  }
  if (waits.length === 0) {
    return null;
  }
  const bins = histogram(waits, width);
  const shares = cumulative(bins);
  const mean = waits.reduce((sum, wait) => sum + wait, 0) / waits.length;
  const longest = longestOf(waits);
  return { bins, shares, mean, longest };
}

/** Whether two values are the same value, contents and all. */
function sameValue(left: unknown, right: unknown): boolean {
  try {
    // The comparison of the house for the plain data: it compares the keys
    // of the objects and not their order, and it compares the numbers by
    // their value.
    deepStrictEqual(left, right);
    return true;
  } catch {
    return false;
  }
}

/**
 * Whether two numbers are the same number.
 *
 * The mean of the waits is a sum divided by a count: the order of the sum
 * may differ between the two paths, and a hair of tolerance keeps a
 * rounding apart from a mistake.
 */
function sameNumber(left: number, right: number): boolean {
  const scale = Math.max(1, Math.abs(left), Math.abs(right));
  return Math.abs(left - right) <= scale * 1e-9;
}

/**
 * The figure of the waits of the reference and the figure of the cache.
 *
 * Returns the reason why they differ, or null when they are the same.
 */
function sameFigure(reference: WaitFigure | null, cached: WaitFigure | null): string | null {
  if (reference === null || cached === null) {
    if (reference === cached) {
      return null;
    }
    return `the reference has ${reference === null ? "no" : "a"} figure and the cache has ${
      cached === null ? "no" : "a"
    } one`;
  }
  if (!sameValue(reference.bins, cached.bins)) {
    return "the bins of the histogram differ";
  }
  if (!sameValue(reference.shares, cached.shares)) {
    return "the shares of the histogram differ";
  }
  if (reference.longest !== cached.longest) {
    return `the longest wait differs (${reference.longest} against ${cached.longest})`;
  }
  if (!sameNumber(reference.mean, cached.mean)) {
    return `the mean wait differs (${reference.mean} against ${cached.mean})`;
  }
  return null;
}

/**
 * The clocks of the sweep.
 *
 * Two groups: clocks spread over the length of the run, so the cost of a
 * paint is seen while the run grows, and the last clocks of the run, one
 * after the other, which is the worst case of a rebuild: every one of them
 * rebuilds the whole history of the run.
 */
function sweepOf(index: ReplayIndex): number[] {
  const start = index.startTime;
  const span = Math.max(index.endTime - start, 0);
  const clocks: number[] = [];
  for (let step = 0; step <= SPREAD_CLOCKS; step += 1) {
    clocks.push(start + (span * step) / SPREAD_CLOCKS);
  }
  const instants = [...new Set(index.frames.map((frame) => frame.time))];
  clocks.push(...instants.slice(-TAIL_CLOCKS));
  // The sweep is ascending: the cache extends its series forward, and a
  // clock that goes back would ask it to rebuild what it already had.
  return [...new Set(clocks)].sort((left, right) => left - right);
}

/**
 * The constructor of the cache, when the module of the cache is there.
 *
 * The cache is a pure object: a fresh instance measures the same work as
 * the one the index carries, and it lets every pass of the sweep pay the
 * same cold start.
 */
async function cacheConstructor(): Promise<CacheConstructor | undefined> {
  let module: { SeriesCache?: new () => SeriesCacheLike };
  try {
    // The name of the module is a variable: a literal that does not exist
    // yet would break the compilation of the script.
    module = (await import(CACHE_MODULE)) as { SeriesCache?: new () => SeriesCacheLike };
  } catch {
    return undefined;
  }
  if (module.SeriesCache === undefined) {
    return undefined;
  }
  try {
    // The cache may want something the script does not know yet.
    new module.SeriesCache();
  } catch {
    return undefined;
  }
  return module.SeriesCache;
}

/** The milliseconds of one paint, family by family, over the sweep. */
interface Costs {
  /** The series of the queues. */
  rows: number[];
  /** The segments of the facilities. */
  segments: number[];
  /** The distribution of the waits. */
  waits: number[];
  /** The whole paint: the three families together. */
  total: number[];
}

function emptyCosts(): Costs {
  return { rows: [], segments: [], waits: [], total: [] };
}

/** The mean of a list of milliseconds. */
function mean(samples: number[]): number {
  if (samples.length === 0) {
    return 0;
  }
  let total = 0;
  for (const sample of samples) {
    total += sample;
  }
  return total / samples.length;
}

/** The median of a list of milliseconds. */
function median(samples: number[]): number {
  if (samples.length === 0) {
    return 0;
  }
  const sorted = [...samples].sort((left, right) => left - right);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1
    ? sorted[middle]
    : (sorted[middle - 1] + sorted[middle]) / 2;
}

/** The biggest of a list of milliseconds. */
function worst(samples: number[]): number {
  let biggest = 0;
  for (const sample of samples) {
    if (sample > biggest) {
      biggest = sample;
    }
  }
  return biggest;
}

/** The printed line of a family and a path. */
function line(family: string, path: string, samples: number[]): string {
  const perPaint = mean(samples);
  // The share of a CPU-second the paints of a second take: a paint of a
  // millisecond at sixty paints per second is six percent of a core.
  const share = ((perPaint * PAINTS_PER_SECOND) / 1000) * 100;
  return (
    `${family.padEnd(9)} ${path.padEnd(7)}` +
    ` ${perPaint.toFixed(4).padStart(10)} ms` +
    ` ${median(samples).toFixed(4).padStart(10)} ms` +
    ` ${worst(samples).toFixed(4).padStart(10)} ms` +
    ` ${share.toFixed(1).padStart(8)} %`
  );
}

const modelDocument = bytes(facadeBin, ["model", model]);
const report = bytes(cliBin, ["simulate", model, "--json", "--seed", "1"]);
const trace = bytes(cliBin, ["trace", model, "--json", "--limit", "10000", "--seed", "1"]);

const index = buildIndex(trace as never, report as never, modelDocument as never);

// The queues of the model, the facilities of the model and the queues that
// had departures to wait for: the three families the charts paint.
const queues = index.queues;
const facilities = index.facilities;
const waited = [...index.waits.keys()];

// The width of the intervals of the waits is chosen once per queue, from the
// longest wait of the whole trace: two instants of the same queue are drawn
// with the same intervals, and the comparison of the two paths is fair.
//
// The width comes from `monitor/series.ts`, which is where the chart of the
// waits takes it from (`WaitDistribution.tsx`): the same function, the same
// value.
const widths = new Map<string, number>();
for (const name of waited) {
  const samples = index.waits.get(name) ?? [];
  widths.set(name, waitWidth(longestOf(samples.map((sample) => sample.wait))));
}

const clocks = sweepOf(index);
const constructor = await cacheConstructor();

// The cold paint: the first one of the process, which pays the compilation
// of the code it runs. It is measured before anything else warms the code
// up, and it is paid once per session, not once per paint.
const coldMark = performance.now();
seriesFrom(queues, index.queuePoints, index.endTime);
const coldRebuild = performance.now() - coldMark;
let coldCache: number | null = null;
if (constructor !== undefined) {
  const cold = new constructor();
  const mark = performance.now();
  cold.rows(index.queuePoints, queues, index.endTime);
  coldCache = performance.now() - mark;
}

// The cache the index carries is the one the app uses; when there is none,
// the script says so instead of measuring a cache that is not there.
const wired = (index as unknown as { series?: SeriesCacheLike }).series;
// The passes of the sweep: with a constructor every pass starts from a
// fresh cache, and without one only the first pass can be measured (the
// cache of the index already holds the series of the previous pass).
const passes = constructor === undefined ? 1 : PASSES;

// The cost of every paint of the sweep, the least measurement of every
// clock among the passes.
const before = emptyCosts();
const after = emptyCosts();
for (let position = 0; position < clocks.length; position += 1) {
  before.rows.push(Number.POSITIVE_INFINITY);
  before.segments.push(Number.POSITIVE_INFINITY);
  before.waits.push(Number.POSITIVE_INFINITY);
  before.total.push(Number.POSITIVE_INFINITY);
  after.rows.push(Number.POSITIVE_INFINITY);
  after.segments.push(Number.POSITIVE_INFINITY);
  after.waits.push(Number.POSITIVE_INFINITY);
  after.total.push(Number.POSITIVE_INFINITY);
}
// The reasons why the two paths are not the same: the proof of the cache is
// that this list stays empty.
const mismatches: string[] = [];
// Whether the cache was there to be measured.
let measuredCache = false;

for (let pass = 0; pass < passes; pass += 1) {
  const cache = constructor === undefined ? wired : new constructor();
  for (let position = 0; position < clocks.length; position += 1) {
    const clock = clocks[position];
    let referenceRows: SeriesRow[] = [];
    let referenceSegments: Segment[][] = [];
    let referenceFigures: (WaitFigure | null)[] = [];

    // The reference: the rebuild the app does today. It is a pure
    // function, so it can be repeated and the least run kept.
    let rowsMs = Number.POSITIVE_INFINITY;
    for (let repeat = 0; repeat < REPEATS; repeat += 1) {
      const mark = performance.now();
      const rows = seriesFrom(queues, index.queuePoints, clock);
      const elapsed = performance.now() - mark;
      if (repeat === 0) {
        referenceRows = rows;
      }
      if (elapsed < rowsMs) {
        rowsMs = elapsed;
      }
    }
    before.rows[position] = Math.min(before.rows[position], rowsMs);

    let segmentsMs = Number.POSITIVE_INFINITY;
    for (let repeat = 0; repeat < REPEATS; repeat += 1) {
      const mark = performance.now();
      const segments = facilities.map((name) =>
        segmentsOf(index.facilityPoints.get(name) ?? [], clock),
      );
      const elapsed = performance.now() - mark;
      if (repeat === 0) {
        referenceSegments = segments;
      }
      if (elapsed < segmentsMs) {
        segmentsMs = elapsed;
      }
    }
    before.segments[position] = Math.min(before.segments[position], segmentsMs);

    let waitsMs = Number.POSITIVE_INFINITY;
    for (let repeat = 0; repeat < REPEATS; repeat += 1) {
      const mark = performance.now();
      const figures = waited.map((name) =>
        referenceWaits(index.waits.get(name) ?? [], widths.get(name) ?? 1, clock),
      );
      const elapsed = performance.now() - mark;
      if (repeat === 0) {
        referenceFigures = figures;
      }
      if (elapsed < waitsMs) {
        waitsMs = elapsed;
      }
    }
    before.waits[position] = Math.min(before.waits[position], waitsMs);
    before.total[position] = Math.min(
      before.total[position],
      rowsMs + segmentsMs + waitsMs,
    );

    if (cache === undefined) {
      continue;
    }

    // The cache: the same three families, read from the cache.
    measuredCache = true;
    const mark = performance.now();
    const cachedRows = cache.rows(index.queuePoints, queues, clock);
    const cachedRowsMs = performance.now() - mark;

    const segmentsMark = performance.now();
    const cachedSegments = facilities.map((name) =>
      cache.segments(index.facilityPoints.get(name) ?? [], clock),
    );
    const cachedSegmentsMs = performance.now() - segmentsMark;

    const waitsMark = performance.now();
    const cachedFigures = waited.map((name) =>
      cache.waits(index.waits.get(name) ?? [], widths.get(name) ?? 1, clock),
    );
    const cachedWaitsMs = performance.now() - waitsMark;

    // The proof: the two paths have to be the same, paint by paint.
    if (pass === 0) {
      if (!sameValue(referenceRows, cachedRows)) {
        mismatches.push(`the rows of the queues at ${clock} s differ`);
      }
      for (let family = 0; family < facilities.length; family += 1) {
        if (!sameValue(referenceSegments[family], cachedSegments[family])) {
          mismatches.push(
            `the segments of the facility ${facilities[family]} at ${clock} s differ`,
          );
        }
      }
      for (let family = 0; family < waited.length; family += 1) {
        const reason = sameFigure(referenceFigures[family], cachedFigures[family]);
        if (reason !== null) {
          mismatches.push(`the waits of the queue ${waited[family]} at ${clock} s: ${reason}`);
        }
      }
    }

    after.rows[position] = Math.min(after.rows[position], cachedRowsMs);
    after.segments[position] = Math.min(after.segments[position], cachedSegmentsMs);
    after.waits[position] = Math.min(after.waits[position], cachedWaitsMs);
    after.total[position] = Math.min(
      after.total[position],
      cachedRowsMs + cachedSegmentsMs + cachedWaitsMs,
    );
  }
}

const traceEvents = (trace as { events: unknown[] }).events.length;
console.log(
  `model=${model}` +
    `  records=${index.records}${index.truncated ? " (truncated)" : ""}` +
    `  events=${traceEvents}` +
    `  frames=${index.frames.length}` +
    `  queues=${queues.length}` +
    `  facilities=${facilities.length}` +
    `  run=${index.startTime}..${index.endTime} s` +
    `  paints=${clocks.length}` +
    `  passes=${passes}` +
    `  cache=${
      constructor !== undefined
        ? "a fresh instance"
        : wired === undefined
          ? "none"
          : "the one of the index"
    }`,
);
console.log(
  "the rows family is the chart of all the queues of the model at once, the" +
    " segments family is the timeline of all the facilities, and the waits" +
    " family is the distribution of the waits of every queue",
);
console.log(
  "the worst paint is the one of the end of the run: it is the steady cost" +
    " of a long replay, where the rebuild walks the whole history",
);
console.log(
  `the cold paint (the first of the process, which pays the compilation of the` +
    ` code) costs ${coldRebuild.toFixed(4)} ms for the rebuild` +
    (coldCache === null ? "" : ` and ${coldCache.toFixed(4)} ms for the cache`) +
    ": it is paid once per session, not once per paint.",
);
console.log("");
console.log(
  "family    path      mean/paint    median      worst    share of a CPU-second",
);
console.log(line("rows", "before", before.rows));
console.log(line("segments", "before", before.segments));
console.log(line("waits", "before", before.waits));
console.log(line("TOTAL", "before", before.total));

if (!measuredCache) {
  console.log("");
  console.log(
    'the cache of the series is not here yet: the "after" path is pending its unit',
  );
  console.log(
    `  (${CACHE_MODULE} and the wiring of the index as \`index.series\`); the numbers`,
  );
  console.log("  above are the cost of the rebuild the app does today.");
} else {
  console.log("");
  console.log(line("rows", "after", after.rows));
  console.log(line("segments", "after", after.segments));
  console.log(line("waits", "after", after.waits));
  console.log(line("TOTAL", "after", after.total));

  const beforeMean = mean(before.total);
  const afterMean = mean(after.total);
  console.log("");
  console.log(
    `the cache paints the whole section ${
      beforeMean > 0 ? (beforeMean / afterMean).toFixed(1) : "?"
    } times cheaper than the rebuild` +
      ` (${beforeMean.toFixed(4)} ms against ${afterMean.toFixed(4)} ms per paint),`,
  );
  console.log(
    `and its worst paint is ${worst(after.total).toFixed(4)} ms against ${worst(
      before.total,
    ).toFixed(4)} ms of the rebuild.`,
  );

  console.log("");
  if (mismatches.length > 0) {
    console.error(`the two paths do NOT agree: ${mismatches.length} differences`);
    for (const mismatch of mismatches.slice(0, 20)) {
      console.error(`  ${mismatch}`);
    }
    // The exit code is the loud part: the script fails when the two paths
    // are not the same. It is not an exit of the process, so everything it
    // printed reaches the reader.
    process.exitCode = 1;
  } else {
    console.log(
      `the two paths agree on every clock of the sweep (${clocks.length} paints):` +
        " the rows, the segments and the figures of the waits are the same.",
    );
  }
}
