import { spawn } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { Plugin } from "vite";

const here = path.dirname(fileURLToPath(import.meta.url));

/** The repository root: the app lives in `crates/genmsi-app`. */
export const repoRoot = path.resolve(here, "../../..");

/** The directory the app offers as models, relative to the repository root. */
export const modelsDir = "fixtures/models";

/** The engine's CLI: the dev bridge returns its stdout verbatim. */
export const genmsiBin =
  process.env.GENMSI_BIN ?? path.join(repoRoot, "target", "debug", "genmsi");

/** The app's engine facade: the same documents, built by the shared crate. */
export const appEngineBin =
  process.env.GENMSI_APP_BIN ?? path.join(repoRoot, "target", "debug", "genmsi-app-engine");

/** The outcome of one engine process. */
export interface RunResult {
  /** Exit code (`127` when the process could not start). */
  code: number;
  /** Everything the process wrote to stdout. */
  stdout: string;
  /** Everything the process wrote to stderr. */
  stderr: string;
}

/** Run one engine process and collect its output. */
export function run(bin: string, args: string[]): Promise<RunResult> {
  return new Promise((resolve) => {
    const child = spawn(bin, args, { cwd: repoRoot });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });
    child.on("error", (error) => {
      resolve({ code: 127, stdout, stderr: error.message });
    });
    child.on("close", (code) => {
      resolve({ code: code ?? 1, stdout, stderr });
    });
  });
}

/**
 * The absolute path of a model, checked against the repository root.
 *
 * The dev server is a local development tool, but it runs commands: a path
 * that leaves the repository is refused instead of being read.
 */
export function resolveInsideRepo(modelPath: string): string {
  const root = path.resolve(repoRoot);
  const resolved = path.resolve(root, modelPath);
  if (resolved !== root && !resolved.startsWith(root + path.sep)) {
    throw new Error(`la ruta sale del repositorio: ${modelPath}`);
  }
  return resolved;
}

/** The engine's message of a failed run, without the `error:` prefix. */
export function failureMessage(result: RunResult): string {
  const text = result.stderr.trim();
  if (text.length > 0) {
    return text.replace(/^error:\s*/, "");
  }
  return `el motor terminó con el código ${result.code}`;
}

/**
 * The dev bridge: a Vite middleware that serves the engine's documents.
 *
 * It moves the engine's bytes and never builds a document of its own: the
 * lint and the report come from the `genmsi` CLI verbatim, and the model's
 * graph comes from the app's engine facade, which builds it with the same
 * engine functions.
 */
export function devBridge(): Plugin {
  return {
    name: "genmsi-dev-bridge",
    apply: "serve",
    configureServer(server) {
      server.middlewares.use("/api", async (req, res) => {
        const url = new URL(req.url ?? "/", "http://localhost");
        const route = url.pathname.replace(/^\/+/, "");
        const modelPath = url.searchParams.get("path") ?? "";
        try {
          switch (route) {
            case "models": {
              const result = await run(appEngineBin, ["models", modelsDir]);
              return send(res, result);
            }
            case "model": {
              resolveInsideRepo(modelPath);
              const result = await run(appEngineBin, ["model", modelPath]);
              return send(res, result);
            }
            case "lint": {
              resolveInsideRepo(modelPath);
              const result = await run(genmsiBin, ["lint", modelPath, "--json"]);
              return send(res, result);
            }
            case "simulate": {
              resolveInsideRepo(modelPath);
              const args = ["simulate", modelPath, "--json"];
              const seed = url.searchParams.get("seed");
              if (seed !== null && seed !== "") {
                args.push("--seed", seed);
              }
              const until = url.searchParams.get("until");
              if (until !== null && until !== "") {
                args.push("--until", until);
              }
              const result = await run(genmsiBin, args);
              return send(res, result);
            }
            default: {
              res.statusCode = 404;
              res.setHeader("content-type", "application/json; charset=utf-8");
              res.end(JSON.stringify({ error: `ruta desconocida: ${route}` }));
            }
          }
        } catch (error) {
          res.statusCode = 400;
          res.setHeader("content-type", "application/json; charset=utf-8");
          res.end(JSON.stringify({ error: (error as Error).message }));
        }
      });
    },
  };
}

/** Write one engine document to the response, byte for byte. */
function send(res: { statusCode: number; setHeader: (name: string, value: string) => void; end: (body?: string) => void }, result: RunResult) {
  res.setHeader("content-type", "application/json; charset=utf-8");
  if (result.code === 0) {
    res.statusCode = 200;
    res.end(result.stdout);
    return;
  }
  res.statusCode = 400;
  res.end(JSON.stringify({ error: failureMessage(result) }));
}
