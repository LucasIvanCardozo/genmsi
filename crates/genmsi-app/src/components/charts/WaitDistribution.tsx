/**
 * The distribution of the waits of a queue.
 *
 * Every transaction that left the queue is one sample: the bar says how
 * many of them waited that time, and the curve says the share of the
 * transactions that left by then.
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
import { cumulative, histogram } from "../../monitor/series.ts";
import type { ReplayIndex } from "../../monitor/replay.ts";

/** The widths a histogram of waits can have. */
const STEPS = [0.1, 0.2, 0.5, 1, 2, 5, 10, 20, 30, 60, 120, 300, 600];

/**
 * The width of the intervals of the histogram.
 *
 * The width is the smallest one that leaves about a dozen intervals: a
 * histogram with a hundred bars is not read, and one with three bars says
 * nothing.
 */
function widthOf(longest: number): number {
  const wanted = Math.max(longest, 1) / 12;
  for (const step of STEPS) {
    if (step >= wanted) {
      return step;
    }
  }
  return STEPS[STEPS.length - 1];
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

  const samples = useMemo(() => {
    if (queue === null) {
      return [];
    }
    return (monitor.waits.get(queue) ?? [])
      .filter((sample) => sample.time <= clock)
      .map((sample) => sample.wait);
  }, [monitor, queue, clock]);

  const chart = useMemo(() => {
    if (samples.length === 0) {
      return null;
    }
    const longest = Math.max(...samples);
    const width = widthOf(longest);
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
  }, [samples]);

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
          <p className="mb-1 text-[11px] text-slate-500">
            {chart.empty ? (
              <>
                Las {number(samples.length)} salidas de esta cola no esperaron
                nada: no hubo cola que esperar.
              </>
            ) : (
              <>
                {number(samples.length)} salidas · media de las salidas{" "}
                {decimals(chart.mean, 2)} s · intervalos de{" "}
                {decimals(chart.width, 2)} s.
              </>
            )}
          </p>
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
                  labelFormatter={(value) => `espera ${number(Number(value))} s`}
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
