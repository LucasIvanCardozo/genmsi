import { useEffect, useMemo, useState } from "react";
import AppHeader from "./components/AppHeader";
import MonitorView from "./components/MonitorView";
import Toolbar from "./components/Toolbar";
import { useApp } from "./state/store";

/** How long the notice of a save stays on the screen. */
const SAVED_NOTICE_MS = 4000;

/** How long the notice of a stopped run stays on the screen. */
const NOTICE_MS = 8000;

/**
 * The app's shell.
 *
 * The header carries the identity of the work and the console of the run;
 * the toolbar holds the commands of the model —the ones that open it and
 * the ones that edit it—; the body
 * is the window of the simulation: the graph of the model, the reproduction of
 * the run and its report, all in one place.
 */
export default function App() {
  const {
    model,
    dirty,
    canUndo,
    canRedo,
    savedMessage,
    notice,
    currentPath,
    clearNotice,
    loadModels,
    loadCatalogue,
    loadRecentModels,
    newModel,
    apply,
    undo,
    redo,
    save,
    saveAs,
    select,
    copy,
    paste,
    relayout,
    addBlock,
    deleteSelection,
    clearSavedMessage,
    watchModel,
    stopWatching,
  } = useApp();

  useEffect(() => {
    void loadModels();
    void loadCatalogue();
    void loadRecentModels();
  }, [loadModels, loadCatalogue, loadRecentModels]);

  // The notice of a save goes away on its own: a notice that stays is a
  // message that takes the place of the work.
  useEffect(() => {
    if (savedMessage === null) {
      return;
    }
    const handle = window.setTimeout(() => clearSavedMessage(), SAVED_NOTICE_MS);
    return () => window.clearTimeout(handle);
  }, [savedMessage, clearSavedMessage]);

  // The notice of a stopped run goes away on its own too.
  useEffect(() => {
    if (notice === null) {
      return;
    }
    const handle = window.setTimeout(() => clearNotice(), NOTICE_MS);
    return () => window.clearTimeout(handle);
  }, [notice, clearNotice]);

  // The app watches the file it has open: when the file changes in the
  // disk —an agent that is building the model writes it— the model is
  // reloaded and run again, so the agent and the reader see the same
  // thing.
  useEffect(() => {
    if (currentPath === null) {
      stopWatching();
      return;
    }
    watchModel(currentPath);
    return () => stopWatching();
  }, [currentPath, watchModel, stopWatching]);

  // The keyboard of the editor: undo, redo, copy and paste.
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      const tag = target?.tagName ?? "";
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") {
        return;
      }
      const withControl = event.ctrlKey || event.metaKey;
      if (!withControl) {
        return;
      }
      const key = event.key.toLowerCase();
      if (key === "z") {
        event.preventDefault();
        if (event.shiftKey) {
          void redo();
        } else {
          void undo();
        }
        return;
      }
      if (key === "y") {
        event.preventDefault();
        void redo();
        return;
      }
      if (key === "c") {
        event.preventDefault();
        copy();
        return;
      }
      if (key === "v") {
        event.preventDefault();
        void paste();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [undo, redo, copy, paste]);

  // The counter that asks the graph to frame the model again.
  const [fitSignal, setFitSignal] = useState(0);

  // The actions of the graph keep their identity: the graph reports the
  // selection on every render, and a new function would start a loop.
  const actions = useMemo(
    () => ({
      onSelect: select,
      onMove: (id: number, x: number, y: number) => void apply({ op: "moveBlock", id, x, y }),
      onMovePort: (
        container: number,
        ioid: number,
        input: boolean,
        x: number,
        y: number,
      ) => void apply({ op: "movePort", block: container, ioid, input, x, y }),
      onConnect: (from: number, fromIoid: number, to: number, toIoid: number) =>
        void apply({ op: "connect", from, fromIoid, to, toIoid }),
      onRemove: (ids: number[]) => {
        if (ids.length === 1) {
          void apply({ op: "removeBlock", id: ids[0] });
        } else {
          void apply({ op: "removeBlocks", ids });
        }
      },
      onRemovePort: (container: number, ioid: number, input: boolean) =>
        void apply({ op: "removePort", block: container, ioid, input }),
      onDisconnect: (container: number, from: number, fromIoid: number) =>
        void apply({ op: "disconnect", container, from, fromIoid }),
      onDropBlock: (className: string, x: number, y: number) =>
        void addBlock(className, x, y),
    }),
    [select, apply, addBlock],
  );

  return (
    <div className="flex h-full flex-col bg-slate-100 text-slate-900">
      <AppHeader />

      <Toolbar
        dirty={dirty}
        canUndo={canUndo}
        canRedo={canRedo}
        hasModel={model !== null}
        onNew={() => void newModel()}
        onSave={() => void save()}
        onSaveAs={() => void saveAs()}
        onUndo={() => void undo()}
        onRedo={() => void redo()}
        onRelayout={() => {
          // The picture is composed again: the reader has to see the new
          // one whole, not the piece of the old one he was looking at.
          void relayout().then(() => setFitSignal((current) => current + 1));
        }}
        onFit={() => setFitSignal((current) => current + 1)}
        onDelete={() => void deleteSelection()}
      />

      <main className="flex flex-1 flex-col gap-3 overflow-hidden p-4">

        <MonitorView actions={actions} fitSignal={fitSignal} />
      </main>
    </div>
  );
}
