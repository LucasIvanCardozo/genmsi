//! The relayout request carries the scope the reader is in.
//!
//! The reader sees one level of the model at a time: the root, or the
//! content of a container —the scope of the breadcrumbs. The request says
//! which level it is talking about, and the engine composes that level and
//! nothing else.
//!
//! Four rules are pinned here: the request of the dev bridge carries the
//! scope, the request of the desktop shell carries it, the store sends the
//! scope the reader is in —and not a hardcoded root— and the composition
//! brings the model back without disturbing the run the reader is watching.

import assert from "node:assert/strict";
import path from "node:path";
import { after, before, test } from "node:test";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";
import type { ViteDevServer } from "vite";
import type { SessionState } from "../src/types/protocol.ts";

const appDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

/**
 * The state the fake engine answers with.
 *
 * The answer of a relayout carries the state of the session: the model as
 * the engine composed it, the diagnostics and what the history can do.
 */
function sessionState(): SessionState {
  return {
    path: "modelo.gmdl",
    model: {
      path: "modelo.gmdl",
      title: "Modelo1",
      blocks: [],
      edges: [],
    },
    diagnostics: {
      status: "ok",
      diagnostics: [],
      message: null,
      blocks: 0,
      definitions: null,
    },
    dirty: false,
    canUndo: true,
    canRedo: false,
  };
}

/**
 * The bridge of the development server, faked.
 *
 * The app posts the request of the session to the bridge; the fake keeps
 * the request and answers with the state of a session.
 */
class FakeBridge {
  /** The requests the app sent, in order. */
  readonly requests: Record<string, unknown>[] = [];

  constructor() {
    (globalThis as Record<string, unknown>).fetch = async (
      _url: string,
      init?: { body?: unknown },
    ): Promise<Response> => {
      this.requests.push(JSON.parse(String(init?.body ?? "{}")) as Record<string, unknown>);
      return new Response(JSON.stringify({ ok: true, state: sessionState() }), {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    };
  }

  /** The relayout request the app sent. */
  relayoutRequest(): Record<string, unknown> {
    const request = this.requests.find((entry) => entry.op === "relayout");
    assert.ok(request !== undefined, "the app did not ask for a relayout");
    return request;
  }
}

/**
 * The shell of the desktop app, faked.
 *
 * The app asks the shell for its work with `invoke`; the fake keeps what it
 * was asked and answers with the state of a session.
 */
class FakeShell {
  /** The commands the app asked for, in order. */
  readonly requests: { command: string; args: Record<string, unknown> }[] = [];

  constructor() {
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (command: string, args?: Record<string, unknown>) => {
        this.requests.push({ command, args: args ?? {} });
        return Promise.resolve(JSON.stringify({ ok: true, state: sessionState() }));
      },
    };
  }

  /** The last request of the session the app sent. */
  sessionRequest(): Record<string, unknown> {
    const request = this.requests.filter((entry) => entry.command === "session").at(-1);
    assert.ok(request !== undefined, "the app did not ask the shell for a session");
    return JSON.parse(request.args.request as string) as Record<string, unknown>;
  }
}

/** The state of the app, as the store holds it. */
interface AppState {
  status: string;
  error: string | null;
  model: { title: string } | null;
  report: unknown;
  trace: unknown;
  activeContainerId: number;
  relayout: () => Promise<void>;
}

/**
 * The server of Vite, which loads the modules of the app.
 *
 * The store's own imports have no extension, so node cannot load them: the
 * server resolves them the way the browser does, and the module that comes
 * back is the same one the app runs.
 */
let server: ViteDevServer;

before(async () => {
  server = await createServer({
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
});

after(async () => {
  await server.close();
});

/** One module of the app, as the browser loads it. */
async function module<T>(path: string): Promise<T> {
  return (await server.ssrLoadModule(path)) as T;
}

test("the relayout request of the dev bridge carries the scope", async () => {
  const bridge = new FakeBridge();
  const { devBridge } = await module<{
    devBridge: { relayout: (scope: number) => Promise<unknown> };
  }>("/src/engine/devBridge.ts");

  await devBridge.relayout(9);

  const request = bridge.relayoutRequest();
  assert.equal(request.op, "relayout", "the request is not a relayout");
  assert.equal(request.scope, 9, "the request did not carry the scope");
});

test("the relayout request of the desktop shell carries the scope", async () => {
  const shell = new FakeShell();
  const { tauri } = await module<{
    tauri: { relayout: (scope: number) => Promise<unknown> };
  }>("/src/engine/tauri.ts");

  await tauri.relayout(4);

  const request = shell.sessionRequest();
  assert.equal(request.op, "relayout", "the request is not a relayout");
  assert.equal(request.scope, 4, "the request did not carry the scope");
});

test("the store sends the scope the reader is in", async () => {
  const shell = new FakeShell();
  const { useApp } = await module<{
    useApp: {
      getState: () => AppState;
      setState: (state: Partial<AppState>) => void;
    };
  }>("/src/state/store.ts");

  // The reader is at the top: the scope is the root of the model.
  useApp.setState({ activeContainerId: 0, status: "ready", error: null });
  await useApp.getState().relayout();
  assert.equal(
    shell.sessionRequest().scope,
    0,
    "the root of the model is not the scope of the reader",
  );

  // The reader went into a container: the scope is that container, and not
  // a root that is hardcoded somewhere.
  useApp.setState({ activeContainerId: 7 });
  await useApp.getState().relayout();
  assert.equal(
    shell.sessionRequest().scope,
    7,
    "the request did not carry the scope of the reader",
  );
});

test("the relayout brings the model back and keeps the run the reader is watching", async () => {
  const shell = new FakeShell();
  const { useApp } = await module<{
    useApp: {
      getState: () => AppState;
      setState: (state: Partial<AppState>) => void;
    };
  }>("/src/state/store.ts");
  const trace = { schema: "genmsi.trace/1", records: [] };
  useApp.setState({
    activeContainerId: 3,
    status: "ready",
    error: null,
    model: null,
    report: null,
    trace,
  });

  await useApp.getState().relayout();

  const state = useApp.getState();
  assert.equal(shell.sessionRequest().scope, 3, "the request did not carry the scope");
  assert.equal(state.model?.title, "Modelo1", "the model did not come back");
  assert.equal(state.trace, trace, "the run the reader was watching went away");
  assert.equal(state.status, "ready", "the app did not go back to its quiet state");
  assert.equal(state.error, null, "the composition of a scope failed");
});
