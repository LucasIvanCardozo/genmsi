//! The state of the run, as the panel shows it.
//!
//! The panel gathers everything that describes the run: what it is, where
//! the clock is, what governs the model and what the model had stored. The
//! numbers come from the report (the published ones) or from the instant
//! the reader is at (the ones of the replay).
//!
//! The rule this pins: what the replay has not reached yet is **not** a
//! zero. A savevalue the report knows and the replay has not reached says
//! "todavía no", and the switch the replay has not touched says the same.

import assert from "node:assert/strict";
import { test } from "node:test";
import {
  busiestQueueOf,
  runStateOf,
} from "../src/components/charts/runState.ts";
import type { QueueNumbers } from "../src/components/charts/entities.ts";
import { instantOf, type Figures } from "../src/monitor/stats.ts";
import type { ReportDocument } from "../src/types";

/** The report of the run, as the panel reads it. */
const report = {
  title: "Urgencias — turno de mañana",
  seed: 20260415,
  start_time: 0,
  end_time: 1200,
  end_reason: "time_limit",
  transactions: 1510,
  events: 48206,
  blocks: [],
  entities: {
    facilities: {},
    queues: {},
    storages: {
      Reactivos: {
        name: "Reactivos",
        capacity: 20,
        in_use: 14,
        area: 300,
        min_use: 0,
        max_use: 20,
        entries: 508,
        retry: 3,
        delay: 6,
      },
      Camillas: {
        name: "Camillas",
        capacity: 10,
        in_use: 2,
        area: 40,
        min_use: 0,
        max_use: 5,
        entries: 90,
        retry: 0,
        delay: 0,
      },
    },
    chains: {},
    logic: {
      "Urgencias abiertas": true,
      "Prioridad crítica": false,
      "Doble turno": true,
    },
    savevalues: {
      "mesas activas": 2,
      "reactivos por paciente": 1,
      "espera máxima": 300,
      "cadena «Paciente»": 0,
    },
    tables: {},
    qtables: {},
  },
  diagnostics: [],
} as unknown as ReportDocument;

/** The figures of the replay at the instant the reader is at. */
const figures = {
  queues: {},
  facilities: {},
  storages: {},
  chains: {},
  tables: {},
  logic: {
    "Urgencias abiertas": true,
    "Prioridad crítica": false,
  },
  values: {
    "mesas activas": 2,
    "reactivos por paciente": 1,
    "espera máxima": 300,
  },
} as unknown as Figures;

/** The instant of the reader: the clock is at the second 960. */
const instant = instantOf(
  { clock: 960, figures, population: 37, known: true },
  null,
  report.start_time,
  report.end_time,
);

/** The state of the run, as the panel shows it. */
const state = runStateOf({ report, instant, generated: 1204 });

test("the panel names the run: its title, its seed and why it ended", () => {
  assert.equal(state.title, "Urgencias — turno de mañana");
  assert.equal(state.seed, 20260415);
  assert.equal(state.duration, 1200, "the run lasted 1200 seconds");
  assert.equal(
    state.endReason,
    "por el tope de tiempo",
    "the reason of the engine, in the words of the app",
  );
});

test("the clock of the panel is the instant the reader is at", () => {
  assert.equal(state.clock, 960);
  assert.equal(state.known, true);
});

test("the transactions of the instant are not the published ones", () => {
  assert.equal(state.transactions, 1204, "the transactions of the instant");
  assert.equal(state.publishedTransactions, 1510, "the transactions of the run");
});

test("the waiting of the end is the sum of the delay of the storages", () => {
  assert.equal(state.waiting, 6, "six transactions were waiting for units");
  assert.deepEqual(state.waitingStorages, ["Reactivos"]);
});

test("a run that ended with nobody waiting says zero", () => {
  const withoutDelay = {
    ...report,
    entities: {
      ...report.entities,
      storages: {
        Reactivos: { ...report.entities.storages.Reactivos, delay: 0 },
      },
    },
  } as unknown as ReportDocument;
  const quiet = runStateOf({ report: withoutDelay, instant, generated: 1204 });
  assert.equal(quiet.waiting, 0);
  assert.deepEqual(quiet.waitingStorages, []);
});

test("the switches of the instant say which ones are on", () => {
  const open = state.switches.find((entry) => entry.name === "Urgencias abiertas");
  assert.equal(open?.on, true, "encendido");
  const critical = state.switches.find((entry) => entry.name === "Prioridad crítica");
  assert.equal(critical?.on, false, "apagado");
});

test("a switch the replay has not reached yet is not an apagado", () => {
  const double = state.switches.find((entry) => entry.name === "Doble turno");
  assert.equal(
    double?.on,
    null,
    "the replay has not reached it: it is not known, not off",
  );
});

test("the savevalues of the instant are the ones the replay knows", () => {
  const tables = state.values.find((entry) => entry.name === "mesas activas");
  assert.equal(tables?.value, 2);
  const wait = state.values.find((entry) => entry.name === "espera máxima");
  assert.equal(wait?.value, 300);
});

test("a savevalue the replay has not reached yet is not a zero", () => {
  const chain = state.values.find((entry) => entry.name === "cadena «Paciente»");
  assert.equal(
    chain?.value,
    null,
    "the replay has not reached it: it is not known, not zero",
  );
});

test("an instant that is not known shows no figure of the instant", () => {
  const unknown = instantOf(
    { clock: 960, figures, population: 37, known: false },
    null,
    report.start_time,
    report.end_time,
  );
  const blind = runStateOf({ report, instant: unknown, generated: 1204 });
  assert.equal(blind.known, false);
  assert.deepEqual(blind.switches, [], "no switch of an instant nobody knows");
  assert.deepEqual(blind.values, [], "no savevalue of an instant nobody knows");
});

test("the summary speaks of the queue that holds the most", () => {
  const wait = {
    name: "Sala de espera",
    content: 50,
    max: 87,
    entries: 340,
    zeroEntries: 20,
    totalTime: 20000,
    area: 36480,
    meanContent: 38,
    departures: 290,
    departureMeanWait: 69,
    engineMeanWait: 107.3,
    zeroShare: 0.059,
    live: true,
  } as QueueNumbers;
  const lab = {
    name: "Laboratorio",
    content: 17,
    max: 41,
    entries: 214,
    zeroEntries: 23,
    totalTime: 4000,
    area: 10944,
    meanContent: 11.4,
    departures: 191,
    departureMeanWait: 21,
    engineMeanWait: 51.1,
    zeroShare: 0.107,
    live: true,
  } as QueueNumbers;
  assert.equal(busiestQueueOf([lab, wait])?.name, "Sala de espera");
  assert.equal(busiestQueueOf([]), null, "a run without queues says nothing");
});
