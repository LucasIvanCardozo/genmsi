import { create } from "zustand";
import { engine } from "../engine";
import type { ModelRef } from "../engine/adapter";
import { relayout } from "../edit/layout";
import {
  buildIndex,
  buildTransitions,
  fittingSpeed,
  type ReplayIndex,
  type TransitionIndex,
} from "../monitor/replay";
import type {
  CatalogueDocument,
  Diagnostics,
  ModelBody,
  ReportDocument,
  TraceDocument,
} from "../types";
import type { EditCommand } from "../types/generated/EditCommand";

/** What the app is doing right now. */
export type Status = "idle" | "loading" | "ready" | "running";

/**
 * How many trace records the monitor asks the engine for.
 *
 * The CLI keeps 100 by default, which is a sample and not a run: the replay
 * needs the whole trace, and the document says when the bound cut it.
 */
export const TRACE_LIMIT = 200_000;

/** How far a pasted copy is moved, so it does not sit on the original. */
const PASTE_OFFSET = { dx: 180, dy: 90 };

export interface AppState {
  /** The models the app offers. */
  models: ModelRef[];
  /** The path of the model that is open. */
  currentPath: string | null;
  /** The open model's graph. */
  model: ModelBody | null;
  /** What the open model compiles to. */
  diagnostics: Diagnostics | null;
  /** The last run's report. */
  report: ReportDocument | null;
  /** The trace of the last run, the one the monitor replays. */
  trace: TraceDocument | null;
  /** The index of the replay: what the monitor reads, built once per run. */
  monitor: ReplayIndex | null;
  /** The moves of the run, read from the trace. */
  transitions: TransitionIndex | null;
  /** The frame the replay is at. */
  playhead: number;
  /** Whether the replay is running. */
  playing: boolean;
  /** How fast the replay runs. */
  speed: number;
  /** The blocks the user selected in the graph. */
  selectedBlockIds: number[];
  /** The blocks the user copied. */
  clipboard: number[];
  /** The seed the next run uses. */
  seed: number;
  /** The horizon the next run uses, in simulated time. */
  until: number | null;
  /** What the app is doing. */
  status: Status;
  /** The message of the last failure, if any. */
  error: string | null;
  /** Whether the model has changes that were not saved. */
  dirty: boolean;
  /** Whether an operation can be undone. */
  canUndo: boolean;
  /** Whether an operation can be redone. */
  canRedo: boolean;
  /** The block classes the editor offers to create. */
  catalogue: CatalogueDocument | null;
  /** The path of a save that waits for the user's confirmation. */
  pendingSave: string | null;
  /** The message of the last save. */
  savedMessage: string | null;
  /** Ask the engine for the models the app offers. */
  loadModels: () => Promise<void>;
  /** Ask the engine for the block classes the editor offers. */
  loadCatalogue: () => Promise<void>;
  /** Open a model: its graph and its diagnostics. */
  openModel: (path: string) => Promise<void>;
  /** Start a new, empty model. */
  newModel: () => Promise<void>;
  /** Apply one editing command. */
  apply: (command: EditCommand) => Promise<void>;
  /** Undo the last operation. */
  undo: () => Promise<void>;
  /** Redo the last undone operation. */
  redo: () => Promise<void>;
  /** Save the model; an invalid model asks for a confirmation first. */
  save: (path?: string) => Promise<void>;
  /** Run the open model and read its report. */
  simulate: () => Promise<void>;
  /** Run the open model and prepare its replay. */
  runMonitor: () => Promise<void>;
  /** Move the replay to a frame. */
  setPlayhead: (frame: number) => void;
  /** Play or pause the replay. */
  setPlaying: (playing: boolean) => void;
  /** Set how fast the replay runs. */
  setSpeed: (speed: number) => void;
  /** Select the blocks of the graph. */
  select: (ids: number[]) => void;
  /** Add a block of a class, at a position or in the first free spot. */
  addBlock: (className: string, x?: number, y?: number) => Promise<void>;
  /** Delete the selected blocks. */
  deleteSelection: () => Promise<void>;
  /** Copy the selected blocks. */
  copy: () => void;
  /** Paste the copied blocks. */
  paste: () => Promise<void>;
  /** Reorder the graph in layers. */
  relayout: () => Promise<void>;
  /** Set the seed of the next run. */
  setSeed: (seed: number) => void;
  /** Set the horizon of the next run. */
  setUntil: (until: number | null) => void;
  /** Answer the confirmation of a save with yes. */
  confirmSave: () => Promise<void>;
  /** Answer the confirmation of a save with no. */
  dismissSave: () => void;
}

