import { EngineError } from "./adapter.ts";

/**
 * The host the app runs in.
 *
 * The desktop shell puts its bridge in the page; the browser does not. The
 * parts of the app that need the shell —the dialogs of the system, the
 * recents of the app— ask here instead of looking for the bridge on their
 * own.
 */

/** The bridge the Tauri shell exposes to the webview. */
interface TauriInternals {
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
}

/** Whether the app runs inside the Tauri shell. */
export function isTauri(): boolean {
  return typeof globalThis !== "undefined" && "__TAURI_INTERNALS__" in globalThis;
}

/** The bridge of the running shell. */
function bridge(): TauriInternals {
  const value = (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
  if (!value) {
    throw new EngineError("la aplicación de escritorio no está disponible");
  }
  return value as TauriInternals;
}

/**
 * Call one command of the shell.
 *
 * The command may be one of the app's own or one of a plugin's: the
 * dialogs of the system are the plugin's, and they need their permissions.
 */
export function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return bridge().invoke<T>(command, args);
}
