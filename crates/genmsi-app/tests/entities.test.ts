//! The numbers of the entities, as the charts read them.
//!
//! The charts show the instant the reader is at: the numbers of the replay,
//! never the numbers the engine published for the whole run. A queue that
//! has not been touched yet shows zero, not the content it will have at the
//! end of the run.

import assert from "node:assert/strict";
import { test } from "node:test";
import {
  facilitiesOf,
  queuesOf,
  storagesOf,
  type Elapsed,
} from "../src/components/charts/entities.ts";
import type { Figures } from "../src/monitor/stats.ts";
import type { ReportDocument } from "../src/types";

const elapsed: Elapsed = { live: 10, published: 100 };

/** A report with one queue, one facility and one storage. */
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

/** The figures of the replay at an instant: one queue, one facility. */
const live = {
  queues: {
    cola: {
      content: 2,
      max: 2,
      entries: 3,
      zeroEntries: 1,
      totalTime: 8,
      area: 12,
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
  storages: {},
  chains: {},
  tables: {},
  logic: {},
  values: {},
} as unknown as Figures;

test("a queue the replay does not know yet shows zero, not the published numbers", () => {
  const queues = queuesOf(report, live, elapsed);
  const cola = queues.find((queue) => queue.name === "cola")!;
  assert.equal(cola.content, 2, "the content of the instant");
  assert.equal(cola.entries, 3, "the entries of the instant");
  assert.equal(cola.totalTime, 8, "the wait of the instant");
  assert.equal(cola.live, true);
});

test("an entity the replay has not touched shows zero", () => {
  const facilities = facilitiesOf(report, live, elapsed);
  const maquina = facilities.find((facility) => facility.name === "maquina")!;
  // The replay knows it: it took it twice.
  assert.equal(maquina.entries, 2);
  assert.equal(maquina.busyTime, 4);
  assert.equal(maquina.live, true);
});

test("a storage the replay has not touched shows zero and no published number", () => {
  const storages = storagesOf(report, live, elapsed);
  const almacen = storages.find((storage) => storage.name === "almacen")!;
  assert.equal(almacen.inUse, 0, "the units of the instant");
  assert.equal(almacen.area, 0, "the area of the instant");
  assert.equal(almacen.maxUse, 0, "the maximum of the instant");
  assert.equal(almacen.live, false, "the replay does not know it yet");
  assert.equal(almacen.capacity, 50, "the capacity is a property of the model");
});

test("the derived metrics of the instant are divided by the time of the instant", () => {
  const queues = queuesOf(report, live, elapsed);
  const cola = queues.find((queue) => queue.name === "cola")!;
  assert.equal(cola.meanContent, 1.2, "the area of 12 over the 10 seconds");
  assert.equal(cola.meanWait, 8 / 3);
});

test("the derived metrics of an entity that did not happen yet are zero", () => {
  const storages = storagesOf(report, live, elapsed);
  const almacen = storages.find((storage) => storage.name === "almacen")!;
  assert.equal(almacen.meanUtilization, 0, "no area, no utilization");
});
