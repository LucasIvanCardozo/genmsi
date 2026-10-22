import type { LintDocument, ModelDocument, ReportDocument } from "../types";

/** A model file the app offers. */
export interface ModelRef {
  /** File name, without the extension. */
  name: string;
  /** Path of the model, relative to the repository root. */
  path: string;
}

/** The options of one simulation. */
export interface SimulationOptions {
  /** Base seed of every random stream. */
  seed?: number;
  /** Horizon of the run, in simulated time. */
  until?: number;
}

/** A failure the engine reported. */
export class EngineError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "EngineError";
  }
}

/**
 * The engine operations the slice needs.
 *
 * There is one adapter and two backends: the dev bridge (a Vite middleware
 * that returns the engine's bytes) and the Tauri shell (commands that call
 * the same engine facade). Neither backend builds a document: they move the
 * engine's bytes.
 */
export interface EngineAdapter {
  /** The models the app offers. */
  listModels(): Promise<ModelRef[]>;
  /** The model's graph (`genmsi.model/1`). */
  loadModel(path: string): Promise<ModelDocument>;
  /** The model's lint (`genmsi.lint/1`). */
  lint(path: string): Promise<LintDocument>;
  /** The run's report (`genmsi.report/1`). */
  simulate(path: string, options?: SimulationOptions): Promise<ReportDocument>;
}
