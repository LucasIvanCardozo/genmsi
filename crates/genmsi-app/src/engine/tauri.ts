import type { EditCommand } from "../types/generated/EditCommand";
import type {
  CatalogueDocument,
  LintDocument,
  ModelDocument,
  ReportDocument,
  SessionAnswer,
  SessionState,
  TraceDocument,
} from "../types";
import { EngineError } from "./adapter.ts";
import type {
  EngineAdapter,
  FileFingerprint,
  ModelRef,
  RunOutcome,
  SaveResult,
  SimulationOptions,
} from "./adapter";
import { invoke } from "./host.ts";

/**
 * Send one request of the session protocol.
 *
 * The shell keeps the same session the dev bridge keeps in its process, and
 * answers with the same JSON.
 */
async function session(request: unknown): Promise<SessionAnswer> {
  const answer = await invoke<string>("session", {
    request: JSON.stringify(request),
  });
  const parsed = JSON.parse(answer) as SessionAnswer;
  if (!parsed.ok) {
    throw new EngineError(parsed.error ?? "el motor no pudo responder");
  }
  return parsed;
}

/** The state the answer carries, or a failure when it carries none. */
function stateOf(answer: SessionAnswer): SessionState {
  if (answer.state === undefined) {
    throw new EngineError("el motor no devolvió el estado del modelo");
  }
  return answer.state;
}

/**
 * The document a command returned, parsed.
 *
 * The shell's commands return the engine's bytes (the document as a JSON
 * string), the same bytes the dev bridge returns as a body: both backends
 * deliver the same document and the frontend sees an object in both.
 */
function parsed<T>(text: string): T {
  return JSON.parse(text) as T;
}

/** The document the answer carries, parsed. */
function documentOf<T>(answer: SessionAnswer): T {
  if (answer.document === undefined) {
    throw new EngineError("el motor no devolvió el documento");
  }
  return JSON.parse(answer.document) as T;
}

/**
 * The Tauri backend: the commands of the desktop shell.
 *
 * The commands call the same engine facade the dev bridge's binary runs, so
 * both backends return the same documents.
 */
export const tauri: EngineAdapter = {
  listModels: () => invoke<ModelRef[]>("list_models"),
  /**
   * The version of the app: the one of the shell, the one that travels in
   * the bundle that was installed.
   */
  appVersion: () => invoke<string>("app_version"),
  loadModel: async (path: string) =>
    parsed<ModelDocument>(await invoke<string>("load_model", { path })),
  lint: async (path: string) =>
    parsed<LintDocument>(await invoke<string>("lint_model", { path })),
  fingerprint: (path: string) =>
    invoke<FileFingerprint | null>("model_fingerprint", { path }),
  simulate: async (path: string, options?: SimulationOptions) =>
    parsed<ReportDocument>(
      await invoke<string>("simulate_model", {
        path,
        seed: options?.seed ?? 1,
        start: options?.start ?? 1,
        until: options?.until ?? null,
      }),
    ),
  trace: async (path: string, options?: SimulationOptions) =>
    parsed<TraceDocument>(
      await invoke<string>("trace_model", {
        path,
        seed: options?.seed ?? 1,
        start: options?.start ?? 1,
        until: options?.until ?? null,
        limit: options?.limit ?? null,
      }),
    ),
  openModel: async (path: string) => stateOf(await session({ op: "open", path })),
  reload: async (path: string) => stateOf(await session({ op: "reload", path })),
  newModel: async () => stateOf(await session({ op: "new" })),
  apply: async (command: EditCommand) => stateOf(await session({ op: "apply", command })),
  relayout: async (scope: number) => stateOf(await session({ op: "relayout", scope })),
  undo: async () => stateOf(await session({ op: "undo" })),
  redo: async () => stateOf(await session({ op: "redo" })),
  save: async (path?: string): Promise<SaveResult> => {
    const answer = await session({ op: "save", path: path ?? null });
    return {
      state: stateOf(answer),
      path: answer.path ?? path ?? "",
      bytes: answer.bytes ?? 0,
    };
  },
  simulateCurrent: async (options?: SimulationOptions) =>
    documentOf<ReportDocument>(
      await session({
        op: "simulateCurrent",
        seed: options?.seed ?? null,
        start: options?.start ?? null,
        until: options?.until ?? null,
      }),
    ),
  traceCurrent: async (options?: SimulationOptions) =>
    documentOf<TraceDocument>(
      await session({
        op: "traceCurrent",
        seed: options?.seed ?? null,
        start: options?.start ?? null,
        until: options?.until ?? null,
        limit: options?.limit ?? null,
      }),
    ),
  runCurrent: async (options?: SimulationOptions): Promise<RunOutcome> => {
    const answer = await session({
      op: "runCurrent",
      seed: options?.seed ?? null,
      start: options?.start ?? null,
      until: options?.until ?? null,
      limit: options?.limit ?? null,
    });
    if (answer.documents === undefined) {
      throw new EngineError("el motor no devolvió el informe ni la traza");
    }
    return {
      report: JSON.parse(answer.documents.report) as ReportDocument,
      trace: JSON.parse(answer.documents.trace) as TraceDocument,
    };
  },
  catalogue: async () => documentOf<CatalogueDocument>(await session({ op: "catalogue" })),
  /**
   * Ask the shell to stop the run in flight.
   *
   * The shell raises the cancellation flag in the engine, stopping the run
   * at the next event. The answer says whether there was a run to stop.
   */
  cancelRun: () => invoke<boolean>("cancel_run"),
};
