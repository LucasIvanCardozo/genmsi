//! The figures of the run: the engine's own arithmetic.

import assert from "node:assert/strict";
import { test } from "node:test";
import { applyChange, closeFigures, emptyFigures } from "../src/monitor/stats.ts";

test("a queue that a transaction enters and leaves", () => {
  // The transaction enters at 0 and leaves at 10, having waited the 10
  // seconds it was in the queue. The area is the content times the time: the
  // content was 0 until the entry and 1 from then on, so the area is 1 * 10.
  let figures = emptyFigures();
  figures = applyChange(figures, { kind: "queue-enqueue", name: "cola", time: 0 });
  figures = applyChange(figures, { kind: "queue-dequeue", name: "cola", time: 10, waited: 10 });
  const queue = figures.queues["cola"];
  assert.equal(queue.content, 0);
  assert.equal(queue.max, 1);
  assert.equal(queue.entries, 1);
  assert.equal(queue.zeroEntries, 0);
  assert.equal(queue.totalTime, 10);
  assert.equal(queue.area, 10);
});

test("a transaction that waits no time at all", () => {
  let figures = emptyFigures();
  figures = applyChange(figures, { kind: "queue-enqueue", name: "cola", time: 5 });
  figures = applyChange(figures, { kind: "queue-dequeue", name: "cola", time: 5, waited: 0 });
  const queue = figures.queues["cola"];
  assert.equal(queue.zeroEntries, 1, "the wait was zero");
  assert.equal(queue.totalTime, 0);
  assert.equal(queue.area, 0, "the content was there for no time");
});

test("the area of a queue is the integral of its content", () => {
  // One transaction enters at 0 and another at 2. The first leaves at 4.
  // The content is 1 from 0 to 2, 2 from 2 to 4: the area is 1*2 + 2*2 = 6.
  let figures = emptyFigures();
  figures = applyChange(figures, { kind: "queue-enqueue", name: "cola", time: 0 });
  figures = applyChange(figures, { kind: "queue-enqueue", name: "cola", time: 2 });
  figures = applyChange(figures, { kind: "queue-dequeue", name: "cola", time: 4, waited: 4 });
  const queue = figures.queues["cola"];
  assert.equal(queue.content, 1);
  assert.equal(queue.max, 2);
  assert.equal(queue.entries, 2);
  assert.equal(queue.area, 6);
});

test("a facility that is taken and released", () => {
  // The facility is held from 0 to 6: six seconds of busy time.
  let figures = emptyFigures();
  figures = applyChange(figures, { kind: "facility-seize", name: "recepcion", time: 0, xact: 7 });
  figures = applyChange(figures, { kind: "facility-release", name: "recepcion", time: 6 });
  const facility = figures.facilities["recepcion"];
  assert.equal(facility.busy, false);
  assert.equal(facility.owner, null);
  assert.equal(facility.entries, 1);
  assert.equal(facility.busyTime, 6);
  assert.equal(facility.inter, 0);
});

test("a facility that is taken by force", () => {
  // The engine restarts the clock of the facility when it is taken by
  // force, and counts the interruption only when there was a holder.
  let figures = emptyFigures();
  figures = applyChange(figures, { kind: "facility-seize", name: "recepcion", time: 0, xact: 7 });
  figures = applyChange(figures, {
    kind: "facility-preempt",
    name: "recepcion",
    time: 2,
    xact: 9,
    interrupted: true,
  });
  let facility = figures.facilities["recepcion"];
  assert.equal(facility.owner, 9, "the new holder has it");
  assert.equal(facility.busy, true);
  assert.equal(facility.inter, 1, "the holder was interrupted");
  assert.equal(facility.entries, 2, "taking it by force is taking it");
  // The busy time is measured from the preemption, not from the first seizure.
  figures = applyChange(figures, { kind: "facility-release", name: "recepcion", time: 5 });
  facility = figures.facilities["recepcion"];
  assert.equal(facility.busyTime, 3);
});

test("a facility that is taken by force with nobody holding it", () => {
  let figures = emptyFigures();
  figures = applyChange(figures, {
    kind: "facility-preempt",
    name: "recepcion",
    time: 3,
    xact: 9,
    interrupted: false,
  });
  const facility = figures.facilities["recepcion"];
  assert.equal(facility.inter, 0, "nobody was interrupted");
  assert.equal(facility.entries, 1);
});

test("a facility that could not be taken", () => {
  let figures = emptyFigures();
  figures = applyChange(figures, { kind: "facility-retry", name: "recepcion" });
  assert.equal(figures.facilities["recepcion"].retry, 1);
});

test("a storage that takes and gives units", () => {
  // Three units are taken at 0 and two are given back at 5: the storage
  // holds 3 units for 5 seconds, so the area is 3 * 5.
  let figures = emptyFigures();
  figures = applyChange(figures, { kind: "storage-enter", name: "almacen", time: 0, units: 3 });
  figures = applyChange(figures, { kind: "storage-leave", name: "almacen", time: 5, units: 2 });
  const storage = figures.storages["almacen"];
  assert.equal(storage.inUse, 1);
  assert.equal(storage.maxUse, 3);
  // The engine starts the minimum at zero, because the storage starts empty:
  // the minimum content it ever had is the empty one.
  assert.equal(storage.minUse, 0);
  assert.equal(storage.entries, 1);
  assert.equal(storage.area, 15);
});

