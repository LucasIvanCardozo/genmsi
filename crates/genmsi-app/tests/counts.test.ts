import assert from "node:assert/strict";
import { test } from "node:test";

import { countsOf, type CountState } from "../src/monitor/counts.ts";
import type { ReportBlockLike } from "../src/monitor/replay.ts";

/** One row of the report, as the replay reads it. */
function row(
  line: number,
  node: number,
  cls: string,
  countsBlock: boolean,
): ReportBlockLike {
  return { line, node, class: cls, current: 0, entries: 0, counts_block: countsBlock };
}

/** The counters of the replay, with the same figure in every line. */
function state(totals: number[]): CountState {
  const blocks: Record<number, number> = {};
  const totalsByLine: Record<number, number> = {};
  totals.forEach((total, index) => {
    blocks[index] = 0;
    totalsByLine[index] = total;
  });
  return { blocks, totals: totalsByLine };
}

test("the figure of a block is the one of the row that counts it", () => {
  // A facility use is five statements of the listing (a queue, a seize, a
  // depart, an advance and a release), and all of them count the 59
  // transactions that went through the block.
  const rows = [
    row(1, 1, "Generate", true),
    row(2, 2, "Queue", true),
    row(3, 2, "Seize", false),
    row(4, 2, "Depart", false),
    row(5, 2, "Advance", false),
    row(6, 2, "Release", false),
  ];
  const counts = countsOf(rows, state([60, 59, 59, 59, 59, 59]));
  // The block of the facility use counts 59 transactions, not 295.
  assert.equal(counts.get(2)?.ever, 59);
  assert.equal(counts.get(2)?.now, 0);
});

test("a generator has no figure", () => {
  const rows = [row(1, 1, "Generate", true), row(2, 2, "Queue", true)];
  const counts = countsOf(rows, state([60, 59]), new Set([1]));
  // The generator itself is not an instance that passed through it.
  assert.equal(counts.get(1), undefined);
  assert.equal(counts.get(2)?.ever, 59);
});

test("the clock keeps its figure although its first statement is a generator", () => {
  // The clock is a generator of ticks, and the statement that counts it is
  // a `GENERATE`: the generator is told apart by the block, not by the
  // statement.
  const rows = [row(1, 4, "Generate", true)];
  const counts = countsOf(rows, state([2]), new Set([1]));
  assert.equal(counts.get(4)?.ever, 2);
});

test("a report that does not say which row counts its block leaves it without a figure", () => {
  const rows = [{ line: 1, node: 1, class: "Queue", current: 0, entries: 0 }];
  const counts = countsOf(rows, state([59]));
  assert.equal(counts.get(1), undefined);
});
