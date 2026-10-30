/**
 * The guardian of the version fails when it has to.
 *
 * The guardian is a test: `tests/version.test.ts` reads the seven places
 * the version lives and fails when they disagree. Proving that it really
 * fails takes a run of it, on a repository where one of the copies says
 * another version.
 *
 * This file is not the guardian: it runs it. The two are apart on purpose
 * —the guardian copies itself into the scratch, and a guardian that
 * carried this test would spawn itself forever.
 */

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

/** The directory of this test. */
const here = path.dirname(fileURLToPath(import.meta.url));

/** The root of the repository: the test lives in `crates/genmsi-app/tests`. */
const repoRoot = path.resolve(here, "../../..");

/** The files the guardian reads, relative to the root of the repository. */
const seven = [
  "Cargo.toml",
  "crates/genmsi-app/engine/Cargo.toml",
  "crates/genmsi-app/package.json",
  "crates/genmsi-app/src-tauri/Cargo.toml",
  "crates/genmsi-app/src-tauri/tauri.conf.json",
  "packaging/arch/PKGBUILD",
  "crates/genmsi-app/LEEME.txt",
];

/**
 * A repository of its own, with the seven files and the guardian.
 *
 * The guardian finds the root from the place of its own file, so a copy
 * placed in the same layout reads the copy.
 */
function scratch(): string {
  const dir = mkdtempSync(path.join(tmpdir(), "genmsi-guardian-"));
  // The workflow comes too: the guardian reads it to check that the
  // formats of the readme are the ones the release publishes.
  for (const file of [...seven, "scripts/bump-version.sh", ".github/workflows/release.yml"]) {
    const target = path.join(dir, file);
    mkdirSync(path.dirname(target), { recursive: true });
    copyFileSync(path.join(repoRoot, file), target);
  }
  const guardian = path.join(dir, "crates/genmsi-app/tests/version.test.ts");
  mkdirSync(path.dirname(guardian), { recursive: true });
  copyFileSync(path.join(repoRoot, "crates/genmsi-app/tests/version.test.ts"), guardian);
  return dir;
}

/** Say another version in the manifest of the app of a copy. */
function misalign(dir: string): void {
  const manifest = path.join(dir, "crates/genmsi-app/package.json");
  const text = readFileSync(manifest, "utf8");
  writeFileSync(
    manifest,
    text.replace(/("version"\s*:\s*")[^"]+(")/, "$19.9.9$2"),
    "utf8",
  );
}

/** Run the guardian of a copy and say what it answered. */
function runGuardian(dir: string): { status: number; output: string } {
  const guardian = path.join(dir, "crates/genmsi-app/tests/version.test.ts");
  // The child does not inherit the context of the test runner: inside it,
  // node sends the results of the child through a channel of its own and
  // says nothing on the standard output.
  const result = spawnSync("node", ["--test", guardian], {
    cwd: dir,
    encoding: "utf8",
    env: { ...process.env, NODE_TEST_CONTEXT: undefined },
  });
  return {
    status: result.status ?? 1,
    output: `${result.stdout ?? ""}${result.stderr ?? ""}`,
  };
}

test("the guardian fails when a copy is out of date", (t) => {
  const dir = scratch();
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  misalign(dir);
  const result = runGuardian(dir);
  assert.notEqual(result.status, 0, "the guardian did not see the copy out of date");
  assert.match(
    result.output,
    /package\.json/,
    "the guardian did not name the copy that was out of date",
  );
});

test("the guardian passes when every copy says the same", (t) => {
  const dir = scratch();
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  const result = runGuardian(dir);
  assert.equal(result.status, 0, `the guardian failed on a whole repository: ${result.output}`);
});
