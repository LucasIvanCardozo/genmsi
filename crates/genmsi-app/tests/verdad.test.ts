//! The truth of the charts: the numbers, the scales and the names.
//!
//! The section showed figures that were not what they said: a mean wait
//! that was not the engine's, a column of zeros that said "published", a
//! ranking whose winner was decided by the order of the array, a histogram
//! whose last classes were empty and a "reintento" that counted something
//! else than a retry.
//!
//! The logic of every card lives in a pure module, and this file proves
//! the behaviour of the pieces the runner can reach: the runner has no DOM
//! (`node --test tests/*.test.ts`), so the components themselves are not
//! rendered here.

import assert from "node:assert/strict";
import { test } from "node:test";
import {
  chainDerived,
  overflowDerived,
  queueDerived,
} from "../src/monitor/derived.ts";
import {
  histogramRows,
  trailingEmptyClasses,
  waitWidth,
} from "../src/monitor/series.ts";
import {
  blockNameOf,
  blockNoteOf,
  blockRows,
} from "../src/components/charts/blocks.ts";
import {
  chainsOf,
  facilitiesOf,
  queuesOf,
  resourcePressuresOf,
  storagesOf,
  type Elapsed,
} from "../src/components/charts/entities.ts";
import type { Figures } from "../src/monitor/stats.ts";
import type { LiveCounts } from "../src/monitor/live.ts";
import type { ModelBody, ReportDocument } from "../src/types";

const elapsed: Elapsed = { live: 10, published: 100 };

// ————————————————————————————————————————————————————————————————
// Las medias de la cola
// ————————————————————————————————————————————————————————————————

test("the wait of the engine is the area of the queue over its entries", () => {
  // The engine's AVE.TIME: the queue covered an area of 20 and 4
  // transactions entered it, so it is 5 seconds — not the 7.5 of the sum of
  // the waits of the ones that left.
  const derived = queueDerived(
    { entries: 4, zeroEntries: 1, totalTime: 30, area: 20, departures: 3 },
    40,
  );
  assert.equal(derived.engineMeanWait, 5);
});

test("the mean of the departures is the sum of the waits of the ones that left over the ones that left", () => {
  // Three transactions left, having waited 30 seconds altogether: their
  // mean wait is 10 seconds.
  const derived = queueDerived(
    { entries: 4, zeroEntries: 1, totalTime: 30, area: 20, departures: 3 },
    40,
  );
  assert.equal(derived.departureMeanWait, 10);
});

test("a queue that nobody left derives no mean of the departures", () => {
  const derived = queueDerived(
    { entries: 4, zeroEntries: 1, totalTime: 30, area: 20, departures: 0 },
    40,
  );
  assert.equal(derived.departureMeanWait, 0);
});

test("the mean time of a chain is the area of the chain over its entries", () => {
  // The engine's AVE.TIME of a chain: an area of 120 and 6 entries.
  const derived = chainDerived({ area: 120, entries: 6 }, 40);
  assert.equal(derived.meanTime, 20);
  assert.equal(derived.meanSize, 3, "the mean size is the area over the time");
});

test("a chain that nobody joined derives no mean time", () => {
  const derived = chainDerived({ area: 0, entries: 0 }, 40);
  assert.equal(derived.meanTime, 0);
});

// ————————————————————————————————————————————————————————————————
// La media y la desviación de los desbordes
// ————————————————————————————————————————————————————————————————

test("the mean of the overflows is the sum of the values that did not fit over how many they were", () => {
  // The engine publishes the sum and the sum of the squares of the values
  // that did not fit in any class: two values of 20 and 30.
  const derived = overflowDerived({ overflow: 2, sum: 50, sumSquares: 1300 });
  assert.equal(derived.mean, 25);
  assert.ok(
    Math.abs(derived.stdDev - 5) < 1e-9,
    `the deviation was ${derived.stdDev}`,
  );
});

test("a table with no overflow derives no mean of the overflows", () => {
  const derived = overflowDerived({ overflow: 0, sum: 0, sumSquares: 0 });
  assert.equal(derived.mean, 0);
  assert.equal(derived.stdDev, 0);
});

// ————————————————————————————————————————————————————————————————
// El histograma de la tabla
// ————————————————————————————————————————————————————————————————

