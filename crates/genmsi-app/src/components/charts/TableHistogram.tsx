/**
 * The histogram of a table of the model.
 *
 * The bars are the frequencies of the **instant**: the table shows what
 * was tabulated up to the clock, not what will be tabulated at the end of
 * the run. The accumulated curve is their running total, and the last
 * class of the table —the one of the values that do not fit— is drawn
 * apart.
 *
 * The empty classes at the end are not drawn: the report of the engine
 * stops the table when the classes add up to the whole and says how many
 * empty ones are left.
 *
 * The mean and the deviation come from the sum and the sum of squares that
 * the engine publishes; the mean and the deviation of the overflows come
 * from the sums that the engine publishes for them.
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
import { overflowDerived, tableDerived } from "../../monitor/derived.ts";
import { histogramRows } from "../../monitor/series.ts";
import type { Figures } from "../../monitor/stats.ts";
import type { ReportDocument, Table } from "../../types";
import { FIGURE_STATES } from "./states.ts";

/** One table of the model, with the numbers the chart reads. */
interface TableEntry {
  /** The name of the table. */
  name: string;
  /** The definition of the table. */
  table: Table;
}

/** One figure of the card, with the state it is in. */
function Figure({
  label,
  value,
  state,
}: {
  /** The name of the figure. */
  label: string;
  /** The figure. */
  value: string;
  /** The state the figure is in. */
  state: "instant" | "derived";
}) {
  return (
    <li className="flex items-baseline justify-between gap-3">
      <span className="text-slate-500">
        {label}
        {state === "derived" ? " *" : ""}
      </span>
      <b className={`font-mono ${FIGURE_STATES[state].textClass}`}>{value}</b>
    </li>
  );
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
    // La media y la desviación de los desbordes las publica el motor: son
    // las de la corrida completa, y el replay no lleva esa cuenta.
    const spilled = overflowDerived({
      overflow: entry.table.overflow,
      sum: entry.table.overflow_sum,
      sumSquares: entry.table.overflow_sum_squares,
    });
    const rows = histogramRows(entry.table, frequencies);
    const barSize = Math.max(4, Math.round(620 / Math.max(rows.length, 1)) - 8);
    return {
      rows,
      derived,
      spilled,
      samples,
      overflow,
      barSize,
    };
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
      <ul className="mb-1 flex flex-col gap-0.5 text-[11px]">
        <Figure
          label="muestras"
          value={number(chart.samples)}
          state="instant"
        />
        <Figure
          label="media"
          value={`${decimals(chart.derived.mean, 2)} s`}
          state="derived"
        />
        <Figure
          label="desviación"
          value={`${decimals(chart.derived.stdDev, 2)} s`}
          state="derived"
        />
        <Figure
          label="desbordes"
          value={number(chart.overflow)}
          state="instant"
        />
        {chart.spilled.mean > 0 && (
          <>
            <Figure
              label="media de los desbordes"
              value={`${decimals(chart.spilled.mean, 2)} s`}
              state="derived"
            />
            <Figure
              label="desviación de los desbordes"
              value={`${decimals(chart.spilled.stdDev, 2)} s`}
              state="derived"
            />
          </>
        )}
      </ul>
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
              labelFormatter={(value, payload) => {
                // El tooltip dice el intervalo, no su punto medio: una
                // espera de 0,05 s pertenece al intervalo que lo dice.
                const row = payload?.[0]?.payload as
                  | { label?: string }
                  | undefined;
                return row?.label ?? `valor ${number(Number(value))}`;
              }}
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
