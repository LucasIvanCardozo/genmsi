import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";

const appDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

/**
 * The shell of the desktop app, faked.
 *
 * The app asks the shell for its work with `invoke`; the fake answers what
 * the test says: the run stays in flight until the test ends it.
 */
class FakeShell {
  /** The commands the app asked for, in order. */
  readonly calls: string[] = [];

  /** Finish the run that is in flight. */
  private finish: ((answer: unknown) => void) | null = null;

  /** Fail the run that is in flight. */
  private fail: ((message: string) => void) | null = null;

  constructor() {
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (command: string) => {
        this.calls.push(command);
        if (command === "cancel_run") {
          // The shell says there was a run to stop.
          return Promise.resolve(true);
        }
        return new Promise((resolve, reject) => {
          this.finish = resolve;
          this.fail = reject;
        });
      },
    };
  }

  /** The run in flight ends with the answer of the shell. */
  answer(answer: unknown): void {
    const finish = this.finish;
    this.finish = null;
    this.fail = null;
    finish?.(answer);
  }

  /** The run in flight ends with a failure, as a stopped run does. */
  failWith(message: string): void {
    const fail = this.fail;
    this.finish = null;
    this.fail = null;
    fail?.(message);
  }
}

/** The state of the app, as the store holds it. */
interface AppState {
  status: string;
  error: string | null;
  notice: string | null;
  cancelling: boolean;
  model: unknown;
  simulate: () => Promise<void>;
  runMonitor: () => Promise<void>;
  cancelRun: () => Promise<void>;
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

test("the run does not block the window: the working state is painted and the loop keeps turning", async () => {
  const shell = new FakeShell();
  const { useApp, close } = await loadStore();
  try {
    useApp.setState({ model: emptyModel, status: "ready", error: null, notice: null });

    // The reader presses "Simular": the call gives back a promise and the
    // app goes on working while the engine runs.
    const running = useApp.getState().simulate();
    assert.ok(running instanceof Promise, "the call did not give back a promise");
    assert.equal(
      useApp.getState().status,
      "running",
      "the working state was not painted before the run ended",
    );

    // The window keeps painting while the run is in flight: the loop of the
    // events turns, and the run has not ended.
    let settled = false;
    void running.then(
      () => {
        settled = true;
      },
      () => {
        settled = true;
      },
    );
    await new Promise((resolve) => setTimeout(resolve, 0));
    assert.equal(settled, false, "the run ended before the loop turned");
    assert.equal(useApp.getState().status, "running", "the app left the working state");

    // The run ends: the app goes back to its quiet state.
    shell.answer(reportAnswer());
    await running;
    assert.equal(useApp.getState().status, "ready", "the app did not go back to its quiet state");
    assert.equal(useApp.getState().error, null, "the run that ended was read as a failure");
  } finally {
    await close();
  }
});

test("the reader can stop the run: the app says what the cancel does and does not do", async () => {
  const shell = new FakeShell();
  const { useApp, close } = await loadStore();
  try {
    useApp.setState({ model: emptyModel, status: "ready", error: null, notice: null });

    const running = useApp.getState().runMonitor();
    assert.equal(useApp.getState().status, "running");

    // The reader presses "Cancelar": the shell is asked to stop the run.
    await useApp.getState().cancelRun();
    assert.ok(
      shell.calls.includes("cancel_run"),
      `the shell was not asked to stop the run: ${shell.calls.join(", ")}`,
    );
    assert.equal(useApp.getState().cancelling, true, "the app does not know the run was stopped");

    // The run ends: the app says the truth about what the cancel did.
    shell.failWith(
      "la corrida se canceló: el motor cortó la simulación en el siguiente evento y la app descartó su resultado.",
    );
    await running;
    assert.equal(useApp.getState().status, "ready", "the app did not go back to its quiet state");
    assert.equal(useApp.getState().error, null, "a stopped run is not a failure");
    const notice = useApp.getState().notice;
    assert.ok(notice !== null, "the app did not say what the cancel did");
    assert.match(notice, /el motor la cortó/, notice);
  } finally {
    await close();
  }
});
