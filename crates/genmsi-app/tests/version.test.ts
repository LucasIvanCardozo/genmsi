/**
 * The guardian of the version.
 *
 * The version of the app lives in seven places and only one of them is the
 * source: the `version` of `[workspace.package]` in the root `Cargo.toml`.
 * The other six follow it, and this test is the one that says when they
 * stop following it.
 *
 * The test reads the repository from its own place: the root is three levels
 * up from `crates/genmsi-app/tests`, and it is found from the URL of the
 * file, not from the directory the test happens to run in.
 */

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
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

/** The file that holds the version every other place follows. */
const source = path.join(repoRoot, "Cargo.toml");

/** The six places that follow the source. */
const copies = {
  engine: path.join(repoRoot, "crates/genmsi-app/engine/Cargo.toml"),
  package: path.join(repoRoot, "crates/genmsi-app/package.json"),
  shell: path.join(repoRoot, "crates/genmsi-app/src-tauri/Cargo.toml"),
  tauri: path.join(repoRoot, "crates/genmsi-app/src-tauri/tauri.conf.json"),
  pkgbuild: path.join(repoRoot, "packaging/arch/PKGBUILD"),
  readme: path.join(repoRoot, "crates/genmsi-app/LEEME.txt"),
};

/** The script that writes the six copies. */
const bumpScript = path.join(repoRoot, "scripts/bump-version.sh");

/** The formats the releases publish: the contract with the packaging unit. */
const publishedFormats = [
  ".deb",
  ".rpm",
  ".pkg.tar.zst",
  ".dmg",
  "-setup.exe",
  ".msi",
  ".flatpak",
  ".tar.gz",
  "SHA256SUMS",
];

/** A version, as the source writes it. */
const versionPattern = /\d+\.\d+\.\d+/g;

/** The version of the source: the `version` of `[workspace.package]`. */
function sourceVersion(): string {
  const text = readFileSync(source, "utf8");
  const section = text.split("[workspace.package]")[1] ?? "";
  const found = section.match(/^version\s*=\s*"([^"]+)"/m);
  assert.ok(found, "the source has no version in [workspace.package]");
  return found[1];
}

/** The version of the `[package]` of a Cargo file. */
function packageVersion(file: string): string {
  const text = readFileSync(file, "utf8");
  const section = text.split("[package]")[1] ?? "";
  const found = section.match(/^version\s*=\s*"([^"]+)"/m);
  assert.ok(found, `${file} has no version in [package]`);
  return found[1];
}

/** The `pkgver` of the PKGBUILD: one line, and only one. */
function pkgbuildVersion(): string {
  const lines = readFileSync(copies.pkgbuild, "utf8")
    .split("\n")
    .filter((line) => /^pkgver=/.test(line));
  assert.equal(lines.length, 1, "the PKGBUILD has exactly one pkgver line");
  return lines[0].slice("pkgver=".length).trim();
}

test("the source of the version is the workspace's Cargo.toml", () => {
  const version = sourceVersion();
  assert.match(version, versionPattern, `the source is not a version: ${version}`);
});

test("the six copies carry the version of the source", () => {
  const version = sourceVersion();
  assert.equal(packageVersion(copies.engine), version, "the engine's Cargo.toml");
  assert.equal(packageVersion(copies.shell), version, "the shell's Cargo.toml");
  const manifest = JSON.parse(readFileSync(copies.package, "utf8")) as {
    version?: string;
  };
  assert.equal(manifest.version, version, "the app's package.json");
  assert.equal(pkgbuildVersion(), version, "the PKGBUILD");
});

test("the Tauri config does not carry a version of its own", () => {
  const config = JSON.parse(readFileSync(copies.tauri, "utf8")) as {
    version?: string;
  };
  assert.equal(
    config.version,
    undefined,
    "the Tauri config has a version: Tauri takes the one of the shell's Cargo.toml",
  );
});

test("the readme of the installer carries the version of the source", () => {
  const version = sourceVersion();
  const text = readFileSync(copies.readme, "utf8");
  const lines = text.split("\n");
  const title = lines[0] ?? "";
  assert.ok(
    title.includes(`GenMSI ${version}`),
    `the title of the readme does not carry the version: ${title}`,
  );
  // No other version lives in the readme: the one of the app is the only
  // version a reader can find there.
  for (const token of text.match(versionPattern) ?? []) {
    assert.equal(token, version, `the readme carries another version: ${token}`);
  }
  // Every format that is published is named, with the version in its line.
  for (const format of publishedFormats) {
    const line = lines.find((candidate) => candidate.includes(format));
    assert.ok(line, `the readme does not name the format ${format}`);
    if (format !== "SHA256SUMS") {
      assert.ok(
        line.includes(version),
        `the line of ${format} does not carry the version: ${line}`,
      );
    }
  }
});

test("the formats the readme names are the ones the workflow publishes", () => {
  // The readme of the installer and the release have to say the same
  // thing: a reader who installs from the readme and a reader who
  // downloads from the release get the same formats.
  const workflow = readFileSync(
    path.join(repoRoot, ".github/workflows/release.yml"),
    "utf8",
  );
  for (const format of publishedFormats) {
    assert.ok(
      workflow.includes(format),
      `the workflow does not publish the format ${format}`,
    );
  }
  // The AppImage stays out: it does not work (the WebKitGTK that travels
  // inside it cannot find its helpers), and an artifact that does not run
  // is worse than no artifact.
  assert.ok(
    !workflow.includes("AppImage"),
    "the workflow publishes an AppImage, and the AppImage does not work",
  );
});

