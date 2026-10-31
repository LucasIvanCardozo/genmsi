/**
 * The indicators of the queues.
 *
 * The table answers the question of every queue: how much it holds right
 * now, how long its transactions waited and how many of them went through
 * it.
 *
 * The wait of the engine is the one of the table: the area of the queue
 * over its entries (`AVE.TIME`). The mean of the departures is the other
 * one —the sum of the waits of the ones that left over the ones that left—
 * and it is a different number with a different name.
 */
import { decimals, number } from "../../format.ts";
import type { QueueNumbers } from "./entities.ts";
import StatTable from "./StatTable.tsx";

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
          label: "Ahora (u)",
          state: (row) => (row.live ? "instant" : "unknown"),
          value: (row) => number(row.content),
        },
        {
          label: "Contenido medio (u)",
          derived: true,
          state: (row) => (row.live ? "derived" : "unknown"),
          value: (row) => decimals(row.meanContent),
        },
        {
          label: "Espera media (s)",
          derived: true,
          state: (row) => (row.live ? "derived" : "unknown"),
          value: (row) => decimals(row.engineMeanWait),
        },
        {
          label: "Media de las salidas (s)",
          derived: true,
          state: (row) => (row.live ? "derived" : "unknown"),
          value: (row) => decimals(row.departureMeanWait),
        },
        {
          label: "Máximo (u)",
          state: (row) => (row.live ? "instant" : "unknown"),
          value: (row) => number(row.max),
        },
        {
          label: "Entradas",
          state: (row) => (row.live ? "instant" : "unknown"),
          value: (row) => number(row.entries),
        },
        {
          label: "Espera cero (%)",
          derived: true,
          state: (row) => (row.live ? "derived" : "unknown"),
          value: (row) => decimals(row.zeroShare * 100),
        },
      ]}
    />
  );
}
