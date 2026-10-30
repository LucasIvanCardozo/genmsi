//! The canvas of a scope: its own origin, its doors beside their blocks.
//!
//! A scope is a canvas of its own, and the canvas draws the scope the way
//! the author left it: the blocks of the scope are drawn where the model
//! places them —the picture is not translated, so a drag of a block moves
//! that block and nothing else—, the doors of a container are drawn beside
//! the block they join, the free spot of the scope is computed in the
//! scope's own space, and a block dropped on the box of a container joins
//! that container.
//!
//! The file of the model does not change: it keeps the absolute
//! coordinates of its blocks, exactly like the reference models do.

import assert from "node:assert/strict";
import path from "node:path";
import { after, before, test } from "node:test";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";
import type { ViteDevServer } from "vite";
import {
  freeSpotOf,
  graphOf,
  scopeOrigin,
} from "../src/graph/graph.ts";
import type { ModelBody } from "../src/types/index.ts";

/** A block of the model, as the engine publishes it. */
function block(
  id: number,
  name: string,
  extra: Partial<ModelBody["blocks"][number]> = {},
): ModelBody["blocks"][number] {
  return {
    id,
    class: name,
    kind: "A",
    name,
    caption: "",
    label: null,
    container: 0,
    layout: { x: 10 * id, y: 20 * id, width: 100, height: 40 },
    properties: [],
    inputs: [],
    outputs: [],
    ...extra,
  };
}

/** A link of the model, as the engine publishes it. */
function edge(
  id: string,
  container: number,
  fromModule: number,
  fromIoid: number,
  toModule: number,
  toIoid: number,
): ModelBody["edges"][number] {
  return {
    id,
    container,
    from_module: fromModule,
    from_ioid: fromIoid,
    from_port: "Salida1",
    to_module: toModule,
    to_ioid: toIoid,
    to_port: "Entrada1",
    label: null,
    hops: [],
  };
}

/** A model with the blocks and the links the caller declares. */
function modelOf(
  blocks: ModelBody["blocks"],
  edges: ModelBody["edges"] = [],
): ModelBody {
  return {
    schema: "genmsi.model/1",
    path: "/tmp/model.gmdl",
    title: "Modelo",
    dirty: false,
    bytes: 0,
    blocks,
    edges,
  } as unknown as ModelBody;
}

/**
 * A model with a container and the two blocks it holds.
 *
 * The container is at the root of the model, and it holds a `Queue` that
 * the flow comes in through and a `Leave` that the flow goes out through.
 */
function scopeModel(): ModelBody {
  return modelOf(
    [
      block(1, "Generate", {
        container: 0,
        layout: { x: 100, y: 200, width: 120, height: 63 },
      }),
      block(10, "", {
        kind: "C",
        class: "",
        container: 0,
        layout: { x: 50, y: 80, width: 130, height: 45 },
        inputs: [{ ioid: 1, name: "Entrada1", caption: "", label: null, layout: null }],
        outputs: [{ ioid: 2, name: "Salida1", caption: "", label: null, layout: null }],
      }),
      block(11, "Queue", {
        container: 10,
        layout: { x: 300, y: 400, width: 100, height: 40 },
      }),
      block(12, "Leave", {
        container: 10,
        layout: { x: 500, y: 700, width: 100, height: 40 },
      }),
    ],
    [
      // The link of the scope that feeds the entry port: the door of the
      // entry joins the block it points at.
      edge("10:10:1->11:1", 10, 10, 1, 11, 1),
      // The link of the scope that ends at the exit port: the door of the
      // exit joins the block the flow leaves by.
      edge("10:12:2->10:2", 10, 12, 2, 10, 2),
    ],
  );
}

test("the origin of a scope is the corner of the blocks it holds", () => {
  const model = scopeModel();
  // The root scope starts at the corner of its own blocks: the container is
  // the topmost one, and the picture starts there.
  assert.deepEqual(scopeOrigin(model, 0), { x: 50, y: 80 });
  // The scope of the container starts at the corner of the blocks the
  // container holds.
  assert.deepEqual(scopeOrigin(model, 10), { x: 300, y: 400 });
  // A scope with no block of its own has no corner: the picture starts at
  // the origin of the canvas.
  assert.deepEqual(scopeOrigin(model, 99), { x: 0, y: 0 });
});

