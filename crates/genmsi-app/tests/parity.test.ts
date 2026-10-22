import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync, spawnSync } from "node:child_process";
import { expect, test } from "bun:test";
import { createServer } from "vite";

const appDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = path.resolve(appDir, "../..");

const genmsiBin = process.env.GENMSI_BIN ?? path.join(repoRoot, "target", "debug", "genmsi");
const appEngineBin =
  process.env.GENMSI_APP_BIN ?? path.join(repoRoot, "target", "debug", "genmsi-app-engine");

const model = "fixtures/models/TP6__Clase 6_1_PuertaBanco.gmdl";

/** Build the engine binaries the test compares. */
function ensureBinaries(): void {
  if (existsSync(genmsiBin) && existsSync(appEngineBin)) {
    return;
  }
  const build = spawnSync("cargo", ["build", "-q", "-p", "genmsi-cli", "-p", "genmsi-app-engine"], {
    cwd: repoRoot,
    stdio: "inherit",
  });
  expect(build.status).toBe(0);
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
  if (url === undefined) {
    throw new Error("the dev server did not report an address");
  }
  return { url, close: () => server.close() };
}

test("the dev bridge returns the bytes of `genmsi lint --json`", async () => {
  ensureBinaries();
  const server = await devServer();
  try {
    const response = await fetch(`${server.url}api/lint?path=${encodeURIComponent(model)}`);
    expect(response.status).toBe(200);
    const body = Buffer.from(await response.arrayBuffer());
    expect(body.equals(cliBytes(["lint", model, "--json"]))).toBe(true);
  } finally {
    await server.close();
  }
}, 180_000);

test("the dev bridge returns the bytes of `genmsi simulate --json`", async () => {
  ensureBinaries();
  const server = await devServer();
  try {
    const response = await fetch(`${server.url}api/simulate?path=${encodeURIComponent(model)}`);
    expect(response.status).toBe(200);
    const body = Buffer.from(await response.arrayBuffer());
    expect(body.equals(cliBytes(["simulate", model, "--json"]))).toBe(true);
  } finally {
    await server.close();
  }
}, 180_000);

test("the engine facade returns the bytes of `genmsi simulate --json`", () => {
  ensureBinaries();
  const bytes = facadeBytes(["simulate", model]);
  expect(bytes.equals(cliBytes(["simulate", model, "--json"]))).toBe(true);
}, 180_000);

test("the dev bridge reports the engine's refusal as an error", async () => {
  ensureBinaries();
  const server = await devServer();
  try {
    const refused = "fixtures/models/TP9__tp9.gmdl";
    const response = await fetch(`${server.url}api/lint?path=${encodeURIComponent(refused)}`);
    expect(response.status).toBe(400);
    const body = (await response.json()) as { error: string };
    expect(body.error).toContain("TP9__tp9.gmdl");
  } finally {
    await server.close();
  }
}, 180_000);
