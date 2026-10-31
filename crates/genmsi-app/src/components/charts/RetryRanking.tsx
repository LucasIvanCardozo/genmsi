/**
 * The waits of every block: where the model is contended.
 *
 * A wait is an attempt that could not enter the block: it was busy, or
 * there were no units. It is not an error — it is the signal of the neck
 * of the bottle.
 *
 * The number is **the waits of the trace**, not the retries the engine
 * counts: the engine only counts the seizure that was rejected by
 * priority, and the trace counts every wait. The card names it for what
 * it counts.
 */
import { number } from "../../format.ts";
import BarList, { type BarItem } from "./BarList.tsx";
import { blockNameOf, type BlockRow } from "./blocks.ts";

/** How many blocks the ranking shows. */
const TOP = 8;

export default function RetryRanking({
  rows,
}: {
  /** The figures of every block of the model. */
  rows: BlockRow[];
}) {
  const withWaits = rows
    .filter((row) => row.retries > 0)
    .sort((a, b) => b.retries - a.retries);
  const biggest = Math.max(...withWaits.map((row) => row.retries), 0);
  const items: BarItem[] = withWaits.slice(0, TOP).map((row) => ({
    name: blockNameOf(row),
    value: number(row.retries),
    share: biggest > 0 ? row.retries / biggest : 0,
    state: row.live ? "instant" : "unknown",
  }));

  return (
    <div>
      <BarList
        items={items}
        color="bg-amber-600"
        empty="Ningún bloque esperó: nada del modelo se rechazó."
      />
      {withWaits.length > TOP && (
        <p className="mt-2 text-[11px] text-slate-400">
          Y {withWaits.length - TOP} bloque(s) más, con menos esperas.
        </p>
      )}
    </div>
  );
}
