import { useEffect } from "react";
import { Play } from "lucide-react";
import ErrorNotice from "./components/ErrorNotice";
import GraphView from "./components/GraphView";
import ModelPicker from "./components/ModelPicker";
import ModelSummary from "./components/ModelSummary";
import PropertiesPanel from "./components/PropertiesPanel";
import ReportView from "./components/ReportView";
import { useApp } from "./state/store";

/**
 * The app's shell.
 *
 * The header opens a model and runs it; the body shows the model's graph and
 * the report of the last run.
 */
export default function App() {
  const {
    model,
    currentPath,
    status,
    error,
    seed,
    until,
    selectedBlockId,
    report,
    view,
    lint,
    loadModels,
    simulate,
    selectBlock,
    setSeed,
    setUntil,
    setView,
  } = useApp();

  useEffect(() => {
    void loadModels();
  }, [loadModels]);

  const selectedBlock =
    model?.blocks.find((block) => block.id === selectedBlockId) ?? null;

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
          disabled={currentPath === null || status === "running"}
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

        <div className="flex gap-1 border-b border-slate-200">
          {(
            [
              { id: "grafo", label: "Grafo" },
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
            <ModelSummary lint={lint} />
            <div className="flex min-h-0 flex-1 gap-3">
              <div className="min-w-0 flex-1">
                <GraphView
                  model={model}
                  selectedBlockId={selectedBlockId}
                  onSelect={(id) => selectBlock(id)}
                />
              </div>
              <aside className="w-80 shrink-0 overflow-auto rounded-lg border border-slate-200 bg-white p-4">
                <PropertiesPanel block={selectedBlock} />
              </aside>
            </div>
          </div>
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
