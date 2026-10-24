import { useState } from "react";
import { number } from "../format";
import { closeFigures, type Figures } from "../monitor/stats";
import { useLive } from "../monitor/live";
import { endReasonText } from "../engine/reasons";
import type { ReplayState } from "../monitor/replay";
import type { ReportDocument } from "../types";

/**
 * One number of the report.
 *
 * The live number is the one the replay knows at this instant; the published
 * one is the engine's, for the run that finished. Which one is on the screen
 * is said out loud: the reader must not mistake the end of a run for the
 * state of the run.
 */
interface Cell {
  /** The text of the number. */
  text: string;
  /** Whether it is the live one. */
  live: boolean;
}

/** A number the engine published, which the replay cannot say. */
function published(value: number): Cell {
  return { text: number(value), live: false };
}

/** A word: the name of a thing, which does not change during the run. */
function text(value: string): Cell {
  return { text: value, live: false };
}

/** A yes or no, the one the engine published. */
function yes(value: boolean): Cell {
  return { text: value ? "sí" : "no", live: false };
}

/** A yes or no the replay knows, and the engine's. */
function either(live: boolean | undefined, value: boolean): Cell {
  if (live === undefined) {
    return yes(value);
  }
  return { text: live ? "sí" : "no", live: true };
}

/** A number the replay knows, and the engine's, for the same thing. */
function at(live: number | undefined, value: number): Cell {
  if (live === undefined) {
    return published(value);
  }
  return { text: number(live), live: true };
}

