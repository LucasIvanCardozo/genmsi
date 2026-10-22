import { useState } from "react";
import { number } from "../format";
import type { ReportDocument } from "../types";

/** A table of the report. */
function DataTable<T>({
  columns,
  rows,
}: {
  columns: { label: string; value: (row: T) => string }[];
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
            {columns.map((column) => (
              <td key={column.label} className="py-1.5 pr-4 font-mono whitespace-nowrap text-slate-800">
                {column.value(row)}
              </td>
            ))}
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
 * renders it and computes nothing.
 */
export default function ReportView({ report }: { report: ReportDocument }) {
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
          Fin <span className="font-mono text-slate-800">{report.end_reason}</span>
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
              { label: "Línea", value: (row) => String(row.line) },
              { label: "Nodo", value: (row) => String(row.node) },
              { label: "Clase", value: (row) => row.class },
              { label: "Etiqueta", value: (row) => row.label ?? "—" },
              { label: "Operandos", value: (row) => row.operands },
              { label: "Entry", value: (row) => number(row.entries) },
              { label: "Current", value: (row) => number(row.current) },
              { label: "Retry", value: (row) => number(row.retries) },
            ]}
          />
        )}
        {active.id === "facilidades" && (
          <DataTable
            rows={Object.values(entities.facilities)}
            columns={[
              { label: "Facilidad", value: (row) => row.name },
              { label: "Entries", value: (row) => number(row.entries) },
              { label: "Ocupado", value: (row) => number(row.busy_time) },
              { label: "Ocupada", value: (row) => (row.busy ? "sí" : "no") },
              { label: "Disponible", value: (row) => (row.available ? "sí" : "no") },
              { label: "Retry", value: (row) => number(row.retry) },
              { label: "Inter", value: (row) => number(row.inter) },
            ]}
          />
        )}
        {active.id === "colas" && (
          <DataTable
            rows={Object.values(entities.queues)}
            columns={[
              { label: "Cola", value: (row) => row.name },
              { label: "Máx", value: (row) => number(row.max) },
              { label: "Contenido", value: (row) => number(row.content) },
              { label: "Entries", value: (row) => number(row.entries) },
              { label: "Entries(0)", value: (row) => number(row.zero_entries) },
              { label: "Tiempo total", value: (row) => number(row.total_time) },
              { label: "Área", value: (row) => number(row.area) },
              { label: "Retry", value: (row) => number(row.retry) },
            ]}
          />
        )}
        {active.id === "almacenes" && (
          <DataTable
            rows={Object.values(entities.storages)}
            columns={[
              { label: "Almacén", value: (row) => row.name },
              { label: "Cap.", value: (row) => number(row.capacity) },
              { label: "En uso", value: (row) => number(row.in_use) },
              { label: "Mín", value: (row) => number(row.min_use) },
              { label: "Máx", value: (row) => number(row.max_use) },
              { label: "Entries", value: (row) => number(row.entries) },
              { label: "Área", value: (row) => number(row.area) },
              { label: "Retry", value: (row) => number(row.retry) },
              { label: "Delay", value: (row) => number(row.delay) },
            ]}
          />
        )}
        {active.id === "cadenas" && (
          <DataTable
            rows={Object.values(entities.chains)}
            columns={[
              { label: "Cadena", value: (row) => row.name },
              { label: "Tamaño", value: (row) => number(row.content) },
              { label: "Máx", value: (row) => number(row.max) },
              { label: "Entries", value: (row) => number(row.entries) },
              { label: "Tiempo total", value: (row) => number(row.total_time) },
              { label: "Área", value: (row) => number(row.area) },
            ]}
          />
        )}
        {active.id === "logica" && (
          <DataTable
            rows={Object.entries(entities.logic).map(([name, state]) => ({ name, state }))}
            columns={[
              { label: "Interruptor", value: (row) => row.name },
              { label: "Estado", value: (row) => (row.state ? "activado" : "desactivado") },
            ]}
          />
        )}
        {active.id === "valores" && (
          <DataTable
            rows={Object.entries(entities.savevalues).map(([name, value]) => ({ name, value }))}
            columns={[
              { label: "Valor", value: (row) => row.name },
              { label: "Cantidad", value: (row) => number(row.value) },
            ]}
          />
        )}
        {active.id === "tablas" && (
          <DataTable
            rows={Object.values(entities.tables)}
            columns={[
              { label: "Tabla", value: (row) => row.name },
              { label: "Primer valor", value: (row) => number(row.first) },
              { label: "Ancho", value: (row) => number(row.width) },
              { label: "Intervalos", value: (row) => number(row.intervals) },
              { label: "Muestras", value: (row) => number(row.samples) },
              { label: "Máximo", value: (row) => number(row.max_value) },
              { label: "Desbordes", value: (row) => number(row.overflow) },
            ]}
          />
        )}
        {active.id === "qtablas" && (
          <DataTable
            rows={Object.values(entities.qtables)}
            columns={[
              { label: "QTabla", value: (row) => row.name },
              { label: "Primer valor", value: (row) => number(row.first) },
              { label: "Ancho", value: (row) => number(row.width) },
              { label: "Intervalos", value: (row) => number(row.intervals) },
              { label: "Muestras", value: (row) => number(row.samples) },
            ]}
          />
        )}
      </div>
    </div>
  );
}
