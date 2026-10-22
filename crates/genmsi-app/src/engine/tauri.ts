import type { EditCommand } from "../types/generated/EditCommand";
import type {
  CatalogueDocument,
  LintDocument,
  ModelDocument,
  ReportDocument,
  SessionAnswer,
  SessionState,
} from "../types";
import { EngineError } from "./adapter";
import type { EngineAdapter, ModelRef, SaveResult, SimulationOptions } from "./adapter";

/** The bridge the Tauri shell exposes to the webview. */
interface TauriInternals {
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
}

/** The Tauri bridge of the running shell. */
function bridge(): TauriInternals {
  const value = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
  if (!value) {
    throw new EngineError("la aplicación de escritorio no está disponible");
  }
  return value as TauriInternals;
}

/**
 * Send one request of the session protocol.
 *
 * The shell keeps the same session the dev bridge keeps in its process, and
 * answers with the same JSON.
 */
async function session(request: unknown): Promise<SessionAnswer> {
  const answer = await bridge().invoke<string>("session", {
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
  listModels: () => bridge().invoke<ModelRef[]>("list_models"),
  loadModel: (path: string) => bridge().invoke<ModelDocument>("load_model", { path }),
  lint: (path: string) => bridge().invoke<LintDocument>("lint_model", { path }),
  simulate: (path: string, options?: SimulationOptions) =>
    bridge().invoke<ReportDocument>("simulate_model", {
      path,
      seed: options?.seed ?? 1,
      until: options?.until ?? null,
    }),
  openModel: async (path: string) => stateOf(await session({ op: "open", path })),
  newModel: async () => stateOf(await session({ op: "new" })),
  apply: async (command: EditCommand) => stateOf(await session({ op: "apply", command })),
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
        until: options?.until ?? null,
      }),
    ),
  catalogue: async () => documentOf<CatalogueDocument>(await session({ op: "catalogue" })),
};
