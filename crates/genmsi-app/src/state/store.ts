import { create } from "zustand";
import { engine } from "../engine";
import { files } from "../engine/files";
import { adviceFor } from "../engine/reasons";
import type { FileFingerprint, ModelRef } from "../engine/adapter";
import {
  buildIndex,
  buildTransitions,
  frameAt,
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
 * The notice of a run that was stopped.
 *
 * The engine cannot be interrupted yet: what the app does is stop waiting
 * for the run and throw its answer away. The notice says exactly that,
 * because a button that says "cancel" and does not cancel is a lie.
 */
export const STOPPED_NOTICE =
  "La corrida se canceló: el motor la cortó y su resultado se descartó.";

/**
 * The horizon of the run when the reader does not choose one.
 *
 * 3600 is the horizon of the original: the `Clock` block has 3600 as its
 * default frequency, and the 66 runs of the comparison against the oracle
 * last 3600 units of simulated time.
 *
 * A run without a horizon is a run that lasts until the engine runs out of
 * patience, so the app puts one there. The reader can change it —or leave
 * it empty— from the toolbar.
 */
export const DEFAULT_UNTIL = 3600;

/**
 * The `START` of the run when the reader does not choose one.
 *
 * The `START` is the counter of terminations of the run: the original has it
 * in the panel of its simulator ("Start:", one by default) and the model does
 * not declare it anywhere, so it is a parameter of the run, like the
 * horizon. One is the default of the original, and with it the runs of the
 * app are the runs of the original.
 */
export const DEFAULT_START = 1;

/**
 * How many trace records the monitor asks the engine for.
 *
 * The CLI keeps 100 by default, which is a sample and not a run: the replay
 * needs the whole trace, and the document says when the bound cut it.
 */
export const TRACE_LIMIT = 200_000;

/**
 * How often the app asks for the fingerprint of the file it has open.
 *
 * Half a second: not faster —the app must not spend its time on a file that
 * did not change— and not slower —the reader has to see the change of the
 * agent soon—.
 */
export const WATCH_INTERVAL_MS = 500;

/**
 * The timer of the watcher of the open file.
 *
 * The watcher lives outside the state: it is a clock, not something the app
 * draws.
 */
let watchTimer: ReturnType<typeof setInterval> | null = null;

/** The file the watcher is watching. */
let watchedPath: string | null = null;

/** How far a pasted copy is moved, so it does not sit on the original. */
const PASTE_OFFSET = { dx: 180, dy: 90 };

/** How many models the list of recents keeps. */
const RECENT_LIMIT = 8;

/** The name the app suggests when the model has no path yet. */
const DEFAULT_MODEL_NAME = "modelo.gmdl";

/**
 * A seed for a run.
 *
 * A run is entirely determined by its seed, so a run that does not take one
 * draws it and says which it used: the reader repeats the run with that
 * number. The number stays inside the safe range of a JavaScript number, so
 * the engine receives it exactly.
 */
export function randomSeed(): number {
  return 1 + Math.floor(Math.random() * 0xffffffff);
}

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
  /** The container currently shown in the canvas (0 = model root). */
  activeContainerId: number;
  /** The blocks the user copied. */
  clipboard: number[];
  /** The seed the next run uses. */
  seed: number;
  /** Whether the seed is the one the app drew, rather than one the reader
   * chose. A run that draws its own seed is a different run every time; one
   * the reader chose is the same run every time. */
  seedIsAuto: boolean;
  /** The horizon the next run uses, in simulated time. */
  until: number | null;
  /** The `START` of the next run: how many termination units end it. */
  start: number;
  /** The commands that were applied, so the app knows whether the change it
   * is undoing moved the blocks or changed the program. */
  history: EditCommand[];
  /** The commands that were undone, ready to be redone. */
  redoStack: EditCommand[];
  /** What the app is doing. */
  status: Status;
  /** Unique generation token for the active run to prevent race conditions. */
  runId: number;
  /** Whether the reader asked for the run in flight to stop. */
  cancelling: boolean;
  /** The message of the last notice, if any. */
  notice: string | null;
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
  /** The models opened lately, the most recent first. */
  recentModels: string[];
  /**
   * The fingerprint of the file the app has open.
   *
   * `undefined` means the app has not looked at the file yet; `null` means
   * the file is not there.
   */
  fingerprint: FileFingerprint | null | undefined;
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
  /** Save the model under a name the user chooses with the system dialog. */
  saveAs: () => Promise<void>;
  /** Run the open model and read its report. */
  simulate: () => Promise<void>;
  /** Run the open model and prepare its replay. */
  runMonitor: () => Promise<void>;
  /** Ask the engine to stop the run in flight. */
  cancelRun: () => Promise<void>;
  /** Take the last notice away. */
  clearNotice: () => void;
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
  /** Say whether the seed is the one the app draws or the one the reader
   * chose. Drawing a seed makes every run a different run; a chosen seed
   * makes it the same run every time. */
  setSeedAuto: (auto: boolean) => void;
  /** Set the horizon of the next run. */
  setUntil: (until: number | null) => void;
  /** Set the `START` of the next run. */
  setStart: (start: number) => void;
  /** Answer the confirmation of a save with yes. */
  confirmSave: () => Promise<void>;
  /** Answer the confirmation of a save with no. */
  dismissSave: () => void;
  /** Take the notice of the last save away. */
  clearSavedMessage: () => void;
  /** Take the error message away. */
  clearError: () => void;
  /** Ask the host for the models opened lately. */
  loadRecentModels: () => Promise<void>;
  /** Watch the file the app has open, and reload the model when it changes. */
  watchModel: (path: string) => void;
  /** Stop watching the file. */
  stopWatching: () => void;
  /** Ask for the fingerprint of the open file, and reload the model if it
   * changed. */
  checkModelFile: () => Promise<void>;
  /** Set which container is shown in the canvas. */
  setActiveContainerId: (id: number) => void;
  /** Reload the model of the disk and run it again. */
  reloadFromDisk: () => Promise<void>;
}