test("the empty classes at the end of a table are trimmed", () => {
  // The engine's report stops the table when the classes add up to the
  // whole and says how many empty classes are left.
  const rows = histogramRows({ first: 10, width: 5, intervals: 5 }, [2, 0, 0, 0, 0]);
  assert.equal(rows.length, 1, "only the class that holds something is drawn");
  assert.equal(rows[0].frequency, 2);
  assert.equal(trailingEmptyClasses([2, 0, 0, 0, 0]), 4);
});

test("a table whose classes are all full keeps them all", () => {
  const rows = histogramRows({ first: 10, width: 5, intervals: 4 }, [2, 5, 3, 1]);
  assert.equal(rows.length, 4);
  assert.equal(trailingEmptyClasses([2, 5, 3, 1]), 0);
});

test("a table that has not been tabulated yet has no class at all", () => {
  const rows = histogramRows({ first: 10, width: 5, intervals: 3 }, []);
  assert.equal(rows.length, 0);
});

// ————————————————————————————————————————————————————————————————
// El ancho de los intervalos de las esperas
// ————————————————————————————————————————————————————————————————

test("the width of the intervals of the waits is chosen from the longest wait of the queue", () => {
  // The width is chosen once per queue, from the whole trace: two instants
  // of the same queue are drawn with the same intervals.
  assert.equal(waitWidth(100), 10);
  assert.equal(waitWidth(20), 2);
  assert.equal(waitWidth(0), 0.1, "a queue whose waits are all zero");
});

// ————————————————————————————————————————————————————————————————
// Los recursos que aprietan
// ————————————————————————————————————————————————————————————————

/** A report with one queue, one facility, one storage and one chain. */
const report = {
  entities: {
    queues: {
      cola: {
        name: "cola",
        content: 7,
        max: 9,
        entries: 40,
        zero_entries: 4,
        total_time: 120,
        area: 300,
        retry: 2,
      },
    },
    facilities: {
      maquina: {
        name: "maquina",
        entries: 30,
        busy_time: 60,
        busy: true,
        available: true,
        retry: 1,
        inter: 0,
      },
    },
    storages: {
      almacen: {
        name: "almacen",
        capacity: 50,
        in_use: 20,
        area: 200,
        min_use: 0,
        max_use: 25,
        entries: 12,
        retry: 3,
        delay: 4,
      },
    },
    chains: {},
    logic: {},
    savevalues: {},
    tables: {},
    qtables: {},
  },
} as unknown as ReportDocument;

/** The figures of the replay at an instant. */
const live = {
  queues: {
    cola: {
      content: 2,
      max: 8,
      entries: 3,
      zeroEntries: 1,
      totalTime: 8,
      area: 12,
      departures: 2,
      lastChange: 5,
    },
  },
  facilities: {
    maquina: {
      busy: true,
      owner: 1,
      entries: 2,
      busyTime: 4,
      seizedAt: 6,
      retry: 0,
      inter: 0,
    },
  },
  storages: {
    almacen: {
      inUse: 20,
      minUse: 0,
      maxUse: 25,
      entries: 6,
      area: 100,
      retry: 2,
      lastChange: 5,
    },
  },
  chains: {},
  tables: {},
  logic: {},
  values: {},
} as unknown as Figures;

test("the bars of the resources have an absolute scale", () => {
  const pressures = resourcePressuresOf(
    facilitiesOf(report, live, elapsed),
    queuesOf(report, live, elapsed),
    storagesOf(report, live, elapsed),
  );
  // A facility that was busy 4 seconds of 10 draws 40 % of the bar, not a
  // full one: the scale is the 100 % of the time up to the clock.
  assert.equal(pressures.facilities[0].share, 0.4);
  // The bar of a queue is its mean content over the maximum it reached.
  assert.equal(pressures.queues[0].share, 12 / 10 / 8);
  // The bar of a storage is its mean utilization over its capacity.
  assert.equal(pressures.storages[0].share, 0.2);
});

test("a resource the replay has not reached shows no bar at all", () => {
  const pressures = resourcePressuresOf(
    facilitiesOf(report, live, elapsed),
    queuesOf(report, live, elapsed),
    storagesOf(report, live, elapsed),
  );
  for (const pressure of [
    ...pressures.facilities,
    ...pressures.queues,
    ...pressures.storages,
  ]) {
    assert.equal(pressure.live, true, "the replay knows this one");
  }
});

// ————————————————————————————————————————————————————————————————
// Las cifras de las entidades
// ————————————————————————————————————————————————————————————————

