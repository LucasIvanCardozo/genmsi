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
import type { ReportDocument } from "../../types";
import StatCard from "./StatCard.tsx";

/** A number with two decimals, the way the app writes the small ones. */
function decimals(value: number): string {
  if (!Number.isFinite(value)) {
    return "0,00";
  }
  return value.toFixed(2).replace(".", ",");
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
  population,
  live,
  clock,
  generated,
}: {
  /** The report of the run. */
  report: ReportDocument;
  /**
   * The transactions inside the model at the instant the reader is at, or
   * null when the live layer does not know that instant.
   */
  population: number | null;
  /** Whether the numbers are the ones of the instant the reader is at. */
  live: boolean;
  /** The clock of the replay. */
  clock: number;
  /** How many transactions were created up to the instant. */
  generated: number;
}) {
  const elapsed = report.end_time - report.start_time;

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
      {/* The population is the one the live layer published with the clock.
          When the layer does not know the instant, the card says so: no
          number is derived from a clock that does not match the figures. */}
      <StatCard
        label="Población"
        value={
          population !== null
            ? number(population)
            : live
              ? "Sin datos."
              : number(publishedPopulation(report))
        }
        note={live ? "dentro del modelo ahora" : "dentro del modelo al terminar"}
        live={population !== null}
      />
    </div>
  );
}
