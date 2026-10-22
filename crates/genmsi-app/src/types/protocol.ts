import type { ModelDocument as ModelBody } from "./generated/ModelDocument";
import type { CatalogueDocument } from "./generated/CatalogueDocument";
import type { DefinitionCounts } from "./generated/DefinitionCounts";

/**
 * The diagnostics of the model in memory.
 *
 * They are continuous: they are computed again after every change, and a
 * model that does not compile is a state the app shows, not an error that
 * stops the editing.
 */
export interface Diagnostics {
  /** `ok` when the compiler was silent, `diagnostics` when it had
   *  observations, `error` when the model does not compile. */
  status: "ok" | "diagnostics" | "error";
  /** The compiler's observations. */
  diagnostics: string[];
  /** The engine's message, when the model does not compile. */
  message: string | null;
  /** The executable statements the model compiles to. */
  blocks: number | null;
  /** The definitions the model declares. */
  definitions: DefinitionCounts | null;
}

/** The state of the editing session: what the app draws. */
export interface SessionState {
  /** Path of the model, as it was opened; empty for a new model. */
  path: string | null;
  /** The graph of the model. */
  model: ModelBody;
  /** What the model compiles to. */
  diagnostics: Diagnostics;
  /** Whether the model has changes that were not saved. */
  dirty: boolean;
  /** Whether an operation can be undone. */
  canUndo: boolean;
  /** Whether an operation can be redone. */
  canRedo: boolean;
}

/**
 * The answer of the facade to one request of the session protocol.
 *
 * The envelope is the app's own (four optional fields), so it is written by
 * hand; the documents it carries are generated from the Rust types.
 */
export interface SessionAnswer {
  /** Whether the request succeeded. */
  ok: boolean;
  /** The message of a failure. */
  error?: string;
  /** The state of the session, for the editing operations. */
  state?: SessionState;
  /** A document of the engine, byte for byte as the CLI prints it. */
  document?: string;
  /** Where a model was written. */
  path?: string;
  /** How many bytes the written model has. */
  bytes?: number;
}

export type { CatalogueDocument };
