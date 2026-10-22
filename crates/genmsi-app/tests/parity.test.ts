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
