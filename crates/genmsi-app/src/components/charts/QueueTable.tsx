/**
 * The indicators of the queues.
 *
 * The table answers the question of every queue: how much it held, how long
 * its transactions waited and how many of them went through it. The
 * columns with an asterisk are derived from the numbers the engine
 * publishes.
 */
import { decimals, number } from "../../format.ts";
import type { QueueNumbers } from "./entities.ts";
import StatTable from "./StatTable.tsx";

/** A share of the entries, in percent. */
function percent(share: number): string {
  return `${Math.round(share * 100)} %`;
}

export default function QueueTable({
  queues,
}: {
  /** The queues of the run, with their numbers. */
  queues: QueueNumbers[];
}) {
  return (
    <StatTable
      rows={queues}
      columns={[
        {
          label: "Cola",
          name: true,
          value: (row) => row.name,
        },
        {
          label: "Contenido medio",
          derived: true,
          live: (row) => row.live,
          value: (row) => decimals(row.meanContent),
        },
        {
          label: "Espera media",
          derived: true,
          live: (row) => row.live,
          value: (row) => decimals(row.meanWait),
        },
        {
          label: "Máximo",
          live: (row) => row.live,
          value: (row) => number(row.max),
        },
        {
          label: "Entradas",
          live: (row) => row.live,
          value: (row) => number(row.entries),
        },
        {
          label: "Espera cero",
          derived: true,
          live: (row) => row.live,
          value: (row) => percent(row.zeroShare),
        },
      ]}
    />
  );
}