/** The message of a failure the engine reported. */
function messageOf(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

/**
 * Whether two fingerprints are the same.
 *
 * The date of the modification is in nanoseconds, so two writes in the same
 * millisecond are two different fingerprints.
 */
function sameFingerprint(
  a: FileFingerprint | null,
  b: FileFingerprint | null,
): boolean {
  if (a === null || b === null) {
    return a === b;
  }
  return a.modifiedNanos === b.modifiedNanos && a.size === b.size;
}

/**
 * The fingerprint of a file, or nothing when it cannot be read.
 *
 * `undefined` is "the app does not know": the file could not be read, and
 * the next look will ask again.
 */
async function fingerprintOf(
  path: string | null,
): Promise<FileFingerprint | null | undefined> {
  if (path === null) {
    return undefined;
  }
  try {
    return await engine().fingerprint(path);
  } catch {
    return undefined;
  }
}

/**
 * What the app says about a failure of the engine.
 *
 * A reason the app knows is an advice: it says what to do about it, and it
 * is not an error —the model is not broken, the run is what could not
 * start. A reason the app does not know is a plain failure with the
 * engine's own message, which is never hidden.
 */
function failureOf(
  error: unknown,
  what: string,
): { error: string | null; notice: string | null } {
  const advice = adviceFor(messageOf(error));
  if (advice !== null) {
    return { error: null, notice: advice };
  }
  return { error: `${what}: ${messageOf(error)}`, notice: null };
}

/**
 * A free spot for a new block: under the model, at its left.
 *
 * The spot stays inside the picture the author left, so the graph does not
 * grow every time a block is added and the view stays readable.
 */
function freeSpot(model: ModelBody, containerId = 0): { x: number; y: number } {
  const placed = model.blocks.filter(
    (block) => block.layout !== null && block.container === containerId && block.id !== 0,
  );
  if (placed.length === 0) {
    return { x: 80, y: 80 };
  }
  const left = Math.min(...placed.map((block) => block.layout?.x ?? 0));
  const bottom = Math.max(
    ...placed.map((block) => (block.layout?.y ?? 0) + (block.layout?.height ?? 0)),
  );
  return { x: left, y: bottom + 40 };
}

/**
 * The commands that only move the blocks.
 *
 * Moving a block does not change the program: what the engine simulates is
 * the program, not the geometry. The run the monitor replays is therefore
 * still the run of the model, and it stays on the screen.
 */
const COSMETIC_COMMANDS: ReadonlySet<string> = new Set([
  "moveBlock",
  "setLayouts",
  "setCaption",
]);

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
  activeContainerId: 0,
  clipboard: [],
  seed: randomSeed(),
  seedIsAuto: true,
  until: DEFAULT_UNTIL,
  start: DEFAULT_START,
  status: "idle",
  runId: 0,
  cancelling: false,
  notice: null,
  history: [],
  redoStack: [],
  error: null,
  dirty: false,
  canUndo: false,
  canRedo: false,
  catalogue: null,
  pendingSave: null,
  savedMessage: null,
  recentModels: [],
  fingerprint: undefined,

  loadModels: async () => {
    // The list of the models of the repository: the development server's
    // tool. The app has no dropdown —a model is opened with the dialog of
    // the system— so the list is not drawn.
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
    const runId = get().runId + 1;
    set({ runId, status: "loading", error: null, currentPath: path });
    try {
      const state = await engine().openModel(path);
      set({
        runId,
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
        activeContainerId: 0,
        clipboard: [],
        history: [],
        redoStack: [],
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
      return;
    }
    // The model joins the recents: the list is short and it holds the ones
    // the reader comes back to. A recents that cannot be written is not a
    // reason to say the model did not open.
    try {
      const recent = await files().rememberModel(path);
      set({ recentModels: recent.slice(0, RECENT_LIMIT) });
    } catch {
      // The recents are a convenience.
    }
  },

  newModel: async () => {
    const runId = get().runId + 1;
    set({ runId, status: "loading", error: null });
    try {
      const state = await engine().newModel();
      set({
        runId,
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
        activeContainerId: 0,
        clipboard: [],
        history: [],
        redoStack: [],
        status: "ready",
        savedMessage: null,
        error: null,
      });
    } catch (error) {
      set({ status: "idle", error: `No se pudo crear el modelo: ${messageOf(error)}` });
    }
  },

  apply: async (command: EditCommand) => {
    const runId = get().runId + 1;
    try {
      const state = await engine().apply(command);
      // A block that moved does not change the program, so the run stays:
      // the reader keeps the reproduction he was watching.
      const keepsRun = COSMETIC_COMMANDS.has(command.op);
      set({
        runId,
        model: state.model,
        diagnostics: state.diagnostics,
        dirty: state.dirty,
        canUndo: state.canUndo,
        canRedo: state.canRedo,
        history: [...get().history, command],
        redoStack: [],
        // The model moved: the run the monitor replays is not the model any
        // more, so the replay goes away with it.
        ...(keepsRun
          ? {}
          : {
              report: null,
              trace: null,
              monitor: null,
              transitions: null,
              playhead: 0,
              playing: false,
            }),
        error: null,
        status: "ready",
      });
    } catch (error) {
      set({ error: `No se pudo aplicar el cambio: ${messageOf(error)}` });
    }
  },

  undo: async () => {
    const runId = get().runId + 1;
    try {
      const state = await engine().undo();
      // The command that is undone is the last one the app applied: the
      // app is the only one that changes the model, so it is the one that
      // knows whether the change moved the blocks or changed the program.
      const history = get().history;
      const undone = history[history.length - 1];
      set({
        runId,
        model: state.model,
        diagnostics: state.diagnostics,
        dirty: state.dirty,
        canUndo: state.canUndo,
        canRedo: state.canRedo,
        history: history.slice(0, -1),
        redoStack: undone === undefined ? get().redoStack : [...get().redoStack, undone],
        ...(undone !== undefined && COSMETIC_COMMANDS.has(undone.op)
          ? {}
          : {
              report: null,
              trace: null,
              monitor: null,
              transitions: null,
              playhead: 0,
              playing: false,
            }),
        error: null,
      });
    } catch (error) {
      set({ error: `No se pudo deshacer: ${messageOf(error)}` });
    }
  },

  redo: async () => {
    const runId = get().runId + 1;
    try {
      const state = await engine().redo();
      const redoStack = get().redoStack;
      const redone = redoStack[redoStack.length - 1];
      set({
        runId,
        model: state.model,
        diagnostics: state.diagnostics,
        dirty: state.dirty,
        canUndo: state.canUndo,
        canRedo: state.canRedo,
        history: redone === undefined ? get().history : [...get().history, redone],
        redoStack: redoStack.slice(0, -1),
        ...(redone !== undefined && COSMETIC_COMMANDS.has(redone.op)
          ? {}
          : {
              report: null,
              trace: null,
              monitor: null,
              transitions: null,
              playhead: 0,
              playing: false,
            }),
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
      // The app wrote the file: the watcher has to remember the file it
      // just wrote, or the model the reader saved would be reloaded as if
      // somebody else had changed it.
      set({ fingerprint: await fingerprintOf(result.state.path) });
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

  clearSavedMessage: () => set({ savedMessage: null }),

  loadRecentModels: async () => {
    try {
      const recent = await files().recentModels();
      set({ recentModels: recent.slice(0, RECENT_LIMIT) });
    } catch (error) {
      set({ error: `No se pudieron leer los modelos recientes: ${messageOf(error)}` });
    }
  },

  /**
   * Watch the file the app has open.
   *
   * The watcher asks for the fingerprint of the file every half second: when
   * the fingerprint changes, the file changed, and the model is reloaded and
   * run again. The first look is right away, so the change that happens
   * after the model was opened is the one the app sees.
   */
  watchModel: (path: string) => {
    if (watchedPath !== path) {
      // Another file: the app looks at it from scratch.
      watchedPath = path;
      set({ fingerprint: undefined });
    }
    if (watchTimer !== null) {
      return;
    }
    watchTimer = setInterval(() => {
      void get().checkModelFile();
    }, WATCH_INTERVAL_MS);
    // The timer of Node keeps the process alive: the watcher must not.
    (watchTimer as unknown as { unref?: () => void }).unref?.();
    void get().checkModelFile();
  },

  stopWatching: () => {
    if (watchTimer !== null) {
      clearInterval(watchTimer);
      watchTimer = null;
    }
    watchedPath = null;
  },

  /**
   * Ask for the fingerprint of the open file, and reload the model when the
   * file changed.
   *
   * A file that cannot be read is not a reason to stop watching: the next
   * look asks again.
   */
  checkModelFile: async () => {
    const path = get().currentPath;
    if (path === null) {
      return;
    }
    let fingerprint: FileFingerprint | null;
    try {
      fingerprint = await engine().fingerprint(path);
    } catch {
      return;
    }
    const previous = get().fingerprint;
    set({ fingerprint });
    if (previous === undefined) {
      // The first look: the app learns what the file looks like.
      return;
    }
    if (sameFingerprint(previous, fingerprint)) {
      return;
    }
    if (fingerprint === null) {
      // The file is not there any more: there is nothing to reload.
      set({ notice: "El archivo del modelo ya no está en disco." });
      return;
    }
    await get().reloadFromDisk();
  },

  /**
   * Reload the model of the disk and run it again.
   *
   * The model of the disk goes over the one in memory only when the reader
   * has nothing pending: his work is not thrown away by a change somebody
   * else made. A model that does not compile leaves the app as it was, with
   * the message of the compiler.
   */
  setActiveContainerId: (id: number) => set({ activeContainerId: id }),

  reloadFromDisk: async () => {
    const state = get();
    const path = state.currentPath;
    if (path === null || state.model === null) {
      return;
    }
    if (state.dirty) {
      set({
        notice:
          "El modelo del disco cambió y tenés cambios sin guardar: no se recargó. Guardá el modelo para ver el del disco.",
      });
      return;
    }
    try {
      const reloaded = await engine().reload(path);
      set({
        model: reloaded.model,
        diagnostics: reloaded.diagnostics,
        dirty: reloaded.dirty,
        canUndo: reloaded.canUndo,
        canRedo: reloaded.canRedo,
        currentPath: reloaded.path ?? path,
        // The run that was on the screen is the run of the model that was
        // there: the run of the model that arrived replaces it.
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
      // The model of the disk does not compile: the app keeps the model it
      // had, and says what the compiler said.
      const failure = failureOf(error, "El modelo del disco no se pudo cargar");
      set({ error: failure.error, notice: failure.notice });
      return;
    }
    // The model that arrived runs again, by the same path as the button
    // "Ejecutar": in the thread of the work, with its cancellation.
    await get().runMonitor();
  },

  saveAs: async () => {
    // The path is the one the user chooses with the dialog of the system:
    // the app does not guess where the model goes.
    const suggested = get().currentPath ?? DEFAULT_MODEL_NAME;
    const path = await files().pickSavePath(suggested);
    if (path === null) {
      // The user gave up: nothing is written and nothing is said.
      return;
    }
    await get().save(path);
  },

  simulate: async () => {
    const runId = get().runId + 1;
    set({ runId, status: "running", error: null, notice: null, cancelling: false });
    // A run that draws its own seed is a new experiment every time; a run
    // with the reader's seed is the same one, so it can be repeated.
    const seed = get().seedIsAuto ? randomSeed() : get().seed;
    try {
      const report = await engine().simulateCurrent({
        seed,
        start: get().start,
        until: get().until ?? undefined,
      });
      if (get().runId !== runId) {
        return;
      }
      if (get().cancelling) {
        set({ status: "ready", cancelling: false, notice: STOPPED_NOTICE });
        return;
      }
      set({
        seed,
        report,
        // The run is not the one the monitor replays: the replay needs the
        // trace, and the monitor asks for it when it runs the model.
        trace: null,
        monitor: null,
        transitions: null,
        playhead: 0,
        playing: false,
        status: "ready",
        cancelling: false,
      });
    } catch (error) {
      if (get().runId !== runId) {
        return;
      }
      // The reader stopped the run: the app does not wait for it any more
      // and says what happened.
      if (get().cancelling) {
        set({ status: "ready", cancelling: false, notice: STOPPED_NOTICE });
        return;
      }
      const failure = failureOf(error, "No se pudo simular el modelo");
      set({ status: "ready", error: failure.error, notice: failure.notice });
    }
  },

  runMonitor: async () => {
    const model = get().model;
    if (model === null) {
      set({ error: "No hay ningún modelo abierto para reproducir." });
      return;
    }
    const runId = get().runId + 1;
    set({ runId, status: "running", error: null, notice: null, cancelling: false });
    const seed = get().seedIsAuto ? randomSeed() : get().seed;
    try {
      const options = {
        seed,
        start: get().start,
        until: get().until ?? undefined,
        limit: TRACE_LIMIT,
      };
      // One run of the engine, with the trace on: the report and the trace
      // come from the same run. The engine is deterministic, so the trace
      // belongs to the report.
      const { report, trace } = await engine().runCurrent(options);
      if (get().runId !== runId) {
        return;
      }
      if (get().cancelling) {
        set({ status: "ready", cancelling: false, notice: STOPPED_NOTICE });
        return;
      }
      const currentModel = get().model;
      if (currentModel === null) {
        return;
      }
      const monitor = buildIndex(trace, report, currentModel);
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
        seed,
        playhead: frameAt(monitor, monitor.startTime),
        playing: monitor.frames.length > 0,
        // The speed is a multiple of the real time, and the run starts at
        // x1: one second of model per second of screen. The reader who wants
        // the run faster asks for it.
        speed: 1,
        status: "ready",
        cancelling: false,
      });
    } catch (error) {
      if (get().runId !== runId) {
        return;
      }
      // The reader stopped the run: the app does not wait for it any more
      // and says what happened.
      if (get().cancelling) {
        set({ status: "ready", cancelling: false, notice: STOPPED_NOTICE });
        return;
      }
      const failure = failureOf(error, "No se pudo reproducir la simulación");
      set({ status: "ready", error: failure.error, notice: failure.notice });
    }
  },

  /**
   * Ask the engine to stop the run in flight.
   *
   * The engine cannot be interrupted yet: the host stops waiting for the
   * run and throws its answer away. The flag tells the store that the run
   * that ends is a run the reader stopped, and not a failure.
   */
  cancelRun: async () => {
    try {
      const stopped = await engine().cancelRun();
      set({ cancelling: stopped });
    } catch (error) {
      set({ error: `No se pudo cancelar la corrida: ${messageOf(error)}` });
    }
  },

  clearNotice: () => set({ notice: null }),

  clearError: () => set({ error: null }),

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
    const active = get().activeContainerId;
    if ((className === "PortIn" || className === "PortOut") && active === 0) {
      set({
        notice: "Las entradas y salidas de submódulo sólo pueden colocarse dentro de un contenedor.",
      });
      return;
    }
    const spot = freeSpot(model, active);
    const left = x ?? spot.x;
    const top = y ?? spot.y;
    await get().apply({
      op: "addBlock",
      class: className,
      container: active,
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
    try {
      // The positions come from the engine: the app and the MCP put the
      // model on the canvas the same way.
      const state = await engine().relayout();
      // The model moved, but the program did not change: the run the reader
      // was watching stays.
      set({
        model: state.model,
        diagnostics: state.diagnostics,
        dirty: state.dirty,
        canUndo: state.canUndo,
        canRedo: state.canRedo,
        error: null,
        status: "ready",
      });
    } catch (error) {
      set({ error: `No se pudo reacomodar el modelo: ${messageOf(error)}` });
    }
  },

  setSeed: (seed: number) => set({ seed, seedIsAuto: false }),

  // The reader fixed the seed: the run repeats instead of drawing a new one.
  setSeedAuto: (auto: boolean) => set({ seedIsAuto: auto }),

  setUntil: (until: number | null) => set({ until }),

  // The `START` of the run is a whole number of one or more: the original
  // refuses a `Start` of zero ("El Start debe ser mayor a cero."), and the
  // app does not let the run start with a counter of zero.
  setStart: (start: number) =>
    set({
      start: Number.isFinite(start) ? Math.max(1, Math.trunc(start)) : DEFAULT_START,
    }),
}));

// El store se ve desde la consola del navegador mientras se desarrolla.
if (import.meta.env?.DEV) {
  (globalThis as { __app?: typeof useApp }).__app = useApp;
}
