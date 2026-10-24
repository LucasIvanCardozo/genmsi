//! The live model: the app watches the file it has open, and reloads and
//! runs the model again when the file changes.
//!
//! The three rules of the cycle are pinned here: the change of the file is
//! seen by itself, the model that arrives is run again by the same path as
//! the button "Ejecutar", and a model that does not compile leaves the app
//! as it was.

import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";
import { fingerprintOf } from "../vite/devBridge.ts";

const appDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

/** The fingerprint of a file, as the shell publishes it. */
interface Fingerprint {
  /** The date of the last modification, in nanoseconds since the epoch. */
  modifiedNanos: string;
  /** How many bytes the file has. */
  size: number;
}

/** The state of the app, as the store holds it. */
interface AppState {
  status: string;
  error: string | null;
  notice: string | null;
  dirty: boolean;
  model: { title: string } | null;
  report: { title: string } | null;
  trace: { title: string } | null;
  currentPath: string | null;
  selectedBlockIds: number[];
  fingerprint: Fingerprint | null | undefined;
  watchModel: (path: string) => void;
  stopWatching: () => void;
  checkModelFile: () => Promise<void>;
  reloadFromDisk: () => Promise<void>;
}

/**
 * The shell of the desktop app, faked.
 *
 * The app asks the shell for its work with `invoke`; the fake answers what
 * the test says: the fingerprint of the file, the state of the reload and
 * the documents of the run.
 */
class FakeShell {
  /** The commands the app asked for, in order. */
  readonly calls: string[] = [];

  /** The operations of the session the app asked for, in order. */
  readonly ops: string[] = [];

  /** The fingerprint the file has right now. */
  fingerprint: Fingerprint | null = { modifiedNanos: "1000", size: 10 };

  /** The state the reload gives: the model of the disk. */
  reloaded: unknown = null;

  /** The failure of the reload, when the model of the disk does not compile. */
  reloadError: string | null = null;

  /** The documents of the run. */
  documents: unknown = null;

  constructor() {
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (command: string, args?: Record<string, unknown>) => {
        this.calls.push(command);
        if (command === "model_fingerprint") {
          return Promise.resolve(this.fingerprint);
        }
        if (command === "session") {
          const request = JSON.parse(String(args?.request)) as { op: string };
          this.ops.push(request.op);
          if (request.op === "reload") {
            if (this.reloadError !== null) {
              return Promise.resolve(
                JSON.stringify({ ok: false, error: this.reloadError }),
              );
            }
            return Promise.resolve(JSON.stringify({ ok: true, state: this.reloaded }));
          }
          if (request.op === "runCurrent") {
            return Promise.resolve(
              JSON.stringify({ ok: true, documents: this.documents }),
            );
          }
          return Promise.resolve(JSON.stringify({ ok: true, state: null }));
        }
        return Promise.resolve(null);
      },
    };
  }
}

/**
 * The store of the app.
 *
 * The store's own imports have no extension, so node cannot load them: the
 * server of Vite resolves them the way the browser does, and the module
 * that comes back is the same one the app runs.
 */
async function loadStore(): Promise<{
  useApp: {
    getState: () => AppState;
    setState: (state: Partial<AppState>) => void;
  };
  close: () => Promise<void>;
}> {
  const server = await createServer({
    configFile: path.join(appDir, "vite.config.ts"),
    root: appDir,
    server: { port: 0, strictPort: false },
    logLevel: "error",
    optimizeDeps: { noDiscovery: true },
  });
  await server.listen();
  const module = (await server.ssrLoadModule("/src/state/store.ts")) as {
    useApp: {
      getState: () => AppState;
      setState: (state: Partial<AppState>) => void;
    };
  };
  return { useApp: module.useApp, close: () => server.close() };
}

