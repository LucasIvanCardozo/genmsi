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

/** The list document of the app (`genmsi.models/1`). */
interface ModelsDocument {
  models: ModelRef[];
}

/**
 * Ask the dev bridge for one document.
 *
 * The bridge answers with the engine's bytes; a failure arrives as a status
 * of 400 and a body with the engine's message.
 */
async function get<T>(
  route: string,
  params: Record<string, string | number | undefined>,
): Promise<T> {
  const query = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== "") {
      query.set(key, String(value));
    }
  }
  const response = await fetch(`/api/${route}?${query.toString()}`);
  const text = await response.text();
  if (!response.ok) {
    throw new EngineError(messageOf(text));
  }
  return JSON.parse(text) as T;
}

/** The engine's message of a failed response. */
function messageOf(text: string): string {
  try {
    const body = JSON.parse(text) as { error?: string };
    if (typeof body.error === "string") {
      return body.error;
    }
  } catch {
    // The body is not JSON: the raw text is the message.
  }
  return text;
}

/**
 * Send one request of the session protocol.
 *
 * The answer is the facade's: the state of the session for the editing
 * operations, and the engine's bytes for the documents.
 */
async function session(request: unknown): Promise<SessionAnswer> {
  const response = await fetch("/api/session", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(request),
  });
  const answer = (await response.json()) as SessionAnswer;
  if (!answer.ok) {
    throw new EngineError(answer.error ?? "el motor no pudo responder");
  }
  return answer;
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

/** The dev bridge backend: the Vite middleware of the development server. */
export const devBridge: EngineAdapter = {
  listModels: async () => (await get<ModelsDocument>("models", {})).models,
  loadModel: (path: string) => get<ModelDocument>("model", { path }),
  lint: (path: string) => get<LintDocument>("lint", { path }),
  simulate: (path: string, options?: SimulationOptions) =>
    get<ReportDocument>("simulate", {
      path,
      seed: options?.seed,
      until: options?.until,
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
