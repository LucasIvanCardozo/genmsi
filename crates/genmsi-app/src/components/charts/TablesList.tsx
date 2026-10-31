/**
 * The tables of the model, one row each.
 *
 * The row says what the instant knows of the table: how many values it
 * tabulated, their mean and their deviation, and how many of them did not
 * fit in any interval. A table the replay has not tabulated yet says
 * "todavía no": it is not a table with no values, it is a table nobody
 * knows yet.
 *
 * The type of the table is in the row: a `TABLE` tabulates a value of the
 * model and a `QTABLE` the waits of a queue, and the two are not the same
 * thing.
 */
import { decimals, number } from "../../format.ts";
import { tableDerived } from "../../monitor/derived.ts";
import type { Figures } from "../../monitor/stats.ts";
import type { ReportDocument, Table } from "../../types";
import StatTable from "./StatTable.tsx";

/** One table of the model, with the numbers the row shows. */
interface Row {
  /** The name of the table. */
  name: string;
  /** What the table tabulates: a value of the model or the waits of a queue. */
  kind: string;
  /** How many values it tabulated. */
  samples: number;
  /** The mean of the values. */
  mean: number;
  /** The deviation of the values. */
  stdDev: number;
  /** How many values did not fit in an interval. */
  overflow: number;
  /** The largest value tabulated. */
  max: number;
  /** Whether the numbers are the ones of the instant. */
  live: boolean;
}

export default function TablesList({
  report,
  live,
}: {
  /** The report of the run. */
  report: ReportDocument;
  /** The figures of the tables at the instant the reader is at. */
  live: Figures | null;
}) {
  const rows: Row[] = [];
  const push = (table: Table, kind: string) => {
    const figure = live?.tables[table.name];
    // Los números son los del instante: lo que se tabuló hasta ahora.
    const samples = figure?.samples ?? 0;
    const sum = figure?.sum ?? 0;
    const sumSquares = figure?.sumSquares ?? 0;
    const overflow = figure?.overflow ?? 0;
    const derived = tableDerived({ samples, sum, sumSquares, overflow });
    rows.push({
      name: table.name,
      kind,
      samples,
      mean: derived.mean,
      stdDev: derived.stdDev,
      overflow,
      max: figure?.maxValue ?? 0,
      live: figure !== undefined,
    });
  };
  for (const table of Object.values(report.entities.tables)) {
    push(table, "tabla");
  }
  for (const table of Object.values(report.entities.qtables)) {
    push(table, "tabla de cola");
  }

  return (
    <StatTable
      rows={rows}
      columns={[
        { label: "Tabla", name: true, value: (row) => row.name },
        {
          label: "Tipo",
          name: true,
          value: (row) => row.kind,
        },
        {
          label: "Muestras",
          state: (row) => (row.live ? "instant" : "unknown"),
          value: (row) => number(row.samples),
        },
        {
          label: "Media",
          derived: true,
          state: (row) => (row.live ? "derived" : "unknown"),
          value: (row) => decimals(row.mean),
        },
        {
          label: "Desviación",
          derived: true,
          state: (row) => (row.live ? "derived" : "unknown"),
          value: (row) => decimals(row.stdDev),
        },
        {
          label: "Desbordes",
          state: (row) => (row.live ? "instant" : "unknown"),
          value: (row) => number(row.overflow),
        },
        {
          label: "Máximo",
          state: (row) => (row.live ? "instant" : "unknown"),
          value: (row) => number(row.max),
        },
      ]}
    />
  );
}
