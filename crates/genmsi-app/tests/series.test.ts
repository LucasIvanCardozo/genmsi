//! The series the charts draw: the value of an entity over the time of the run.

import assert from "node:assert/strict";
import { test } from "node:test";
import { countAt, cumulative, histogram, histogramRows, segmentsOf, seriesFrom } from "../src/monitor/series.ts";

/** The points of one entity. */
function points(...rows: [number, number][]): Map<string, { time: number; value: number }[]> {
  return new Map([["qA", rows.map(([time, value]) => ({ time, value }))]]);
}

test("a series starts at zero, even when the first change is later", () => {
  // The queue was empty until the second 30: the series says so, and the
  // chart's axis starts at zero.
  const rows = seriesFrom(["qA"], points([30, 4]), 100);
  assert.deepEqual(rows, [
    { time: 0, qA: 0 },
    { time: 30, qA: 4 },
    { time: 100, qA: 4 },
  ]);
});

test("a change at zero is the value the series starts with", () => {
  const rows = seriesFrom(["qA"], points([0, 1], [10, 0]), 100);
  assert.deepEqual(rows, [
    { time: 0, qA: 1 },
    { time: 10, qA: 0 },
    { time: 100, qA: 0 },
  ]);
});

test("the series stops at the instant the reader is at", () => {
  const rows = seriesFrom(["qA"], points([0, 1], [10, 0], [20, 3]), 20);
  assert.deepEqual(rows, [
    { time: 0, qA: 1 },
    { time: 10, qA: 0 },
    { time: 20, qA: 3 },
  ]);
});

test("the last row of the series is the instant the reader is at", () => {
  // The reader is at the second 15: the series ends there, so the filled
  // area of the chart reaches the cursor.
  const rows = seriesFrom(["qA"], points([0, 1], [10, 0]), 15);
  assert.deepEqual(rows, [
    { time: 0, qA: 1 },
    { time: 10, qA: 0 },
    { time: 15, qA: 0 },
  ]);
});

test("every entity of the series carries its own value", () => {
  const two = new Map<string, { time: number; value: number }[]>([
    ["qA", [{ time: 0, value: 1 }]],
    ["qB", [{ time: 5, value: 2 }]],
  ]);
  const rows = seriesFrom(["qA", "qB"], two, 10);
  assert.deepEqual(rows, [
    { time: 0, qA: 1, qB: 0 },
    { time: 5, qA: 1, qB: 2 },
    { time: 10, qA: 1, qB: 2 },
  ]);
});

test("the segments of a facility say when it was busy", () => {
  // The facility was taken at the second 3 and freed at the second 4.
  const segments = segmentsOf(
    [
      { time: 3, value: 1 },
      { time: 4, value: 0 },
    ],
    10,
  );
  assert.deepEqual(segments, [
    { start: 0, end: 3, busy: false },
    { start: 3, end: 4, busy: true },
    { start: 4, end: 10, busy: false },
  ]);
});

test("a facility that is never taken is free the whole run", () => {
  const segments = segmentsOf([], 10);
  assert.deepEqual(segments, [{ start: 0, end: 10, busy: false }]);
});

test("the histogram of the waits counts the values in their interval", () => {
  // The intervals are two seconds wide and start at zero: 1 is in [0, 2),
  // 5 is in [4, 6) and 7 is in [6, 8).
  const bins = histogram([1, 1, 5, 7], 2);
  assert.deepEqual(
    bins.map((bin) => [bin.low, bin.high, bin.count]),
    [
      [0, 2, 2],
      [2, 4, 0],
      [4, 6, 1],
      [6, 8, 1],
    ],
  );
});

test("the histogram of no waits is empty", () => {
  assert.deepEqual(histogram([], 2), []);
});

test("the accumulation of a histogram is the running share of the samples", () => {
  const bins = histogram([1, 3, 5, 7], 2);
  assert.deepEqual(cumulative(bins), [25, 50, 75, 100]);
});

test("the number of instants at or before an instant is counted", () => {
  assert.equal(countAt([0, 10, 20, 30], 20), 3);
  assert.equal(countAt([0, 10, 20, 30], 5), 1);
  assert.equal(countAt([], 5), 0);
});

test("the segments of a facility stop at the instant the reader is at", () => {
  // The facility was taken at the second 3, freed at the second 4 and taken
  // again at the second 8. The reader is at the second 5: the segments say
  // what happened up to there, and the last one ends at the clock.
  const segments = segmentsOf(
    [
      { time: 3, value: 1 },
      { time: 4, value: 0 },
      { time: 8, value: 1 },
    ],
    5,
  );
  assert.deepEqual(segments, [
    { start: 0, end: 3, busy: false },
    { start: 3, end: 4, busy: true },
    { start: 4, end: 5, busy: false },
  ]);
});

test("the intervals of a table histogram are correctly stepped", () => {
  const rows = histogramRows({ first: 10, width: 5, intervals: 4 }, [2, 5, 3, 1]);
  assert.equal(rows.length, 4);
  assert.equal(
    rows[0].label,
    "≤ 10",
    "the first class holds every value up to its first bound: it has no lower bound",
  );
  assert.equal(rows[1].label, "10 - 15");
  assert.equal(rows[2].label, "15 - 20");
  assert.equal(rows[3].label, "> 20");
  assert.equal(rows[3].overflowClass, true);
  assert.equal(rows[1].x, 12.5);
  assert.equal(rows[2].x, 17.5);
});
