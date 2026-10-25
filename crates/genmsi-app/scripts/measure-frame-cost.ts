// Measure the cost of one frame of the app's replay.
//
// The app draws the replay with requestAnimationFrame: on every frame it
// looks for the frame of the clock's instant (a binary search) and rebuilds
// the state (the queues, the facilities, the storages). When that work per
// frame stays well below the 16 ms of a 60 Hz frame, the animation does not
// drop frames because of the engine.
//
//     node crates/genmsi-app/scripts/measure-frame-cost.ts
//
// The binaries it reads are the debug ones: build them first
// (`cargo build -p genmsi-cli -p genmsi-app-engine`).
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { buildIndex, frameAt, stateAt } from "../src/monitor/replay.ts";

const repoRoot = fileURLToPath(new URL("../../..", import.meta.url));
const cliBin = `${repoRoot}target/debug/genmsi`;
const facadeBin = `${repoRoot}target/debug/genmsi-app-engine`;
const model = "fixtures/models/Parcial 6-11-23__Veron - Parcial.gmdl";

function bytes(bin: string, args: string[]): unknown {
  return JSON.parse(
    execFileSync(bin, args, { cwd: repoRoot, maxBuffer: 512 * 1024 * 1024 }).toString("utf8"),
  );
}

for (const limit of [100, 10_000]) {
  const modelDocument = bytes(facadeBin, ["model", model]);
  const report = bytes(cliBin, ["simulate", model, "--json", "--seed", "1"]);
  const trace = bytes(cliBin, ["trace", model, "--json", "--limit", String(limit), "--seed", "1"]);

  const t0 = performance.now();
  const index = buildIndex(trace as never, report as never, modelDocument as never);
  const t1 = performance.now();

  const frames = index.frames.length;
  // The work of one frame: find the frame of the instant and rebuild the state.
  const t2 = performance.now();
  for (let i = 0; i < frames; i += 1) {
    const frame = frameAt(index, index.frames[i].time);
    stateAt(index, frame);
  }
  const t3 = performance.now();

  const perFrame = frames > 0 ? (t3 - t2) / frames : 0;
  console.log(
    `events=${String((trace as { events: unknown[] }).events.length).padStart(7)}` +
      `  frames=${String(frames).padStart(6)}` +
      `  precomputed=${(t1 - t0).toFixed(1)} ms` +
      `  per frame=${perFrame.toFixed(4)} ms` +
      `  ceiling=${(1000 / Math.max(perFrame, 0.0001)).toFixed(0)} frames/s`,
  );
}
