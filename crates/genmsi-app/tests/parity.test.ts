import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";

const appDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = path.resolve(appDir, "../..");

const genmsiBin = process.env.GENMSI_BIN ?? path.join(repoRoot, "target", "debug", "genmsi");
const appEngineBin =
  process.env.GENMSI_APP_BIN ?? path.join(repoRoot, "target", "debug", "genmsi-app-engine");

const model = "fixtures/models/TP6__Clase 6_1_PuertaBanco.gmdl";
const refused = "fixtures/models/TP9__tp9.gmdl";
const refusedBridge = "fixtures/models/Problema 2__puenteLevadizo.gmdl";

/** Build the engine binaries the test compares. */
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
  return execFileSync(genmsiBin, args, { cwd: repoRoot, maxBuffer: 32 * 1024 * 1024 });
}

/** The bytes the app's engine facade prints. */
function facadeBytes(args: string[]): Buffer {
  return execFileSync(appEngineBin, args, { cwd: repoRoot, maxBuffer: 32 * 1024 * 1024 });
}

/** The dev server with the dev bridge. */
async function devServer(): Promise<{ url: string; close: () => Promise<void> }> {
  const server = await createServer({
    configFile: path.join(appDir, "vite.config.ts"),
    root: appDir,
    server: { port: 0, strictPort: false },
    logLevel: "error",
    // The esbuild pre-bundler scans the source while the tests run and
    // restarts underneath itself, which prints a long Vite stack trace on
    // stderr. The dependencies are the ones the config already lists, so the
    // scan discovers nothing new and can stay quiet.
    optimizeDeps: { noDiscovery: true },
  });
  await server.listen();
  const url = server.resolvedUrls?.local[0];
  assert.ok(url !== undefined, "the dev server did not report an address");
  return { url, close: () => server.close() };
}

test("the dev bridge returns the bytes of `genmsi lint --json`", { timeout: 180_000 }, async () => {
  ensureBinaries();
  const server = await devServer();
  try {
    const response = await fetch(`${server.url}api/lint?path=${encodeURIComponent(model)}`);
    assert.equal(response.status, 200);
    const body = Buffer.from(await response.arrayBuffer());
    assert.ok(body.equals(cliBytes(["lint", model, "--json"])), "the lint bytes differ");
  } finally {
    await server.close();
  }
});

test(
  "the dev bridge returns the bytes of `genmsi simulate --json`",
  { timeout: 180_000 },
  async () => {
    ensureBinaries();
    const server = await devServer();
    try {
      const response = await fetch(`${server.url}api/simulate?path=${encodeURIComponent(model)}`);
      assert.equal(response.status, 200);
      const body = Buffer.from(await response.arrayBuffer());
      assert.ok(body.equals(cliBytes(["simulate", model, "--json"])), "the report bytes differ");
    } finally {
      await server.close();
    }
  },
);

test(
  "the engine facade returns the bytes of `genmsi simulate --json`",
  { timeout: 180_000 },
  () => {
    ensureBinaries();
    const bytes = facadeBytes(["simulate", model]);
    assert.ok(bytes.equals(cliBytes(["simulate", model, "--json"])), "the facade bytes differ");
  },
);

test(
  "the dev bridge reports the engine's refusal as an error",
  { timeout: 180_000 },
  async () => {
    ensureBinaries();
    const server = await devServer();
    try {
      const response = await fetch(`${server.url}api/lint?path=${encodeURIComponent(refused)}`);
      assert.equal(response.status, 400);
      const body = (await response.json()) as { error: string };
      assert.ok(body.error.includes("TP9__tp9.gmdl"), body.error);
    } finally {
      await server.close();
    }
  },
);

test(
  "the dev bridge returns the CLI's lint bytes for the other reviewed models",
  { timeout: 180_000 },
  async () => {
    ensureBinaries();
    const linted = [
      "fixtures/models/TP2__problema1.gmdl",
      "fixtures/models/Parcial 6-11-23__Veron - Parcial.gmdl",
    ];
    const server = await devServer();
    try {
      for (const path of linted) {
        const response = await fetch(`${server.url}api/lint?path=${encodeURIComponent(path)}`);
        assert.equal(response.status, 200, path);
        const body = Buffer.from(await response.arrayBuffer());
        assert.ok(body.equals(cliBytes(["lint", path, "--json"])), `the lint bytes differ for ${path}`);
      }
    } finally {
      await server.close();
    }
  },
);

test(
  "the facade refuses a model exactly as the CLI does",
  { timeout: 180_000 },
  () => {
    ensureBinaries();
    for (const path of [refused, refusedBridge]) {
      const cli = spawnSync(genmsiBin, ["simulate", path, "--json"], {
        cwd: repoRoot,
        maxBuffer: 32 * 1024 * 1024,
        encoding: "buffer",
      });
      const facade = spawnSync(appEngineBin, ["simulate", path], {
        cwd: repoRoot,
        maxBuffer: 32 * 1024 * 1024,
        encoding: "buffer",
      });
      assert.equal(cli.status, 1, path);
      assert.equal(facade.status, 1, path);
      assert.equal(facade.stdout.length, 0, `${path}: the refused facade printed to stdout`);
      assert.ok(
        facade.stderr.equals(cli.stderr),
        `the refusal message differs for ${path}`, 
      );
    }
  },
);

test(
  "the facade honours `--seed` and `--until` exactly as the CLI does",
  { timeout: 180_000 },
  () => {
    ensureBinaries();
    const runs: string[][] = [
      ["--seed", "7"],
      ["--until", "50"],
    ];
    for (const knobs of runs) {
      const bytes = facadeBytes(["simulate", model, ...knobs]);
      assert.ok(
        bytes.equals(cliBytes(["simulate", model, "--json", ...knobs])),
        `the report bytes differ for ${knobs.join(" ")}`,
      );
    }
  },
);

test(
  "the facade still publishes the model document of a refused model",
  { timeout: 180_000 },
  () => {
    ensureBinaries();
    const document = JSON.parse(
      facadeBytes(["model", refused]).toString("utf8"),
    ) as { schema: string; blocks: unknown[]; edges: unknown[] };
    assert.equal(document.schema, "genmsi.model/1");
    assert.ok(document.blocks.length > 0, "the refused model still has a graph");
  },
);
