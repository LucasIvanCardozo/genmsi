/**
 * The distribution of the waits of a queue.
 *
 * Every transaction that left the queue is one sample: the bar says how
 * many of them waited that time, and the curve says the share of the
 * transactions that left by then.
 *
 * The width of the intervals is chosen **once per queue**, from the
 * longest wait of the whole trace: a width that changed with the clock
 * would re-bin the histogram while the run advances, and two instants of
 * the same queue would not be comparable.
 *
 * The mean of the histogram is the mean **of the samples** (the
 * transactions that left), which is not the mean wait of the queue: that
 * one divides the accumulated wait by every entry, including the
 * transactions that are still waiting.
 */
import { useMemo, useState } from "react";
import {
  Bar,
  CartesianGrid,
  ComposedChart,
  Line,
  ReferenceLine,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { decimals, number } from "../../format.ts";
import { cumulative, histogram, waitWidth } from "../../monitor/series.ts";
import type { ReplayIndex } from "../../monitor/replay.ts";
import { FIGURE_STATES } from "./states.ts";

/** The longest wait of a list of samples. */
function longestOf(waits: number[]): number {
  let longest = 0;
  for (const wait of waits) {
    if (wait > longest) {
      longest = wait;
    }
  }
  return longest;
}

export default function WaitDistribution({
  monitor,
  clock,
}: {
  /** The index of the replay: the waits of every departure. */
  monitor: ReplayIndex;
  /** The instant the reader is at. */
  clock: number;
}) {
  const names = useMemo(() => [...monitor.waits.keys()], [monitor]);
  const [selected, setSelected] = useState<string | null>(null);
  const queue = selected ?? names[0] ?? null;

  const all = useMemo(
    () => (queue === null ? [] : (monitor.waits.get(queue) ?? [])),
    [monitor, queue],
  );
  // The width of the intervals is chosen once per queue, from the whole
  // trace: two instants of the same queue are drawn with the same
  // intervals.
  const width = useMemo(
    () => waitWidth(longestOf(all.map((sample) => sample.wait))),
    [all],
  );

  const samples = useMemo(
    () =>
      all
        .filter((sample) => sample.time <= clock)
        .map((sample) => sample.wait),
    [all, clock],
  );

  const chart = useMemo(() => {
    if (samples.length === 0) {
      return null;
    }
    const longest = longestOf(samples);
    const bins = histogram(samples, width);
    const shares = cumulative(bins);
    const mean = samples.reduce((sum, wait) => sum + wait, 0) / samples.length;
    const barSize = Math.max(4, Math.round(620 / Math.max(bins.length, 1)) - 8);
    const rows = bins.map((bin, index) => ({
      x: bin.low + width / 2,
      label: `${number(bin.low)} - ${number(bin.high)}`,
      frequency: bin.count,
      accumulated: shares[index],
    }));
    return { rows, mean, width, longest, empty: longest === 0, barSize };
  }, [samples, width]);

  if (names.length === 0) {
    return (
      <p className="py-6 text-center text-xs text-slate-400">
        Ninguna cola tuvo salidas que esperar.
      </p>
    );
  }

  return (
    <div>
      <div className="mb-2 flex flex-wrap gap-1">
        {names.map((name) => (
          <button
            key={name}
            type="button"
            onClick={() => setSelected(name)}
            className={[
              "rounded-md border px-2 py-1 text-[11px] font-medium transition",
              name === queue
                ? "border-indigo-500 bg-indigo-50 text-indigo-700"
                : "border-slate-300 bg-white text-slate-500 hover:bg-slate-50",
            ].join(" ")}
          >
            {name}
          </button>
        ))}
      </div>
      {chart === null ? (
        <p className="py-6 text-center text-xs text-slate-400">
          Todavía no salió ninguna transacción de esta cola.
        </p>
      ) : (
        <>
          <ul className="mb-1 flex flex-col gap-0.5 text-[11px]">
            <li className="flex items-baseline justify-between gap-3">
              <span className="text-slate-500">salidas</span>
              <b className={`font-mono ${FIGURE_STATES.instant.textClass}`}>
                {number(samples.length)}
              </b>
            </li>
            <li className="flex items-baseline justify-between gap-3">
              <span className="text-slate-500">media de las salidas *</span>
              <b className={`font-mono ${FIGURE_STATES.derived.textClass}`}>
                {decimals(chart.mean, 2)} s
              </b>
            </li>
            <li className="flex items-baseline justify-between gap-3">
              <span className="text-slate-500">intervalos de</span>
              <b className="font-mono text-slate-700">
                {decimals(chart.width, 2)} s
              </b>
            </li>
          </ul>
          {monitor.truncated && (
            <p className="mb-1 text-[10px] leading-snug text-slate-400">
              La traza se cortó en {number(monitor.limit)} registros: las
              esperas que pasaron después no están.
            </p>
          )}
          <div className={chart.empty ? "hidden" : "h-64"}>
            <ResponsiveContainer width="100%" height="100%">
              <ComposedChart
                data={chart.rows}
                margin={{ top: 8, right: 16, bottom: 8, left: 0 }}
              >
                <CartesianGrid stroke="#f1f5f9" />
                <XAxis
                  dataKey="x"
                  type="number"
                  domain={[0, chart.longest + chart.width]}
                  tickFormatter={(value) => decimals(value, 2)}
                  fontSize={10}
                  minTickGap={30}
                />
                <YAxis
                  yAxisId="left"
                  fontSize={10}
                  allowDecimals={false}
                  width={44}
                />
                <YAxis
                  yAxisId="right"
                  orientation="right"
                  fontSize={10}
                  domain={[0, 100]}
                  width={40}
                />
                <Tooltip
                  formatter={(value, name) =>
                    name === "acumulado"
                      ? `${number(Number(value))} %`
                      : number(Number(value))
                  }
                  labelFormatter={(value, payload) => {
                    // El tooltip dice el intervalo, no su punto medio: una
                    // espera de 0,05 s pertenece al intervalo que lo dice.
                    const row = payload?.[0]?.payload as
                      | { label?: string }
                      | undefined;
                    return row?.label ?? `espera ${number(Number(value))} s`;
                  }}
                />
                <ReferenceLine
                  x={chart.mean}
                  stroke="#0f172a"
                  strokeWidth={1.5}
                />
                <Bar
                  yAxisId="left"
                  dataKey="frequency"
                  name="frecuencia"
                  fill="#6366f1"
                  barSize={chart.barSize}
                  isAnimationActive={false}
                />
                <ReferenceLine
                  x={chart.mean}
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
        </>
      )}
    </div>
  );
}