test("the nodes of a scope are drawn where the model places them", () => {
  const model = scopeModel();
  const root = graphOf(model, 0);
  const container = root.nodes.find((node) => node.id === "10");
  const generator = root.nodes.find((node) => node.id === "1");
  // The picture is not translated: the place of a block on the canvas is
  // the place the model gives it.
  assert.deepEqual(container?.position, { x: 50, y: 80 });
  assert.deepEqual(generator?.position, { x: 100, y: 200 });

  const scope = graphOf(model, 10);
  const queue = scope.nodes.find((node) => node.id === "11");
  const leave = scope.nodes.find((node) => node.id === "12");
  assert.deepEqual(queue?.position, { x: 300, y: 400 });
  assert.deepEqual(leave?.position, { x: 500, y: 700 });
});

test("a block that is dragged moves only that block", () => {
  const model = scopeModel();
  const dragged = scopeModel();
  // The block at the corner of the scope is dragged to the other corner of
  // the canvas: the origin of the scope moves with it, and the picture of
  // the scope does not slide under the reader.
  dragged.blocks = dragged.blocks.map((block) =>
    block.id === 11
      ? { ...block, layout: { x: 0, y: 0, width: 100, height: 40 } }
      : block,
  );
  const before = graphOf(model, 10).nodes;
  const after = graphOf(dragged, 10).nodes;
  const leaveBefore = before.find((node) => node.id === "12");
  const leaveAfter = after.find((node) => node.id === "12");
  assert.deepEqual(
    leaveAfter?.position,
    leaveBefore?.position,
    "the picture of the scope slid when the corner block moved",
  );
  // The origin of the scope did move, though: the corner of a scope is the
  // corner of the blocks it holds.
  assert.deepEqual(scopeOrigin(model, 10), { x: 300, y: 400 });
  assert.deepEqual(scopeOrigin(dragged, 10), { x: 0, y: 0 });
});

test("the model keeps the absolute coordinates of the blocks of a scope", () => {
  const model = scopeModel();
  // The picture is drawn where the model places the blocks, and the file
  // does not change: the blocks of the scope keep the coordinates the
  // author gave them.
  graphOf(model, 10);
  const queue = model.blocks.find((block) => block.id === 11);
  assert.deepEqual(queue?.layout, { x: 300, y: 400, width: 100, height: 40 });
});

test("a door is drawn where the model puts it", () => {
  // The door is a block of the picture: the model carries its place, and
  // the canvas draws it there.
  const model = scopeModel();
  const container = model.blocks.find((block) => block.id === 10);
  assert.ok(container !== undefined);
  container.inputs[0].layout = { x: 250, y: 300, width: 140, height: 44 };
  const { nodes } = graphOf(model, 10);
  const entry = nodes.find((node) => node.id === "door:10:1:in");
  assert.deepEqual(entry?.position, { x: 250, y: 300 });
});

test("a door with no place goes to the free spot of the scope", () => {
  // A door the reader has not placed yet —a new one— goes to the free
  // spot of the scope, under the doors that are already there: two doors
  // never share a place.
  const model = scopeModel();
  const { nodes } = graphOf(model, 10);
  const entry = nodes.find((node) => node.id === "door:10:1:in");
  const exit = nodes.find((node) => node.id === "door:10:2:out");
  assert.ok(entry !== undefined && exit !== undefined);
  assert.notDeepEqual(entry.position, exit.position);
  // And the place of the door is in the canvas of the scope.
  const spot = freeSpotOf(model, 10).position;
  assert.deepEqual(entry.position, spot);
  assert.equal(exit.position.x, spot.x);
  assert.ok(exit.position.y > entry.position.y);
});

test("moving a block does not move the doors", () => {
  // The doors are blocks of their own: moving a block of the scope leaves
  // the doors where they are.
  const model = scopeModel();
  const container = model.blocks.find((block) => block.id === 10);
  assert.ok(container !== undefined);
  container.inputs[0].layout = { x: 250, y: 300, width: 140, height: 44 };
  const before = graphOf(model, 10);
  const queue = model.blocks.find((block) => block.id === 11);
  assert.ok(queue?.layout !== undefined && queue.layout !== null);
  queue.layout = { ...queue.layout, x: 320, y: 460 };
  const after = graphOf(model, 10);
  const door = (graph: ReturnType<typeof graphOf>) =>
    graph.nodes.find((node) => node.id === "door:10:1:in")?.position;
  assert.deepEqual(door(after), door(before));
});

