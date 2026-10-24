import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  advanceClock,
  buildIndex,
  buildTransitions,
  checkFidelity,
  frameAt,
  mergeSeries,
  stateAt,
  type ModelDocumentLike,
  type ReportDocumentLike,
  type TraceDocumentLike,
  type TraceEventLike,
} from "../src/monitor/replay.ts";

const appDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = path.resolve(appDir, "../..");

const genmsiBin = process.env.GENMSI_BIN ?? path.join(repoRoot, "target", "debug", "genmsi");
const appEngineBin =
  process.env.GENMSI_APP_BIN ?? path.join(repoRoot, "target", "debug", "genmsi-app-engine");

/** The reference model of the app's slice. */
const model = "fixtures/models/TP6__Clase 6_1_PuertaBanco.gmdl";
/** The big model: its trace is the performance guard's case. */
const bigModel = "fixtures/models/Parcial 6-11-23__Veron - Parcial.gmdl";

/** Build the engine binaries the test reads. */
function ensureBinaries(): void {
  if (existsSync(genmsiBin) && existsSync(appEngineBin)) {
    return;
  }
  const build = spawnSync("cargo", ["build", "-q", "-p", "genmsi-cli", "-p", "genmsi-app-engine"], {
    cwd: repoRoot,
    stdio: "inherit",
  });
  assert.equal(build.status, 0, "the engine binaries could not be built");
}

/** The bytes the CLI prints. */
function cliBytes(args: string[]): Buffer {
  return execFileSync(genmsiBin, args, { cwd: repoRoot, maxBuffer: 64 * 1024 * 1024 });
}

/** The bytes the app's engine facade prints. */
function facadeBytes(args: string[]): Buffer {
  return execFileSync(appEngineBin, args, { cwd: repoRoot, maxBuffer: 64 * 1024 * 1024 });
}

/** The documents of one run: the model, the report and the trace. */
function documents(
  path: string,
  limit: number,
): {
  model: ModelDocumentLike;
  report: ReportDocumentLike;
  trace: TraceDocumentLike;
} {
  ensureBinaries();
  const modelDocument = JSON.parse(facadeBytes(["model", path]).toString("utf8"));
  const report = JSON.parse(cliBytes(["simulate", path, "--json", "--seed", "1"]).toString("utf8"));
  const trace = JSON.parse(cliBytes(["trace", path, "--json", "--limit", String(limit), "--seed", "1"]).toString("utf8"));
  return { model: modelDocument, report, trace };
}

/** One trace event. */
function event(
  time: number,
  xact: number,
  block: number,
  className: string,
  name: string,
  detail = "",
): TraceEventLike {
  return { time, xact, block, class: className, label: null, event: name, detail };
}

/** The small model of the unit tests: a queue, a depart, a seize and a release. */
const smallModel: ModelDocumentLike = {
  blocks: [
    { id: 1, properties: [{ name: "Queue", value: "qA" }] },
    { id: 2, properties: [{ name: "Queue", value: "qA" }] },
    { id: 3, properties: [{ name: "Facility", value: "F" }] },
    { id: 4, properties: [{ name: "Facility", value: "F" }] },
  ],
};

const smallReport: ReportDocumentLike = {
  blocks: [
    { line: 1, node: 1, class: "Queue", current: 0, entries: 1 },
    { line: 2, node: 2, class: "Depart", current: 0, entries: 1 },
    { line: 3, node: 3, class: "Seize", current: 0, entries: 1 },
    { line: 4, node: 4, class: "Release", current: 0, entries: 1 },
  ],
  entities: { queues: { qA: { name: "qA", content: 0 } } },
};

/** The events of the small run: a transaction queues, seizes and releases. */
const smallTrace: TraceDocumentLike = {
  start_time: 0,
  end_time: 10,
  limit: 100,
  records: 5,
  truncated: false,
  events: [
    event(0, 1, 0, "Queue", "enter"),
    event(1, 1, 1, "Depart", "enter"),
    event(2, 1, 2, "Seize", "enter"),
    event(3, 1, 2, "Seize", "leave"),
    event(4, 1, 3, "Release", "enter"),
  ],
};

test("the queue's content follows the queue and the depart blocks", () => {
  const index = buildIndex(smallTrace, smallReport, smallModel);
  const state = stateAt(index, 0);
  assert.equal(state.queues["qA"], 1, "the transaction is in the queue");
  assert.equal(stateAt(index, 1).queues["qA"], 0, "the transaction left the queue");
});

