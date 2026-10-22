//! The numbers of the app, the way the original shows them.

import assert from "node:assert/strict";
import { test } from "node:test";
import { chronometer, number } from "../src/format.ts";

test("a duration reads as a clock", () => {
  assert.equal(chronometer(0), "0:00:00");
  assert.equal(chronometer(59), "0:00:59");
  assert.equal(chronometer(60), "0:01:00");
  assert.equal(chronometer(2399.673), "0:39:59", "the time of a one-hour run");
  assert.equal(chronometer(3600), "1:00:00");
  assert.equal(chronometer(28800), "8:00:00", "the run of TP10");
});

test("a duration that makes no sense reads as zero", () => {
  assert.equal(chronometer(-1), "0:00:00");
  assert.equal(chronometer(Number.NaN), "0:00:00");
});

test("the numbers keep the comma decimals of the original", () => {
  assert.equal(number(1.5), "1,500");
  assert.equal(number(42), "42");
});
