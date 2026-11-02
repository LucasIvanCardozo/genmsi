//! The loop of the proposal, on the side of the app.
//!
//! The tick that watches the file the app has open also looks at the
//! staging area of the app: when an agent leaves a proposal, the app puts
//! it on the canvas —dirty and undoable, without writing anything— and the
//! person decides with the band of the status line.
//!
//! The rules of the decision are in a pure module and are pinned by
//! `proposal.test.ts`. What is pinned here is the wiring: the tick asks,
//! the adoption happens **once**, the acceptance saves the file, and the
//! note of the reader travels with the rejection.

import assert from "node:assert/strict";
import { test } from "node:test";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";

const appDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

/** The state of the app, as the store holds it. */
interface AppState {
  status: string;
  error: string | null;
  notice: string | null;
  dirty: boolean;
  model: { title: string } | null;
  proposal: { id: string; state: string } | null;
  proposalOnCanvas: boolean;
  pendingProposal: { id: string } | null;
  savedMessage: string | null;
  currentPath: string | null;
  fingerprint: { modifiedNanos: string; size: number } | null | undefined;
  watchModel: (path: string) => void;
  stopWatching: () => void;
  undo: () => Promise<void>;
  acceptProposal: () => Promise<void>;
  discardProposal: (note: string) => Promise<void>;
}

/**
 * The document of a proposal that waits, as the engine publishes it.
 */
function proposalDocument(id = "1700000000000-ab"): unknown {
  return {
    schema: "genmsi.proposal/1",
    id,
    model: `/home/lucas/.local/share/genmsi/staging/${id}.gmdl`,
    target: "/models/live.gmdl",
    summary: "el bus del banco",
    state: "pending",
    note: null,
  };
}

/**
 * The state of the session, as the engine answers it.
 *
 * The state says whether the proposal is on the canvas: that is what the
 * app draws the band with.
 */
function stateOf(
  title: string,
  proposal: { id: string; state: string; summary: string | null; note: string | null } | null,
  onCanvas: boolean,
): unknown {
  return {
    path: "/models/live.gmdl",
    model: {
      schema: "genmsi.model/1",
      path: "",
      title,
      blocks: [],
      edges: [],
    },
    diagnostics: { status: "ok", diagnostics: [], message: null },
    dirty: false,
    canUndo: true,
    canRedo: false,
    proposal,
    proposalOnCanvas: onCanvas,
  };
}

/**
 * The shell of the desktop app, faked.
 *
 * The app asks the shell for its work with `invoke`; the fake answers what
 * the test says and keeps the operations the app asked for, in order.
 */
class FakeShell {
  /** The operations of the session the app asked for, in order. */
  readonly ops: string[] = [];

  /** The proposal that waits in the staging area. */
  pending: unknown = null;

  /** The note that travelled with the rejection. */
  note: string | null = null;

  /** The proposal the session holds. */
  held: { id: string; state: string; summary: string | null; note: string | null } | null = null;

