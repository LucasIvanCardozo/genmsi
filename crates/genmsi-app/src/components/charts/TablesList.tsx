/**
 * The tables of the model, one row each.
 *
 * The row says what the engine published of the table: how many values it
 * tabulated, their mean and their deviation, and how many of them did not
 * fit in any interval.
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
  const push = (table: Table) => {
    const figure = live?.tables[table.name];
    // Los números son los del instante: lo que se tabuló hasta ahora.
    const samples = figure?.samples ?? 0;
    const sum = figure?.sum ?? 0;
    const sumSquares = figure?.sumSquares ?? 0;
    const overflow = figure?.overflow ?? 0;
    const derived = tableDerived({ samples, sum, sumSquares, overflow });
    rows.push({
      name: table.name,
      samples,
      mean: derived.mean,
      stdDev: derived.stdDev,
      overflow,
      max: figure?.maxValue ?? 0,
      live: figure !== undefined,
    });
  };
  for (const table of Object.values(report.entities.tables)) {
    push(table);
  }
  for (const table of Object.values(report.entities.qtables)) {
    push(table);
  }

  return (
    <StatTable
      rows={rows}
      columns={[
        { label: "Tabla", name: true, value: (row) => row.name },
        {
          label: "Muestras",
          live: (row) => row.live,
          value: (row) => number(row.samples),
        },
        {
          label: "Media",
          derived: true,
          live: (row) => row.live,
          value: (row) => decimals(row.mean),
        },
        {
          label: "Desviación",
          derived: true,
          live: (row) => row.live,
          value: (row) => decimals(row.stdDev),
        },
        {
          label: "Desbordes",
          live: (row) => row.live,
          value: (row) => number(row.overflow),
        },
        {
          label: "Máximo",
          live: (row) => row.live,
          value: (row) => number(row.max),
        },
      ]}
    />
  );
}