test("the free spot of a scope is in the scope's own space", () => {
  const model = scopeModel();
  const origin = scopeOrigin(model, 10);
  const spot = freeSpotOf(model, 10);
  // The spot is under the lowest block of the scope, at the left of its
  // picture, in the space of the scope.
  assert.deepEqual(spot.spot, { x: 0, y: 340 + 40 });
  // And the position of the model is that spot plus the origin of the
  // scope: the file keeps the absolute coordinates.
  assert.deepEqual(spot.position, {
    x: spot.spot.x + origin.x,
    y: spot.spot.y + origin.y,
  });
  assert.deepEqual(spot.position, { x: 300, y: 780 });
});

test("the free spot of the root scope is under the lowest block of the root", () => {
  const model = scopeModel();
  const origin = scopeOrigin(model, 0);
  const spot = freeSpotOf(model, 0);
  // The root holds the `Generate` and the container: the spot is under the
  // lowest of them, which is the `Generate`.
  assert.deepEqual(spot.spot, { x: 0, y: 120 + 63 + 40 });
  assert.deepEqual(spot.position, { x: origin.x, y: origin.y + 120 + 63 + 40 });
});

/**
 * The state of the app, as the store holds it.
 */
interface AppState {
  status: string;
  error: string | null;
  model: ModelBody | null;
  focusBlockId: number | null;
  focusToken: number;
  activeContainerId: number;
  addBlock: (className: string, x?: number, y?: number) => Promise<void>;
  focusBlock: (id: number) => void;
}

/**
 * The shell of the desktop app, faked.
 *
 * The app asks the shell for its work with `invoke`; the fake keeps what
 * it was asked and answers with the state of a session. The fake does what
 * the engine does with an `addBlock`: the block arrives in the model, with
 * a fresh id.
 */
class FakeShell {
  /** The commands the app asked for, in order. */
  readonly requests: { command: string; args: Record<string, unknown> }[] = [];
  /** The model the fake keeps in memory. */
  model: ModelBody;

  constructor(model: ModelBody) {
    this.model = model;
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: (command: string, args?: Record<string, unknown>) => {
        this.requests.push({ command, args: args ?? {} });
        this.apply();
        return Promise.resolve(JSON.stringify({ ok: true, state: this.state() }));
      },
    };
  }

  /**
   * What the engine does with the edit the app asked for.
   *
   * Only the addition of a block is faked: the block arrives in the model
   * with a fresh id, the way the engine gives it one.
   */
  apply(): void {
    const request = this.requests.at(-1);
    if (request === undefined || request.command !== "session") {
      return;
    }
    const parsed = JSON.parse(request.args.request as string) as {
      op?: string;
      command?: { op?: string; class?: string; container?: number; x?: number; y?: number };
    };
    const command = parsed.command;
    if (parsed.op !== "apply" || command?.op !== "addBlock") {
      return;
    }
    const id = Math.max(0, ...this.model.blocks.map((block) => block.id)) + 1;
    this.model = {
      ...this.model,
      blocks: [
        ...this.model.blocks,
        {
          id,
          class: command.class ?? "",
          kind: "A",
          name: command.class ?? "",
          caption: "",
          label: null,
          container: command.container ?? 0,
          layout: { x: command.x ?? 0, y: command.y ?? 0, width: 80, height: 32 },
          properties: [],
          inputs: [],
          outputs: [],
        },
      ],
    } as unknown as ModelBody;
  }

  /** The state of the session the fake answers with. */
  state(): Record<string, unknown> {
    return {
      path: "modelo.gmdl",
      model: this.model,
      diagnostics: {
        status: "ok",
        diagnostics: [],
        message: null,
        blocks: this.model.blocks.length,
        definitions: null,
      },
      dirty: false,
      canUndo: true,
      canRedo: false,
    };
  }

  /** The request of the session the app sent last. */
  sessionRequest(): Record<string, unknown> {
    const request = this.requests
      .filter((entry) => entry.command === "session")
      .at(-1);
    assert.ok(request !== undefined, "the app did not ask the shell for a session");
    return JSON.parse(request.args.request as string) as Record<string, unknown>;
  }

  /** The command of the edit the app sent last. */
  editCommand(): Record<string, unknown> {
    const request = this.sessionRequest();
    assert.equal(request.op, "apply", "the app did not ask the shell to edit the model");
    return request.command as Record<string, unknown>;
  }
}

