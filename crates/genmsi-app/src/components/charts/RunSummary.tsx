/**
 * The run in numbers.
 *
 * The card answers the first question a reader asks about a run: how much
 * happened, how long it lasted and how much of it is still inside the
 * model. Every number is the engine's own, except the two that are ratios
 * of the engine's numbers (the throughput and the population), and the
 * card says which are the ones of the instant.
 */
import { number } from "../../format.ts";
import { endReasonText } from "../../engine/reasons.ts";
import type { ReplayState } from "../../monitor/replay.ts";
import type { ReportDocument } from "../../types";
import StatCard from "./StatCard.tsx";

/** A number with two decimals, the way the app writes the small ones. */
function decimals(value: number): string {
  if (!Number.isFinite(value)) {
    return "0,00";
  }
  return value.toFixed(2).replace(".", ",");
}

/** The transactions that are inside the model at this instant. */
function populationOf(state: ReplayState | null): number | null {
  if (state === null) {
    return null;
  }
  let total = 0;
  for (const count of Object.values(state.blocks)) {
    total += count;
  }
  return total;
}

/** The transactions the model ended with, inside it. */
function publishedPopulation(report: ReportDocument): number {
  let total = 0;
  for (const block of report.blocks) {
    total += block.current;
  }
  return total;
}

export default function RunSummary({
  report,
  state,
  clock,
  generated,
}: {
  /** The report of the run. */
  report: ReportDocument;
  /** The state of the replay at the instant the reader is at. */
  state: ReplayState | null;
  /** The clock of the replay. */
  clock: number;
  /** How many transactions were created up to the instant. */
  generated: number;
}) {
  const elapsed = report.end_time - report.start_time;
  const live = state !== null;
  const population = live ? populationOf(state) : null;

  return (
    <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-6">
      <StatCard
        label="Transacciones"
        value={number(live ? generated : report.transactions)}
        note={live ? "creadas hasta ahora" : "creadas en la corrida"}
        live={live}
      />
      <StatCard
        label="Eventos"
        value={number(report.events)}
        note="procesados por el motor"
      />
      <StatCard
        label="Reloj"
        value={number(live ? clock : elapsed)}
        note={live ? "de la corrida, en segundos" : "la duración de la corrida"}
        live={live}
      />
      <StatCard
        label="Fin"
        value={endReasonText(report.end_reason)}
        note="por qué terminó la corrida"
      />
      <StatCard
        label="Caudal"
        value={`${decimals(
          live
            ? clock - report.start_time > 0
              ? generated / (clock - report.start_time)
              : 0
            : elapsed > 0
              ? report.transactions / elapsed
              : 0,
        )} tx/s`}
        note="transacciones por segundo"
        live={live}
      />
      <StatCard
        label="Población"
        value={number(population ?? publishedPopulation(report))}
        note={live ? "dentro del modelo ahora" : "dentro del modelo al terminar"}
        live={live}
      />
    </div>
  );
}
