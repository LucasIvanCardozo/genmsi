import type { EditCommand } from "../types/generated/EditCommand";
import type {
  CatalogueDocument,
  LintDocument,
  ModelDocument,
  ReportDocument,
  SessionState,
} from "../types";

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

/** The outcome of a save. */
export interface SaveResult {
  /** The state of the session after the save. */
  state: SessionState;
  /** Where the model was written. */
  path: string;
  /** How many bytes the file has. */
  bytes: number;
}

/** A failure the engine reported. */
export class EngineError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "EngineError";
  }
}

/**
 * The engine operations the app needs.
 *
 * There is one adapter and two backends: the dev bridge (a Vite middleware
 * that answers with the engine's bytes) and the Tauri shell (commands that
 * call the same engine facade). Neither backend builds a document: they move
 * the engine's bytes.
 *
 * The editing operations go through the session: the model lives in the
 * facade, with its undo stack, and every mutation is a command that the
 * format crate applies.
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
  /** Open a model for editing. */
  openModel(path: string): Promise<SessionState>;
  /** Start a new, empty model. */
  newModel(): Promise<SessionState>;
  /** Apply one editing command. */
  apply(command: EditCommand): Promise<SessionState>;
  /** Undo the last operation. */
  undo(): Promise<SessionState>;
  /** Redo the last undone operation. */
  redo(): Promise<SessionState>;
  /** Save the model, to its own path or to a new one. */
  save(path?: string): Promise<SaveResult>;
  /** The report of a run of the model in memory. */
  simulateCurrent(options?: SimulationOptions): Promise<ReportDocument>;
  /** The block classes the editor offers to create. */
  catalogue(): Promise<CatalogueDocument>;
}