test("the facility's owner follows the seize and the release blocks", () => {
  const index = buildIndex(smallTrace, smallReport, smallModel);
  // The enter of the seize block changes nothing: the transaction may wait.
  assert.equal(stateAt(index, 2).facilities["F"], undefined);
  // The leave of the seize block is the moment the transaction takes it.
  assert.equal(stateAt(index, 3).facilities["F"], 1);
  assert.equal(stateAt(index, 4).facilities["F"], null, "the release frees it");
});

test("the state at an event is a pure function of the prefix of the trace", () => {
  const index = buildIndex(smallTrace, smallReport, smallModel);
  const full = stateAt(index, 3);

  // The same prefix, walked again, gives the same state.
  assert.deepEqual(stateAt(index, 3), full);

  // And the state does not depend on what comes after: a trace that stops at
  // the event gives the same state at that event.
  const prefix: TraceDocumentLike = { ...smallTrace, events: smallTrace.events.slice(0, 4) };
  const shorter = buildIndex(prefix, smallReport, smallModel);
  assert.deepEqual(stateAt(shorter, 3), full);
});

test("the same trace always gives the same replay", () => {
  const first = buildIndex(smallTrace, smallReport, smallModel);
  const second = buildIndex(smallTrace, smallReport, smallModel);
  assert.deepEqual(second.frames, first.frames);
  assert.deepEqual(second.queuePoints, first.queuePoints);
  assert.deepEqual(second.facilityPoints, first.facilityPoints);
});

test("the series of an entity tells every instant its value changes", () => {
  const index = buildIndex(smallTrace, smallReport, smallModel);
  const queue = index.queuePoints.get("qA") ?? [];
  assert.deepEqual(queue, [
    { time: 0, value: 1 },
    { time: 1, value: 0 },
  ]);
  const facility = index.facilityPoints.get("F") ?? [];
  assert.deepEqual(facility, [
    { time: 3, value: 1 },
    { time: 4, value: 0 },
  ]);
});

test("the rows of a chart carry the value each entity had at that instant", () => {
  const index = buildIndex(smallTrace, smallReport, smallModel);
  const rows = mergeSeries(["qA"], index.queuePoints);
  assert.deepEqual(rows, [
    { time: 0, qA: 1 },
    { time: 1, qA: 0 },
  ]);
  const facilities = mergeSeries(["F"], index.facilityPoints);
  assert.deepEqual(facilities, [
    { time: 3, F: 1 },
    { time: 4, F: 0 },
  ]);
});

test("a transaction that backs up is counted in the block it came from", () => {
  // The trace of a transaction that waits at a shut gate: the engine counts
  // it in the queue it came from, and the replay counts it there too.
  const report: ReportDocumentLike = {
    blocks: [
      { line: 1, node: 1, class: "Queue", current: 1, entries: 1 },
      { line: 2, node: 3, class: "Gate", current: 0, entries: 0 },
    ],
    entities: { queues: { qA: { name: "qA", content: 1 } } },
  };
  const trace: TraceDocumentLike = {
    start_time: 0,
    end_time: 10,
    limit: 100,
    records: 5,
    truncated: false,
    events: [
      event(0, 1, 0, "Queue", "enter", "qA"),
      event(1, 1, 0, "Queue", "leave", "-> 1"),
      event(1, 1, 1, "Gate", "enter"),
      event(1, 1, 1, "Gate", "wait", "gate"),
      event(1, 1, 1, "Gate", "backup", "-> 0"),
    ],
  };
  const index = buildIndex(trace, report, smallModel);
  const state = stateAt(index, index.frames.length - 1);
  assert.equal(state.blocks[0], 1, "the transaction is counted in the queue");
  assert.equal(state.blocks[1], 0, "and not in the gate");
  // The entities are not touched: the transaction neither entered a queue
  // nor left one when it backed up.
  assert.equal(state.queues["qA"], 1, "the queue kept its content");
  // The backup is a change of the occupation, and not an entry.
  const backup = index.frames[index.frames.length - 1];
  assert.equal(backup.event, "backup");
  assert.equal(backup.change?.left, 1, "the gate loses the transaction");
  assert.equal(backup.change?.entered, 0, "the queue gains it");
  assert.equal(backup.change?.entered_count, undefined, "and no entry is counted");
});