test("a storage whose units do not fit", () => {
  let figures = emptyFigures();
  figures = applyChange(figures, { kind: "storage-retry", name: "almacen" });
  assert.equal(figures.storages["almacen"].retry, 1);
});

test("a chain that a transaction joins and leaves", () => {
  // The transaction joins at 0 and leaves at 8: the chain holds one for
  // eight seconds.
  let figures = emptyFigures();
  figures = applyChange(figures, { kind: "chain-enter", name: "cadena", time: 0, xact: 3 });
  figures = applyChange(figures, {
    kind: "chain-leave",
    name: "cadena",
    time: 8,
    linked: [0],
  });
  const chain = figures.chains["cadena"];
  assert.equal(chain.content, 0);
  assert.equal(chain.max, 1);
  assert.equal(chain.entries, 1);
  assert.equal(chain.totalTime, 8);
  assert.equal(chain.area, 8);
});

test("a chain that loses several members at once", () => {
  // The engine takes the members out and says which ones: the chain holds
  // two of them and the time of each is added up.
  let figures = emptyFigures();
  figures = applyChange(figures, { kind: "chain-enter", name: "cadena", time: 0, xact: 3 });
  figures = applyChange(figures, { kind: "chain-enter", name: "cadena", time: 2, xact: 4 });
  figures = applyChange(figures, {
    kind: "chain-leave",
    name: "cadena",
    time: 6,
    linked: [0, 2],
  });
  const chain = figures.chains["cadena"];
  assert.equal(chain.content, 0, "both members left");
  assert.equal(chain.entries, 2);
  assert.equal(chain.totalTime, 10, "six seconds and four seconds");
});

test("a table that tabulates values", () => {
  // The values fall in the intervals the engine's own arithmetic says: the
  // first value is in the first interval, and the next one is in the
  // second.
  let figures = emptyFigures();
  figures = applyChange(figures, {
    kind: "table-sample",
    name: "tabla",
    time: 0,
    value: 3,
    index: 1,
    intervals: 4,
  });
  figures = applyChange(figures, {
    kind: "table-sample",
    name: "tabla",
    time: 1,
    value: 9,
    index: 3,
    intervals: 4,
  });
  const table = figures.tables["tabla"];
  assert.equal(table.samples, 2);
  assert.equal(table.maxValue, 9);
  assert.deepEqual(table.frequencies, [0, 1, 0, 1]);
});

test("a switch that is set and a value that is assigned", () => {
  let figures = emptyFigures();
  figures = applyChange(figures, { kind: "logic-set", name: "S1", set: true });
  figures = applyChange(figures, { kind: "value-set", name: "V1", value: 42 });
  assert.equal(figures.logic["S1"], true);
  assert.equal(figures.values["V1"], 42);
});

test("closing the figures closes the integrals that are open", () => {
  // A queue that still holds a transaction and a facility that is still
  // held: at the end of the run both close their integral.
  let figures = emptyFigures();
  figures = applyChange(figures, { kind: "queue-enqueue", name: "cola", time: 0 });
  figures = applyChange(figures, { kind: "facility-seize", name: "recepcion", time: 0, xact: 7 });
  const closed = closeFigures(figures, 10);
  assert.equal(closed.queues["cola"].area, 10, "the queue held one for ten seconds");
  assert.equal(closed.facilities["recepcion"].busyTime, 10);
  // The figures that were given are not touched.
  assert.equal(figures.queues["cola"].area, 0, "the original was not changed");
});

test("the figures that did not change keep their identity", () => {
  let figures = emptyFigures();
  figures = applyChange(figures, { kind: "queue-enqueue", name: "cola", time: 0 });
  figures = applyChange(figures, { kind: "queue-enqueue", name: "otra", time: 0 });
  const before = figures.queues["cola"];
  figures = applyChange(figures, { kind: "queue-dequeue", name: "otra", time: 1, waited: 1 });
  assert.equal(figures.queues["cola"], before, "the queue that did not move is the same one");
  assert.notEqual(figures.queues["otra"], before);
});

test("the figures that were given are not changed", () => {
  const figures = emptyFigures();
  const after = applyChange(figures, { kind: "queue-enqueue", name: "cola", time: 0 });
  assert.equal(figures.queues["cola"], undefined, "the original has no queue yet");
  assert.equal(after.queues["cola"].entries, 1);
});

test("a time that goes backwards is not counted", () => {
  // The engine never does this, but the arithmetic is the engine's: the
  // difference is never negative.
  let figures = emptyFigures();
  figures = applyChange(figures, { kind: "queue-enqueue", name: "cola", time: 10 });
  figures = applyChange(figures, { kind: "queue-dequeue", name: "cola", time: 5, waited: 0 });
  assert.equal(figures.queues["cola"].area, 0, "the area does not go back");
});

test("a change of a kind that does not exist leaves the figures alone", () => {
  const figures = emptyFigures();
  const after = applyChange(figures, { kind: "unknown" } as never);
  assert.equal(after, figures);
});
