//! The cache of the series: the same answers as the reference, built once.
//!
//! The cache is not a second implementation of the series: it is the same
//! answers, extended instead of rebuilt. These tests pin that it agrees
//! with `seriesFrom`, `segmentsOf` and the histogram of the waits at every
//! instant, and that it keeps the identity of what did not change.

import assert from "node:assert/strict";
import { test } from "node:test";
import { cumulative, histogram, segmentsOf, seriesFrom } from "../src/monitor/series.ts";
import type { SeriesPoint } from "../src/monitor/series.ts";
import type { WaitSample } from "../src/monitor/replay.ts";
import { SeriesCache } from "../src/monitor/seriesCache.ts";

/** The instants the sweep asks for: on a change, between changes, at the end. */
const CLOCKS = [0, 5, 10, 12, 15, 20, 25, 30, 40, 45, 100, Number.POSITIVE_INFINITY];

/** The entities of the run of the tests. */
const ENTITIES = ["qA", "qB"];

/** The points of a run: interleaved, repeated and zero changes. */
function runPoints(): Map<string, SeriesPoint[]> {
  return new Map<string, SeriesPoint[]>([
    [
      "qA",
      [
        { time: 0, value: 1 },
        { time: 10, value: 3 },
        { time: 10, value: 4 },
        { time: 20, value: 0 },
        { time: 40, value: 7 },
      ],
    ],
    [
      "qB",
      [
        { time: 5, value: 2 },
        { time: 15, value: 0 },
        { time: 15, value: 1 },
        { time: 30, value: 1 },
      ],
    ],
  ]);
}

/** The waits of a queue: a departure at every few seconds. */
const WAIT_SAMPLES: WaitSample[] = [
  { time: 1, wait: 0.5 },
  { time: 3, wait: 2.5 },
  { time: 7, wait: 0.5 },
  { time: 9, wait: 6 },
];

/** What the chart of the waits computes at an instant. */
function waitsAt(samples: WaitSample[], width: number, clock: number) {
  const values = samples
    .filter((sample) => sample.time <= clock)
    .map((sample) => sample.wait);
  if (values.length === 0) {
    return null;
  }
  const bins = histogram(values, width);
  const shares = cumulative(bins);
  const mean = values.reduce((sum, wait) => sum + wait, 0) / values.length;
  const longest = values.reduce((max, wait) => (wait > max ? wait : max), 0);
  return { bins, shares, mean, longest };
}

test("the cached rows are the rows of the reference, at every instant", () => {
  const cache = new SeriesCache();
  const points = runPoints();
  for (const clock of CLOCKS) {
    assert.deepEqual(
      cache.rows(points, ENTITIES, clock),
      seriesFrom(ENTITIES, points, clock),
      `at the second ${clock}`,
    );
  }
});

test("the sweep does not depend on the order the instants are asked for", () => {
  // The reader scrubs the playhead: the cache is asked for an instant
  // before the one it answered last.
  const cache = new SeriesCache();
  const points = runPoints();
  const fresh = new SeriesCache();
  for (const clock of [...CLOCKS].reverse()) {
    assert.deepEqual(
      cache.rows(points, ENTITIES, clock),
      fresh.rows(points, ENTITIES, clock),
      `at the second ${clock}`,
    );
  }
});

test("the series starts at zero, even when the first change is later", () => {
  const cache = new SeriesCache();
  const points = new Map<string, SeriesPoint[]>([["qA", [{ time: 30, value: 4 }]]]);
  const rows = cache.rows(points, ["qA"], 100);
  assert.deepEqual(rows[0], { time: 0, qA: 0 });
});

test("the series starts at zero even when the reader is before the run", () => {
  const cache = new SeriesCache();
  const points = new Map<string, SeriesPoint[]>([["qA", [{ time: 10, value: 4 }]]]);
  assert.deepEqual(cache.rows(points, ["qA"], -5), seriesFrom(["qA"], points, -5));
  assert.deepEqual(cache.rows(points, ["qA"], -5), [{ time: 0, qA: 0 }]);
});

test("a change at zero is the value the series starts with", () => {
  const cache = new SeriesCache();
  const points = new Map<string, SeriesPoint[]>([["qA", [{ time: 0, value: 1 }, { time: 10, value: 0 }]]]);
  const rows = cache.rows(points, ["qA"], 100);
  assert.deepEqual(rows[0], { time: 0, qA: 1 });
});

test("the last row is the instant the reader is at, with the values of the last change", () => {
  const cache = new SeriesCache();
  const rows = cache.rows(runPoints(), ENTITIES, 22);
  assert.deepEqual(rows[rows.length - 1], { time: 22, qA: 0, qB: 1 });
});

test("there are no rows after the instant the reader is at", () => {
  const cache = new SeriesCache();
  const points = runPoints();
  for (const clock of CLOCKS) {
    for (const row of cache.rows(points, ENTITIES, clock)) {
      assert.ok(
        row.time <= clock,
        `the row of the second ${row.time} is after the clock of the second ${clock}`,
      );
    }
  }
});

test("the same instant gives the same array, and another instant another one", () => {
  const cache = new SeriesCache();
  const points = runPoints();
  const first = cache.rows(points, ENTITIES, 20);
  assert.strictEqual(cache.rows(points, ENTITIES, 20), first);
  assert.notStrictEqual(cache.rows(points, ENTITIES, 21), first);
});

