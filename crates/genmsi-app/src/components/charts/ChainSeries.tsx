/**
 * The size of the chains of the run, over the time of the run.
 *
 * A chain holds transactions that are waiting for each other: the curve
 * says how many are linked at every instant.
 */
import { useMemo, useState } from "react";
import { decimals, number } from "../../format.ts";
import { seriesFrom } from "../../monitor/series.ts";
import type { ReplayIndex } from "../../monitor/replay.ts";
import type { ChainNumbers } from "./entities.ts";
import StatCard from "./StatCard.tsx";
import TimeSeriesChart from "./TimeSeriesChart.tsx";

export default function ChainSeries({
  monitor,
  chains,
  clock,
}: {
  /** The index of the replay: the instants the chains change. */
  monitor: ReplayIndex;
  /** The chains of the run, with their numbers. */
  chains: ChainNumbers[];
  /** The instant the reader is at. */
  clock: number;
}) {
  const [selected, setSelected] = useState<string | null>(null);
  const name = selected ?? chains[0]?.name ?? null;
  const chain = chains.find((candidate) => candidate.name === name) ?? null;

  const data = useMemo(
    () => (name === null ? [] : seriesFrom([name], monitor.chainPoints, clock)),
    [monitor, name, clock],
  );

  if (chain === null) {
    return (
      <p className="py-6 text-center text-xs text-slate-400">
        El modelo no tiene cadenas.
      </p>
    );
  }

  const meanTime = chain.entries > 0 ? chain.totalTime / chain.entries : 0;

  return (
    <div>
      {chains.length > 1 && (
        <div className="mb-2 flex flex-wrap gap-1">
          {chains.map((candidate) => (
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
      <TimeSeriesChart
        data={data}
        series={[{ name: chain.name, color: "#6366f1" }]}
        endTime={monitor.endTime}
        clock={clock}
        height={200}
      />
      <div className="mt-3 grid grid-cols-2 gap-3 lg:grid-cols-4">
        <StatCard
          label="Ahora"
          value={number(chain.content)}
          note="transacciones encadenadas"
          live={chain.live}
        />
        <StatCard
          label="Máximo"
          value={number(chain.max)}
          note="la cadena más larga"
          live={chain.live}
        />
        <StatCard
          label="Entradas"
          value={number(chain.entries)}
          note="enlaces hechos en la corrida"
          live={chain.live}
        />
        <StatCard
          label="Tiempo medio"
          value={`${decimals(meanTime)} s`}
          note="derivado: tiempo total / entradas"
          live={chain.live}
        />
      </div>
    </div>
  );
}
