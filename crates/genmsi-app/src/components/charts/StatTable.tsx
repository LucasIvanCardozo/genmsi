/**
 * A table of indicators.
 *
 * The table is the same everywhere: a row of headers and the rows of the
 * entities. Every number says which state it is in, and the states come
 * from the vocabulary of the section (`states.ts`): a derived number says
 * so with an asterisk —and the footnote of the card says what the
 * derivation is— and a number the replay has not reached yet says "todavía
 * no" instead of showing a zero that looks published.
 */
import { FIGURE_STATES, type FigureState } from "./states.ts";

export interface StatColumn<T> {
  /** The name of the column. */
  label: string;
  /** The width of the column, in pixels. */
  width?: number;
  /** The number of the column, written the way the app writes it. */
  value: (row: T) => string;
  /**
   * Whether the number of the column is a ratio of two numbers of the
   * engine: the header carries the asterisk and the footnote of the card
   * carries the formula.
   */
  derived?: boolean;
  /** Whether the name of the entity goes in this column. */
  name?: boolean;
  /**
   * The state of the number of the column: the same one for every row, or
   * one per row when the entity is not always known.
   */
  state?: FigureState | ((row: T) => FigureState);
}

/** The state of the number of a column in a row. */
function stateOf<T>(
  column: StatColumn<T>,
  row: T,
): FigureState {
  if (typeof column.state === "function") {
    return column.state(row);
  }
  return column.state ?? "instant";
}

export default function StatTable<T>({
  columns,
  rows,
}: {
  /** The columns of the table. */
  columns: StatColumn<T>[];
  /** The rows of the table. */
  rows: T[];
}) {
  if (rows.length === 0) {
    return <p className="py-6 text-center text-xs text-slate-400">Sin datos.</p>;
  }
  return (
    <table className="w-full border-collapse text-xs">
      <thead>
        <tr className="border-b border-slate-200 text-left text-slate-500">
          {columns.map((column) => {
            const derived =
              column.derived === true ||
              rows.some((row) => stateOf(column, row) === "derived");
            return (
              <th
                key={column.label}
                className="py-1.5 pr-4 font-medium whitespace-nowrap"
                style={column.width === undefined ? undefined : { width: column.width }}
              >
                {column.label}
                {derived && <span className="text-amber-700"> *</span>}
              </th>
            );
          })}
        </tr>
      </thead>
      <tbody>
        {rows.map((row, index) => (
          <tr key={index} className="border-b border-slate-100">
            {columns.map((column) => {
              if (column.name === true) {
                return (
                  <td
                    key={column.label}
                    className="py-1.5 pr-4 whitespace-nowrap text-slate-800"
                  >
                    {column.value(row)}
                  </td>
                );
              }
              const state = stateOf(column, row);
              const rule = FIGURE_STATES[state];
              return (
                <td
                  key={column.label}
                  className={[
                    "py-1.5 pr-4 whitespace-nowrap font-mono",
                    rule.textClass,
                  ].join(" ")}
                >
                  {state === "unknown" ? (
                    <span className="inline-block rounded border border-dashed border-slate-400 px-1 font-sans text-[10px] font-normal">
                      {rule.label}
                    </span>
                  ) : (
                    column.value(row)
                  )}
                </td>
              );
            })}
          </tr>
        ))}
      </tbody>
    </table>
  );
}