test("a transaction that waits for a facility takes it when it leaves", () => {
  // The transaction waits at the seize: the engine counts it in the block it
  // came from, and the trace says so with the `backup` event. When the
  // facility is released, the transaction leaves the block it is counted in,
  // and it took the facility: the entity belongs to the block it waited in.
  const report: ReportDocumentLike = {
    blocks: [
      { line: 1, node: 1, class: "Queue", current: 0, entries: 1 },
      { line: 2, node: 3, class: "Seize", current: 0, entries: 1 },
      { line: 3, node: 4, class: "Release", current: 0, entries: 1 },
    ],
    entities: { queues: { qA: { name: "qA", content: 1 } } },
  };
  const trace: TraceDocumentLike = {
    start_time: 0,
    end_time: 10,
    limit: 100,
    records: 7,
    truncated: false,
    events: [
      event(0, 1, 0, "Queue", "enter", "qA"),
      event(1, 1, 0, "Queue", "leave", "-> 1"),
      event(1, 1, 1, "Seize", "enter", "F"),
      event(1, 1, 1, "Seize", "wait", "F"),
      event(1, 1, 1, "Seize", "backup", "-> 0"),
      event(5, 1, 0, "Queue", "leave", "-> 2"),
      event(5, 1, 2, "Release", "enter", "F"),
    ],
  };
  const index = buildIndex(trace, report, smallModel);
  // The leave of the queue is the moment the transaction takes the facility.
  const taken = stateAt(index, 5);
  assert.equal(taken.facilities["F"], 1, "the transaction owns the facility");
  assert.equal(taken.blocks[0], 0, "the queue holds nobody");
  assert.equal(taken.blocks[1], 0, "and the seize holds nobody");
  // The release frees the facility again.
  const state = stateAt(index, index.frames.length - 1);
  assert.equal(state.facilities["F"], null, "the release frees the facility");
});

test("the replay of the reference model ends in the state the report publishes", { timeout: 300_000 }, () => {
  const documentsOf = documents(model, 200_000);
  const index = buildIndex(documentsOf.trace, documentsOf.report, documentsOf.model);
  const fidelity = checkFidelity(index, documentsOf.report);

  assert.deepEqual(fidelity.queueMismatches, [], "the queues differ");
  assert.deepEqual(fidelity.blockMismatches, [], "the blocks differ");
  assert.ok(fidelity.queues && fidelity.blocks, "the projection is not faithful");
  // The generator blocks hold the generator itself in the engine's count, so
  // they are not part of the comparison.
  assert.ok(fidelity.generators > 0, "the model has no generator");
});

test("the live figures of the run are the ones the engine publishes", { timeout: 300_000 }, () => {
  const documentsOf = documents(model, 200_000);
  const index = buildIndex(documentsOf.trace, documentsOf.report, documentsOf.model);
  const fidelity = checkFidelity(index, documentsOf.report);

  // The figures the app shows while the run is replayed are rebuilt from the
  // trace with the engine's own arithmetic, so at the end of the run they
  // have to be the report's numbers, and not an approximation of them.
  assert.deepEqual(
    fidelity.figureMismatches,
    [],
    "the live figures differ from the engine's own",
  );
  assert.ok(fidelity.figures, "the live figures are not the engine's");
});

test("the samples of a table, the switches and the values are read from the trace", () => {
  // The engine publishes what the report counts and the trace did not
  // carry: the samples of a table, the state of a switch and the value of a
  // savevalue.
  const tableReport: ReportDocumentLike = {
    blocks: smallReport.blocks,
    entities: {
      ...smallReport.entities,
      tables: { T: { name: "T", first: 0, width: 10, intervals: 4 } },
    },
  };
  const trace: TraceDocumentLike = {
    start_time: 0,
    end_time: 10,
    limit: 100,
    records: 5,
    truncated: false,
    events: [
      event(0, 1, 0, "Queue", "enter"),
      event(1, 1, 4, "Tabulate", "table", "T 5"),
      event(2, 1, 5, "Tabulate", "table", "T 25"),
      event(3, 1, 6, "Logic", "logic", "S1 1"),
      event(4, 1, 7, "Assign", "value", "V1 42"),
    ],
  };
  const index = buildIndex(trace, tableReport, smallModel);
  const state = stateAt(index, index.frames.length - 1);
  // The first value falls in the second interval and the second one in the
  // fourth: the engine's own arithmetic.
  assert.deepEqual(state.figures.tables["T"]?.frequencies, [0, 1, 0, 1]);
  assert.equal(state.figures.tables["T"]?.samples, 2);
  assert.equal(state.figures.tables["T"]?.maxValue, 25);
  assert.equal(state.figures.logic["S1"], true);
  assert.equal(state.figures.values["V1"], 42);
});

