/**
 * The indicators of the facilities.
 *
 * The utilization and the mean service are derived from the numbers the
 * engine publishes; the interruptions and the retries are the ones it
 * counts.
 *
 * The column **Ahora** says who holds the facility at the instant the
 * reader is at: it is the one thing about the state of a run that no other
 * table says.
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
          live: (row) => row.live,
          value: (row) => (row.owner === null ? "libre" : `tx ${row.owner}`),
        },
        {
          label: "Utilización",
          derived: true,
          live: (row) => row.live,
          value: (row) => `${Math.round(row.utilization * 100)} %`,
        },
        {
          label: "Servicio medio",
          derived: true,
          live: (row) => row.live,
          value: (row) => `${decimals(row.meanService)} s`,
        },
        {
          label: "Entradas",
          live: (row) => row.live,
          value: (row) => number(row.entries),
        },
        {
          label: "Interrupciones",
          live: (row) => row.live,
          value: (row) => number(row.inter),
        },
        {
          label: "Reintentos",
          live: (row) => row.live,
          value: (row) => number(row.retry),
        },
      ]}
    />
  );
}
