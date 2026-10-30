/**
 * The app says which version it is.
 *
 * The version that travels in the bundle is the one of the shell's crate,
 * and the version the development server answers is the one of the
 * manifest of the app. Both have to be the same number: the reader who
 * asks "which version is this" gets one answer, not two.
 */

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";
import { devBridge } from "../src/engine/devBridge.ts";
import { tauri } from "../src/engine/tauri.ts";

/** The directory of this test. */
const here = path.dirname(fileURLToPath(import.meta.url));

/** The root of the repository: the test lives in `crates/genmsi-app/tests`. */
const repoRoot = path.resolve(here, "../../..");

/**
 * The version of the source: the `version` of `[workspace.package]`.
 *
 * The source is the one place a person changes, so it is the number every
 * other place has to answer.
 */
function sourceVersion(): string {
  const text = readFileSync(path.join(repoRoot, "Cargo.toml"), "utf8");
  const section = text.split("[workspace.package]")[1] ?? "";
  const found = section.match(/^version\s*=\s*"([^"]+)"/m);
  assert.ok(found, "the source has no version in [workspace.package]");
  return found[1];
}

test("the Tauri backend asks the shell for the version of the app", async (t) => {
  const commands: string[] = [];
  (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
    invoke: async (command: string) => {
      commands.push(command);
      return sourceVersion();
    },
  };
  t.after(() => {
    delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
  });
  const version = await tauri.appVersion();
  assert.equal(version, sourceVersion(), "the version the shell answered");
  assert.deepEqual(commands, ["app_version"], "the command the shell was asked");
});

test("the dev backend asks the bridge for the version of the app", async (t) => {
  const urls: string[] = [];
  const original = globalThis.fetch;
  globalThis.fetch = (async (input: string) => {
    urls.push(String(input));
    return {
      ok: true,
      status: 200,
      text: async () => JSON.stringify(sourceVersion()),
    } as unknown as Response;
  }) as typeof fetch;
  t.after(() => {
    globalThis.fetch = original;
  });
  const version = await devBridge.appVersion();
  assert.equal(version, sourceVersion(), "the version the bridge answered");
  assert.equal(urls.length, 1, "the bridge was asked once");
  assert.ok(urls[0].startsWith("/api/version"), `the route of the bridge: ${urls[0]}`);
});

test("the dev bridge answers with the version of the manifest", { timeout: 60_000 }, async (t) => {
  const server = await createServer({
    configFile: path.join(repoRoot, "crates/genmsi-app/vite.config.ts"),
    root: path.join(repoRoot, "crates/genmsi-app"),
    server: { port: 0, strictPort: false },
    logLevel: "error",
    // The esbuild pre-bundler scans the source while the tests run and
    // restarts underneath itself, which prints a long Vite stack trace on
    // stderr. The dependencies are the ones the config already lists, so the
    // scan discovers nothing new and can stay quiet.
    optimizeDeps: { noDiscovery: true },
  });
  await server.listen();
  t.after(() => server.close());
  const url = server.resolvedUrls?.local[0];
  assert.ok(url !== undefined, "the dev server did not report an address");
  const response = await fetch(`${url}api/version`);
  assert.equal(response.status, 200);
  assert.equal(await response.text(), JSON.stringify(sourceVersion()));
});
