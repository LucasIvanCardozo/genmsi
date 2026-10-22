import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  buildIndex,
  checkFidelity,
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
  const report = JSON.parse(cliBytes(["simulate", path, "--json"]).toString("utf8"));
  const trace = JSON.parse(cliBytes(["trace", path, "--json", "--limit", String(limit)]).toString("utf8"));
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
    { line: 1, node: 1, class: "Queue", current: 0 },
    { line: 2, node: 2, class: "Depart", current: 0 },
    { line: 3, node: 3, class: "Seize", current: 0 },
    { line: 4, node: 4, class: "Release", current: 0 },
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
  assert.deepEqual(second.series, first.series);
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

test("the replay of a big trace is faithful and is built once", { timeout: 600_000 }, () => {
  const documentsOf = documents(bigModel, 200_000);
  assert.equal(documentsOf.trace.truncated, false, "the trace of the big model fits");

  const started = Date.now();
  const index = buildIndex(documentsOf.trace, documentsOf.report, documentsOf.model);
  const elapsed = Date.now() - started;

  const fidelity = checkFidelity(index, documentsOf.report);
  assert.deepEqual(fidelity.queueMismatches, [], "the queues differ");
  assert.deepEqual(fidelity.blockMismatches, [], "the blocks differ");

  // The index is what the animation reads: it is built once and it is much
  // smaller than the trace it comes from.
  assert.ok(index.frames.length > 10_000, `${index.frames.length} frames`);
  assert.ok(index.series.length <= 300, `${index.series.length} points`);
  assert.ok(elapsed < 10_000, `the index took ${elapsed} ms`);
});

test("a truncated trace says so instead of pretending it replayed everything", { timeout: 300_000 }, () => {
  const documentsOf = documents(model, 5);
  const index = buildIndex(documentsOf.trace, documentsOf.report, documentsOf.model);
  assert.equal(index.truncated, true);
  assert.equal(index.records, 5);
  assert.equal(index.limit, 5);
});
