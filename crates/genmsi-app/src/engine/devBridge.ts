import type { LintDocument, ModelDocument, ReportDocument } from "../types";
import { EngineError } from "./adapter";
import type { EngineAdapter, ModelRef, SimulationOptions } from "./adapter";

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
};
