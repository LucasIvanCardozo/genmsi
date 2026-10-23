import { useEffect, useMemo, useState } from "react";
import {
  Bar,
  CartesianGrid,
  ComposedChart,
  Legend,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { number } from "../format";
import { mergeSeries, type ReplayIndex } from "../monitor/replay";
import { useLive } from "../monitor/live";
import type { Figures } from "../monitor/stats";
import type { ReportDocument, Table } from "../types";

/** How many bars a histogram has at most. */
export const MAX_BARS = 200;

/** How many entities a time series shows at once by default. */
const DEFAULT_SERIES = 3;

/** One bar of a histogram: an interval (or a group of intervals) of a table. */
export interface HistogramBar {
  /** The range of the interval, as the report prints it. */
  label: string;
  /** The transactions the interval holds. */
  frequency: number;
  /** The accumulated share of the samples, in percent. */
  accumulated: number;
}

/** The histogram of one table of the model. */
export interface Histogram {
  /** The name of the table. */
  name: string;
  /** The bars of the histogram. */
  bars: HistogramBar[];
  /** How many intervals each bar covers (one when the table is drawn as it is). */
  group: number;
  /** The values the table recorded. */
  samples: number;
}

/**
 * The histogram of a table.
 *
 * The bars are the **published frequencies** and the accumulated curve is
 * their running total: no metric the engine computes is derived here. A
 * table with more intervals than the chart has bars is grouped, and the
 * label says how many intervals each bar covers.
 *
 * The frequencies of the instant can be given instead, so the histogram
 * grows while the run is replayed: they are the engine's own arithmetic,
 * rebuilt from the samples the trace carries.
 */
export function histogramOf(table: Table, frequencies?: number[]): Histogram {
  const counts = frequencies ?? table.frequencies;
  const total =
    frequencies === undefined
      ? table.samples
      : counts.reduce((sum, count) => sum + count, 0);
  const group = Math.max(1, Math.ceil(table.intervals / MAX_BARS));
  const samples = Math.max(total, 1);
  const bars: HistogramBar[] = [];
  let cumulative = 0;
  for (let index = 0; index < counts.length; index += group) {
    let frequency = 0;
    const last = Math.min(index + group, counts.length);
    for (let step = index; step < last; step += 1) {
      frequency += counts[step];
    }
    cumulative += frequency;
    const low = index === 0 ? 0 : table.first + (index - 1) * table.width + 1;
    const high = table.first + (last - 1) * table.width;
    bars.push({
      label: `${number(low)} - ${number(high)}`,
      frequency,
      accumulated: (cumulative / samples) * 100,
    });
  }
  return { name: table.name, bars, group, samples: total };
}

/** The tables of the model: the ones of `TABULATE` and the ones of the queues. */
function tablesOf(report: ReportDocument | null): { name: string; table: Table }[] {
  if (report === null) {
    return [];
  }
  const tables = Object.values(report.entities.tables).map((table) => ({
    name: table.name,
    table,
  }));
  const qtables = Object.values(report.entities.qtables).map((table) => ({
    name: table.name,
    table,
  }));
  return [...tables, ...qtables];
}

/** The number of an axis: the simulated time, rounded to a second. */
function axisTime(value: number): string {
  return String(Math.round(value));
}

/**
 * The charts of the run: the time series of the entities and the histograms
 * of the tables.
 *
 * The time series comes from the replay (the state over the run, rebuilt by
 * walking the trace) and the histograms come from the report document (the
 * published frequencies). The charts read the documents; they compute
 * nothing that the engine computes.
 */
export default function MonitorCharts({
  report,
  monitor,
  figures,
}: {
  report: ReportDocument | null;
  monitor: ReplayIndex;
  /** The figures of the run at the instant the replay is at. */
  figures?: Figures | null;
}) {
  const tables = useMemo(() => tablesOf(report), [report]);
  const [selectedQueues, setSelectedQueues] = useState<string[]>([]);
  const [selectedFacilities, setSelectedFacilities] = useState<string[]>([]);
  const [selectedTable, setSelectedTable] = useState<string | null>(null);
  // The clock of the replay: the charts show the run as it is now, not the
  // run as it will be.
  const clock = useLive((state) => state.clock);

  // A new run has its own entities: the selection follows the run.
  useEffect(() => {
    setSelectedQueues(monitor.queues.slice(0, DEFAULT_SERIES));
    setSelectedFacilities(monitor.facilities.slice(0, DEFAULT_SERIES));
    setSelectedTable(tables[0]?.name ?? null);
  }, [monitor, tables]);

  const queueData = useMemo(
    () => mergeSeries(selectedQueues, monitor.queuePoints, clock),
    [monitor, selectedQueues, clock],
  );

  const facilityData = useMemo(
    () => mergeSeries(selectedFacilities, monitor.facilityPoints, clock),
    [monitor, selectedFacilities, clock],
  );

  const histogram = useMemo(() => {
    const found = tables.find((candidate) => candidate.name === selectedTable);
    if (found === undefined) {
      return null;
    }
    // The frequencies of the instant, when the replay knows the table's
    // samples.
    const live = figures?.tables[found.table.name]?.frequencies;
    return histogramOf(found.table, live);
  }, [tables, selectedTable, figures]);

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3 overflow-auto">
      <section className="rounded-lg border border-slate-200 bg-white p-4">
        <header className="mb-2 flex flex-wrap items-baseline justify-between gap-2">
          <h3 className="text-sm font-semibold text-slate-800">
            Contenido de las colas a lo largo del tiempo
          </h3>
          <span className="text-[11px] text-slate-500">
            {queueData.length} instantes en que cambia el estado
          </span>
        </header>
        <EntityPicker
          entities={monitor.queues}
          selected={selectedQueues}
          onToggle={(name) =>
            setSelectedQueues((current) =>
              current.includes(name)
                ? current.filter((candidate) => candidate !== name)
                : [...current, name],
            )
          }
        />
        {selectedQueues.length === 0 ? (
          <p className="py-6 text-center text-xs text-slate-400">
            Elige una cola para verla.
          </p>
        ) : (
          <div className="h-64">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={queueData} margin={{ top: 8, right: 16, bottom: 8, left: 0 }}>
                <CartesianGrid stroke="#f1f5f9" />
                <XAxis
                  dataKey="time"
                  tickFormatter={axisTime}
                  fontSize={10}
                  minTickGap={40}
                />
                <YAxis fontSize={10} allowDecimals={false} />
                <Tooltip
                  formatter={(value) => number(Number(value))}
                  labelFormatter={(value) => `t = ${number(Number(value))}`}
                />
                <Legend wrapperStyle={{ fontSize: 11 }} />
                {selectedQueues.map((name) => (
                  <Line
                    key={name}
                    type="stepAfter"
                    dataKey={name}
                    stroke="#4f46e5"
                    strokeWidth={1.75}
                    dot={false}
                    isAnimationActive={false}
                  />
                ))}
              </LineChart>
            </ResponsiveContainer>
          </div>
        )}
      </section>

      {monitor.facilities.length > 0 && (
        <section className="rounded-lg border border-slate-200 bg-white p-4">
          <header className="mb-2">
            <h3 className="text-sm font-semibold text-slate-800">
              Facilities a lo largo del tiempo
            </h3>
            <p className="text-[11px] text-slate-500">
              1 cuando la facility está ocupada, 0 cuando está libre. Cada
              escalón es un cambio del estado.
            </p>
          </header>
          <EntityPicker
            entities={monitor.facilities}
            selected={selectedFacilities}
            onToggle={(name) =>
              setSelectedFacilities((current) =>
                current.includes(name)
                  ? current.filter((candidate) => candidate !== name)
                  : [...current, name],
              )
            }
          />
          {selectedFacilities.length === 0 ? (
            <p className="py-6 text-center text-xs text-slate-400">
              Elige una facility para verla.
            </p>
          ) : (
            <div className="h-64">
              <ResponsiveContainer width="100%" height="100%">
                <LineChart data={facilityData} margin={{ top: 8, right: 16, bottom: 8, left: 0 }}>
                  <CartesianGrid stroke="#f1f5f9" />
                  <XAxis
                    dataKey="time"
                    tickFormatter={axisTime}
                    fontSize={10}
                    minTickGap={40}
                  />
                  <YAxis fontSize={10} domain={[0, 1]} ticks={[0, 1]} />
                  <Tooltip
                    formatter={(value) => (Number(value) === 1 ? "ocupada" : "libre")}
                    labelFormatter={(value) => `t = ${number(Number(value))}`}
                  />
                  <Legend wrapperStyle={{ fontSize: 11 }} />
                  {selectedFacilities.map((name) => (
                    <Line
                      key={name}
                      type="stepAfter"
                      dataKey={name}
                      stroke="#0d9488"
                      strokeWidth={1.75}
                      dot={false}
                      isAnimationActive={false}
                    />
                  ))}
                </LineChart>
              </ResponsiveContainer>
            </div>
          )}
        </section>
      )}

      {tables.length > 0 && (
        <section className="rounded-lg border border-slate-200 bg-white p-4">
          <header className="mb-2 flex flex-wrap items-baseline justify-between gap-2">
            <h3 className="text-sm font-semibold text-slate-800">
              Histograma de las tablas del modelo
            </h3>
            <div className="flex flex-wrap gap-1">
              {tables.map((candidate) => (
                <button
                  key={candidate.name}
                  type="button"
                  onClick={() => setSelectedTable(candidate.name)}
                  className={[
                    "rounded-md border px-2 py-1 text-[11px] font-medium transition",
                    candidate.name === selectedTable
                      ? "border-indigo-500 bg-indigo-50 text-indigo-700"
                      : "border-slate-300 bg-white text-slate-600 hover:bg-slate-50",
                  ].join(" ")}
                >
                  {candidate.name}
                </button>
              ))}
            </div>
          </header>
          {histogram === null ? (
            <p className="py-6 text-center text-xs text-slate-400">Elige una tabla.</p>
          ) : (
            <>
              <p className="mb-2 text-[11px] text-slate-500">
                Las barras son las frecuencias de la tabla y la curva es su suma
                acumulada, sobre {number(histogram.samples)} valores.
                {histogram.group > 1 && (
                  <> Cada barra cubre {histogram.group} intervalos.</>
                )}
              </p>
              <div className="h-72">
                <ResponsiveContainer width="100%" height="100%">
                  <ComposedChart data={histogram.bars} margin={{ top: 8, right: 16, bottom: 8, left: 0 }}>
                    <CartesianGrid stroke="#f1f5f9" />
                    <XAxis
                      dataKey="label"
                      fontSize={10}
                      minTickGap={30}
                      interval="preserveStartEnd"
                    />
                    <YAxis yAxisId="left" fontSize={10} allowDecimals={false} />
                    <YAxis yAxisId="right" orientation="right" fontSize={10} domain={[0, 100]} />
                    <Tooltip
                      formatter={(value, name) =>
                        name === "acumulado"
                          ? `${number(Number(value))} %`
                          : number(Number(value))
                      }
                    />
                    <Legend wrapperStyle={{ fontSize: 11 }} />
                    <Bar
                      yAxisId="left"
                      dataKey="frequency"
                      name="frecuencia"
                      fill="#6366f1"
                      isAnimationActive={false}
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
            </>
          )}
        </section>
      )}
    </div>
  );
}

/** The entities the user chooses to see in the time series. */
function EntityPicker({
  entities,
  selected,
  onToggle,
}: {
  entities: string[];
  selected: string[];
  onToggle: (name: string) => void;
}) {
  if (entities.length === 0) {
    return null;
  }
  return (
    <div className="mb-2 flex flex-wrap gap-1">
      {entities.map((name) => {
        const active = selected.includes(name);
        return (
          <button
            key={name}
            type="button"
            onClick={() => onToggle(name)}
            className={[
              "rounded-md border px-2 py-1 text-[11px] font-medium transition",
              active
                ? "border-indigo-500 bg-indigo-50 text-indigo-700"
                : "border-slate-300 bg-white text-slate-500 hover:bg-slate-50",
            ].join(" ")}
          >
            {name}
          </button>
        );
      })}
    </div>
  );
}
