import { spawn, type ChildProcess } from "node:child_process";
import { mkdirSync, statSync } from "node:fs";
import type { IncomingMessage, ServerResponse } from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { Plugin } from "vite";

const here = path.dirname(fileURLToPath(import.meta.url));

/** The repository root: the app lives in `crates/genmsi-app`. */
export const repoRoot = path.resolve(here, "../../..");

/** The directory the app offers as models, relative to the repository root. */
export const modelsDir = "fixtures/models";

/**
 * The directory the development path writes.
 *
 * The reference models of `fixtures/models` are read-only oracle material, so
 * the copy the user edits lives here. Only the development server writes, and
 * only here.
 */
export const workDir = "crates/genmsi-app/.devmodels";

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
 * The fingerprint of a file: its date of modification, in nanoseconds, and
 * its size.
 *
 * The date is in nanoseconds because two writes in the same millisecond
 * have to look different: the app watches this number to know that the file
 * it has open changed.
 */
export function fingerprintOf(file: string): {
  modifiedNanos: string;
  size: number;
} | null {
  try {
    const stats = statSync(file, { bigint: true });
    return { modifiedNanos: stats.mtimeNs.toString(), size: Number(stats.size) };
  } catch {
    // The file is not there: the app has nothing to reload.
    return null;
  }
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

/** The answer of the session process to one request. */
export interface SessionAnswer {
  /** Whether the request succeeded. */
  ok: boolean;
  /** The message of a failure. */
  error?: string;
  /** The state of the session, for the editing operations. */
  state?: unknown;
  /** A document of the engine, byte for byte as the CLI prints it. */
  document?: string;
  /** Where a model was written. */
  path?: string;
  /** How many bytes the written model has. */
  bytes?: number;
}

/**
 * The long-lived session process of the dev bridge.
 *
 * One request per line, one answer per line. The process keeps the model in
 * memory while the user edits it: that is what makes the undo stack, the
 * diagnostics after every change and the save possible.
 */
class SessionProcess {
  /** The child process, once it is started. */
  private child: ChildProcess | null = null;
  /** What arrived but is not a whole line yet. */
  private buffer = "";
  /** The requests waiting for their answer, in order. */
  private waiters: Array<(line: string) => void> = [];

  /** Start the process, or return the one that runs. */
  private start(): ChildProcess {
    if (this.child !== null) {
      return this.child;
    }
    const child = spawn(appEngineBin, ["session"], {
      cwd: repoRoot,
      stdio: ["pipe", "pipe", "pipe"],
    });
    child.stdout?.setEncoding("utf8");
    child.stdout?.on("data", (chunk: string) => {
      this.read(chunk);
    });
    child.stderr?.on("data", (chunk: string) => {
      process.stderr.write(`[genmsi-session] ${chunk}`);
    });
    child.on("close", () => {
      this.child = null;
    });
    this.child = child;
    return child;
  }

  /** Take the whole lines out of what arrived. */
  private read(chunk: string): void {
    this.buffer += chunk;
    let index = this.buffer.indexOf("\n");
    while (index >= 0) {
      const line = this.buffer.slice(0, index);
      this.buffer = this.buffer.slice(index + 1);
      const waiter = this.waiters.shift();
      if (waiter !== undefined) {
        waiter(line);
      }
      index = this.buffer.indexOf("\n");
    }
  }

  /** Send one request and wait for its answer. */
  send(request: unknown): Promise<SessionAnswer> {
    const child = this.start();
    return new Promise((resolve) => {
      this.waiters.push((line) => {
        try {
          resolve(JSON.parse(line) as SessionAnswer);
        } catch (error) {
          resolve({
            ok: false,
            error: `la respuesta del motor no se pudo leer: ${(error as Error).message}`,
          });
        }
      });
      child.stdin?.write(`${JSON.stringify(request)}\n`);
    });
  }
}

/** The one session of the dev server. */
const session = new SessionProcess();

/** Read the body of a request. */
async function readBody(request: IncomingMessage): Promise<string> {
  const chunks: Buffer[] = [];
  for await (const chunk of request) {
    chunks.push(chunk as Buffer);
  }
  return Buffer.concat(chunks).toString("utf8");
}

/**
 * The dev bridge: a Vite middleware that serves the engine's documents.
 *
 * It moves the engine's bytes and never builds a document of its own: the
 * lint and the report come from the `genmsi` CLI verbatim, the model's graph
 * comes from the app's engine facade, and the editing session is answered by
 * the same facade in a long-lived process.
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
              // The models of the reference set and of the development work
              // directory, where the copy the user edits lives.
              const answer = await session.send({
                op: "models",
                dirs: [modelsDir, workDir],
              });
              res.statusCode = answer.ok ? 200 : 400;
              res.setHeader("content-type", "application/json; charset=utf-8");
              res.end(answer.document ?? JSON.stringify({ error: answer.error ?? "" }));
              return;
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
            case "fingerprint": {
              // The fingerprint of the file the app has open: the same
              // answer the desktop shell gives.
              const resolved = resolveInsideRepo(modelPath);
              res.statusCode = 200;
              res.setHeader("content-type", "application/json; charset=utf-8");
              res.end(JSON.stringify({ fingerprint: fingerprintOf(resolved) }));
              return;
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
              const start = url.searchParams.get("start");
              if (start !== null && start !== "") {
                args.push("--start", start);
              }
              const result = await run(genmsiBin, args);
              return send(res, result);
            }
            case "trace": {
              resolveInsideRepo(modelPath);
              const args = ["trace", modelPath, "--json"];
              const seed = url.searchParams.get("seed");
              if (seed !== null && seed !== "") {
                args.push("--seed", seed);
              }
              const until = url.searchParams.get("until");
              if (until !== null && until !== "") {
                args.push("--until", until);
              }
              const start = url.searchParams.get("start");
              if (start !== null && start !== "") {
                args.push("--start", start);
              }
              const limit = url.searchParams.get("limit");
              if (limit !== null && limit !== "") {
                args.push("--limit", limit);
              }
              const result = await run(genmsiBin, args);
              return send(res, result);
            }
            case "session": {
              // The editing protocol: one request, one answer, both JSON.
              const body = await readBody(req);
              const request = JSON.parse(body) as unknown;
              const answer = await session.send(request);
              res.statusCode = answer.ok ? 200 : 400;
              res.setHeader("content-type", "application/json; charset=utf-8");
              res.end(JSON.stringify(answer));
              return;
            }
            case "catalogue": {
              const answer = await session.send({ op: "catalogue" });
              res.statusCode = answer.ok ? 200 : 400;
              res.setHeader("content-type", "application/json; charset=utf-8");
              res.end(JSON.stringify(answer));
              return;
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
      // The dev bridge is the only path that writes, and it writes here.
      mkdirSync(path.resolve(repoRoot, workDir), { recursive: true });
      server.config.logger.info(
        `el puente de desarrollo escribe solo en desarrollo, en ${workDir}`,
      );
    },
  };
}

/** Write one engine document to the response, byte for byte. */
function send(res: ServerResponse, result: RunResult): void {
  res.setHeader("content-type", "application/json; charset=utf-8");
  if (result.code === 0) {
    res.statusCode = 200;
    res.end(result.stdout);
    return;
  }
  res.statusCode = 400;
  res.end(JSON.stringify({ error: failureMessage(result) }));
}