test("the members that leave a chain are read from the trace", () => {
  // The engine says which transactions left the chain, so the chain's time
  // and its content are the engine's own.
  const chainReport: ReportDocumentLike = {
    blocks: smallReport.blocks,
    entities: { ...smallReport.entities },
  };
  const trace: TraceDocumentLike = {
    start_time: 0,
    end_time: 10,
    limit: 100,
    records: 4,
    truncated: false,
    events: [
      event(0, 1, 8, "Link", "enter", "C"),
      event(1, 2, 8, "Link", "enter", "C"),
      event(6, 1, 9, "Unlink", "enter", "C"),
      event(6, 1, 9, "Unlink", "unlink", "1,2"),
    ],
  };
  const index = buildIndex(trace, chainReport, smallModel);
  const state = stateAt(index, index.frames.length - 1);
  const chain = state.figures.chains["C"];
  assert.equal(chain?.content, 0, "both members left");
  assert.equal(chain?.entries, 2);
  // The first was linked at 0 and the second at 1, and they left at 6.
  assert.equal(chain?.totalTime, 6 + 5);
});

test("the replay of a big trace is faithful and is built once", { timeout: 600_000 }, () => {
  const documentsOf = documents(bigModel, 200_000);
  assert.equal(documentsOf.trace.truncated, false, "the trace of the big model fits");

  const started = Date.now();
  const index = buildIndex(documentsOf.trace, documentsOf.report, documentsOf.model);
  const elapsed = Date.now() - started;

  const fidelity = checkFidelity(index, documentsOf.report);
  assert.deepEqual(fidelity.queueMismatches, [], "the queues differ");
  assert.deepEqual(fidelity.blockMismatches, [], "the blocks differ");
  assert.deepEqual(
    fidelity.entryMismatches,
    [],
    "the entries rebuilt from the trace differ from the engine's own totals",
  );

  // The index is what the animation reads: it is built once and it holds one
  // point per change of the state, not one per event.
  assert.ok(index.frames.length > 10_000, `${index.frames.length} frames`);
  const points = [...index.queuePoints.values()].reduce((total, list) => total + list.length, 0);
  assert.ok(points > 100, `${points} points`);
  assert.ok(elapsed < 10_000, `the index took ${elapsed} ms`);
});

test("a truncated trace says so instead of pretending it replayed everything", { timeout: 300_000 }, () => {
  const documentsOf = documents(model, 5);
  const index = buildIndex(documentsOf.trace, documentsOf.report, documentsOf.model);
  assert.equal(index.truncated, true);
  assert.equal(index.records, 5);
  assert.equal(index.limit, 5);
});

test("the clock of the replay accumulates instead of restarting at every frame", () => {
  // The bug this pins: the clock used to be re-anchored at the time of the
  // current frame on every paint, so a paint that advanced less than the gap
  // between two frames never crossed the next one and the replay stood still
  // at the slow speeds.
  assert.equal(advanceClock(100, 0.5, 10), 105, "the clock moves by the wall time spent");
  assert.equal(advanceClock(105, 0.5, 10), 110, "and it keeps the distance it had");

  // The clock of a replay at 1x is the real time: one second of wall clock
  // moves the simulation one second.
  assert.equal(advanceClock(0, 1, 1), 1, "at 1x a second is a second");
  assert.equal(advanceClock(0, 1, 250), 250, "at 250x a second is 250 seconds");

  // A whole replay at 60 paints per second: with a clock that accumulates,
  // the run crosses every frame.
  const index = buildIndex(smallTrace, smallReport, smallModel);
  const span = Math.max(index.endTime - index.startTime, 1);
  const rate = span / 12;
  let clock = index.frames[0]?.time ?? index.startTime;
  let playhead = 0;
  let advanced = 0;
  for (let paint = 0; paint < 60 * 12; paint += 1) {
    clock = advanceClock(clock, 1 / 60, rate);
    if (clock >= index.endTime) {
      break;
    }
    const frame = frameAt(index, clock);
    if (frame !== playhead) {
      playhead = frame;
      advanced += 1;
    }
  }
  assert.ok(advanced > 0, "the replay moved");
  assert.ok(
    clock >= index.endTime - 1,
    `the clock reached the end of the run (${clock} of ${index.endTime})`,
  );
});

test("the entries of every block are rebuilt from the trace and match the engine", () => {
  // The totals the monitor shows under each block: the entries every block
  // counted since the run started. They are rebuilt from the trace, so they
  // have to come out equal to the engine's own numbers.
  const index = buildIndex(smallTrace, smallReport, smallModel);
  const state = stateAt(index, index.frames.length - 1);
  assert.equal(state.totals[0], 1, "the queue counted one entry");
  assert.equal(state.totals[1], 1, "the depart counted one entry");
  assert.equal(state.totals[2], 1, "the seize counted one entry");
  assert.equal(state.totals[3], 1, "the release counted one entry");

  const fidelity = checkFidelity(index, smallReport);
  assert.deepEqual(fidelity.entryMismatches, [], "the entries differ from the report");
  assert.equal(fidelity.entries, true);
});