  constructor() {
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (command: string, args?: Record<string, unknown>) => {
        if (command === "model_fingerprint") {
          return Promise.resolve({ modifiedNanos: "1000", size: 10 });
        }
        if (command !== "session") {
          return Promise.resolve(null);
        }
        const request = JSON.parse(String(args?.request)) as {
          op: string;
          note?: string | null;
        };
        this.ops.push(request.op);
        if (request.op === "pendingProposal") {
          return Promise.resolve(
            JSON.stringify(
              this.pending === null ? { ok: true } : { ok: true, document: JSON.stringify(this.pending) },
            ),
          );
        }
        if (request.op === "adoptProposal") {
          const document = this.pending as { id: string; summary: string | null };
          this.held = {
            id: document.id,
            state: "pending",
            summary: document.summary,
            note: null,
          };
          return Promise.resolve(
            JSON.stringify({ ok: true, state: stateOf("el propuesto", this.held, true) }),
          );
        }
        if (request.op === "acceptProposal") {
          if (this.held !== null) {
            this.held = { ...this.held, state: "accepted" };
          }
          return Promise.resolve(
            JSON.stringify({
              ok: true,
              state: stateOf("el propuesto", this.held, false),
              path: "/models/live.gmdl",
              bytes: 42,
            }),
          );
        }
        if (request.op === "discardProposal") {
          this.note = request.note ?? null;
          if (this.held !== null) {
            this.held = { ...this.held, state: "rejected", note: this.note };
          }
          return Promise.resolve(
            JSON.stringify({ ok: true, state: stateOf("el viejo", this.held, false) }),
          );
        }
        if (request.op === "undo") {
          // The adoption comes undone: the proposal is still the one of
          // the session, but it is not on the canvas.
          return Promise.resolve(
            JSON.stringify({ ok: true, state: stateOf("el viejo", this.held, false) }),
          );
        }
        return Promise.resolve(JSON.stringify({ ok: true, state: null }));
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

/** Wait a little, so the tick has time to ask again. */
async function tick(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, 1200));
}

test("the tick of the watcher adopts the proposal that waits, and only once", async () => {
  const shell = new FakeShell();
  const { useApp, close } = await loadStore();
  try {
    shell.pending = proposalDocument();
    useApp.setState({
      currentPath: "/models/live.gmdl",
      model: { title: "el viejo" } as AppState["model"],
      status: "ready",
      error: null,
      notice: null,
      dirty: false,
      proposal: null,
      proposalOnCanvas: false,
      pendingProposal: null,
      fingerprint: undefined,
    });

    // The app starts watching the file: the same tick looks at the
    // staging area of the app.
    useApp.getState().watchModel("/models/live.gmdl");

    await waitFor(() => useApp.getState().proposalOnCanvas === true);
    assert.equal(
      useApp.getState().proposal?.id,
      "1700000000000-ab",
      "the app does not hold the proposal of the agent",
    );
    assert.equal(
      useApp.getState().pendingProposal?.id,
      "1700000000000-ab",
      "the app does not know that a proposal waits",
    );

    // The adoption is not a loop: the tick asks again and does not adopt
    // the same proposal twice.
    await tick();
    const adoptions = shell.ops.filter((op) => op === "adoptProposal").length;
    assert.equal(adoptions, 1, `the app adopted the proposal ${adoptions} times`);
    assert.ok(
      shell.ops.includes("pendingProposal"),
      `the app did not look at the staging area: ${shell.ops}`,
    );
  } finally {
    useApp.getState().stopWatching();
    await close();
  }
});

test("the proposal that the reader undid is not adopted again behind their back", async () => {
  const shell = new FakeShell();
  const { useApp, close } = await loadStore();
  try {
    shell.pending = proposalDocument();
    useApp.setState({
      currentPath: "/models/live.gmdl",
      model: { title: "el viejo" } as AppState["model"],
      status: "ready",
      error: null,
      notice: null,
      dirty: false,
      proposal: null,
      proposalOnCanvas: false,
      pendingProposal: null,
      fingerprint: undefined,
    });
    useApp.getState().watchModel("/models/live.gmdl");
    await waitFor(() => useApp.getState().proposalOnCanvas === true);

    // The reader undoes the adoption: the proposal is still the one of the
    // session, but it is not on the canvas.
    await useApp.getState().undo();
    assert.equal(
      useApp.getState().proposalOnCanvas,
      false,
      "the proposal is still on the canvas after the undo",
    );

    // The tick does not fight the reader.
    await tick();
    const adoptions = shell.ops.filter((op) => op === "adoptProposal").length;
    assert.equal(adoptions, 1, `the app adopted the proposal ${adoptions} times`);
  } finally {
    useApp.getState().stopWatching();
    await close();
  }
});

test("a reader with changes that are not saved keeps them: the proposal waits", async () => {
  const shell = new FakeShell();
  const { useApp, close } = await loadStore();
  try {
    shell.pending = proposalDocument();
    useApp.setState({
      currentPath: "/models/live.gmdl",
      model: { title: "el viejo" } as AppState["model"],
      status: "ready",
      error: null,
      notice: null,
      dirty: true,
      proposal: null,
      proposalOnCanvas: false,
      pendingProposal: null,
      fingerprint: undefined,
    });
    useApp.getState().watchModel("/models/live.gmdl");

    // The app knows that a proposal waits...
    await waitFor(() => useApp.getState().pendingProposal !== null);
    assert.equal(
      useApp.getState().pendingProposal?.id,
      "1700000000000-ab",
      "the app does not know that a proposal waits",
    );
    // ...and it does not step on the work of the person.
    await tick();
    assert.equal(
      shell.ops.filter((op) => op === "adoptProposal").length,
      0,
      `the app adopted the proposal over the work of the reader: ${shell.ops}`,
    );
    assert.equal(
      useApp.getState().proposalOnCanvas,
      false,
      "the proposal is on the canvas although the reader has changes",
    );
  } finally {
    useApp.getState().stopWatching();
    await close();
  }
});

test("accepting the proposal saves the file", async () => {
  const shell = new FakeShell();
  const { useApp, close } = await loadStore();
  try {
    shell.pending = proposalDocument();
    useApp.setState({
      currentPath: "/models/live.gmdl",
      model: { title: "el viejo" } as AppState["model"],
      status: "ready",
      error: null,
      notice: null,
      dirty: false,
      proposal: null,
      proposalOnCanvas: false,
      pendingProposal: null,
      fingerprint: undefined,
    });
    useApp.getState().watchModel("/models/live.gmdl");
    await waitFor(() => useApp.getState().proposalOnCanvas === true);

    await useApp.getState().acceptProposal();

    assert.ok(
      shell.ops.includes("acceptProposal"),
      `the app did not accept the proposal: ${shell.ops}`,
    );
    assert.equal(
      useApp.getState().proposal?.state,
      "accepted",
      "the proposal is not accepted",
    );
    assert.match(
      useApp.getState().savedMessage ?? "",
      /aceptó la propuesta/,
      "the app did not say that the proposal was accepted",
    );
    assert.equal(useApp.getState().error, null, "the acceptance was read as a failure");
  } finally {
    useApp.getState().stopWatching();
    await close();
  }
});

test("the note of the reader travels with the rejection", async () => {
  const shell = new FakeShell();
  const { useApp, close } = await loadStore();
  try {
    shell.pending = proposalDocument();
    useApp.setState({
      currentPath: "/models/live.gmdl",
      model: { title: "el viejo" } as AppState["model"],
      status: "ready",
      error: null,
      notice: null,
      dirty: false,
      proposal: null,
      proposalOnCanvas: false,
      pendingProposal: null,
      fingerprint: undefined,
    });
    useApp.getState().watchModel("/models/live.gmdl");
    await waitFor(() => useApp.getState().proposalOnCanvas === true);

    await useApp.getState().discardProposal("  el bus no cierra  ");

    assert.equal(
      shell.note,
      "el bus no cierra",
      "the note did not travel with the rejection",
    );
    assert.equal(
      useApp.getState().proposal?.state,
      "rejected",
      "the proposal is not rejected",
    );
    assert.match(
      useApp.getState().notice ?? "",
      /el bus no cierra/,
      "the app did not say that the note travelled",
    );
  } finally {
    useApp.getState().stopWatching();
    await close();
  }
});
