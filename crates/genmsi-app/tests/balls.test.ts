//! The balls that cross the links of the model.

import assert from "node:assert/strict";
import { test } from "node:test";
import { buildTransitions, type TraceDocumentLike } from "../src/monitor/replay.ts";
import {
  FLIGHT_SECONDS,
  firstMoveAfter,
  travellersAt,
  type Move,
} from "../src/monitor/balls.ts";

/** A trace with the moves of two transactions. */
const trace: TraceDocumentLike = {
  start_time: 0,
  end_time: 100,
  limit: 100,
  records: 8,
  truncated: false,
  events: [
    { time: 0, xact: 1, block: 0, class: "Generate", label: null, event: "generate", detail: "" },
    { time: 0, xact: 1, block: 1, class: "Queue", label: "A", event: "enter", detail: "" },
    // The move from the queue to the advance: the trace says where it goes.
    { time: 0, xact: 1, block: 1, class: "Queue", label: "A", event: "leave", detail: "-> 2" },
    { time: 5, xact: 1, block: 2, class: "Advance", label: "B", event: "enter", detail: "" },
    { time: 5, xact: 1, block: 2, class: "Advance", label: "B", event: "leave", detail: "-> 3" },
    { time: 9, xact: 1, block: 3, class: "Seize", label: "C", event: "enter", detail: "" },
    // A move that stays inside the same block of the model: not visible.
    { time: 9, xact: 1, block: 3, class: "Seize", label: "C", event: "leave", detail: "-> 4" },
    { time: 9, xact: 1, block: 4, class: "Depart", label: "C", event: "enter", detail: "" },
  ],
};

/** The program blocks of the model: 3 and 4 are one block of the model. */
const nodeOf = (block: number) => {
  if (block === 3 || block === 4) {
    return 3;
  }
  return block;
};

test("the moves of the run are read from the trace", () => {
  const index = buildTransitions(trace, nodeOf);
  // The queue to the advance, and the advance to the seize.
  assert.equal(index.moves.length, 2);
  assert.deepEqual(index.moves[0], { xact: 1, from: 1, to: 2, time: 0 });
  assert.deepEqual(index.moves[1], { xact: 1, from: 2, to: 3, time: 5 });
});

test("a move inside the same block of the model is not a move the reader sees", () => {
  const index = buildTransitions(trace, nodeOf);
  // The seize to the depart is one move of the trace, but both belong to the
  // same block of the model, so no ball crosses for it.
  assert.equal(index.moves.some((move) => move.from === 3 && move.to === 3), false);
});

test("the moves are ordered by the time they happened", () => {
  const index = buildTransitions(trace, nodeOf);
  const times = index.times;
  for (let i = 1; i < times.length; i += 1) {
    assert.ok(times[i] >= times[i - 1], `${times[i]} after ${times[i - 1]}`);
  }
});

test("the search finds where the moves of an instant begin", () => {
  const times = [0, 0, 5, 5, 9, 9];
  assert.equal(firstMoveAfter(times, -1), 0);
  assert.equal(firstMoveAfter(times, 0), 2);
  assert.equal(firstMoveAfter(times, 4), 2);
  assert.equal(firstMoveAfter(times, 5), 4);
  assert.equal(firstMoveAfter(times, 100), 6);
});

test("the ball lands exactly when the instance reaches its destination", () => {
  // The move happened at the instant the instance arrived: the ball is
  // already there, at the end of its flight.
  const moves: Move[] = [{ from: 1, to: 2, time: 10, xact: 7 }];

  const atArrival = travellersAt(moves, [10], 10, 1);
  assert.equal(atArrival.length, 1);
  assert.equal(atArrival[0].progress, 1, "the ball is at its destination");
  assert.equal(atArrival[0].to, 2);
  assert.equal(atArrival[0].xact, 7, "the ball says which transaction it is");

  // Before that, the ball is on its way: it left a flight before the arrival.
  const onTheWay = travellersAt(moves, [10], 9.5, 1);
  assert.equal(onTheWay.length, 1);
  assert.equal(onTheWay[0].progress, 0.5, "half way between the two blocks");
  assert.equal(onTheWay[0].from, 1);
  assert.equal(onTheWay[0].to, 2);
});

test("a ball that has not left yet is not on screen", () => {
  const moves: Move[] = [{ from: 1, to: 2, time: 10, xact: 7 }];
  // The flight is one second: the ball leaves at 9.
  assert.equal(travellersAt(moves, [10], 8.5, 1).length, 0);
  assert.equal(travellersAt(moves, [10], 9, 1).length, 1, "it just left");
});

test("a move of a big run is found without walking them all", () => {
  // The cost of a frame is the number of balls, not the number of moves: the
  // search finds where to start.
  const count = 30_000;
  const times: number[] = [];
  const moves: Move[] = [];
  for (let i = 0; i < count; i += 1) {
    times.push(i);
    moves.push({ from: 1, to: 2, time: i, xact: i });
  }
  const started = performance.now();
  const travellers = travellersAt(moves, times, 29_999.1, 2);
  const elapsed = performance.now() - started;
  // The moves of the last instants are the only ones on screen: the one that
  // just landed, and the one that is about to.
  assert.ok(travellers.length <= 2, `${travellers.length} balls on screen`);
  assert.ok(elapsed < 50, `the frame took ${elapsed} ms`);
});

test("the flight is a fixed stretch of the run at every speed", () => {
  // The ball crosses in 0.2 s of the run, so the faster the reader runs the
  // reproduction, the quicker the ball crosses: 200 ms of screen at 1x, 100 ms
  // at 2x.
  const moves: Move[] = [{ from: 1, to: 2, time: 10, xact: 7 }];
  const half = travellersAt(moves, [10], 10 - FLIGHT_SECONDS / 2, FLIGHT_SECONDS);
  assert.equal(half.length, 1);
  // The progress is a fraction of the flight: half the flight, half way.
  assert.ok(
    Math.abs(half[0].progress - 0.5) < 1e-9,
    `half way at half the flight, got ${half[0].progress}`,
  );
});
