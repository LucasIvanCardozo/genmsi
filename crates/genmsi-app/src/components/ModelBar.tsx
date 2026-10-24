import { useEffect, useRef, useState } from "react";
import { FolderOpen, History } from "lucide-react";
import { files } from "../engine/files";
import { useApp } from "../state/store";

/**
 * The models of the app: the one that is open and the ones opened lately.
 *
 * The app does not offer a list of models to choose from: a model is opened
 * with the dialog of the system, and the ones opened lately are in a menu,
 * so the reader comes back to them without looking for them.
 *
 * The name of the open model is a state, not a control: the bar says which
 * model is on screen.
 */
export default function ModelBar() {
  const { currentPath, recentModels, status, openModel } = useApp();
  const [menu, setMenu] = useState(false);
  const menuRef = useRef<HTMLDivElement | null>(null);

  // The menu closes when the reader clicks outside of it.
  useEffect(() => {
    if (!menu) {
      return;
    }
    const onClick = (event: MouseEvent) => {
      if (menuRef.current !== null && !menuRef.current.contains(event.target as Node)) {
        setMenu(false);
      }
    };
    document.addEventListener("mousedown", onClick);
    return () => document.removeEventListener("mousedown", onClick);
  }, [menu]);

  /** Open a model with the dialog of the system. */
  const onOpen = async () => {
    const path = await files().pickModel();
    if (path === null) {
      return;
    }
    await openModel(path);
  };

  /** Open one of the models opened lately. */
  const onRecent = async (path: string) => {
    setMenu(false);
    await openModel(path);
  };

  const button =
    "flex items-center gap-1.5 rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs font-medium text-slate-700 shadow-sm transition hover:bg-slate-50 disabled:cursor-not-allowed disabled:text-slate-300";

  return (
    <div className="flex items-center gap-2">
      <span
        className="max-w-56 truncate text-sm font-medium text-slate-800"
        title={currentPath ?? undefined}
      >
        {currentPath === null ? "Sin modelo" : nameOf(currentPath)}
      </span>
      <button
        type="button"
        className={button}
        onClick={() => void onOpen()}
        disabled={status === "loading"}
        title="Abrir un modelo del disco"
      >
        <FolderOpen className="size-3.5" aria-hidden />
        Abrir…
      </button>
      <div ref={menuRef} className="relative">
        <button
          type="button"
          className={button}
          onClick={() => setMenu((open) => !open)}
          disabled={recentModels.length === 0}
          title="Los modelos abiertos últimamente"
        >
          <History className="size-3.5" aria-hidden />
          Recientes
        </button>
        {menu && (
          <ul className="absolute left-0 top-full z-20 mt-1 w-72 rounded-md border border-slate-200 bg-white py-1 shadow-lg">
            {recentModels.map((path) => (
              <li key={path}>
                <button
                  type="button"
                  onClick={() => void onRecent(path)}
                  title={path}
                  className="block w-full truncate px-3 py-1.5 text-left text-xs text-slate-700 hover:bg-slate-50"
                >
                  {nameOf(path)}
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}

/** The name of a path, without its directory. */
function nameOf(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || path;
}
