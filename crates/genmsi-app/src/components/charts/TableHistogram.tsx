/**
 * The histogram of a table of the model.
 *
 * The bars are the **published frequencies** and the accumulated curve is
 * their running total: the engine's own numbers, not a metric the app
 * computes. The mean and the deviation come from the sum and the sum of
 * squares that the engine publishes, and the last class of the table —the
 * one of the values that do not fit— is drawn apart.
 */
import { useMemo, useState } from "react";
import {
  Bar,
  CartesianGrid,
  Cell,
  ComposedChart,
  Line,
  ReferenceArea,
  ReferenceLine,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { decimals, number } from "../../format.ts";
import { tableDerived } from "../../monitor/derived.ts";
import type { Figures } from "../../monitor/stats.ts";
import type { ReportDocument, Table } from "../../types";

/** One table of the model, with the numbers the chart reads. */
interface TableEntry {
  /** The name of the table. */
  name: string;
  /** The definition of the table. */
  table: Table;
}

export default function TableHistogram({
  report,
  live,
}: {
  /** The report of the run. */
  report: ReportDocument;
  /** The figures of the tables at the instant the reader is at. */
  live: Figures | null;
}) {
  const tables = useMemo<TableEntry[]>(() => {
    const entries: TableEntry[] = [];
    for (const table of Object.values(report.entities.tables)) {
      entries.push({ name: table.name, table });
    }
    for (const table of Object.values(report.entities.qtables)) {
      entries.push({ name: table.name, table });
    }
    return entries;
  }, [report]);

  const [selected, setSelected] = useState<string | null>(null);
  const name = selected ?? tables[0]?.name ?? null;
  const entry = tables.find((candidate) => candidate.name === name) ?? null;

  const chart = useMemo(() => {
    if (entry === null) {
      return null;
    }
    // Las frecuencias son las del instante: la tabla muestra lo que se
    // tabuló hasta ahora, no lo que se tabulará al final de la corrida.
    const figure = live?.tables[entry.table.name];
    const frequencies = figure?.frequencies ?? [];
    const samples = figure?.samples ?? 0;
    const sum = figure?.sum ?? 0;
    const sumSquares = figure?.sumSquares ?? 0;
    const overflow = figure?.overflow ?? 0;
    const derived = tableDerived({ samples, sum, sumSquares, overflow });
    const total = frequencies.reduce((sum2, count) => sum2 + count, 0);
    let running = 0;
    const rows = frequencies.map((count, index) => {
      running += count;
      const low =
        index === 0
          ? 0
          : entry.table.first + (index - 1) * entry.table.width + entry.table.width;
      const high = entry.table.first + index * entry.table.width;
      const isOverflow = index === frequencies.length - 1 && entry.table.intervals > 1;
      return {
        x: (low + high) / 2,
        label: isOverflow ? `> ${number(high)}` : `${number(low)} - ${number(high)}`,
        frequency: count,
        accumulated: total > 0 ? (running / total) * 100 : 0,
        // The name is not `overflow`: Recharts gives every property of a
        // row to the shape it draws, and `overflow` is an attribute of the
        // SVG that a boolean would break.
        overflowClass: isOverflow,
      };
    });
    const barSize = Math.max(4, Math.round(620 / Math.max(rows.length, 1)) - 8);
    return { rows, derived, samples, overflow, width: entry.table.width, barSize };
  }, [entry, live]);

  if (entry === null || chart === null) {
    return (
      <p className="py-6 text-center text-xs text-slate-400">
        El modelo no tiene tablas.
      </p>
    );
  }

  return (
    <div>
      {tables.length > 1 && (
        <div className="mb-2 flex flex-wrap gap-1">
          {tables.map((candidate) => (
            <button
              key={candidate.name}
              type="button"
              onClick={() => setSelected(candidate.name)}
              className={[
                "rounded-md border px-2 py-1 text-[11px] font-medium transition",
                candidate.name === name
                  ? "border-indigo-500 bg-indigo-50 text-indigo-700"
                  : "border-slate-300 bg-white text-slate-500 hover:bg-slate-50",
              ].join(" ")}
            >
              {candidate.name}
            </button>
          ))}
        </div>
      )}
      <p className="mb-1 text-[11px] text-slate-500">
        {number(chart.samples)} valores · media {decimals(chart.derived.mean, 2)}{" "}
        · desviación {decimals(chart.derived.stdDev, 2)} ·{" "}
        {number(chart.overflow)} desbordes. La banda es la desviación.
      </p>
      <div className="h-72">
        <ResponsiveContainer width="100%" height="100%">
          <ComposedChart data={chart.rows} margin={{ top: 8, right: 16, bottom: 8, left: 0 }}>
            <CartesianGrid stroke="#f1f5f9" />
            <XAxis
              dataKey="x"
              type="number"
              tickFormatter={(value) => decimals(value, 1)}
              fontSize={10}
              minTickGap={30}
            />
            <YAxis yAxisId="left" fontSize={10} allowDecimals={false} width={44} />
            <YAxis
              yAxisId="right"
              orientation="right"
              fontSize={10}
              domain={[0, 100]}
              width={40}
            />
            <Tooltip
              formatter={(value, name) =>
                name === "acumulado" ? `${number(Number(value))} %` : number(Number(value))
              }
              labelFormatter={(value) => `valor ${number(Number(value))}`}
            />
            <ReferenceArea
              x1={chart.derived.mean - chart.derived.stdDev}
              x2={chart.derived.mean + chart.derived.stdDev}
              fill="#6366f12e"
              strokeOpacity={0}
            />
            <Bar
              yAxisId="left"
              dataKey="frequency"
              name="frecuencia"
              barSize={chart.barSize}
              isAnimationActive={false}
            >
              {chart.rows.map((row, index) => (
                <Cell
                  key={index}
                  fill={row.overflowClass ? "#d97706" : "#6366f1"}
                  stroke="none"
                />
              ))}
            </Bar>
            <ReferenceLine
              x={chart.derived.mean}
              stroke="#0f172a"
              strokeWidth={1.5}
            />
            <Line
              yAxisId="right"
              type="stepAfter"
              dataKey="accumulated"
              name="acumulado"
              stroke="#ea580c"
              strokeWidth={1.75}
              dot={false}
              isAnimationActive={false}
            />
          </ComposedChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
