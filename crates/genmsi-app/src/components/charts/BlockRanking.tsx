/**
 * The transactions that went through each block.
 *
 * The ranking says where the work of the model accumulates: the bar that
 * is longest is the block every transaction passes through.
 */
import { number } from "../../format.ts";
import BarList, { type BarItem } from "./BarList.tsx";
import type { BlockRow } from "./blocks.ts";

/** How many blocks the ranking shows. */
const TOP = 8;

export default function BlockRanking({
  rows,
}: {
  /** The figures of every block of the model. */
  rows: BlockRow[];
}) {
  const withEntries = rows.filter((row) => row.entries > 0);
  const biggest = Math.max(...withEntries.map((row) => row.entries), 0);
  const items: BarItem[] = withEntries.slice(0, TOP).map((row) => ({
    name: row.name,
    value: number(row.entries),
    share: biggest > 0 ? row.entries / biggest : 0,
  }));

  return (
    <div>
      <BarList
        items={items}
        color="bg-indigo-500"
        empty="Ningún bloque tuvo transacciones."
      />
      {withEntries.length > TOP && (
        <p className="mt-2 text-[11px] text-slate-400">
          Y {withEntries.length - TOP} bloque(s) más, con menos transacciones.
        </p>
      )}
    </div>
  );
}