test("the script that writes the copies is there", () => {
  assert.ok(existsSync(bumpScript), "scripts/bump-version.sh does not exist");
  assert.ok(statSync(bumpScript).mode & 0o111, "the script is not executable");
});

/** A copy of the files the script touches, in a directory of its own. */
function scratch(): string {
  const dir = mkdtempSync(path.join(tmpdir(), "genmsi-version-"));
  const files = [
    "Cargo.toml",
    "crates/genmsi-app/package.json",
    "crates/genmsi-app/LEEME.txt",
    "crates/genmsi-app/engine/Cargo.toml",
    "crates/genmsi-app/src-tauri/Cargo.toml",
    "crates/genmsi-app/src-tauri/tauri.conf.json",
    "packaging/arch/PKGBUILD",
    "scripts/bump-version.sh",
  ];
  for (const file of files) {
    const target = path.join(dir, file);
    mkdirSync(path.dirname(target), { recursive: true });
    copyFileSync(path.join(repoRoot, file), target);
  }
  return dir;
}

/** Run the script of a scratch copy and say what it answered. */
function runScript(dir: string, version: string): { code: number; stderr: string } {
  const result = spawnSync("bash", [path.join(dir, "scripts/bump-version.sh"), version], {
    cwd: dir,
    encoding: "utf8",
  });
  return { code: result.status ?? 1, stderr: result.stderr };
}

/** The version of a file of the scratch copy. */
function versionIn(dir: string, file: string): string {
  const text = readFileSync(path.join(dir, file), "utf8");
  if (file.endsWith(".json")) {
    return (JSON.parse(text) as { version?: string }).version ?? "";
  }
  if (file.endsWith("PKGBUILD")) {
    return text.match(/^pkgver=(.*)$/m)?.[1]?.trim() ?? "";
  }
  if (file.endsWith("Cargo.toml")) {
    const section = text.split("[package]")[1] ?? text.split("[workspace.package]")[1] ?? "";
    return section.match(/^version\s*=\s*"([^"]+)"/m)?.[1] ?? "";
  }
  return "";
}

test("the script writes the six copies with the version it is given", (t) => {
  const dir = scratch();
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  // The source of the scratch copy says the version of the repository, and
  // the script is asked for another one: the source is out of date and the
  // script says so.
  const result = runScript(dir, "9.9.9");
  assert.notEqual(result.code, 0, "the script did not say the source is out of date");
  assert.match(result.stderr, /Cargo\.toml/, "the script did not name the source");
  // Nothing is written: a tree that carries two versions is a tree that
  // nobody can install, and the script refuses to make one.
  for (const file of [
    "crates/genmsi-app/engine/Cargo.toml",
    "crates/genmsi-app/src-tauri/Cargo.toml",
    "crates/genmsi-app/package.json",
    "packaging/arch/PKGBUILD",
  ]) {
    assert.notEqual(
      versionIn(dir, file),
      "9.9.9",
      `the script wrote ${file} even though the source was out of date`,
    );
  }
  const readme = readFileSync(path.join(dir, "crates/genmsi-app/LEEME.txt"), "utf8");
  assert.ok(!readme.includes("9.9.9"), "the script wrote the readme out of date");
  // The source is not the script's business: the human changes it by hand.
  assert.equal(
    versionIn(dir, "Cargo.toml"),
    sourceVersion(),
    "the script changed the source",
  );
});

test("the script is happy when the source already says the version", (t) => {
  const dir = scratch();
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  const cargo = path.join(dir, "Cargo.toml");
  const text = readFileSync(cargo, "utf8");
  writeFileSync(
    cargo,
    text.replace(/^(version\s*=\s*")[^"]+(")/m, "$19.9.9$2"),
    "utf8",
  );
  const result = runScript(dir, "9.9.9");
  assert.equal(result.code, 0, `the script failed: ${result.stderr}`);
  assert.equal(
    versionIn(dir, "crates/genmsi-app/package.json"),
    "9.9.9",
    "the app's package.json",
  );
});

test("the script does not dirty the tree when the version did not change", (t) => {
  const dir = scratch();
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  const before = readFileSync(path.join(dir, "crates/genmsi-app/package.json"), "utf8");
  const result = runScript(dir, sourceVersion());
  assert.equal(result.code, 0, `the script failed: ${result.stderr}`);
  const after = readFileSync(path.join(dir, "crates/genmsi-app/package.json"), "utf8");
  assert.equal(after, before, "the script changed the tree with the same version");
});

test("the script fails when it cannot leave a copy with the version", (t) => {
  const dir = scratch();
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  rmSync(path.join(dir, "crates/genmsi-app/engine/Cargo.toml"));
  const result = runScript(dir, sourceVersion());
  assert.notEqual(result.code, 0, "the script did not fail with a copy missing");
});

/** The seven files the version lives in, relative to a root. */
const seven = [
  "Cargo.toml",
  "crates/genmsi-app/engine/Cargo.toml",
  "crates/genmsi-app/package.json",
  "crates/genmsi-app/src-tauri/Cargo.toml",
  "crates/genmsi-app/src-tauri/tauri.conf.json",
  "packaging/arch/PKGBUILD",
  "crates/genmsi-app/LEEME.txt",
];

test("the script does not write anything when the source is out of date", (t) => {
  const dir = scratch();
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  const before = seven.map((file) =>
    createHash("sha256").update(readFileSync(path.join(dir, file))).digest("hex"),
  );
  const result = runScript(dir, "9.9.9");
  assert.notEqual(result.code, 0, "the script did not say the source is out of date");
  const after = seven.map((file) =>
    createHash("sha256").update(readFileSync(path.join(dir, file))).digest("hex"),
  );
  assert.deepEqual(after, before, "the script wrote something with the source out of date");
});

