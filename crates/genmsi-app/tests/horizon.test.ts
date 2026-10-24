//! The horizon of the run and the advice about a model that cannot finish.
//!
//! A run without a horizon is a run that lasts until the engine runs out of
//! patience, so the app puts one there: the 3600 of the original, the one
//! the `Clock` block has by default and the one the 66 runs of the oracle
//! last.
//!
//! And a model that cannot finish —no `Clock`, no `Terminate` with its
//! operand A— is not an error the reader has to decode: the app says what
//! is wrong and what to do about it, in its own words.

import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";
import { adviceFor, endReasonText, reasonOf } from "../src/engine/reasons.ts";

const appDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

/** The message the engine gives when the model cannot finish. */
const NEVER_FINISHES =
  "(node 3): no TERMINATE block with operand A and no CLOCK block: " +
  "the simulation would never finish";

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

  /** Fail the run that is in flight. */
  private fail: ((message: string) => void) | null = null;

  constructor() {
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (command: string, args?: Record<string, unknown>) => {
        this.requests.push({ command, args: args ?? {} });
        return new Promise((resolve, reject) => {
          this.finish = resolve;
          this.fail = reject;
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
    this.fail = null;
    finish?.(answer);
  }

  /** The run in flight ends with a failure, as a refused model does. */
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
  until: number | null;
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

test("the run has a horizon by default: the state starts at 3600 and the engine is asked for it", async () => {
  const shell = new FakeShell();
  const { useApp, close } = await loadStore();
  try {
    assert.equal(
      useApp.getState().until,
      3600,
      "the app does not start with a horizon of its own",
    );

    const running = useApp.getState().simulate();
    assert.equal(
      shell.sessionRequest().until,
      3600,
      "the engine was not asked for the horizon of the app",
    );
    shell.answer(reportAnswer());
    await running;
    assert.equal(useApp.getState().status, "ready", "the app did not go back to its quiet state");
  } finally {
    await close();
  }
});

test("a model that cannot finish is an advice with the remedy, not a red error", async () => {
  const shell = new FakeShell();
  const { useApp, close } = await loadStore();
  try {
    useApp.setState({ model: emptyModel, status: "ready", error: null, notice: null });

    const running = useApp.getState().simulate();
    shell.failWith(NEVER_FINISHES);
    await running;

    assert.equal(useApp.getState().status, "ready", "the app did not go back to its quiet state");
    assert.equal(useApp.getState().error, null, "the advice came as a red error");
    const notice = useApp.getState().notice;
    assert.ok(notice !== null, "the app did not say that the model cannot finish");
    assert.match(notice, /no puede terminar/, notice);
    assert.match(notice, /Reloj/, notice);
    assert.match(notice, /Terminar/, notice);
    assert.match(notice, /Hasta/, notice);
  } finally {
    await close();
  }
});

test("the run of the monitor says the same thing about a model that cannot finish", async () => {
  const shell = new FakeShell();
  const { useApp, close } = await loadStore();
  try {
    useApp.setState({ model: emptyModel, status: "ready", error: null, notice: null });

    const running = useApp.getState().runMonitor();
    shell.failWith(NEVER_FINISHES);
    await running;

    assert.equal(useApp.getState().status, "ready", "the app did not go back to its quiet state");
    assert.equal(useApp.getState().error, null, "the advice came as a red error");
    const notice = useApp.getState().notice;
    assert.ok(notice !== null, "the app did not say that the model cannot finish");
    assert.match(notice, /no puede terminar/, notice);
  } finally {
    await close();
  }
});

test("a failure the app does not know is still the failure of the engine", async () => {
  const shell = new FakeShell();
  const { useApp, close } = await loadStore();
  try {
    useApp.setState({ model: emptyModel, status: "ready", error: null, notice: null });

    const running = useApp.getState().simulate();
    shell.failWith("node 4 (Avanzar): the operand B must not be null");
    await running;

    assert.equal(useApp.getState().status, "ready", "the app did not go back to its quiet state");
    assert.equal(useApp.getState().notice, null, "an unknown reason was translated");
    const error = useApp.getState().error;
    assert.ok(error !== null, "the failure of the engine was hidden");
    assert.match(error, /operand B must not be null/, error);
  } finally {
    await close();
  }
});

test("the failure of the monitor keeps the words it always had", async () => {
  const shell = new FakeShell();
  const { useApp, close } = await loadStore();
  try {
    useApp.setState({ model: emptyModel, status: "ready", error: null, notice: null });

    const running = useApp.getState().runMonitor();
    shell.failWith("node 4 (Avanzar): the operand B must not be null");
    await running;

    assert.equal(useApp.getState().notice, null, "an unknown reason was translated");
    const error = useApp.getState().error;
    assert.ok(error !== null, "the failure of the engine was hidden");
    assert.match(error, /No se pudo reproducir la simulación/, error);
    assert.match(error, /operand B must not be null/, error);
  } finally {
    await close();
  }
});

test("the reason of the engine is the reason the app knows", () => {
  assert.equal(reasonOf(NEVER_FINISHES), "never-finishes");
  const advice = adviceFor(NEVER_FINISHES);
  assert.ok(advice !== null, "the app does not know what to say about the model");
  assert.match(advice, /no puede terminar/, advice);
  assert.match(advice, /Hasta/, advice);
});

test("a reason the app does not know is not translated", () => {
  assert.equal(reasonOf("node 4 (Avanzar): unsupported block"), "unknown");
  assert.equal(adviceFor("node 4 (Avanzar): unsupported block"), null);
});

test("the reason a run ended is said in the words of the app", () => {
  // The engine names the reason in English and the report shows it raw: the
  // app says it in its own words, and says what it does not know as it comes.
  assert.equal(endReasonText("terminated"), "terminada");
  assert.equal(endReasonText("time_limit"), "por el tope de tiempo");
  assert.equal(endReasonText("cancelled"), "cancelada");
  assert.equal(endReasonText("no_events"), "sin eventos: la cadena de eventos se agotó");
  assert.equal(endReasonText("algo_raro"), "algo_raro");
});
