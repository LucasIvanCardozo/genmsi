/**
 * The retries of every block: where the model is contended.
 *
 * A retry is an attempt that could not enter the block: it was busy, or
 * there were no units. It is not an error — it is the signal of the neck
 * of the bottle.
 */
import { number } from "../../format.ts";
import BarList, { type BarItem } from "./BarList.tsx";
import type { BlockRow } from "./blocks.ts";

/** How many blocks the ranking shows. */
const TOP = 8;

export default function RetryRanking({
  rows,
}: {
  /** The figures of every block of the model. */
  rows: BlockRow[];
}) {
  const withRetries = rows
    .filter((row) => row.retries > 0)
    .sort((a, b) => b.retries - a.retries);
  const biggest = Math.max(...withRetries.map((row) => row.retries), 0);
  const items: BarItem[] = withRetries.slice(0, TOP).map((row) => ({
    name: row.name,
    value: number(row.retries),
    share: biggest > 0 ? row.retries / biggest : 0,
  }));

  return (
    <div>
      <BarList
        items={items}
        color="bg-amber-600"
        empty="Ningún bloque tuvo reintentos: nada del modelo se rechazó."
      />
      {withRetries.length > TOP && (
        <p className="mt-2 text-[11px] text-slate-400">
          Y {withRetries.length - TOP} bloque(s) más, con menos reintentos.
        </p>
      )}
    </div>
  );
}
