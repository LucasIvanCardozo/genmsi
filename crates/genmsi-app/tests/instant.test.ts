//! The instant the charts show: the clock and the figures of one instant.
//!
//! The bug this pins: the charts took the clock of the live layer and the
//! figures of the replay at the playhead, and the playhead does not move
//! while the reproduction runs. Closing the frozen figures at the clock
//! that advances made the utilization of a facility that was held tend to
//! a hundred percent, and the mean content of a queue decay towards zero.
//!
//! The live layer publishes the clock and the figures of the frame the
//! replay reached, together: the snapshot below is what it carries, and
//! the instant is built out of it.

import assert from "node:assert/strict";
import { test } from "node:test";
import {
  buildIndex,
  frameAt,
  stateAt,
  type ModelDocumentLike,
  type ReportDocumentLike,
  type TraceDocumentLike,
  type TraceEventLike,
} from "../src/monitor/replay.ts";
import { instantOf, populationOf } from "../src/monitor/stats.ts";
import { facilitiesOf, queuesOf } from "../src/components/charts/entities.ts";
import type { ReportDocument } from "../src/types";

/** One trace event. */
function event(
  time: number,
  xact: number,
  block: number,
  className: string,
  name: string,
): TraceEventLike {
  return { time, xact, block, class: className, label: null, event: name, detail: "" };
}

/** The model of the run: a queue, a depart, a seize and a release. */
const model: ModelDocumentLike = {
  blocks: [
    { id: 1, properties: [{ name: "Queue", value: "qA" }] },
    { id: 2, properties: [{ name: "Queue", value: "qA" }] },
    { id: 3, properties: [{ name: "Facility", value: "F" }] },
    { id: 4, properties: [{ name: "Facility", value: "F" }] },
  ],
};

const report: ReportDocumentLike = {
  blocks: [
    { line: 1, node: 1, class: "Queue", current: 0, entries: 3 },
    { line: 2, node: 2, class: "Depart", current: 0, entries: 3 },
    { line: 3, node: 3, class: "Seize", current: 0, entries: 1 },
    { line: 4, node: 4, class: "Release", current: 0, entries: 1 },
  ],
  entities: { queues: { qA: { name: "qA", content: 0 } } },
};

/**
 * The run of the test.
 *
 * The queue is filled and emptied twice (at the seconds 0 and 2), the
 * facility is taken at the fourth second and released at the sixth, and the
 * queue is filled again at the seventh.
 */
const trace: TraceDocumentLike = {
  start_time: 0,
  end_time: 20,
  limit: 100,
  records: 7,
  truncated: false,
  events: [
    event(0, 1, 0, "Queue", "enter"),
    event(1, 1, 1, "Depart", "enter"),
    event(2, 2, 0, "Queue", "enter"),
    event(3, 2, 1, "Depart", "enter"),
    event(4, 3, 2, "Seize", "leave"),
    event(6, 3, 3, "Release", "enter"),
    event(7, 4, 0, "Queue", "enter"),
  ],
};

/** The report of the run, as the charts read it. */
const charts = {
  entities: {
    queues: {
      qA: {
        name: "qA",
        content: 0,
        max: 1,
        entries: 3,
        zero_entries: 0,
        total_time: 2,
        area: 2,
        retry: 0,
      },
    },
    facilities: {
      F: {
        name: "F",
        entries: 1,
        busy_time: 2,
        busy: false,
        retry: 0,
        inter: 0,
      },
    },
    storages: {},
    chains: {},
    logic: {},
    savevalues: {},
    tables: {},
    qtables: {},
  },
} as unknown as ReportDocument;

/** The frame the playhead is at: the facility has just been taken. */
const PLAYHEAD = 4;

/** The clocks the reader goes through while the run is replayed. */
const CLOCKS = [4, 6, 7, 10, 19];

/**
 * The time the facility was busy, up to the clock.
 *
 * The engine's own arithmetic: it was taken at the fourth second and
 * released at the sixth.
 */
function busyTimeOf(clock: number): number {
  return Math.max(0, Math.min(clock, 6) - 4);
}

