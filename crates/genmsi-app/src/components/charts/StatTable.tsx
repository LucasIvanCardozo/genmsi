/**
 * A table of indicators.
 *
 * The table is the same everywhere: a row of headers and the rows of the
 * entities. A column whose number is **derived** says so with an asterisk,
 * and the footnote of the card says what the derivation is.
 */
export interface StatColumn<T> {
  /** The name of the column. */
  label: string;
  /** The width of the column, in pixels. */
  width?: number;
  /** The number of the column, written the way the app writes it. */
  value: (row: T) => string;
  /** Whether the number is derived from the numbers of the engine. */
  derived?: boolean;
  /** Whether the name of the entity goes in this column. */
  name?: boolean;
  /** Whether the number is the one of the instant the reader is at. */
  live?: (row: T) => boolean;
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
          {columns.map((column) => (
            <th
              key={column.label}
              className="py-1.5 pr-4 font-medium whitespace-nowrap"
              style={column.width === undefined ? undefined : { width: column.width }}
            >
              {column.label}
              {column.derived === true && <span className="text-indigo-500"> *</span>}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {rows.map((row, index) => (
          <tr key={index} className="border-b border-slate-100">
            {columns.map((column) => {
              const live = column.live?.(row) === true;
              return (
                <td
                  key={column.label}
                  className={[
                    "py-1.5 pr-4 whitespace-nowrap",
                    column.name === true ? "text-slate-800" : "font-mono text-slate-700",
                    column.derived === true ? "text-indigo-700" : "",
                    live === true ? "text-indigo-700" : "",
                  ].join(" ")}
                >
                  {column.value(row)}
                </td>
              );
            })}
          </tr>
        ))}
      </tbody>
    </table>
  );
}