/** Wait until the app gets where the test wants it to be. */
async function waitFor(condition: () => boolean, timeout = 5000): Promise<void> {
  const started = Date.now();
  while (!condition()) {
    if (Date.now() - started > timeout) {
      throw new Error("the app did not get there in time");
    }
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
}

/** A model with one block: the graph of the app has something to draw. */
function modelOf(title: string): unknown {
  return {
    schema: "genmsi.model/1",
    path: "",
    title,
    blocks: [
      {
        id: 1,
        class: "Generate",
        kind: "A",
        name: "Generate",
        caption: "Generate",
        label: null,
        container: 0,
        layout: null,
        properties: [],
        inputs: [],
        outputs: [],
      },
    ],
    edges: [],
  };
}

/** The state of the session, with the model of the disk. */
function stateOf(title: string): unknown {
  return {
    path: "/models/live.gmdl",
    model: modelOf(title),
    diagnostics: { status: "ok", diagnostics: [], message: null },
    dirty: false,
    canUndo: true,
    canRedo: false,
  };
}

/** The report of a run. */
function reportOf(title: string): unknown {
  return {
    schema: "genmsi.report/1",
    title,
    seed: 1,
    start_time: 0,
    end_time: 10,
    end_reason: "terminated",
    transactions: 1,
    events: 2,
    blocks: [
      {
        line: 1,
        node: 1,
        class: "Generate",
        label: null,
        operands: "",
        entries: 1,
        current: 0,
        retries: 0,
      },
    ],
    entities: {
      facilities: {},
      storages: {},
      queues: {},
      chains: {},
      logic: {},
      savevalues: {},
      tables: {},
      qtables: {},
    },
    diagnostics: [],
  };
}

/** The trace of a run: the events the monitor replays. */
function traceOf(title: string): unknown {
  return {
    schema: "genmsi.trace/1",
    title,
    seed: 1,
    start_time: 0,
    end_time: 10,
    end_reason: "terminated",
    limit: 100,
    records: 2,
    truncated: false,
    events: [
      {
        time: 0,
        xact: 1,
        block: 0,
        class: "Generate",
        label: null,
        event: "enter",
        detail: "",
      },
      {
        time: 1,
        xact: 1,
        block: 0,
        class: "Generate",
        label: null,
        event: "leave",
        detail: "",
      },
    ],
  };
}

test("the development bridge publishes the same fingerprint as the shell", async () => {
  // The scratch directory is outside the repository, and the test removes
  // it when it is done.
  const dir = mkdtempSync(path.join(tmpdir(), "genmsi-live-"));
  try {
    const file = path.join(dir, "modelo.gmdl");
    writeFileSync(file, "contenido");

    const fingerprint = fingerprintOf(file);
    assert.ok(fingerprint !== null, "the bridge did not answer");
    assert.equal(fingerprint.size, "contenido".length);
    // The date is in nanoseconds: the epoch in nanoseconds is bigger than a
    // thousand billion, and the epoch in milliseconds is not.
    assert.ok(
      BigInt(fingerprint.modifiedNanos) > 1_000_000_000_000_000_000n,
      `the date is not in nanoseconds: ${fingerprint.modifiedNanos}`,
    );

    // A file that is not there has no fingerprint.
    assert.equal(fingerprintOf(path.join(dir, "no.gmdl")), null);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("the app sees the change of the file by itself: the model is reloaded and run again", async () => {
  const shell = new FakeShell();
  const { useApp, close } = await loadStore();
  try {
    // The model the app has open, with a block the reader had selected.
    useApp.setState({
      currentPath: "/models/live.gmdl",
      model: modelOf("el viejo") as AppState["model"],
      status: "ready",
      error: null,
      notice: null,
      dirty: false,
      report: reportOf("el viejo") as AppState["report"],
      trace: traceOf("el viejo") as AppState["trace"],
      selectedBlockIds: [7],
      fingerprint: undefined,
    });

    // The app starts watching the file.
    useApp.getState().watchModel("/models/live.gmdl");
    await waitFor(() => useApp.getState().fingerprint !== undefined);
    assert.deepEqual(
      useApp.getState().fingerprint,
      { modifiedNanos: "1000", size: 10 },
      "the app did not learn what the file looks like",
    );

    // Somebody else writes the file: the model of the disk is another one.
    shell.fingerprint = { modifiedNanos: "2000", size: 24 };
    shell.reloaded = stateOf("el nuevo");
    shell.documents = {
      report: JSON.stringify(reportOf("el nuevo")),
      trace: JSON.stringify(traceOf("el nuevo")),
    };

    // The app sees the change by itself: nobody pressed anything.
    await waitFor(() => useApp.getState().report?.title === "el nuevo");

    // The model of the disk is the one in the app, and it ran again.
    assert.equal(useApp.getState().model?.title, "el nuevo", "the model did not arrive");
    assert.equal(useApp.getState().report?.title, "el nuevo", "the model did not run again");
    assert.equal(useApp.getState().trace?.title, "el nuevo", "the trace is not the new one");
    assert.equal(useApp.getState().status, "ready", "the app did not go back to its quiet state");
    assert.equal(useApp.getState().error, null, "the reload was read as a failure");

    // The reload and the run went through the session, in that order.
    assert.deepEqual(shell.ops, ["reload", "runCurrent"], `the app asked for: ${shell.ops}`);

    // The reload did not take the reader's work away: the selection is
    // still there.
    assert.deepEqual(
      useApp.getState().selectedBlockIds,
      [7],
      "the reload took the selection away",
    );
  } finally {
    useApp.getState().stopWatching();
    await close();
  }
});

test("a model that does not compile leaves the app as it was, with the message of the compiler", async () => {
  const shell = new FakeShell();
  const { useApp, close } = await loadStore();
  try {
    useApp.setState({
      currentPath: "/models/live.gmdl",
      model: modelOf("el viejo") as AppState["model"],
      status: "ready",
      error: null,
      notice: null,
      dirty: false,
      report: reportOf("el viejo") as AppState["report"],
      trace: traceOf("el viejo") as AppState["trace"],
      selectedBlockIds: [7],
      fingerprint: { modifiedNanos: "1000", size: 10 },
    });

    // The agent breaks the model: the compiler refuses it.
    shell.fingerprint = { modifiedNanos: "2000", size: 26 };
    shell.reloadError = "el bloque 3 no existe";

    await useApp.getState().checkModelFile();

    // The app says what the compiler said.
    const error = useApp.getState().error;
    assert.ok(error !== null, "the app did not say what was wrong");
    assert.match(error, /el bloque 3 no existe/, error);

    // The model the app had is still there, with its run and its
    // selection: nothing broke.
    assert.equal(useApp.getState().model?.title, "el viejo", "the app lost the model");
    assert.equal(useApp.getState().report?.title, "el viejo", "the app lost the run");
    assert.deepEqual(useApp.getState().selectedBlockIds, [7], "the app lost the selection");
    assert.equal(useApp.getState().status, "ready", "the app is not in its quiet state");
    // The model did not run: it does not compile.
    assert.deepEqual(shell.ops, ["reload"], `the app asked for: ${shell.ops}`);
    assert.ok(
      !shell.ops.includes("runCurrent"),
      "the app ran a model that does not compile",
    );
  } finally {
    await close();
  }
});

test("the model of the disk does not go over the changes the reader has not saved", async () => {
  const shell = new FakeShell();
  const { useApp, close } = await loadStore();
  try {
    useApp.setState({
      currentPath: "/models/live.gmdl",
      model: modelOf("el viejo") as AppState["model"],
      status: "ready",
      error: null,
      notice: null,
      // The reader has work that is not on the disk.
      dirty: true,
      report: null,
      trace: null,
      selectedBlockIds: [],
      fingerprint: { modifiedNanos: "1000", size: 10 },
    });

    shell.fingerprint = { modifiedNanos: "2000", size: 24 };
    shell.reloaded = stateOf("el nuevo");

    await useApp.getState().checkModelFile();

    // The app says that the model of the disk is waiting.
    const notice = useApp.getState().notice;
    assert.ok(notice !== null, "the app did not say anything");
    assert.match(notice, /cambios sin guardar/, notice);

    // The model the reader has is still there: it was not thrown away.
    assert.equal(useApp.getState().model?.title, "el viejo", "the app lost the model");
    assert.deepEqual(shell.ops, [], `the app asked for: ${shell.ops}`);
  } finally {
    await close();
  }
});