/**
 * The area the queue covered, up to the clock.
 *
 * The engine's own arithmetic: the queue held one unit in [0,1), in [2,3)
 * and in [7,∞).
 */
function areaOf(clock: number): number {
  return Math.min(clock, 1) + Math.max(0, Math.min(clock, 3) - 2) + Math.max(0, clock - 7);
}

/**
 * The instant the charts are at, at a clock of the run.
 *
 * The live layer carries the clock and the figures of the frame the replay
 * reached; the figures of the playhead are the ones the charts used before
 * the fix, and they do not move while the reproduction runs.
 */
function instantAt(clock: number): ReturnType<typeof instantOf> {
  const index = buildIndex(trace, report, model);
  const frame = frameAt(index, clock);
  const state = stateAt(index, frame);
  const playhead = stateAt(index, PLAYHEAD);
  return instantOf(
    {
      clock,
      figures: state.figures,
      population: populationOf(state.blocks),
      known: true,
    },
    { figures: playhead.figures, time: playhead.time },
    trace.start_time,
    trace.end_time,
  );
}

test("the utilization of the charts is the engine's at every instant", () => {
  for (const clock of CLOCKS) {
    const instant = instantAt(clock);
    const facilities = facilitiesOf(charts, instant.figures, {
      live: instant.live,
      published: instant.published,
    });
    const machine = facilities.find((facility) => facility.name === "F")!;
    assert.equal(
      machine.utilization,
      busyTimeOf(clock) / clock,
      `the utilization at the second ${clock}`,
    );
  }
});

test("the utilization of a facility that was released does not tend to a hundred percent", () => {
  // The clock goes on after the facility was released: the utilization goes
  // down, it does not go up. The frozen figures of the playhead would say
  // that the facility was still held, and the utilization would grow with
  // the clock until it was a hundred percent.
  const instant = instantAt(19);
  const facilities = facilitiesOf(charts, instant.figures, {
    live: instant.live,
    published: instant.published,
  });
  const machine = facilities.find((facility) => facility.name === "F")!;
  assert.equal(machine.utilization, 2 / 19, "the time it was busy over the clock");
  assert.ok(machine.utilization < 0.2, "the utilization does not tend to a hundred percent");
});

test("the mean content of a queue is its area over the time of the instant", () => {
  for (const clock of CLOCKS) {
    const instant = instantAt(clock);
    const queues = queuesOf(charts, instant.figures, {
      live: instant.live,
      published: instant.published,
    });
    const queue = queues.find((candidate) => candidate.name === "qA")!;
    assert.equal(
      queue.meanContent,
      areaOf(clock) / clock,
      `the mean content at the second ${clock}`,
    );
  }
});

test("the mean content of a queue does not decay towards zero", () => {
  // The queue was filled again at the seventh second: the mean content of
  // the instant says so. The frozen figures of the playhead would see an
  // empty queue and the mean would decay towards zero with the clock.
  const instant = instantAt(19);
  const queues = queuesOf(charts, instant.figures, {
    live: instant.live,
    published: instant.published,
  });
  const queue = queues.find((candidate) => candidate.name === "qA")!;
  assert.equal(queue.meanContent, 14 / 19, "the area of the queue over the clock");
  assert.ok(queue.meanContent > 0.5, "the mean content does not decay towards zero");
});

test("an instant that is not known derives nothing", () => {
  // The clock was moved by hand and the window has not published the
  // figures of that instant yet: no metric is derived from a pair that is
  // not of one instant.
  const index = buildIndex(trace, report, model);
  const state = stateAt(index, index.frames.length - 1);
  const instant = instantOf(
    { clock: 19, figures: state.figures, population: 3, known: false },
    { figures: state.figures, time: state.time },
    trace.start_time,
    trace.end_time,
  );
  assert.equal(instant.known, false);
  assert.equal(instant.figures, null, "there are no figures of that instant");
  assert.equal(instant.population, null, "and no population of it");
  assert.equal(instant.live, 0, "and no time to divide them by");
  const facilities = facilitiesOf(charts, instant.figures, {
    live: instant.live,
    published: instant.published,
  });
  const machine = facilities.find((facility) => facility.name === "F")!;
  assert.equal(machine.utilization, 0, "no metric is derived");
});
