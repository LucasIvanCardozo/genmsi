//! The population of the instant: the transactions inside the model.
//!
//! The card used to add up the blocks of the state of the replay at the
//! playhead, and the playhead does not move while the reproduction runs:
//! the card showed the population of a frame that had gone by. The
//! population comes from the live layer now, which publishes it with the
//! clock of the frame the replay reached.

import assert from "node:assert/strict";
import { test } from "node:test";
import {
  buildIndex,
  stateAt,
  type ModelDocumentLike,
  type ReportDocumentLike,
  type TraceDocumentLike,
  type TraceEventLike,
} from "../src/monitor/replay.ts";
import { emptyFigures, instantOf } from "../src/monitor/stats.ts";
import { useLive } from "../src/monitor/live.ts";

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

/** The model of the run: a queue and a depart. */
const model: ModelDocumentLike = {
  blocks: [
    { id: 1, properties: [{ name: "Queue", value: "qA" }] },
    { id: 2, properties: [{ name: "Queue", value: "qA" }] },
  ],
};

const report: ReportDocumentLike = {
  blocks: [
    { line: 1, node: 1, class: "Queue", current: 0, entries: 1 },
    { line: 2, node: 2, class: "Depart", current: 0, entries: 1 },
  ],
  entities: { queues: { qA: { name: "qA", content: 0 } } },
};

/**
 * The run of the test: a transaction enters the queue at the second 0,
 * enters the depart at the second 1 and leaves the model at the second 2.
 */
const trace: TraceDocumentLike = {
  start_time: 0,
  end_time: 10,
  limit: 100,
  records: 3,
  truncated: false,
  events: [
    event(0, 1, 0, "Queue", "enter"),
    event(1, 1, 1, "Depart", "enter"),
    event(2, 1, 1, "Depart", "leave"),
  ],
};

test("the live layer publishes the population with the clock", () => {
  useLive.getState().reset();
  // The picture carries the clock, the figures and the population of the
  // same frame: the instant is known.
  useLive.getState().publish({
    clock: 3,
    counts: new Map(),
    balls: new Map(),
    figures: emptyFigures(),
    population: 4,
  });
  assert.equal(useLive.getState().population, 4, "the population of the picture");
  assert.equal(useLive.getState().known, true, "the picture carried the figures and the population");

  // A picture without the population leaves the one of the last published
  // instant: the clock is not there yet, and the instant is not known.
  useLive.getState().publish({
    clock: 5,
    counts: new Map(),
    balls: new Map(),
    figures: emptyFigures(),
  });
  assert.equal(useLive.getState().known, false, "the picture carried no population");
});

test("the population of the instant is the one the live layer published", () => {
  const index = buildIndex(trace, report, model);
  // The transaction is inside the model at the second 0 and 1, and it left
  // it at the second 2.
  const state = stateAt(index, 0);
  // The playhead is at the end of the run: its population is zero, and it
  // must not be the one the card shows.
  const playhead = stateAt(index, index.frames.length - 1);
  const instant = instantOf(
    { clock: 0, figures: state.figures, population: 1, known: true },
    { figures: playhead.figures, time: playhead.time },
    trace.start_time,
    trace.end_time,
  );
  assert.equal(instant.population, 1, "the population of the instant");
  assert.equal(instant.known, true, "the instant is known");
});

test("an instant that is not known has no population", () => {
  // The clock was moved by hand and the window has not published the
  // figures of that instant yet: the card says so instead of showing a
  // number of another instant.
  const instant = instantOf(
    { clock: 5, figures: null, population: 4, known: false },
    null,
    trace.start_time,
    trace.end_time,
  );
  assert.equal(instant.known, false);
  assert.equal(instant.population, null, "there is no population of an instant nobody knows");
  assert.equal(instant.live, 0, "and no time to divide the numbers by");
});
