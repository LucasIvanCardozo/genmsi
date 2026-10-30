//! The selection of the reader: blocks and doors.
//!
//! A door is a block of the picture, and it is selected like one: the
//! button that deletes what is selected deletes a door as well, and not
//! only the blocks.
//!
//! The store of the app is loaded through the server of Vite, because its
//! own imports have no extension and node cannot load them.

import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";

const appDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

/**
 * The shell of the desktop app, faked.
 *
 * The app asks the shell for its work with `invoke`; the fake keeps what
 * it was asked and answers what the test says.
 */
class FakeShell {
  /** The commands the app asked for, in order. */
  readonly requests: { command: string; args: Record<string, unknown> }[] = [];

  /** The answer that is waiting for the request in flight. */
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

  /** The request in flight is answered. */
  answer(answer: unknown): void {
    const finish = this.finish;
    this.finish = null;
    finish?.(answer);
  }
}

/** The store of the app. */
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

/** The state of the app, as the store holds it. */
interface AppState {
  status: string;
  error: string | null;
  model: unknown;
  selectedBlockIds: number[];
  selectedDoors: { container: number; ioid: number; input: boolean }[];
  select: (ids: number[], doors?: { container: number; ioid: number; input: boolean }[]) => void;
  deleteSelection: () => Promise<void>;
}

/** A model with a container that has an entry door. */
const modelWithADoor = {
  schema: "genmsi.model/1",
  path: "",
  title: "Modelo",
  dirty: false,
  bytes: 0,
  blocks: [
    {
      id: 2,
      class: "",
      kind: "C",
      name: "Contenedor",
      caption: "",
      label: null,
      container: 0,
      layout: { x: 0, y: 0, width: 160, height: 56 },
      properties: [],
      inputs: [
        {
          ioid: 1,
          name: "Entrada1",
          caption: "",
          label: null,
          layout: { x: 0, y: 0, width: 140, height: 44 },
        },
      ],
      outputs: [],
    },
  ],
  edges: [],
};

/** The answer of the shell to a command of the session. */
function sessionAnswer(): unknown {
  return {
    ok: true,
    state: {
      path: null,
      model: modelWithADoor,
      diagnostics: { errors: [], warnings: [] },
      dirty: true,
      canUndo: true,
      canRedo: false,
    },
  };
}

test("the button that deletes what is selected deletes a door too", async () => {
  const shell = new FakeShell();
  const { useApp, close } = await loadStore();
  try {
    useApp.setState({
      model: modelWithADoor,
      selectedBlockIds: [],
      selectedDoors: [{ container: 2, ioid: 1, input: true }],
    });

    const deleting = useApp.getState().deleteSelection();
    shell.answer(sessionAnswer());
    await deleting;

    const request = shell.sessionRequest();
    assert.equal(request.op, "apply");
    assert.deepEqual(
      request.command,
      { op: "removePort", block: 2, ioid: 1, input: true },
      "the door was not deleted",
    );
    // And the selection is empty: the door is gone.
    assert.deepEqual(useApp.getState().selectedDoors, []);
  } finally {
    await close();
  }
});

test("the selection of the doors is the selection of the reader", async () => {
  const { useApp, close } = await loadStore();
  try {
    useApp.setState({ model: modelWithADoor, selectedBlockIds: [], selectedDoors: [] });

    // The canvas says what is selected: a block, a door, or both.
    useApp.getState().select([5], [{ container: 2, ioid: 1, input: true }]);
    assert.deepEqual(useApp.getState().selectedBlockIds, [5]);
    assert.deepEqual(useApp.getState().selectedDoors, [
      { container: 2, ioid: 1, input: true },
    ]);

    // The same selection does not change the state: the canvas reports it
    // again on every render, and a new array would start a loop.
    const before = useApp.getState().selectedDoors;
    useApp.getState().select([5], [{ container: 2, ioid: 1, input: true }]);
    assert.equal(useApp.getState().selectedDoors, before);
  } finally {
    await close();
  }
});
