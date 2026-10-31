//! The metrics the app derives from the numbers the engine publishes.
//!
//! A derived metric is a ratio of two published numbers: the area and the
//! time of the run, the time occupied and the time of the run. The engine
//! does not publish them, so the app computes them and says so.

import assert from "node:assert/strict";
import { test } from "node:test";
import {
  chainDerived,
  facilityDerived,
  queueDerived,
  storageDerived,
  tableDerived,
} from "../src/monitor/derived.ts";

test("the mean content of a queue is its area over the time of the run", () => {
  // The queue held 2 units for 10 seconds: the area is 20 and the run took
  // 40 seconds, so the mean content is 0.5.
  const derived = queueDerived({ entries: 4, zeroEntries: 1, totalTime: 30, area: 20 }, 40);
  assert.equal(derived.meanContent, 0.5);
});

test("the mean wait of a queue is its total time over its entries", () => {
  const derived = queueDerived({ entries: 4, zeroEntries: 1, totalTime: 30, area: 20 }, 40);
  assert.equal(derived.meanWait, 7.5);
});

test("the mean wait of the engine is the area over the entries", () => {
  // The engine's AVE.TIME: the queue covered an area of 20 and 4
  // transactions entered it, so it is 5 seconds — not the 7.5 of the sum of
  // the waits of the ones that left.
  const derived = queueDerived({ entries: 4, zeroEntries: 1, totalTime: 30, area: 20 }, 40);
  assert.equal(derived.engineMeanWait, 5);
});

test("a queue that saw nobody derives no wait of the engine", () => {
  const derived = queueDerived({ entries: 0, zeroEntries: 0, totalTime: 0, area: 0 }, 40);
  assert.equal(derived.engineMeanWait, 0);
});

test("the share of waits that were zero is the zero entries over the entries", () => {
  const derived = queueDerived({ entries: 4, zeroEntries: 1, totalTime: 30, area: 20 }, 40);
  assert.equal(derived.zeroShare, 0.25);
});

test("a queue that saw nothing derives nothing", () => {
  const derived = queueDerived({ entries: 0, zeroEntries: 0, totalTime: 0, area: 0 }, 40);
  assert.equal(derived.meanContent, 0);
  assert.equal(derived.meanWait, 0);
  assert.equal(derived.zeroShare, 0);
});

test("a run of no time divides by nothing", () => {
  const derived = queueDerived({ entries: 4, zeroEntries: 1, totalTime: 30, area: 20 }, 0);
  assert.equal(derived.meanContent, 0, "there was no time to divide by");
});

test("the utilization of a facility is the time it was busy over the run", () => {
  // The facility was busy 30 seconds of a run of 40: three quarters of it.
  const derived = facilityDerived({ entries: 6, busyTime: 30 }, 40);
  assert.equal(derived.utilization, 0.75);
});

test("the mean service of a facility is its busy time over its entries", () => {
  const derived = facilityDerived({ entries: 6, busyTime: 30 }, 40);
  assert.equal(derived.meanService, 5);
});

test("a facility that was never taken derives nothing", () => {
  const derived = facilityDerived({ entries: 0, busyTime: 0 }, 40);
  assert.equal(derived.utilization, 0);
  assert.equal(derived.meanService, 0);
});

test("the utilization of a storage is its area over the time and the capacity", () => {
  // The storage held 20 units for 10 seconds of a 40 second run with a
  // capacity of 50: a fifth of the capacity, half of the time.
  const derived = storageDerived({ capacity: 50, area: 200 }, 40);
  assert.equal(derived.meanUtilization, 0.1);
});

test("a storage with no capacity derives nothing", () => {
  const derived = storageDerived({ capacity: 0, area: 200 }, 40);
  assert.equal(derived.meanUtilization, 0);
});

test("the mean size of a chain is its area over the time of the run", () => {
  const derived = chainDerived({ area: 120 }, 40);
  assert.equal(derived.meanSize, 3);
});

test("the mean of a table is its sum over its samples", () => {
  const derived = tableDerived({ samples: 4, sum: 20, sumSquares: 120, overflow: 1 });
  assert.equal(derived.mean, 5);
});

test("the deviation of a table comes from the sum of squares the engine publishes", () => {
  // Four samples: 2, 4, 6, 8. The mean is 5 and the deviation is 2.449...
  const derived = tableDerived({ samples: 4, sum: 20, sumSquares: 120, overflow: 0 });
  assert.ok(Math.abs(derived.stdDev - Math.sqrt(5)) < 1e-9, `the deviation was ${derived.stdDev}`);
});

test("a table with no samples derives nothing", () => {
  const derived = tableDerived({ samples: 0, sum: 0, sumSquares: 0, overflow: 0 });
  assert.equal(derived.mean, 0);
  assert.equal(derived.stdDev, 0);
  assert.equal(derived.overflowRate, 0);
});

test("the rate of overflow of a table is its overflow over its samples", () => {
  const derived = tableDerived({ samples: 100, sum: 20, sumSquares: 120, overflow: 3 });
  assert.equal(derived.overflowRate, 0.03);
});
