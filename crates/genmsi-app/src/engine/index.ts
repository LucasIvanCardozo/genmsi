import { devBridge } from "./devBridge";
import { tauri } from "./tauri";
import type { EngineAdapter } from "./adapter";

export type { EngineAdapter, ModelRef, SimulationOptions } from "./adapter";
export { EngineError } from "./adapter";

/** Whether the app runs inside the Tauri shell. */
export function isTauri(): boolean {
  return typeof globalThis !== "undefined" && "__TAURI_INTERNALS__" in globalThis;
}

/**
 * The adapter of the running host.
 *
 * Two backends, one interface: the dev bridge when the app runs under Vite
 * (which is how the slice is reviewed) and the Tauri commands when it runs
 * inside the desktop shell.
 */
export function engine(): EngineAdapter {
  return isTauri() ? tauri : devBridge;
}
