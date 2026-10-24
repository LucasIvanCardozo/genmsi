import { invoke, isTauri } from "./host";

/**
 * How the app reaches the files of the disk.
 *
 * Two ways, one interface: the desktop shell asks the dialogs of the system
 * and keeps the recents in the app's data directory; the browser (the
 * development server) has no system dialog, so it asks for a path with a
 * prompt and keeps the recents in the browser's storage.
 */
export interface FileGateway {
  /** The path of the model to open, or null when the user gives up. */
  pickModel(): Promise<string | null>;
  /** The path to write the model to, or null when the user gives up. */
  pickSavePath(suggested: string): Promise<string | null>;
  /** The models opened lately, the most recent first. */
  recentModels(): Promise<string[]>;
  /** Remember a model as one of the last opened ones. */
  rememberModel(path: string): Promise<string[]>;
}

/** The filters of the dialogs: the models of the app. */
const MODEL_FILTER = { name: "Modelo GenMSI", extensions: ["gmdl"] };

/**
 * The path a dialog returned, or null when there is none.
 *
 * The dialogs answer with the path they chose; the shape is read defensively
 * so a change of the plugin does not break the app.
 */
function pathOf(chosen: unknown): string | null {
  if (typeof chosen === "string") {
    return chosen;
  }
  if (Array.isArray(chosen)) {
    return pathOf(chosen[0]);
  }
  if (chosen !== null && typeof chosen === "object" && "path" in chosen) {
    const path = (chosen as { path: unknown }).path;
    return typeof path === "string" ? path : null;
  }
  return null;
}

/** The desktop shell: the dialogs of the system and the recents of the app. */
const tauriFiles: FileGateway = {
  pickModel: async () =>
    pathOf(
      await invoke<unknown>("plugin:dialog|open", {
        options: {
          title: "Abrir modelo",
          multiple: false,
          directory: false,
          filters: [MODEL_FILTER],
        },
      }),
    ),
  pickSavePath: async (suggested) =>
    pathOf(
      await invoke<unknown>("plugin:dialog|save", {
        options: {
          title: "Guardar modelo",
          defaultPath: suggested,
          filters: [MODEL_FILTER],
        },
      }),
    ),
  recentModels: async () => invoke<string[]>("recent_models"),
  rememberModel: async (path) => invoke<string[]>("remember_model", { path }),
};

/** The key the browser keeps the recents under. */
const RECENT_KEY = "genmsi.recent-models";

/** How many models the list of recents keeps. */
const RECENT_LIMIT = 8;

/** The recents the browser keeps. */
function browserRecents(): string[] {
  try {
    const raw = globalThis.localStorage?.getItem(RECENT_KEY);
    if (raw === null || raw === undefined) {
      return [];
    }
    const list = JSON.parse(raw) as unknown;
    if (!Array.isArray(list)) {
      return [];
    }
    return list.filter((item): item is string => typeof item === "string");
  } catch {
    // The browser refuses to read its storage: there are no recents.
    return [];
  }
}

/**
 * Ask the user for a path, with the prompt of the browser.
 *
 * The browser has no dialog of the system: this is the development path,
 * and the prompt says so.
 */
function askPath(question: string): string | null {
  const asked = globalThis.prompt?.(question) ?? null;
  if (asked === null) {
    return null;
  }
  const path = asked.trim();
  return path === "" ? null : path;
}

/** The development server: a prompt for the path and the browser's storage. */
const browserFiles: FileGateway = {
  pickModel: async () => askPath("Ruta del modelo (sólo en desarrollo)"),
  pickSavePath: async (suggested) =>
    askPath(`Ruta para guardar el modelo (sólo en desarrollo): ${suggested}`),
  recentModels: async () => browserRecents(),
  rememberModel: async (path) => {
    const list = [path, ...browserRecents().filter((item) => item !== path)].slice(
      0,
      RECENT_LIMIT,
    );
    try {
      globalThis.localStorage?.setItem(RECENT_KEY, JSON.stringify(list));
    } catch {
      // The browser refuses to write its storage: the recents stay as they
      // were, and the app keeps working.
    }
    return list;
  },
};

/**
 * The gateway of the running host.
 *
 * The desktop shell has the dialogs of the system; the development server
 * does not, and says so when it asks for a path.
 */
export function files(): FileGateway {
  return isTauri() ? tauriFiles : browserFiles;
}
