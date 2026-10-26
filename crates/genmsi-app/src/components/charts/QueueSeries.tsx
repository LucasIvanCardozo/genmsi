/**
 * The content of the queues over the time of the run.
 *
 * The series is the **exact** history of the queue: the steps are the
 * instants its content changes. The axis of the time is the time of the
 * run, from zero to its end, and the vertical line marks the instant the
 * reader is at.
 */
import { useEffect, useMemo, useState } from "react";
import { decimals } from "../../format.ts";
import { seriesFrom } from "../../monitor/series.ts";
import type { ReplayIndex } from "../../monitor/replay.ts";
import type { QueueNumbers } from "./entities.ts";
import { colorOf } from "./palette.ts";
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
          <p className="mb-1 text-[11px] text-slate-500">
            Las líneas punteadas son el contenido medio de cada cola, derivado
            del área que cubrió:{" "}
            {series
              .filter((entry) => entry.mean !== undefined)
              .map((entry) => `${entry.name} ${decimals(entry.mean ?? 0)}`)
              .join(" · ")}
            .
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
