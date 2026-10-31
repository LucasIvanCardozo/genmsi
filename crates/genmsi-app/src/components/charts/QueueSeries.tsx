/**
 * The content of the queues over the time of the run.
 *
 * The series is the **exact** history of the queue: the steps are the
 * instants its content changes. The axis of the time is the time of the
 * run, from zero to its end, and the vertical line marks the instant the
 * reader is at.
 */
import { useEffect, useMemo, useState } from "react";
import { decimals, number } from "../../format.ts";
import { seriesFrom } from "../../monitor/series.ts";
import type { ReplayIndex } from "../../monitor/replay.ts";
import type { QueueNumbers } from "./entities.ts";
import { colorOf } from "./palette.ts";
import { FIGURE_STATES } from "./states.ts";
import EntityPicker from "./EntityPicker.tsx";
import TimeSeriesChart from "./TimeSeriesChart.tsx";

/** How many queues the chart shows at once by default. */
const DEFAULT_SERIES = 3;

export default function QueueSeries({
  monitor,
  queues,
  clock,
}: {
  /** The index of the replay: the instants the queues change. */
  monitor: ReplayIndex;
  /** The queues of the run, with their numbers. */
  queues: QueueNumbers[];
  /** The instant the reader is at. */
  clock: number;
}) {
  const [selected, setSelected] = useState<string[]>([]);

  // A new run has its own queues: the selection follows the run.
  useEffect(() => {
    setSelected(monitor.queues.slice(0, DEFAULT_SERIES));
  }, [monitor]);

  const data = useMemo(
    () => seriesFrom(selected, monitor.queuePoints, clock),
    [monitor, selected, clock],
  );

  const series = selected.map((name, index) => {
    const queue = queues.find((candidate) => candidate.name === name);
    return {
      name,
      color: colorOf(index),
      mean: queue?.meanContent,
      content: queue?.content ?? 0,
      live: queue?.live ?? false,
    };
  });

  return (
    <div>
      <EntityPicker
        entities={monitor.queues}
        selected={selected}
        onToggle={(name) =>
          setSelected((current) =>
            current.includes(name)
              ? current.filter((candidate) => candidate !== name)
              : [...current, name],
          )
        }
      />
      {selected.length === 0 ? (
        <p className="py-6 text-center text-xs text-slate-400">
          Elige una cola para verla.
        </p>
      ) : (
        <>
          <ul className="mb-1 flex flex-col gap-0.5 text-[11px]">
            {series.map((entry) => (
              <li
                key={entry.name}
                className="flex items-baseline justify-between gap-3"
              >
                <span className="truncate text-slate-500">{entry.name}</span>
                {entry.live ? (
                  <span className="flex items-baseline gap-3">
                    <span className={`font-mono ${FIGURE_STATES.instant.textClass}`}>
                      {number(entry.content)} u
                    </span>
                    <span className="font-mono">
                      <span className={FIGURE_STATES.derived.textClass}>
                        {decimals(entry.mean ?? 0)} u
                      </span>
                      <span className="text-amber-700"> *</span>
                    </span>
                  </span>
                ) : (
                  <span className="inline-block rounded border border-dashed border-slate-400 px-1 font-sans text-[10px] text-slate-300">
                    {FIGURE_STATES.unknown.label}
                  </span>
                )}
              </li>
            ))}
          </ul>
          <p className="mb-1 text-[10px] leading-snug text-slate-400">
            La línea punteada de cada cola es su contenido medio; el eje de
            las ordenadas cuenta unidades en la cola.
          </p>
          <TimeSeriesChart
            data={data}
            series={series}
            endTime={monitor.endTime}
            clock={clock}
          />
        </>
      )}
    </div>
  );
}
