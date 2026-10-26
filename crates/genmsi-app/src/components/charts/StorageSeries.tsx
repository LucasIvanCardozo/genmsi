/**
 * The units in use of a storage, over the time of the run.
 *
 * The chart says how many units the storage held at every instant, and the
 * dashed line is its capacity: the model is blocked when the curve reaches
 * it.
 */
import { useMemo, useState } from "react";
import { number } from "../../format.ts";
import { seriesFrom } from "../../monitor/series.ts";
import type { ReplayIndex } from "../../monitor/replay.ts";
import type { StorageNumbers } from "./entities.ts";
import StatCard from "./StatCard.tsx";
import TimeSeriesChart from "./TimeSeriesChart.tsx";

export default function StorageSeries({
  monitor,
  storages,
  clock,
}: {
  /** The index of the replay: the instants the units change. */
  monitor: ReplayIndex;
  /** The storages of the run, with their numbers. */
  storages: StorageNumbers[];
  /** The instant the reader is at. */
  clock: number;
}) {
  const [selected, setSelected] = useState<string | null>(null);
  const name = selected ?? storages[0]?.name ?? null;
  const storage = storages.find((candidate) => candidate.name === name) ?? null;

  const data = useMemo(
    () => (name === null ? [] : seriesFrom([name], monitor.storagePoints, clock)),
    [monitor, name, clock],
  );

  if (storage === null) {
    return (
      <p className="py-6 text-center text-xs text-slate-400">
        El modelo no tiene almacenes.
      </p>
    );
  }

  const meanUtilization =
    storage.capacity > 0 ? storage.meanUtilization : 0;

  return (
    <div>
      {storages.length > 1 && (
        <div className="mb-2 flex flex-wrap gap-1">
          {storages.map((candidate) => (
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
      {storage.capacity > 0 && (
        <p className="mb-1 text-[11px] text-slate-500">
          El almacén tiene {number(storage.capacity)} unidades: la línea
          punteada es su límite.
        </p>
      )}
      <TimeSeriesChart
        data={data}
        series={[{ name: storage.name, color: "#0d9488" }]}
        endTime={monitor.endTime}
        clock={clock}
        reference={
          storage.capacity > 0
            ? { y: storage.capacity, color: "#d97706", label: `capacidad ${number(storage.capacity)}` }
            : undefined
        }
        yDomain={[0, Math.max(storage.capacity, 1)]}
      />
      <div className="mt-3 grid grid-cols-2 gap-3 lg:grid-cols-4">
        <StatCard
          label="En uso ahora"
          value={`${number(storage.inUse)} u`}
          note={
            storage.capacity > 0
              ? `de ${number(storage.capacity)} disponibles`
              : "sin capacidad declarada"
          }
          live={storage.live}
        />
        <StatCard
          label="Máximo"
          value={`${number(storage.maxUse)} u`}
          note="unidades ocupadas a la vez"
          live={storage.live}
        />
        <StatCard
          label="Utilización media"
          value={`${Math.round(meanUtilization * 100)} %`}
          note="derivada: área / tiempo / capacidad"
          live={storage.live}
        />
        <StatCard
          label="Reintentos"
          value={number(storage.retry)}
          note="intentos que no tuvieron unidades"
          live={storage.live}
        />
      </div>
    </div>
  );
}