test("the array of an instant never changes its content", () => {
  // The rows of the cache are shared by the answers that hold them: an
  // answer that is still around must not change under the reader.
  const cache = new SeriesCache();
  const points = runPoints();
  const atTwenty = cache.rows(points, ENTITIES, 20);
  const before = [...atTwenty];
  cache.rows(points, ENTITIES, 40);
  assert.deepEqual(atTwenty, before);
});

test("a new run does not see the rows of the previous one", () => {
  const cache = new SeriesCache();
  const first = new Map<string, SeriesPoint[]>([
    ["qA", [{ time: 0, value: 1 }, { time: 10, value: 2 }]],
  ]);
  const second = new Map<string, SeriesPoint[]>([
    ["qA", [{ time: 0, value: 7 }, { time: 10, value: 8 }]],
  ]);
  cache.rows(first, ["qA"], 20);
  const rows = cache.rows(second, ["qA"], 20);
  assert.deepEqual(rows, seriesFrom(["qA"], second, 20));
  assert.deepEqual(rows, [
    { time: 0, qA: 7 },
    { time: 10, qA: 8 },
    { time: 20, qA: 8 },
  ]);
});

test("an entity named time overwrites the instant of the row, as the reference does", () => {
  const cache = new SeriesCache();
  const points = new Map<string, SeriesPoint[]>([
    ["time", [{ time: 0, value: 5 }, { time: 10, value: 7 }]],
  ]);
  for (const clock of [0, 5, 10, 20]) {
    assert.deepEqual(
      cache.rows(points, ["time"], clock),
      seriesFrom(["time"], points, clock),
      `at the second ${clock}`,
    );
  }
});

test("the cached segments are the segments of the reference, at every instant", () => {
  const cache = new SeriesCache();
  const points: SeriesPoint[] = [
    { time: 3, value: 1 },
    { time: 4, value: 0 },
    { time: 8, value: 1 },
  ];
  for (const clock of [0, 2, 3, 3.5, 4, 5, 8, 9, 20]) {
    assert.deepEqual(
      cache.segments(points, clock),
      segmentsOf(points, clock),
      `at the second ${clock}`,
    );
  }
});

test("the segments of a facility that is never taken are free the whole run", () => {
  const cache = new SeriesCache();
  assert.deepEqual(cache.segments([], 10), [{ start: 0, end: 10, busy: false }]);
});

test("the segments of a facility stop at the instant the reader is at", () => {
  const cache = new SeriesCache();
  const points: SeriesPoint[] = [
    { time: 3, value: 1 },
    { time: 4, value: 0 },
    { time: 8, value: 1 },
  ];
  // The reader scrubs back: the segments of the earlier instant are the
  // ones of that instant, and not the ones of the later one.
  cache.segments(points, 20);
  assert.deepEqual(cache.segments(points, 5), [
    { start: 0, end: 3, busy: false },
    { start: 3, end: 4, busy: true },
    { start: 4, end: 5, busy: false },
  ]);
});

test("the same instant gives the same segments, and another instant another one", () => {
  const cache = new SeriesCache();
  const points: SeriesPoint[] = [{ time: 3, value: 1 }];
  const first = cache.segments(points, 10);
  assert.strictEqual(cache.segments(points, 10), first);
  assert.notStrictEqual(cache.segments(points, 11), first);
});

test("the cached waits are the waits of the reference, at every instant", () => {
  const cache = new SeriesCache();
  const width = 2;
  for (const clock of [0, 1, 2, 3, 5, 7, 8, 9, 20]) {
    assert.deepEqual(
      cache.waits(WAIT_SAMPLES, width, clock),
      waitsAt(WAIT_SAMPLES, width, clock),
      `at the second ${clock}`,
    );
  }
});

test("there are no waits before the first departure of the queue", () => {
  const cache = new SeriesCache();
  assert.equal(cache.waits(WAIT_SAMPLES, 2, 0), null);
});

test("the width of the intervals of the waits does not change with the clock", () => {
  // The width is chosen once per queue: two instants of the same queue are
  // drawn with the same intervals.
  const cache = new SeriesCache();
  const width = 2;
  const early = cache.waits(WAIT_SAMPLES, width, 1);
  const late = cache.waits(WAIT_SAMPLES, width, 9);
  assert.deepEqual(early?.bins.map((bin) => bin.low), [0]);
  assert.deepEqual(late?.bins.map((bin) => bin.low), [0, 2, 4, 6]);
  assert.deepEqual(late?.shares, [50, 75, 75, 100]);
  for (const bin of late?.bins ?? []) {
    assert.equal(bin.low % width, 0);
    assert.equal(bin.high - bin.low, width);
  }
});

test("a width that changed rebuilds the histogram of the queue", () => {
  const cache = new SeriesCache();
  assert.deepEqual(cache.waits(WAIT_SAMPLES, 10, 9)?.bins.map((bin) => bin.low), [0]);
  assert.deepEqual(cache.waits(WAIT_SAMPLES, 2, 9)?.bins.map((bin) => bin.low), [0, 2, 4, 6]);
  assert.deepEqual(cache.waits(WAIT_SAMPLES, 10, 9)?.bins.map((bin) => bin.low), [0]);
});

test("the same instant gives the same waits, and another instant another one", () => {
  const cache = new SeriesCache();
  const first = cache.waits(WAIT_SAMPLES, 2, 9);
  assert.strictEqual(cache.waits(WAIT_SAMPLES, 2, 9), first);
  assert.notStrictEqual(cache.waits(WAIT_SAMPLES, 2, 10), first);
});
