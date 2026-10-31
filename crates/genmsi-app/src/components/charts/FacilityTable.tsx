/**
 * The indicators of the facilities.
 *
 * The utilization and the mean service are derived from the numbers of the
 * instant, and the foot of the card says the divisor. The interruptions
 * are the ones of the instant.
 *
 * The column **Ahora** says who holds the facility at the instant the
 * reader is at: it is the one thing about the state of a run that no other
 * table says.
 *
 * The retries are the ones the **engine** published: the replay never
 * emits a facility retry, so the figure of the instant would always be a
 * zero that says nothing. The engine counts the transactions that a gate
 * on the facility rejected, and that is what the column says.
 */
import { decimals, number } from "../../format.ts";
import type { FacilityNumbers } from "./entities.ts";
import StatTable from "./StatTable.tsx";

export default function FacilityTable({
  facilities,
}: {
  /** The facilities of the run, with their numbers. */
  facilities: FacilityNumbers[];
}) {
  return (
    <StatTable
      rows={facilities}
      columns={[
        { label: "Facility", name: true, value: (row) => row.name },
        {
          label: "Ahora",
          state: (row) => (row.live ? "instant" : "unknown"),
          value: (row) => (row.owner === null ? "libre" : `tx ${row.owner}`),
        },
        {
          label: "Utilización",
          derived: true,
          state: (row) => (row.live ? "derived" : "unknown"),
          value: (row) => `${Math.round(row.utilization * 100)} %`,
        },
        {
          label: "Servicio medio",
          derived: true,
          state: (row) => (row.live ? "derived" : "unknown"),
          value: (row) => `${decimals(row.meanService)} s`,
        },
        {
          label: "Entradas",
          state: (row) => (row.live ? "instant" : "unknown"),
          value: (row) => number(row.entries),
        },
        {
          label: "Interrupciones",
          state: (row) => (row.live ? "instant" : "unknown"),
          value: (row) => number(row.inter),
        },
        {
          label: "Reintentos de las compuertas",
          state: (row) =>
            row.publishedRetry === null ? "unknown" : "published",
          value: (row) => number(row.publishedRetry ?? 0),
        },
      ]}
    />
  );
}
