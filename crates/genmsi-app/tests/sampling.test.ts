//! The sampler of the live picture: what the charts show, and how often.
//!
//! The bug this pins: every picture the replay published reached the
//! charts, and every picture redrew the whole section. Sixty times a
//! second is what the reader cannot see and what a slow machine cannot
//! afford. The sampler keeps the newest picture and shows it at the rate
//! the reader asked for, and the picture that waits is never lost.

import assert from "node:assert/strict";
import { test } from "node:test";
import { Sampler } from "../src/monitor/sampling.ts";

test("a sampler that refreshes on every frame shows every value", () => {
  let clock = 0;
  const sampler = new Sampler<number>(0, () => clock);
  assert.equal(sampler.offer(1), 1, "the first value is due at once");
  clock = 16;
  assert.equal(sampler.offer(2), 2, "and so is the next one");
});

test("a sampler that refreshes every quarter second keeps the value that waits", () => {
  let clock = 0;
  const sampler = new Sampler<number>(250, () => clock);
  assert.equal(sampler.offer(1), 1, "the first value is due at once");
  clock = 100;
  assert.equal(sampler.offer(2), null, "the second one waits for its turn");
  assert.equal(sampler.due(), false, "its time has not come");
  clock = 250;
  assert.equal(sampler.due(), true, "and now it has");
  assert.equal(sampler.take(), 2, "the value that waits is not lost");
});

test("when several values wait, the newest one is the one shown", () => {
  let clock = 0;
  const sampler = new Sampler<number>(250, () => clock);
  sampler.offer(1);
  clock = 10;
  sampler.offer(2);
  clock = 20;
  sampler.offer(3);
  assert.equal(sampler.take(), 3, "the picture of the instant is the newest one");
});

test("a sampler with nothing waiting has nothing to take", () => {
  const sampler = new Sampler<number>(250, () => 0);
  assert.equal(sampler.take(), null);
  assert.equal(sampler.due(), false);
});
