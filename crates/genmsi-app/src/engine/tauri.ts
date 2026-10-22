import type { LintDocument, ModelDocument, ReportDocument } from "../types";
import { EngineError } from "./adapter";
import type { EngineAdapter, ModelRef, SimulationOptions } from "./adapter";

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
};
