/**
 * The transactions that went through each block.
 *
 * The ranking says where the work of the model accumulates: the bar that
 * is longest is the block every transaction passes through.
 *
 * Two numbers sit next to each other: the count of the **instant**, in
 * indigo, and the count the **engine published** for the whole run, in
 * slate. A block the replay has not reached yet says "todavía no": it is
 * not a block without transactions.
 *
 * A generator is not a block that receives the transactions: it creates
 * them, and its row says so.
 */
import { number } from "../../format.ts";
import BarList, { type BarItem } from "./BarList.tsx";
import { blockNameOf, blockNoteOf, type BlockRow } from "./blocks.ts";

/** How many blocks the ranking shows. */
const TOP = 8;

export default function BlockRanking({
  rows,
}: {
  /** The figures of every block of the model. */
  rows: BlockRow[];
}) {
  // A block that the run has not reached yet is still worth a row when the
  // engine counted something in it: it says "todavía no" and the published
  // count.
  const seen = rows.filter(
    (row) => row.entries > 0 || (row.published ?? 0) > 0,
  );
  const biggest = Math.max(...seen.map((row) => row.entries), 0);
  const items: BarItem[] = seen.slice(0, TOP).map((row) => ({
    name: blockNameOf(row),
    value: number(row.entries),
    published: row.published === null ? undefined : number(row.published),
    share: biggest > 0 ? row.entries / biggest : 0,
    state: row.live ? "instant" : "unknown",
    note: blockNoteOf(row),
  }));

  return (
    <div>
      <BarList
        items={items}
        color="bg-indigo-500"
        empty="Ningún bloque tuvo transacciones."
      />
      {seen.length > TOP && (
        <p className="mt-2 text-[11px] text-slate-400">
          Y {seen.length - TOP} bloque(s) más, con menos transacciones.
        </p>
      )}
    </div>
  );
}
