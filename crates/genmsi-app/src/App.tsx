import { useEffect, useMemo, useState } from "react";
import { Play } from "lucide-react";
import BlockPalette from "./components/BlockPalette";
import DiagnosticsPanel from "./components/DiagnosticsPanel";
import ErrorNotice from "./components/ErrorNotice";
import GraphView from "./components/GraphView";
import ModelPicker from "./components/ModelPicker";
import ModelSummary from "./components/ModelSummary";
import MonitorView from "./components/MonitorView";
import PropertiesPanel from "./components/PropertiesPanel";
import ReportView from "./components/ReportView";
import Toolbar from "./components/Toolbar";
import { useApp } from "./state/store";

/**
 * The app's shell.
 *
 * The header opens a model and runs it; the toolbar holds the editing
 * actions; the body shows the model's graph, the properties of the selected
 * block and the report of the last run.
 */
export default function App() {
  const {
    model,
    currentPath,
    status,
    error,
    seed,
    until,
    selectedBlockIds,
    report,
    view,
    diagnostics,
    dirty,
    canUndo,
    canRedo,
    catalogue,
    pendingSave,
    savedMessage,
    loadModels,
    loadCatalogue,
    newModel,
    apply,
    undo,
    redo,
    save,
    simulate,
    select,
    copy,
    paste,
    relayout,
    addBlock,
    deleteSelection,
    setSeed,
    setUntil,
    setView,
    confirmSave,
    dismissSave,
  } = useApp();

  useEffect(() => {
    void loadModels();
    void loadCatalogue();
  }, [loadModels, loadCatalogue]);

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

  const selectedBlock =
    model?.blocks.find((block) => block.id === selectedBlockIds[0]) ?? null;
  // The counter that asks the graph to frame the model again.
  const [fitSignal, setFitSignal] = useState(0);

  // The actions of the graph keep their identity: the graph reports the
  // selection on every render, and a new function would start a loop.
  const actions = useMemo(
    () => ({
      onSelect: select,
      onMove: (id: number, x: number, y: number) => void apply({ op: "moveBlock", id, x, y }),
      onConnect: (from: number, fromIoid: number, to: number, toIoid: number) =>
        void apply({ op: "connect", from, fromIoid, to, toIoid }),
      onRemove: (ids: number[]) => {
        if (ids.length === 1) {
          void apply({ op: "removeBlock", id: ids[0] });
        } else {
          void apply({ op: "removeBlocks", ids });
        }
      },
      onDisconnect: (container: number, from: number, fromIoid: number) =>
        void apply({ op: "disconnect", container, from, fromIoid }),
      onDropBlock: (className: string, x: number, y: number) =>
        void addBlock(className, x, y),
    }),
    [select, apply, addBlock],
  );

  return (
    <div className="flex h-full flex-col bg-slate-100 text-slate-900">
      <header className="flex items-center gap-5 border-b border-slate-200 bg-white px-5 py-2.5">
        <div className="flex items-baseline gap-2">
          <span className="text-base font-semibold tracking-tight">GenMSI</span>
          <span className="text-[11px] text-slate-500">Simulación GPSS</span>
        </div>

        <ModelPicker />

        <div className="flex items-center gap-1.5">
          <label className="text-[11px] font-medium uppercase tracking-wide text-slate-500">
            Semilla
          </label>
          <input
            aria-label="Semilla"
            type="number"
            min={1}
            value={seed}
            onChange={(event) => setSeed(Number(event.target.value) || 1)}
            className="w-16 rounded-md border border-slate-300 bg-white px-2 py-1 text-xs font-mono text-slate-800 outline-none focus:border-slate-500 focus:ring-2 focus:ring-slate-200"
          />
          <label className="text-[11px] font-medium uppercase tracking-wide text-slate-500">
            Hasta
          </label>
          <input
            aria-label="Hasta"
            type="number"
            min={0}
            placeholder="—"
            value={until ?? ""}
            onChange={(event) => setUntil(event.target.value === "" ? null : Number(event.target.value))}
            className="w-20 rounded-md border border-slate-300 bg-white px-2 py-1 text-xs font-mono text-slate-800 outline-none focus:border-slate-500 focus:ring-2 focus:ring-slate-200"
          />
        </div>

        <button
          type="button"
          onClick={() => void simulate()}
          disabled={currentPath === null && model === null}
          className="flex items-center gap-1.5 rounded-md bg-indigo-600 px-3 py-1.5 text-xs font-medium text-white shadow-sm transition hover:bg-indigo-500 disabled:cursor-not-allowed disabled:bg-slate-300"
        >
          <Play className="size-3.5" aria-hidden />
          {status === "running" ? "Simulando…" : "Simular"}
        </button>

        <div className="ml-auto flex items-center gap-3 text-xs text-slate-500">
          {model !== null && <span className="font-medium text-slate-700">{model.title}</span>}
          {status === "loading" && <span>Cargando…</span>}
        </div>
      </header>

      <main className="flex flex-1 flex-col gap-3 overflow-hidden p-4">
        {error !== null && <ErrorNotice message={error} />}
        {savedMessage !== null && (
          <p className="rounded-lg border border-emerald-200 bg-emerald-50 px-3 py-1.5 text-xs text-emerald-900">
            {savedMessage}
          </p>
        )}

        <div className="flex flex-wrap items-center gap-2">
          <Toolbar
            dirty={dirty}
            canUndo={canUndo}
            canRedo={canRedo}
            hasModel={model !== null}
            pendingSave={pendingSave}
            onNew={() => void newModel()}
            onSave={() => void save()}
            onSaveAs={(path) => void save(path)}
            onUndo={() => void undo()}
            onRedo={() => void redo()}
            onRelayout={() => void relayout()}
            onFit={() => setFitSignal((current) => current + 1)}
            onDelete={() => void deleteSelection()}
            onConfirmSave={() => void confirmSave()}
            onDismissSave={dismissSave}
          />
          <BlockPalette catalogue={catalogue} onAdd={(className) => void addBlock(className)} />
        </div>

        <div className="flex gap-1 border-b border-slate-200">
          {(
            [
              { id: "grafo", label: "Grafo" },
              { id: "monitor", label: "Monitor" },
              { id: "reporte", label: "Reporte" },
            ] as const
          ).map((candidate) => (
            <button
              key={candidate.id}
              type="button"
              onClick={() => setView(candidate.id)}
              className={[
                "-mb-px rounded-t-md border-b-2 px-4 py-1.5 text-sm font-medium transition",
                view === candidate.id
                  ? "border-indigo-500 text-indigo-700"
                  : "border-transparent text-slate-500 hover:text-slate-800",
              ].join(" ")}
            >
              {candidate.label}
            </button>
          ))}
        </div>

        {model === null ? (
          <div className="flex flex-1 items-center justify-center rounded-lg border border-dashed border-slate-300 bg-white/60 text-sm text-slate-500">
            Selecciona un modelo para ver su grafo.
          </div>
        ) : view === "grafo" ? (
          <div className="flex min-h-0 flex-1 flex-col gap-3">
            <ModelSummary diagnostics={diagnostics} />
            <DiagnosticsPanel diagnostics={diagnostics} />
            <div className="flex min-h-0 flex-1 gap-3">
              <div className="min-w-0 flex-1">
                <GraphView model={model} actions={actions} fitSignal={fitSignal} />
              </div>
              <aside className="w-80 shrink-0 overflow-auto rounded-lg border border-slate-200 bg-white p-4">
                <PropertiesPanel
                  block={selectedBlock}
                  catalogue={catalogue}
                  onSetProperty={(id, property, value) =>
                    void apply({ op: "setProperty", id, property, value })
                  }
                  onSetCaption={(id, caption) => void apply({ op: "setCaption", id, caption })}
                />
              </aside>
            </div>
          </div>
        ) : view === "monitor" ? (
          <MonitorView />
        ) : report === null ? (
          <div className="flex flex-1 items-center justify-center rounded-lg border border-dashed border-slate-300 bg-white/60 text-sm text-slate-500">
            Ejecuta la simulación para ver su reporte.
          </div>
        ) : (
          <ReportView report={report} />
        )}
      </main>
    </div>
  );
}
