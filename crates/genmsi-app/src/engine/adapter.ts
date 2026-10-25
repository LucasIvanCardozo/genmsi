import type { EditCommand } from "../types/generated/EditCommand";
import type {
  CatalogueDocument,
  LintDocument,
  ModelDocument,
  ReportDocument,
  SessionState,
  TraceDocument,
} from "../types";

/** A model file the app offers. */
export interface ModelRef {
  /** File name, without the extension. */
  name: string;
  /** Path of the model, relative to the repository root. */
  path: string;
}

/**
 * The fingerprint of a file on disk.
 *
 * The date of the modification is in **nanoseconds** and travels as a
 * string: two writes in the same millisecond have to look different, and a
 * JavaScript number cannot hold nanoseconds since the epoch exactly.
 */
export interface FileFingerprint {
  /** The date of the last modification, in nanoseconds since the epoch. */
  modifiedNanos: string;
  /** How many bytes the file has. */
  size: number;
}

/** The options of one simulation. */
export interface SimulationOptions {
  /** Base seed of every random stream. */
  seed?: number;
  /** The `START` of the run: how many termination units end it. */
  start?: number;
  /** Horizon of the run, in simulated time. */
  until?: number;
  /** How many trace records the run keeps. */
  limit?: number;
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

/** The two documents of one run. */
export interface RunOutcome {
  /** The report of the run (`genmsi.report/1`). */
  report: ReportDocument;
  /** The trace of the run (`genmsi.trace/1`). */
  trace: TraceDocument;
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
  /**
   * The fingerprint of a file on disk, or null when the file is not there.
   *
   * The app watches the fingerprint of the model it has open: when it
   * changes, the file changed.
   */
  fingerprint(path: string): Promise<FileFingerprint | null>;
  /** The run's report (`genmsi.report/1`). */
  simulate(path: string, options?: SimulationOptions): Promise<ReportDocument>;
  /** The run's trace (`genmsi.trace/1`). */
  trace(path: string, options?: SimulationOptions): Promise<TraceDocument>;
  /** Open a model for editing. */
  openModel(path: string): Promise<SessionState>;
  /**
   * Reload the model of the disk into the session.
   *
   * The model is compiled before it replaces the one in memory: a model that
   * does not compile leaves the session as it was, and the failure says what
   * the compiler said.
   */
  reload(path: string): Promise<SessionState>;
  /** Start a new, empty model. */
  newModel(): Promise<SessionState>;
  /** Apply one editing command. */
  apply(command: EditCommand): Promise<SessionState>;
  /**
   * Compose the model on the canvas.
   *
   * The positions come from the engine, the same composition the MCP applies
   * before it writes a model: there is one algorithm.
   */
  relayout(): Promise<SessionState>;
  /** Undo the last operation. */
  undo(): Promise<SessionState>;
  /** Redo the last undone operation. */
  redo(): Promise<SessionState>;
  /** Save the model, to its own path or to a new one. */
  save(path?: string): Promise<SaveResult>;
  /** The report of a run of the model in memory. */
  simulateCurrent(options?: SimulationOptions): Promise<ReportDocument>;
  /** The trace of a run of the model in memory. */
  traceCurrent(options?: SimulationOptions): Promise<TraceDocument>;
  /**
   * The report and the trace of one run of the model in memory.
   *
   * The engine runs **once**, with the trace on: the two documents come from
   * the same run, and they are the ones the two separate runs gave.
   */
  runCurrent(options?: SimulationOptions): Promise<RunOutcome>;
  /**
   * Ask the engine to stop the run in flight.
   *
   * The answer says whether there was a run to stop. The engine cannot be
   * interrupted yet: what the host does is stop waiting for the run and
   * throw its answer away.
   */
  cancelRun(): Promise<boolean>;
  /** The block classes the editor offers to create. */
  catalogue(): Promise<CatalogueDocument>;
}
