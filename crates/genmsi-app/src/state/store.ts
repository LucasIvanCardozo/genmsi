import { create } from "zustand";
import { engine } from "../engine";
import type { ModelRef } from "../engine/adapter";
import type { LintDocument, ModelDocument, ReportDocument } from "../types";

/** What the app is doing right now. */
export type Status = "idle" | "loading" | "ready" | "running";

/** The view the body shows. */
export type View = "grafo" | "reporte";

export interface AppState {
  /** The models the app offers. */
  models: ModelRef[];
  /** The path of the model that is open. */
  currentPath: string | null;
  /** The open model's graph. */
  model: ModelDocument | null;
  /** The open model's lint. */
  lint: LintDocument | null;
  /** The last run's report. */
  report: ReportDocument | null;
  /** The block the user selected in the graph. */
  selectedBlockId: number | null;
  /** The seed the next run uses. */
  seed: number;
  /** The horizon the next run uses, in simulated time. */
  until: number | null;
  /** What the app is doing. */
  status: Status;
  /** The view the body shows. */
  view: View;
  /** The message of the last failure, if any. */
  error: string | null;
  /** Ask the engine for the models the app offers. */
  loadModels: () => Promise<void>;
  /** Open a model: its graph and its lint. */
  openModel: (path: string) => Promise<void>;
  /** Run the open model and read its report. */
  simulate: () => Promise<void>;
  /** Select a block of the graph. */
  selectBlock: (id: number | null) => void;
  /** Show one of the views. */
  setView: (view: View) => void;
  /** Set the seed of the next run. */
  setSeed: (seed: number) => void;
  /** Set the horizon of the next run. */
  setUntil: (until: number | null) => void;
}

/** The message of a failure the engine reported. */
function messageOf(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

export const useApp = create<AppState>((set, get) => ({
  models: [],
  currentPath: null,
  model: null,
  lint: null,
  report: null,
  selectedBlockId: null,
  seed: 1,
  until: null,
  status: "idle",
  view: "grafo",
  error: null,

  loadModels: async () => {
    try {
      const models = await engine().listModels();
      set({ models, error: null });
    } catch (error) {
      set({ error: `No se pudieron listar los modelos: ${messageOf(error)}` });
    }
  },

  openModel: async (path: string) => {
    set({ status: "loading", error: null, currentPath: path });
    try {
      const [model, lint] = await Promise.all([
        engine().loadModel(path),
        engine().lint(path),
      ]);
      set({
        model,
        lint,
        report: null,
        selectedBlockId: null,
        status: "ready",
        error: null,
      });
    } catch (error) {
      set({
        status: "idle",
        model: null,
        lint: null,
        report: null,
        error: `No se pudo abrir el modelo: ${messageOf(error)}`,
      });
    }
  },

  simulate: async () => {
    const path = get().currentPath;
    if (path === null) {
      return;
    }
    set({ status: "running", error: null });
    try {
      const report = await engine().simulate(path, {
        seed: get().seed,
        until: get().until ?? undefined,
      });
      set({ report, status: "ready", view: "reporte" });
    } catch (error) {
      set({ status: "ready", error: `No se pudo simular el modelo: ${messageOf(error)}` });
    }
  },

  selectBlock: (id: number | null) => set({ selectedBlockId: id }),

  setView: (view: View) => set({ view }),

  setSeed: (seed: number) => set({ seed }),

  setUntil: (until: number | null) => set({ until }),
}));