/**
 * The server of Vite, which loads the modules of the app.
 *
 * The store's own imports have no extension, so node cannot load them: the
 * server resolves them the way the browser does, and the module that comes
 * back is the same one the app runs.
 */
const appDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

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
async function module<T>(modulePath: string): Promise<T> {
  return (await server.ssrLoadModule(modulePath)) as T;
}

test("a block dropped on the box of a container joins that container", async () => {
  const model = scopeModel();
  const shell = new FakeShell(model);
  const { useApp } = await module<{
    useApp: {
      getState: () => AppState;
      setState: (state: Partial<AppState>) => void;
    };
  }>("/src/state/store.ts");

  // The reader is at the root of the model and drops a block on the box of
  // the container.
  useApp.setState({ model, activeContainerId: 0, status: "ready", error: null });
  await useApp.getState().addBlock("Advance", 60, 90);

  const command = shell.editCommand();
  assert.equal(command.op, "addBlock", "the app did not add a block");
  assert.equal(
    command.container,
    10,
    "the block did not join the container under the pointer",
  );
  // The block lands inside the picture of the container it joined: the
  // place it was dropped at belongs to the scope the reader was in.
  const spot = freeSpotOf(model, 10);
  assert.deepEqual(
    { x: command.x, y: command.y },
    spot.position,
    "the block did not land inside the picture of the container",
  );
});

test("a block dropped outside any container joins the scope of the reader", async () => {
  const model = scopeModel();
  const shell = new FakeShell(model);
  const { useApp } = await module<{
    useApp: {
      getState: () => AppState;
      setState: (state: Partial<AppState>) => void;
    };
  }>("/src/state/store.ts");

  // The reader is at the root and drops the block far from the container.
  useApp.setState({ model, activeContainerId: 0, status: "ready", error: null });
  await useApp.getState().addBlock("Advance", 900, 900);

  const command = shell.editCommand();
  assert.equal(command.op, "addBlock", "the app did not add a block");
  assert.equal(command.container, 0, "the block did not join the scope of the reader");
  assert.deepEqual(
    { x: command.x, y: command.y },
    { x: 900, y: 900 },
    "the block did not land where it was dropped",
  );
});

test("a block added inside a container lands inside the picture of the scope", async () => {
  const model = scopeModel();
  const shell = new FakeShell(model);
  const { useApp } = await module<{
    useApp: {
      getState: () => AppState;
      setState: (state: Partial<AppState>) => void;
    };
  }>("/src/state/store.ts");

  // The reader is inside the container and adds a block without saying
  // where: the block goes to the free spot of the scope.
  useApp.setState({ model, activeContainerId: 10, status: "ready", error: null });
  await useApp.getState().addBlock("Advance");

  const command = shell.editCommand();
  assert.equal(command.op, "addBlock", "the app did not add a block");
  assert.equal(command.container, 10, "the block did not join the scope of the reader");
  const spot = freeSpotOf(model, 10);
  assert.deepEqual(
    { x: command.x, y: command.y },
    spot.position,
    "the block did not land in the free spot of the scope",
  );
  // And the canvas does **not** follow the block that was added: the
  // reader stays where he is.
  const state = useApp.getState();
  assert.equal(
    state.focusBlockId,
    null,
    "the canvas moved to the block that was added",
  );
});

test("the canvas follows the block the reader picks in the tree", async () => {
  const model = scopeModel();
  new FakeShell(model);
  const { useApp } = await module<{
    useApp: {
      getState: () => AppState;
      setState: (state: Partial<AppState>) => void;
    };
  }>("/src/state/store.ts");

  useApp.setState({ model, activeContainerId: 0, status: "ready", error: null });
  const before = useApp.getState().focusToken;

  // The reader picks a block in the tree: the canvas follows it.
  useApp.getState().focusBlock(4);

  const state = useApp.getState();
  assert.equal(state.focusBlockId, 4, "the canvas was not told to follow the block");
  assert.ok(
    state.focusToken > before,
    "the request to follow a block did not grow the token",
  );

  // And the reader picks the same block again: the canvas follows it
  // again, because the request is a new one.
  const token = state.focusToken;
  useApp.getState().focusBlock(4);
  assert.ok(
    useApp.getState().focusToken > token,
    "the same block did not get a new request",
  );
  assert.equal(useApp.getState().focusBlockId, 4, "the canvas lost the block");
});
