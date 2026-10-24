//! The `START` of the run, in the toolbar and in the request to the engine.
//!
//! The `START` is the counter of terminations of the run: the original has
//! it in the panel of its simulator ("Start:", one by default) and the model
//! does not declare it anywhere, so it is a parameter of the run, like the
//! horizon. The app puts it in the toolbar, next to the horizon, and it
//! travels to the engine with the rest of the knobs of the run.

import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";

const appDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

/**
 * The shell of the desktop app, faked.
 *
 * The app asks the shell for its work with `invoke`; the fake keeps what it
 * was asked and answers what the test says.
 */
class FakeShell {
  /** The commands the app asked for, in order. */
  readonly requests: { command: string; args: Record<string, unknown> }[] = [];

  /** Finish the run that is in flight. */
  private finish: ((answer: unknown) => void) | null = null;

  constructor() {
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (command: string, args?: Record<string, unknown>) => {
        this.requests.push({ command, args: args ?? {} });
        return new Promise((resolve) => {
          this.finish = resolve;
        });
      },
    };
  }

  /** The request of the session the app sent. */
  sessionRequest(): Record<string, unknown> {
    const request = this.requests.find((entry) => entry.command === "session");
    assert.ok(request !== undefined, "the app did not ask the shell for a session");
    return JSON.parse(request.args.request as string) as Record<string, unknown>;
  }

  /** The run in flight ends with the answer of the shell. */
  answer(answer: unknown): void {
    const finish = this.finish;
    this.finish = null;
    finish?.(answer);
  }
}

/** The state of the app, as the store holds it. */
interface AppState {
  status: string;
  error: string | null;
  notice: string | null;
  until: number | null;
  start: number;
  setStart: (start: number) => void;
  model: unknown;
  simulate: () => Promise<void>;
  runMonitor: () => Promise<void>;
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
    // The esbuild pre-bundler scans the source while the tests run and
    // restarts underneath itself, which prints a long Vite stack trace on
    // stderr. The dependencies are the ones the config already lists, so the
    // scan discovers nothing new and can stay quiet.
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

/** A model with nothing in it: the run does not need one to be in flight. */
const emptyModel = {
  schema: "genmsi.model/1",
  path: "",
  title: "Modelo1",
  blocks: [],
  edges: [],
};

/** The answer of the shell to a run that gave its report. */
function reportAnswer(): string {
  const report = { schema: "genmsi.report/1", blocks: [], tables: [] };
  return JSON.stringify({ ok: true, document: JSON.stringify(report) });
}

test("the run has a START by default: the state starts at one and the engine is asked for it", async () => {
  const shell = new FakeShell();
  const { useApp, close } = await loadStore();
  try {
    assert.equal(
      useApp.getState().start,
      1,
      "the app does not start with the START of the original",
    );

    const running = useApp.getState().simulate();
    assert.equal(
      shell.sessionRequest().start,
      1,
      "the engine was not asked for the START of the app",
    );
    shell.answer(reportAnswer());
    await running;
    assert.equal(useApp.getState().status, "ready", "the app did not go back to its quiet state");
  } finally {
    await close();
  }
});

test("the START the reader chooses is the one the engine is asked for", async () => {
  const shell = new FakeShell();
  const { useApp, close } = await loadStore();
  try {
    useApp.setState({ start: 5 });

    const running = useApp.getState().simulate();
    assert.equal(
      shell.sessionRequest().start,
      5,
      "the engine was not asked for the START the reader chose",
    );
    shell.answer(reportAnswer());
    await running;
  } finally {
    await close();
  }
});

test("the START of the run of the monitor is the one the reader chose", async () => {
  const shell = new FakeShell();
  const { useApp, close } = await loadStore();
  try {
    useApp.setState({ model: emptyModel, status: "ready", error: null, notice: null, start: 3 });

    const running = useApp.getState().runMonitor();
    assert.equal(
      shell.sessionRequest().start,
      3,
      "the engine was not asked for the START the reader chose",
    );
    shell.answer(reportAnswer());
    await running;
  } finally {
    await close();
  }
});

test("the START is never less than one, like in the original", async () => {
  const shell = new FakeShell();
  const { useApp, close } = await loadStore();
  try {
    // The original refuses a Start of zero ("El Start debe ser mayor a
    // cero."): the app does not let the field hold it.
    useApp.getState().setStart(0);
    assert.equal(
      useApp.getState().start,
      1,
      "the app let the START of the run be zero",
    );

    const running = useApp.getState().simulate();
    assert.equal(shell.sessionRequest().start, 1, "the engine was asked for a START of zero");
    shell.answer(reportAnswer());
    await running;
  } finally {
    await close();
  }
});