test("a move carries the time the transaction spent in the block it left", () => {
  // The transaction enters the block at 0 and leaves it at 4: the move
  // happens at 4 and carries the four seconds it spent in the block, which
  // is the flight of the ball that crosses the link.
  const trace: TraceDocumentLike = {
    start_time: 0,
    end_time: 10,
    limit: 100,
    records: 3,
    truncated: false,
    events: [
      event(0, 1, 0, "Queue", "enter"),
      event(4, 1, 0, "Queue", "leave", "-> 1"),
      event(4, 1, 1, "Advance", "enter"),
    ],
  };
  const index = buildTransitions(trace, (block) => block);
  assert.equal(index.moves.length, 1);
  assert.equal(index.moves[0].flight, 4, "the flight is the time in the block");
  assert.equal(index.moves[0].time, 4, "the move still happens at the arrival");
});

test("the flight of a move is the time the transaction spent in the block of the model", () => {
  // A block of the model may be several blocks of the program (a queue and
  // its depart): the transaction enters the block of the model at 0 and
  // leaves it at 6, so the flight of the move that takes it out is 6.
  const trace: TraceDocumentLike = {
    start_time: 0,
    end_time: 10,
    limit: 100,
    records: 4,
    truncated: false,
    events: [
      event(0, 1, 0, "Queue", "enter"),
      event(2, 1, 1, "Depart", "enter"),
      event(6, 1, 1, "Depart", "leave", "-> 2"),
      event(6, 1, 2, "Advance", "enter"),
    ],
  };
  const nodeOf = (block: number) => (block === 0 || block === 1 ? 10 : block);
  const index = buildTransitions(trace, nodeOf);
  assert.equal(index.moves.length, 1);
  assert.equal(index.moves[0].from, 10, "the move leaves the block of the model");
  assert.equal(index.moves[0].flight, 6, "the flight is the time in the block");
});

test("two consecutive moves of the same transaction do not overlap", () => {
  // The transaction enters the first block at 0, leaves it at 4 (the first
  // move), enters the second at 4 and leaves it at 9 (the second move): the
  // flight of the first ends exactly when the flight of the second begins,
  // so the two balls never share the link.
  const trace: TraceDocumentLike = {
    start_time: 0,
    end_time: 10,
    limit: 100,
    records: 5,
    truncated: false,
    events: [
      event(0, 1, 0, "Queue", "enter"),
      event(4, 1, 0, "Queue", "leave", "-> 1"),
      event(4, 1, 1, "Advance", "enter"),
      event(9, 1, 1, "Advance", "leave", "-> 2"),
      event(9, 1, 2, "Seize", "enter"),
    ],
  };
  const index = buildTransitions(trace, (block) => block);
  assert.equal(index.moves.length, 2);
  const [first, second] = index.moves;
  assert.equal(first.flight, 4, "the first flight is the time in the first block");
  assert.equal(second.flight, 5, "the second flight is the time in the second block");
  // The first ball lands exactly when the second one leaves.
  assert.equal(first.time, second.time - second.flight);
});

test("a move whose entry is not known has no flight", () => {
  // The trace does not say when the transaction entered the block: the
  // flight is zero, and the ball flies the shortest flight there is.
  const trace: TraceDocumentLike = {
    start_time: 0,
    end_time: 10,
    limit: 100,
    records: 1,
    truncated: false,
    events: [event(3, 1, 0, "Queue", "leave", "-> 1")],
  };
  const index = buildTransitions(trace, (block) => block);
  assert.equal(index.moves.length, 1);
  assert.equal(index.moves[0].flight, 0);
});

test("the slider is a clock: a time selects the last record at or before it", () => {
  // The slider of the monitor is a time, not an index of records: dragging it
  // picks a moment of the run and the replay goes to the last record that
  // happened at or before it.
  const index = buildIndex(smallTrace, smallReport, smallModel);
  const times = index.frames.map((frame) => frame.time);
  const at = (time: number) => times[frameAt(index, time)];
  assert.equal(at(0), 0, "the beginning of the run");
  assert.equal(at(1.5), 1, "the last record at or before 1.5");
  assert.equal(at(2), 2, "the exact instant of a record");
  assert.equal(at(9.9), 4, "the last record of the run");
  assert.equal(frameAt(index, -5), 0, "before the run, the first record");
});
