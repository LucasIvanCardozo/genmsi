/**
 * A list of bars: the ranking of a family of entities.
 *
 * The bar of every entity is a share of the longest one: the ranking says
 * which one is the biggest, and the numbers say how much.
 */
export interface BarItem {
  /** The name of the entity. */
  name: string;
  /** The number, written the way the app writes it. */
  value: string;
  /** The share of the bar the entity takes, from 0 to 1. */
  share: number;
}

export default function BarList({
  items,
  color,
  empty,
}: {
  /** The entities of the ranking. */
  items: BarItem[];
  /** The color of the bars. */
  color: string;
  /** What to say when there is nothing to show. */
  empty?: string;
}) {
  if (items.length === 0) {
    return (
      <p className="py-4 text-center text-xs text-slate-400">
        {empty ?? "Sin entidades."}
      </p>
    );
  }
  return (
    <ul className="flex flex-col gap-2">
      {items.map((item, index) => (
        <li key={`${item.name} ${index}`} className="flex flex-col gap-1">
          <div className="flex items-baseline justify-between gap-2">
            <span className="truncate text-[11px] text-slate-600">{item.name}</span>
            <span className="font-mono text-[11px] text-slate-800">{item.value}</span>
          </div>
          <div className="h-1.5 w-full overflow-hidden rounded-full bg-slate-100">
            <div
              className={`h-full rounded-full ${color}`}
              style={{ width: `${Math.max(2, item.share * 100)}%` }}
            />
          </div>
        </li>
      ))}
    </ul>
  );
}
