//! The balls that cross the links of the model.

import assert from "node:assert/strict";
import { test } from "node:test";
import { buildTransitions, type TraceDocumentLike } from "../src/monitor/replay.ts";
import {
  FLIGHT_SECONDS,
  MIN_FLIGHT,
  ballFade,
  firstMoveAfter,
  firstMoveAtOrAfter,
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

test("the move of a transaction born in a block flies the whole flight", () => {
  // A transaction born in a generator never entered a block: it appears and
  // leaves in the same instant, so it has no time in the block it came from.
  // Its ball leaves the block the whole flight before the instance reaches
  // the next one, so every ball crosses in the same time.
  const born: TraceDocumentLike = {
    start_time: 0,
    end_time: 100,
    limit: 100,
    records: 3,
    truncated: false,
    events: [
      { time: 10, xact: 1, block: 0, class: "Generate", label: null, event: "generate", detail: "" },
      { time: 10, xact: 1, block: 0, class: "Generate", label: null, event: "leave", detail: "-> 1" },
      { time: 10, xact: 1, block: 1, class: "Queue", label: "A", event: "enter", detail: "" },
    ],
  };
  const index = buildTransitions(born, nodeOf);
  assert.equal(index.moves.length, 1);
  assert.equal(index.moves[0].flight, FLIGHT_SECONDS);
  // And the ball is on screen the whole flight before the instance arrives:
  // it left the generator two seconds before the instance reached the queue.
  const travellers = travellersAt(index.moves, index.times, 9);
  assert.equal(travellers.length, 1);
  assert.equal(travellers[0].progress, 0.5);
});

test("a move of a transaction that waited less than the flight flies that wait", () => {
  // The instance was in the block for a second and a half: its ball takes
  // that time and no more.
  const waited: TraceDocumentLike = {
    start_time: 0,
    end_time: 100,
    limit: 100,
    records: 4,
    truncated: false,
    events: [
      { time: 0, xact: 1, block: 1, class: "Queue", label: "A", event: "enter", detail: "" },
      { time: 1.5, xact: 1, block: 1, class: "Queue", label: "A", event: "leave", detail: "-> 2" },
      { time: 1.5, xact: 1, block: 2, class: "Advance", label: "B", event: "enter", detail: "" },
    ],
  };
  const index = buildTransitions(waited, nodeOf);
  assert.equal(index.moves.length, 1);
  assert.equal(index.moves[0].flight, 1.5);
});

test("the moves of the run are read from the trace", () => {
  const index = buildTransitions(trace, nodeOf);
  // The queue to the advance, and the advance to the seize.
  assert.equal(index.moves.length, 2);
  assert.deepEqual(index.moves[0], { xact: 1, from: 1, to: 2, time: 0, flight: 0 });
  assert.deepEqual(index.moves[1], { xact: 1, from: 2, to: 3, time: 5, flight: 0 });
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

test("the search finds where the moves that are still to happen begin", () => {
  // The ball arrives with the transaction: the move that carries it happens
  // after the instant the ball is looked at.
  const times = [0, 0, 5, 5, 9, 9];
  assert.equal(firstMoveAtOrAfter(times, -1), 0);
  assert.equal(firstMoveAtOrAfter(times, 0), 0, "the move of this very instant");
  assert.equal(firstMoveAtOrAfter(times, 1), 2);
  assert.equal(firstMoveAtOrAfter(times, 5), 2);
  assert.equal(firstMoveAtOrAfter(times, 10), 6, "there is none left");
});

test("the ball lands exactly when the instance reaches its destination", () => {
  // The move happened at the instant the instance arrived: the ball is
  // already there, at the end of its flight.
  const moves: Move[] = [{ from: 1, to: 2, time: 10, xact: 7, flight: 1 }];

  const atArrival = travellersAt(moves, [10], 10);
  assert.equal(atArrival.length, 1);
  assert.equal(atArrival[0].progress, 1, "the ball is at its destination");
  assert.equal(atArrival[0].to, 2);
  assert.equal(atArrival[0].xact, 7, "the ball says which transaction it is");

  // Before that, the ball is on its way: it left a flight before the arrival.
  const onTheWay = travellersAt(moves, [10], 9.5);
  assert.equal(onTheWay.length, 1);
  assert.equal(onTheWay[0].progress, 0.5, "half way between the two blocks");
  assert.equal(onTheWay[0].from, 1);
  assert.equal(onTheWay[0].to, 2);
});

test("a ball that has not left yet is not on screen", () => {
  const moves: Move[] = [{ from: 1, to: 2, time: 10, xact: 7, flight: 1 }];
  // The flight is one second: the ball leaves at 9.
  assert.equal(travellersAt(moves, [10], 8.5).length, 0);
  assert.equal(travellersAt(moves, [10], 9).length, 1, "it just left");
});

test("a ball that has already arrived is not on the screen anymore", () => {
  // The ball arrives with the instance and vanishes there: it does not sit
  // on the edge of the block it reached.
  const moves: Move[] = [{ from: 1, to: 2, time: 10, xact: 7, flight: 1 }];
  assert.equal(travellersAt(moves, [10], 10.5).length, 0, "the ball is gone");
});

test("the flight of a ball is bounded on both sides", () => {
  // A transaction that waited an hour in the block it left: the ball does
  // not crawl for an hour.
  const waited: Move[] = [{ from: 1, to: 2, time: 3600, xact: 7, flight: 3600 }];
  const crawling = travellersAt(waited, [3600], 3600 - FLIGHT_SECONDS / 2);
  assert.equal(crawling.length, 1);
  assert.equal(crawling[0].flight, FLIGHT_SECONDS, "the flight is the longest one");

  // A transaction that passed through the block in no time: the ball still
  // flies, so the reader sees it leave and arrive.
  const instant: Move[] = [{ from: 1, to: 2, time: 10, xact: 7, flight: 0 }];
  const quick = travellersAt(instant, [10], 10);
  assert.equal(quick.length, 1);
  assert.equal(quick[0].flight, MIN_FLIGHT, "the flight is the shortest one");
  assert.equal(quick[0].progress, 1, "the ball arrives with the instance");
});

test("the ball crosses the link in the flight of its move", () => {
  // The flight of the move is the time the transaction spent in the block
  // it came from: the ball crosses the link in that time, at every speed.
  const moves: Move[] = [{ from: 1, to: 2, time: 10, xact: 7, flight: FLIGHT_SECONDS }];
  const half = travellersAt(moves, [10], 10 - FLIGHT_SECONDS / 2);
  assert.equal(half.length, 1);
  // The progress is a fraction of the flight: half the flight, half way.
  assert.ok(
    Math.abs(half[0].progress - 0.5) < 1e-9,
    `half way at half the flight, got ${half[0].progress}`,
  );
});

test("the ball shrinks as it reaches the block", () => {
  // At the start the ball is whole: the reader must see it leave the block
  // the transaction came from. Over the last fifth of the flight it shrinks,
  // so it vanishes as it arrives instead of sitting on the edge.
  assert.equal(ballFade(0), 1, "the ball leaves whole");
  assert.equal(ballFade(0.79), 1, "the ball is whole until the last fifth");
  assert.equal(ballFade(0.8), 1, "the fade starts at the last fifth");
  assert.ok(Math.abs(ballFade(0.9) - 0.5) < 1e-9, "half way through the fade");
  assert.equal(ballFade(1), 0, "the ball vanishes as it arrives");
});

test("a move of a big run is found without walking them all", () => {
  // The cost of a frame is the number of balls, not the number of moves: the
  // search finds where to start.
  const count = 30_000;
  const times: number[] = [];
  const moves: Move[] = [];
  for (let i = 0; i < count; i += 1) {
    times.push(i);
    moves.push({ from: 1, to: 2, time: i, xact: i, flight: 2 });
  }
  const started = performance.now();
  const travellers = travellersAt(moves, times, 29_998.5);
  const elapsed = performance.now() - started;
  // The moves of the next instants are the only ones on screen: the one
  // that is on the way, and the one that has not left yet.
  assert.equal(travellers.length, 1, `${travellers.length} balls on screen`);
  assert.equal(travellers[0].xact, 29_999, "the ball is the one that is on the way");
  assert.ok(
    Math.abs(travellers[0].progress - 0.75) < 1e-9,
    `three quarters of the flight, got ${travellers[0].progress}`,
  );
  assert.ok(elapsed < 50, `the frame took ${elapsed} ms`);
});
