//! The ranking of the blocks: the counts of the instant.
//!
//! The rows of the ranking used to be read from the state of the replay at
//! the playhead, and the playhead does not move while the reproduction runs:
//! the ranking showed the transactions of a frame that had gone by. The
//! counts and the retries come from the live layer now, which publishes
//! them with the clock of the frame the replay reached.

import assert from "node:assert/strict";
import { test } from "node:test";
import { blockRows } from "../src/components/charts/blocks.ts";
import type { LiveCounts } from "../src/monitor/live.ts";
import type { ReportDocument } from "../src/types";

/** The report of a run with four program rows. */
const report = {
  blocks: [
    { line: 1, node: 1, class: "Queue", current: 0, entries: 5 },
    { line: 2, node: 2, class: "Depart", current: 0, entries: 5 },
    { line: 3, node: 3, class: "Seize", current: 0, entries: 1 },
    { line: 4, node: 4, class: "Release", current: 0, entries: 1 },
  ],
} as unknown as ReportDocument;

/** The counts of the live layer: what the blocks hold right now. */
const counts = new Map<number, LiveCounts>([
  [1, { now: 2, ever: 5 }],
  [3, { now: 0, ever: 1 }],
]);

/** The times every program row could not be entered. */
const retries = { 0: 3, 2: 1 };

test("the ranking of the blocks takes the counts of the instant", () => {
  const rows = blockRows(report, null, counts, retries);
  const queue = rows.find((row) => row.name === "bloque 1")!;
  assert.equal(queue.entries, 5, "the transactions that went through it");
  assert.equal(queue.current, 2, "the transactions inside it right now");
  assert.equal(queue.live, true, "the live layer knows the block");
});

test("the retries of a block are the ones of its rows", () => {
  const rows = blockRows(report, null, counts, retries);
  const queue = rows.find((row) => row.name === "bloque 1")!;
  const seize = rows.find((row) => row.name === "bloque 3")!;
  assert.equal(queue.retries, 3, "the retries of the row of the queue");
  assert.equal(seize.retries, 1, "the retries of the row of the seize");
});

test("a block the live layer does not know shows nothing", () => {
  const rows = blockRows(report, null, counts, retries);
  const depart = rows.find((row) => row.name === "bloque 2")!;
  assert.equal(depart.entries, 0, "no transactions");
  assert.equal(depart.current, 0, "and none inside it");
  assert.equal(depart.live, false, "the live layer does not know it yet");
});

test("the ranking is ordered by the transactions that went through the block", () => {
  const rows = blockRows(report, null, counts, retries);
  assert.deepEqual(
    rows.map((row) => row.name),
    ["bloque 1", "bloque 3", "bloque 2", "bloque 4"],
    "the blocks with more transactions come first",
  );
});