/** A table of the report. */
function DataTable<T>({
  columns,
  rows,
}: {
  columns: { label: string; value: (row: T) => Cell }[];
  rows: T[];
}) {
  if (rows.length === 0) {
    return <p className="px-1 py-6 text-sm text-slate-500">Sin datos.</p>;
  }
  return (
    <table className="w-full border-collapse text-xs">
      <thead>
        <tr className="border-b border-slate-300 text-left text-slate-500">
          {columns.map((column) => (
            <th key={column.label} className="py-2 pr-4 font-medium whitespace-nowrap">
              {column.label}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {rows.map((row, index) => (
          <tr key={index} className="border-b border-slate-100">
            {columns.map((column) => {
              const cell = column.value(row);
              return (
                <td
                  key={column.label}
                  className={[
                    "py-1.5 pr-4 font-mono whitespace-nowrap",
                    cell.live ? "text-indigo-700" : "text-slate-800",
                  ].join(" ")}
                >
                  {cell.text}
                </td>
              );
            })}
          </tr>
        ))}
      </tbody>
    </table>
  );
}

/** One tab of the report. */
interface Tab {
  id: string;
  label: string;
  count: number;
}

/**
 * The report of the last run.
 *
 * Every table comes from the engine's `genmsi.report/1` document; the app
 * renders it and computes nothing. The numbers the replay knows are shown as
 * they are at this instant, and the ones the replay cannot say keep the
 * engine's value: the trace does not carry the samples of a table, nor the
 * state of a switch, nor the value of a savevalue.
 */
export default function ReportView({
  report,
  state,
  figures,
}: {
  report: ReportDocument;
  /** The state of the replay at the instant the reader is looking at. */
  state: ReplayState | null;
  /** The figures of the entities at that instant. */
  figures: Figures | null;
}) {
  const entities = report.entities;
  const tabs: Tab[] = [
    { id: "bloques", label: "Bloques", count: report.blocks.length },
    { id: "facilidades", label: "Facilidades", count: Object.keys(entities.facilities).length },
    { id: "colas", label: "Colas", count: Object.keys(entities.queues).length },
    { id: "almacenes", label: "Almacenes", count: Object.keys(entities.storages).length },
    { id: "cadenas", label: "Cadenas", count: Object.keys(entities.chains).length },
    { id: "logica", label: "Lógica", count: Object.keys(entities.logic).length },
    { id: "valores", label: "Valores", count: Object.keys(entities.savevalues).length },
    { id: "tablas", label: "Tablas", count: Object.keys(entities.tables).length },
    { id: "qtablas", label: "QTablas", count: Object.keys(entities.qtables).length },
  ];
  const [tab, setTab] = useState("bloques");
  const active = tabs.find((candidate) => candidate.id === tab) ?? tabs[0];
  // The state of the replay at the instant: the numbers it knows are the
  // live ones.
  const atBlock = (line: number) => (state === null ? undefined : line - 1);
  // The clock of the replay: the integrals that are still open are closed at
  // this instant, so the area of a queue is the area it covered so far.
  const clock = useLive((live) => live.clock);
  const closed = figures === null ? null : closeFigures(figures, clock);

  return (
    <div className="flex h-full flex-col gap-3 overflow-hidden">
      <div className="flex flex-wrap items-baseline gap-x-6 gap-y-1 rounded-lg border border-slate-200 bg-white px-4 py-3 text-xs">
        <span className="text-sm font-semibold text-slate-800">{report.title}</span>
        <span className="text-slate-500">
          Semilla <span className="font-mono text-slate-800">{report.seed}</span>
        </span>
        <span className="text-slate-500">
          Tiempos{" "}
          <span className="font-mono text-slate-800">
            {number(report.start_time)} → {number(report.end_time)}
          </span>
        </span>
        <span className="text-slate-500">
          Fin <span className="font-mono text-slate-800">{endReasonText(report.end_reason)}</span>
        </span>
        <span className="text-slate-500">
          Transacciones{" "}
          <span className="font-mono text-slate-800">{report.transactions}</span>
        </span>
        <span className="text-slate-500">
          Eventos <span className="font-mono text-slate-800">{report.events}</span>
        </span>
      </div>

      <div className="flex gap-1 overflow-x-auto border-b border-slate-200">
        {tabs.map((candidate) => (
          <button
            key={candidate.id}
            type="button"
            onClick={() => setTab(candidate.id)}
            className={[
              "-mb-px rounded-t-md border-b-2 px-3 py-1.5 text-xs font-medium transition",
              candidate.id === active.id
                ? "border-indigo-500 text-indigo-700"
                : "border-transparent text-slate-500 hover:text-slate-800",
            ].join(" ")}
          >
            {candidate.label}
            <span className="ml-1.5 font-mono text-[10px] text-slate-400">{candidate.count}</span>
          </button>
        ))}
      </div>

      <div className="flex-1 overflow-auto rounded-lg border border-slate-200 bg-white px-4 py-3">
        {active.id === "bloques" && (
          <DataTable
            rows={report.blocks}
            columns={[
              { label: "Línea", value: (row) => published(row.line) },
              { label: "Nodo", value: (row) => published(row.node) },
              { label: "Clase", value: (row) => text(row.class) },
              { label: "Etiqueta", value: (row) => text(row.label ?? "—") },
              { label: "Operandos", value: (row) => text(row.operands) },
              {
                label: "Entry",
                value: (row) =>
                  at(
                    state === null ? undefined : state.totals[atBlock(row.line) ?? -1],
                    row.entries,
                  ),
              },
              {
                label: "Current",
                value: (row) =>
                  at(
                    state === null ? undefined : state.blocks[atBlock(row.line) ?? -1],
                    row.current,
                  ),
              },
              {
                label: "Retry",
                value: (row) =>
                  at(
                    state === null ? undefined : state.retries[atBlock(row.line) ?? -1],
                    row.retries,
                  ),
              },
            ]}
          />
        )}
        {active.id === "facilidades" && (
          <DataTable
            rows={Object.values(entities.facilities)}
            columns={[
              { label: "Facilidad", value: (row) => text(row.name) },
              {
                label: "Entries",
                value: (row) => at(closed?.facilities[row.name]?.entries, row.entries),
              },
              {
                label: "Ocupado",
                value: (row) =>
                  at(closed?.facilities[row.name]?.busyTime, row.busy_time),
              },
              {
                label: "Ocupada",
                value: (row) => either(closed?.facilities[row.name]?.busy, row.busy),
              },
              {
                label: "Disponible",
                value: (row) => yes(row.available),
              },
              {
                label: "Inter",
                value: (row) => at(closed?.facilities[row.name]?.inter, row.inter),
              },
            ]}
          />
        )}
        {active.id === "colas" && (
          <DataTable
            rows={Object.values(entities.queues)}
            columns={[
              { label: "Cola", value: (row) => text(row.name) },
              {
                label: "Máx",
                value: (row) => at(closed?.queues[row.name]?.max, row.max),
              },
              {
                label: "Contenido",
                value: (row) => at(closed?.queues[row.name]?.content, row.content),
              },
              {
                label: "Entries",
                value: (row) => at(closed?.queues[row.name]?.entries, row.entries),
              },
              {
                label: "Entries(0)",
                value: (row) => at(closed?.queues[row.name]?.zeroEntries, row.zero_entries),
              },
              {
                label: "Tiempo total",
                value: (row) => at(closed?.queues[row.name]?.totalTime, row.total_time),
              },
              {
                label: "Área",
                value: (row) => at(closed?.queues[row.name]?.area, row.area),
              },
            ]}
          />
        )}
        {active.id === "almacenes" && (
          <DataTable
            rows={Object.values(entities.storages)}
            columns={[
              { label: "Almacén", value: (row) => text(row.name) },
              {
                label: "En uso",
                value: (row) => at(closed?.storages[row.name]?.inUse, row.in_use),
              },
              {
                label: "Mín",
                value: (row) => at(closed?.storages[row.name]?.minUse, row.min_use),
              },
              {
                label: "Máx",
                value: (row) => at(closed?.storages[row.name]?.maxUse, row.max_use),
              },
              {
                label: "Entries",
                value: (row) => at(closed?.storages[row.name]?.entries, row.entries),
              },
              {
                label: "Área",
                value: (row) => at(closed?.storages[row.name]?.area, row.area),
              },
            ]}
          />
        )}
        {active.id === "cadenas" && (
          <DataTable
            rows={Object.values(entities.chains)}
            columns={[
              { label: "Cadena", value: (row) => text(row.name) },
              {
                label: "Tamaño",
                value: (row) => at(closed?.chains[row.name]?.content, row.content),
              },
              {
                label: "Máx",
                value: (row) => at(closed?.chains[row.name]?.max, row.max),
              },
              {
                label: "Entries",
                value: (row) => at(closed?.chains[row.name]?.entries, row.entries),
              },
              {
                label: "Tiempo total",
                value: (row) => at(closed?.chains[row.name]?.totalTime, row.total_time),
              },
              {
                label: "Área",
                value: (row) => at(closed?.chains[row.name]?.area, row.area),
              },
            ]}
          />
        )}
        {active.id === "logica" && (
          <DataTable
            rows={Object.entries(entities.logic).map(([name, state]) => ({ name, state }))}
            columns={[
              { label: "Interruptor", value: (row) => text(row.name) },
              {
                label: "Estado",
                value: (row) => either(closed?.logic[row.name], row.state),
              },
            ]}
          />
        )}
        {active.id === "valores" && (
          <DataTable
            rows={Object.entries(entities.savevalues).map(([name, value]) => ({ name, value }))}
            columns={[
              { label: "Valor", value: (row) => text(row.name) },
              {
                label: "Cantidad",
                value: (row) => at(closed?.values[row.name], row.value),
              },
            ]}
          />
        )}
        {active.id === "tablas" && (
          <DataTable
            rows={Object.values(entities.tables)}
            columns={[
              { label: "Tabla", value: (row) => text(row.name) },
              { label: "Primer valor", value: (row) => published(row.first) },
              { label: "Ancho", value: (row) => published(row.width) },
              { label: "Intervalos", value: (row) => published(row.intervals) },
              {
                label: "Muestras",
                value: (row) => at(closed?.tables[row.name]?.samples, row.samples),
              },
              {
                label: "Máximo",
                value: (row) => at(closed?.tables[row.name]?.maxValue, row.max_value),
              },
              { label: "Desbordes", value: (row) => published(row.overflow) },
            ]}
          />
        )}
        {active.id === "qtablas" && (
          <DataTable
            rows={Object.values(entities.qtables)}
            columns={[
              { label: "QTabla", value: (row) => text(row.name) },
              { label: "Primer valor", value: (row) => published(row.first) },
              { label: "Ancho", value: (row) => published(row.width) },
              { label: "Intervalos", value: (row) => published(row.intervals) },
              {
                label: "Muestras",
                value: (row) => at(closed?.tables[row.name]?.samples, row.samples),
              },
            ]}
          />
        )}
      </div>
    </div>
  );
}
