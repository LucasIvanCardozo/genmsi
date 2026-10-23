//! The live layer: the store that carries the picture of the replay.

import assert from "node:assert/strict";
import { test } from "node:test";
import { useLive, type LiveBall, type LiveCounts } from "../src/monitor/live.ts";

/** The counts of a block. */
const counts = (now: number, ever: number): LiveCounts => ({ now, ever });

/** A ball on a link. */
const ball = (xact: number, progress: number): LiveBall => ({ xact, progress });

test("the store starts empty", () => {
  useLive.getState().reset();
  const state = useLive.getState();
  assert.equal(state.clock, 0);
  assert.equal(state.counts.size, 0);
  assert.equal(state.balls.size, 0);
});

test("the counts of a block are published", () => {
  useLive.getState().reset();
  useLive.getState().publish({
    clock: 12.5,
    counts: new Map([[7, counts(3, 145)]]),
    balls: new Map(),
  });
  const state = useLive.getState();
  assert.equal(state.clock, 12.5);
  assert.deepEqual(state.counts.get(7), { now: 3, ever: 145 });
});

test("a count that did not change keeps its identity", () => {
  useLive.getState().reset();
  useLive.getState().publish({
    clock: 1,
    counts: new Map([
      [1, counts(2, 10)],
      [2, counts(5, 50)],
    ]),
    balls: new Map(),
  });
  const before = useLive.getState().counts.get(1);
  // Only the second block moved.
  useLive.getState().publish({
    clock: 2,
    counts: new Map([
      [1, counts(2, 10)],
      [2, counts(6, 51)],
    ]),
    balls: new Map(),
  });
  const after = useLive.getState().counts;
  // The block that did not move keeps the very same object: that is what
  // keeps the graph from redrawing when the clock ticks.
  assert.equal(after.get(1), before, "the untouched count kept its identity");
  assert.deepEqual(after.get(2), { now: 6, ever: 51 });
});

test("a count that changed is a new one", () => {
  useLive.getState().reset();
  useLive.getState().publish({
    clock: 1,
    counts: new Map([[3, counts(1, 4)]]),
    balls: new Map(),
  });
  const before = useLive.getState().counts.get(3);
  useLive.getState().publish({
    clock: 2,
    counts: new Map([[3, counts(2, 5)]]),
    balls: new Map(),
  });
  const after = useLive.getState().counts.get(3);
  assert.notEqual(after, before, "the moved count is a new object");
  assert.deepEqual(after, { now: 2, ever: 5 });
});

test("a block that the run left behind goes away", () => {
  useLive.getState().reset();
  useLive.getState().publish({
    clock: 1,
    counts: new Map([
      [1, counts(1, 1)],
      [2, counts(1, 1)],
    ]),
    balls: new Map(),
  });
  useLive.getState().publish({
    clock: 2,
    counts: new Map([[1, counts(1, 1)]]),
    balls: new Map(),
  });
  assert.equal(useLive.getState().counts.has(2), false);
});

test("a link without a ball keeps no entry", () => {
  useLive.getState().reset();
  useLive.getState().publish({
    clock: 1,
    counts: new Map(),
    balls: new Map([["e1", [ball(4, 0.5)]]]),
  });
  const before = useLive.getState().balls;
  useLive.getState().publish({
    clock: 2,
    counts: new Map(),
    balls: new Map([["e1", [ball(4, 0.6)]]]),
  });
  const after = useLive.getState().balls;
  // The link's ball moved, so its list is a new one.
  assert.notEqual(after.get("e1"), before.get("e1"));
  assert.equal(after.get("e1")?.[0].progress, 0.6);
  // And the link that has no ball is not in the map at all, so the component
  // that draws it is not redrawn.
  assert.equal(after.size, 1);
});

test("a ball that is still the same keeps the list", () => {
  useLive.getState().reset();
  const balls = [ball(4, 0.5)];
  useLive.getState().publish({ clock: 1, counts: new Map(), balls: new Map([["e1", balls]]) });
  const before = useLive.getState().balls.get("e1");
  useLive.getState().publish({ clock: 2, counts: new Map(), balls: new Map([["e1", balls]]) });
  assert.equal(useLive.getState().balls.get("e1"), before);
});

test("a link that loses its ball goes away", () => {
  useLive.getState().reset();
  useLive.getState().publish({
    clock: 1,
    counts: new Map(),
    balls: new Map([["e1", [ball(4, 0.5)]]]),
  });
  useLive.getState().publish({ clock: 2, counts: new Map(), balls: new Map() });
  assert.equal(useLive.getState().balls.size, 0);
});

test("the clock always moves, even when nothing else does", () => {
  useLive.getState().reset();
  useLive.getState().publish({ clock: 1, counts: new Map(), balls: new Map() });
  useLive.getState().publish({ clock: 2, counts: new Map(), balls: new Map() });
  assert.equal(useLive.getState().clock, 2);
});

test("a reset empties the picture", () => {
  useLive.getState().reset();
  useLive.getState().publish({
    clock: 9,
    counts: new Map([[1, counts(1, 1)]]),
    balls: new Map([["e1", [ball(1, 0.2)]]]),
  });
  useLive.getState().reset();
  const state = useLive.getState();
  assert.equal(state.clock, 0);
  assert.equal(state.counts.size, 0);
  assert.equal(state.balls.size, 0);
});
