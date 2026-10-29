import assert from "node:assert/strict";
import { test } from "node:test";

import {
  containerCountsOf,
  countsOf,
  countsWithContainers,
  crossingsOf,
  type CountState,
} from "../src/monitor/counts.ts";
import { containerPair } from "../src/graph/node.ts";
import { containerShape } from "../src/graph/frames.ts";
import type { ReportBlockLike } from "../src/monitor/replay.ts";
import type { ModelBlock, ModelEdge } from "../src/types/index.ts";

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

/** A block of the model, as the engine publishes it. */
function block(
  id: number,
  extra: Partial<ModelBlock> = {},
): ModelBlock {
  return {
    id,
    class: "Queue",
    kind: "A",
    name: `Bloque${id}`,
    caption: "",
    label: null,
    container: 0,
    layout: { x: 0, y: 0, width: 100, height: 40 },
    properties: [],
    inputs: [],
    outputs: [],
    ...extra,
  };
}

/** A container of the model, with the doors it declares. */
function container(
  id: number,
  inputs: ModelBlock["inputs"] = [],
  outputs: ModelBlock["outputs"] = [],
): ModelBlock {
  return block(id, { class: "", kind: "C", inputs, outputs });
}

/** A port of a block, as the engine publishes it. */
function port(ioid: number, name: string): ModelBlock["inputs"][number] {
  return { ioid, name, caption: "", label: null, layout: null };
}

/** A link of the model, as the engine publishes it. */
function link(
  from: number,
  fromIoid: number,
  to: number,
  toIoid = 1,
): ModelEdge {
  return {
    id: `${from}:${fromIoid}->${to}:${toIoid}`,
    container: from,
    from_module: from,
    from_ioid: fromIoid,
    from_port: "Salida1",
    to_module: to,
    to_ioid: toIoid,
    to_port: "Entrada1",
    label: null,
    hops: [],
  };
}

/** The counters of the replay, with a figure in every line. */
function counters(lines: Array<{ now: number; ever: number }>): CountState {
  const blocks: Record<number, number> = {};
  const totals: Record<number, number> = {};
  lines.forEach((line, index) => {
    blocks[index] = line.now;
    totals[index] = line.ever;
  });
  return { blocks, totals };
}

test("a container shows the sum of the figures of the blocks it holds", () => {
  // The container has no row of the report —the engine compiles it inline—,
  // so its figure is computed: the entities inside it are the ones of the
  // blocks it holds.
  const rows = [
    row(1, 11, "Queue", true),
    // The rest of the statements of a facility use: they do not count the
    // block, so their figures are not added up.
    row(2, 11, "Seize", false),
    row(3, 12, "Depart", true),
  ];
  const model = {
    blocks: [
      container(10, [port(2, "Entrada 1")]),
      block(11, { container: 10 }),
      block(12, { container: 10 }),
    ],
    edges: [link(10, 2, 11)],
  };
  const counts = countsWithContainers(
    rows,
    counters([
      { now: 3, ever: 20 },
      { now: 9, ever: 99 },
      { now: 2, ever: 15 },
    ]),
    model,
  );
  const figure = counts.get(10);
  // Three entities in the queue and two in the depart: the row that does
  // not count the block is not added up.
  assert.equal(figure?.now, 5);
});

test("the total of a container is the figure of the block its entry door leads to", () => {
  const rows = [row(1, 11, "Queue", true), row(2, 12, "Depart", true)];
  const model = {
    blocks: [
      container(10, [port(2, "Entrada 1")]),
      block(11, { container: 10 }),
      block(12, { container: 10 }),
    ],
    // The flow enters the container by the depart: its total of entries is
    // the total that went through the container.
    edges: [link(10, 2, 12)],
  };
  const counts = countsWithContainers(
    rows,
    counters([
      { now: 3, ever: 20 },
      { now: 2, ever: 15 },
    ]),
    model,
  );
  assert.equal(counts.get(10)?.ever, 15);
  assert.equal(counts.get(10)?.now, 5);
});

test("a container without an entry door has passed nothing", () => {
  const rows = [row(1, 11, "Queue", true)];
  const model = {
    blocks: [container(20), block(11, { container: 20 })],
    edges: [],
  };
  const counts = countsWithContainers(
    rows,
    counters([{ now: 3, ever: 20 }]),
    model,
  );
  const figure = counts.get(20);
  assert.equal(figure?.now, 3);
  assert.equal(figure?.ever, 0);
  // The container is there and it is empty: it says 0/0.
  assert.equal(containerPair({ now: 0, ever: 0 }), "0/0");
  assert.equal(containerPair(figure), "3/0");
});

test("a container that was just made shows 0/0", () => {
  const model = { blocks: [container(30)], edges: [] };
  const counts = containerCountsOf(model, new Map());
  assert.deepEqual(counts.get(30), { now: 0, ever: 0 });
  assert.equal(containerPair(counts.get(30)), "0/0");
});

test("a normal block keeps the figure of its row", () => {
  const rows = [row(1, 11, "Queue", true), row(2, 12, "Depart", true)];
  const model = {
    blocks: [
      container(10, [port(2, "Entrada 1")]),
      block(11, { container: 10 }),
      block(12, { container: 10 }),
    ],
    edges: [link(10, 2, 11)],
  };
  const counts = countsWithContainers(
    rows,
    counters([
      { now: 3, ever: 20 },
      { now: 2, ever: 15 },
    ]),
    model,
  );
  // The figure of a block is the one of the row that counts it, and the
  // container does not change it.
  assert.deepEqual(counts.get(11), { now: 3, ever: 20 });
  assert.deepEqual(counts.get(12), { now: 2, ever: 15 });
});

test("the container reserves the zone of its figure", () => {
  // The container shows its figures, so the zone of the figure is there
  // even when the plate is too small for it on its own.
  const shape = containerShape({ width: 40, height: 20 }, { inputs: [], outputs: [] });
  assert.notEqual(shape.figure, null);
});

test("the total of a container is the count of the moves that entered it", () => {
  // A move that comes from outside and goes in crosses the border of the
  // container: that is an entity that went through it. A move that leaves
  // is not counted again — the container counts the entities that entered
  // it, and an entity that goes in and out went through it once.
  const model = {
    blocks: [
      container(10, [port(2, "Entrada 1")]),
      block(11, { container: 10 }),
      block(12, { container: 0 }),
    ],
    edges: [],
  };
  const moves = [
    { from: 12, to: 11, time: 10 },
    { from: 11, to: 12, time: 20 },
  ];
  assert.equal(crossingsOf(model, moves, 100).get(10), 1);
  // And the count stops at the time the replay has reached.
  assert.equal(crossingsOf(model, moves, 15).get(10), 1);
  assert.equal(crossingsOf(model, moves, 5).get(10), undefined);
});

test("the container shows the entities that crossed its border", () => {
  // The figure of a container is the one of the crossings: the entities
  // inside it, and the entities that entered it.
  const model = {
    blocks: [
      container(10, [port(2, "Entrada 1")]),
      block(11, { container: 10 }),
    ],
    edges: [],
  };
  const rows = [row(1, 11, "Queue", true)];
  const crossings = crossingsOf(
    model,
    [{ from: 0, to: 11, time: 10 }],
    100,
  );
  const counts = countsWithContainers(
    rows,
    counters([{ now: 2, ever: 7 }]),
    model,
    crossings,
  );
  assert.deepEqual(counts.get(10), { now: 2, ever: 1 });
  assert.equal(containerPair(counts.get(10)), "2/1");
});