/** The message of a failure the engine reported. */
function messageOf(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

/**
 * A free spot for a new block: under the model, at its left.
 *
 * The spot stays inside the picture the author left, so the graph does not
 * grow every time a block is added and the view stays readable.
 */
function freeSpot(model: ModelBody): { x: number; y: number } {
  const placed = model.blocks.filter((block) => block.layout !== null);
  if (placed.length === 0) {
    return { x: 40, y: 40 };
  }
  const left = Math.min(...placed.map((block) => block.layout?.x ?? 0));
  const bottom = Math.max(
    ...placed.map((block) => (block.layout?.y ?? 0) + (block.layout?.height ?? 0)),
  );
  return { x: left, y: bottom + 60 };
}

/**
 * The container a point belongs to.
 *
 * A block dropped inside a container joins it; a block dropped outside any
 * container joins the model root.
 */
function containerAt(model: ModelBody, x: number, y: number): number {
  const root = model.blocks.find((block) => block.class === "main");
  const containers = model.blocks.filter(
    (block) =>
      block.kind === "C" &&
      block.layout !== null &&
      block.id !== root?.id,
  );
  const hits = containers
    .filter((block) => {
      const layout = block.layout;
      if (layout === null) {
        return false;
      }
      return (
        x >= layout.x &&
        x <= layout.x + layout.width &&
        y >= layout.y &&
        y <= layout.y + layout.height
      );
    })
    .sort(
      (a, b) =>
        (a.layout?.width ?? 0) * (a.layout?.height ?? 0) -
        (b.layout?.width ?? 0) * (b.layout?.height ?? 0),
    );
  return hits[0]?.id ?? root?.id ?? 0;
}

export const useApp = create<AppState>((set, get) => ({
  models: [],
  currentPath: null,
  model: null,
  diagnostics: null,
  report: null,
  trace: null,
  monitor: null,
  transitions: null,
  playhead: 0,
  playing: false,
  speed: 1,
  selectedBlockIds: [],
  clipboard: [],
  seed: 1,
  until: null,
  status: "idle",
  error: null,
  dirty: false,
  canUndo: false,
  canRedo: false,
  catalogue: null,
  pendingSave: null,
  savedMessage: null,

  loadModels: async () => {
    try {
      const models = await engine().listModels();
      set({ models, error: null });
    } catch (error) {
      set({ error: `No se pudieron listar los modelos: ${messageOf(error)}` });
    }
  },

  loadCatalogue: async () => {
    try {
      const catalogue = await engine().catalogue();
      set({ catalogue, error: null });
    } catch (error) {
      set({ error: `No se pudo cargar el catálogo de bloques: ${messageOf(error)}` });
    }
  },

  openModel: async (path: string) => {
    set({ status: "loading", error: null, currentPath: path });
    try {
      const state = await engine().openModel(path);
      set({
        model: state.model,
        diagnostics: state.diagnostics,
        currentPath: state.path,
        dirty: state.dirty,
        canUndo: state.canUndo,
        canRedo: state.canRedo,
        report: null,
        trace: null,
        monitor: null,
        transitions: null,
        playhead: 0,
        playing: false,
        selectedBlockIds: [],
        clipboard: [],
        status: "ready",
        savedMessage: null,
        error: null,
      });
    } catch (error) {
      set({
        status: "idle",
        model: null,
        diagnostics: null,
        report: null,
        error: `No se pudo abrir el modelo: ${messageOf(error)}`,
      });
    }
  },

  newModel: async () => {
    set({ status: "loading", error: null });
    try {
      const state = await engine().newModel();
      set({
        model: state.model,
        diagnostics: state.diagnostics,
        currentPath: state.path,
        dirty: state.dirty,
        canUndo: state.canUndo,
        canRedo: state.canRedo,
        report: null,
        trace: null,
        monitor: null,
        transitions: null,
        playhead: 0,
        playing: false,
        selectedBlockIds: [],
        clipboard: [],
        status: "ready",
        savedMessage: null,
        error: null,
      });
    } catch (error) {
      set({ status: "idle", error: `No se pudo crear el modelo: ${messageOf(error)}` });
    }
  },

  apply: async (command: EditCommand) => {
    try {
      const state = await engine().apply(command);
      set({
        model: state.model,
        diagnostics: state.diagnostics,
        dirty: state.dirty,
        canUndo: state.canUndo,
        canRedo: state.canRedo,
        // The model moved: the run the monitor replays is not the model any
        // more, so the replay goes away with it.
        report: null,
        trace: null,
        monitor: null,
        transitions: null,
        playhead: 0,
        playing: false,
        error: null,
        status: "ready",
      });
    } catch (error) {
      set({ error: `No se pudo aplicar el cambio: ${messageOf(error)}` });
    }
  },

  undo: async () => {
    try {
      const state = await engine().undo();
      set({
        model: state.model,
        diagnostics: state.diagnostics,
        dirty: state.dirty,
        canUndo: state.canUndo,
        canRedo: state.canRedo,
        report: null,
        trace: null,
        monitor: null,
        transitions: null,
        playhead: 0,
        playing: false,
        error: null,
      });
    } catch (error) {
      set({ error: `No se pudo deshacer: ${messageOf(error)}` });
    }
  },

  redo: async () => {
    try {
      const state = await engine().redo();
      set({
        model: state.model,
        diagnostics: state.diagnostics,
        dirty: state.dirty,
        canUndo: state.canUndo,
        canRedo: state.canRedo,
        report: null,
        trace: null,
        monitor: null,
        transitions: null,
        playhead: 0,
        playing: false,
        error: null,
      });
    } catch (error) {
      set({ error: `No se pudo rehacer: ${messageOf(error)}` });
    }
  },

  save: async (path?: string) => {
    const state = get();
    // A model the engine refuses is saved anyway, but the user is asked
    // first: the saving is never blocked (S10).
    if (state.diagnostics?.status === "error" && state.pendingSave === null) {
      set({ pendingSave: path ?? "" });
      return;
    }
    try {
      const result = await engine().save(path);
      set({
        model: result.state.model,
        diagnostics: result.state.diagnostics,
        dirty: result.state.dirty,
        canUndo: result.state.canUndo,
        canRedo: result.state.canRedo,
        currentPath: result.state.path,
        pendingSave: null,
        savedMessage: `Se guardó ${result.path} (${result.bytes} bytes).`,
        error: null,
      });
    } catch (error) {
      set({ error: `No se pudo guardar el modelo: ${messageOf(error)}` });
    }
  },

  confirmSave: async () => {
    const path = get().pendingSave;
    set({ pendingSave: null });
    await get().save(path === null || path === "" ? undefined : path);
  },

  dismissSave: () => set({ pendingSave: null }),

  simulate: async () => {
    set({ status: "running", error: null });
    try {
      const report = await engine().simulateCurrent({
        seed: get().seed,
        until: get().until ?? undefined,
      });
      set({
        report,
        // The run is not the one the monitor replays: the replay needs the
        // trace, and the monitor asks for it when it runs the model.
        trace: null,
        monitor: null,
        transitions: null,
        playhead: 0,
        playing: false,
        status: "ready",
      });
    } catch (error) {
      set({ status: "ready", error: `No se pudo simular el modelo: ${messageOf(error)}` });
    }
  },

  runMonitor: async () => {
    const model = get().model;
    if (model === null) {
      set({ error: "No hay ningún modelo abierto para reproducir." });
      return;
    }
    set({ status: "running", error: null });
    try {
      const options = {
        seed: get().seed,
        until: get().until ?? undefined,
        limit: TRACE_LIMIT,
      };
      // The report and the trace are two runs of the same model with the same
      // seed: the engine is deterministic, so the trace belongs to the report.
      const report = await engine().simulateCurrent(options);
      const trace = await engine().traceCurrent(options);
      const monitor = buildIndex(trace, report, model);
      // The moves of the run, read from the trace. The program blocks are
      // mapped to the blocks of the model, because a ball travels between the
      // rectangles the reader sees.
      const nodeOf = (block: number) => report.blocks[block]?.node ?? null;
      const transitions = buildTransitions(trace, nodeOf);
      set({
        report,
        trace,
        monitor,
        transitions,
        playhead: 0,
        playing: monitor.frames.length > 0,
        // The speed is a multiple of the real time. The default is the one
        // that fits the run in a few seconds, and the monitor says which one
        // it is: the reader always knows the scale.
        speed: fittingSpeed(monitor.endTime - monitor.startTime),
        status: "ready",
      });
    } catch (error) {
      set({
        status: "ready",
        error: `No se pudo reproducir la simulación: ${messageOf(error)}`,
      });
    }
  },

  setPlayhead: (frame: number) => set({ playhead: frame }),

  setPlaying: (playing: boolean) => set({ playing }),

  setSpeed: (speed: number) => set({ speed }),

  select: (ids: number[]) => {
    // The same selection does not change the state: the graph reports it
    // again on every render, and a new array would start a loop.
    const current = get().selectedBlockIds;
    const same =
      current.length === ids.length && current.every((id) => ids.includes(id));
    if (!same) {
      set({ selectedBlockIds: ids });
    }
  },

  addBlock: async (className: string, x?: number, y?: number) => {
    const model = get().model;
    if (model === null) {
      return;
    }
    const spot = freeSpot(model);
    const left = x ?? spot.x;
    const top = y ?? spot.y;
    await get().apply({
      op: "addBlock",
      class: className,
      container: containerAt(model, left, top),
      x: left,
      y: top,
    });
  },

  deleteSelection: async () => {
    const ids = get().selectedBlockIds;
    if (ids.length === 0) {
      return;
    }
    if (ids.length === 1) {
      await get().apply({ op: "removeBlock", id: ids[0] });
    } else {
      await get().apply({ op: "removeBlocks", ids });
    }
    set({ selectedBlockIds: [] });
  },

  copy: () => set({ clipboard: [...get().selectedBlockIds] }),

  paste: async () => {
    const clipboard = get().clipboard;
    if (clipboard.length === 0) {
      return;
    }
    await get().apply({
      op: "paste",
      ids: clipboard,
      dx: PASTE_OFFSET.dx,
      dy: PASTE_OFFSET.dy,
    });
  },

  relayout: async () => {
    const model = get().model;
    if (model === null) {
      return;
    }
    const root = model.blocks.find((block) => block.class === "main");
    if (root === undefined) {
      return;
    }
    const positions = relayout(model.blocks, model.edges, root.id);
    if (positions.length === 0) {
      return;
    }
    await get().apply({
      op: "setLayouts",
      positions: positions.map((position) => ({
        id: position.id,
        x: position.x,
        y: position.y,
        width: position.width,
        height: position.height,
      })),
    });
  },

  setSeed: (seed: number) => set({ seed }),

  setUntil: (until: number | null) => set({ until }),
}));

// El store se ve desde la consola del navegador mientras se desarrolla.
if (import.meta.env?.DEV) {
  (globalThis as { __app?: typeof useApp }).__app = useApp;
}