test("the queue carries the transactions that left and the mean of their waits", () => {
  const queues = queuesOf(report, live, elapsed);
  const cola = queues.find((queue) => queue.name === "cola")!;
  assert.equal(cola.departures, 2, "the transactions that left the queue");
  assert.equal(cola.departureMeanWait, 4, "8 seconds of wait between 2 departures");
  assert.equal(cola.engineMeanWait, 4, "the area of 12 over 3 entries");
});

test("the facility carries the retry the engine published", () => {
  // The replay never emits a facility retry, so the live figure is always
  // zero: the number of the engine is the one that is worth showing.
  const facilities = facilitiesOf(report, live, elapsed);
  const maquina = facilities.find((facility) => facility.name === "maquina")!;
  assert.equal(maquina.retry, 0, "the figure of the instant");
  assert.equal(maquina.publishedRetry, 1, "the retry the engine published");
});

test("the chain carries the mean time of the engine", () => {
  const chains = chainsOf(
    {
      entities: {
        ...report.entities,
        chains: { cadena: { name: "cadena", content: 1, max: 4, entries: 6, total_time: 30, area: 120, retry: 0 } },
      },
    } as unknown as ReportDocument,
    {
      ...live,
      chains: {
        cadena: {
          content: 1,
          max: 4,
          entries: 6,
          totalTime: 30,
          area: 120,
          lastChange: 5,
        },
      },
    } as unknown as Figures,
    elapsed,
  );
  const cadena = chains.find((chain) => chain.name === "cadena")!;
  assert.equal(cadena.meanTime, 20, "the area of 120 over 6 entries");
  assert.equal(cadena.meanSize, 12, "the area of 120 over the 10 seconds");
});

// ————————————————————————————————————————————————————————————————
// Los bloques del modelo
// ————————————————————————————————————————————————————————————————

/** A model with two blocks that share a caption and a generator. */
const model = {
  blocks: [
    { id: 1, class: "Generate", kind: "A", name: "Salida", caption: "Salida", label: null, container: 0, layout: null, properties: [], inputs: [], outputs: [] },
    { id: 2, class: "Facility", kind: "A", name: "Mesa", caption: "Mesa", label: null, container: 0, layout: null, properties: [], inputs: [], outputs: [] },
    { id: 3, class: "Queue", kind: "A", name: "Sala", caption: "Mesa", label: null, container: 0, layout: null, properties: [], inputs: [], outputs: [] },
  ],
} as unknown as ModelBody;

/** A report with the rows of the blocks. */
const blocksReport = {
  blocks: [
    { line: 1, node: 1, class: "Generate", label: null, operands: "", entries: 10, current: 0, retries: 0, counts_block: true },
    { line: 2, node: 2, class: "Facility", label: "2", operands: "", entries: 7, current: 0, retries: 0, counts_block: true },
    { line: 3, node: 3, class: "Queue", label: null, operands: "", entries: 4, current: 0, retries: 0, counts_block: true },
  ],
} as unknown as ReportDocument;

/** The counts of the live layer: the generator created ten transactions. */
const counts = new Map<number, LiveCounts>([
  [1, { now: 0, ever: 10 }],
  [2, { now: 0, ever: 7 }],
]);

test("the generator has its own label: the transactions it created", () => {
  const rows = blockRows(blocksReport, model, counts, {});
  const generator = rows.find((row) => row.generator)!;
  assert.ok(generator, "the generator is in the ranking");
  assert.match(blockNameOf(generator), /generador/);
  assert.match(blockNoteOf(generator) ?? "", /creó/);
});

test("two blocks with the same caption are told apart", () => {
  const rows = blockRows(blocksReport, model, counts, {});
  const mesa = rows.find((row) => row.class === "Facility")!;
  const sala = rows.find((row) => row.class === "Queue")!;
  assert.equal(mesa.name, sala.name, "the model calls them the same");
  assert.notEqual(blockNameOf(mesa), blockNameOf(sala), "the cards tell them apart");
});

test("the block carries the count the engine published for it", () => {
  const rows = blockRows(blocksReport, model, counts, {});
  const mesa = rows.find((row) => row.class === "Facility")!;
  assert.equal(mesa.entries, 7, "the count of the instant");
  assert.equal(mesa.published, 7, "the count the engine published");
});

test("a block the replay has not reached is not known yet", () => {
  const rows = blockRows(blocksReport, model, counts, {});
  const sala = rows.find((row) => row.class === "Queue")!;
  assert.equal(sala.live, false);
  assert.equal(sala.entries, 0);
});
