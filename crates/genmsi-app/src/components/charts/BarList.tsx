/**
 * A list of bars: the ranking of a family of entities.
 *
 * The bar of every entity is a share of the scale of its panel: the
 * ranking says which one is the biggest, and the numbers say how much.
 * The number is painted in the color of its state —the indigo of the
 * instant— and the bar keeps the color of the series.
 *
 * A figure the replay has not reached yet is not a zero: it says "todavía
 * no" with the mark of an empty value.
 */
import { FIGURE_STATES, type FigureState } from "./states.ts";

export interface BarItem {
  /** The name of the entity. */
  name: string;
  /** The number, written the way the app writes it. */
  value: string;
  /** The share of the bar the entity takes, from 0 to 1. */
  share: number;
  /** The state the number is in. */
  state?: FigureState;
  /**
   * A second number next to the first one: the one the engine published
   * for the whole run, next to the one of the instant.
   */
  published?: string;
  /** What the number counts, when it is not the obvious thing. */
  note?: string;
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
      {items.map((item, index) => {
        const state = item.state ?? "instant";
        const rule = FIGURE_STATES[state];
        return (
          <li key={`${item.name} ${index}`} className="flex flex-col gap-1">
            <div className="flex items-baseline justify-between gap-2">
              <span className="truncate text-[11px] text-slate-600">{item.name}</span>
              <span className="font-mono text-[11px]">
                {state === "unknown" ? (
                  <span className="inline-block rounded border border-dashed border-slate-400 px-1 font-sans text-[10px] text-slate-300">
                    {rule.label}
                  </span>
                ) : (
                  <span className={rule.textClass}>{item.value}</span>
                )}
                {item.published !== undefined && (
                  <>
                    <span className="text-slate-300"> / </span>
                    <span className={FIGURE_STATES.published.textClass}>
                      {item.published}
                    </span>
                  </>
                )}
              </span>
            </div>
            {item.note !== undefined && (
              <p className="text-[10px] leading-snug text-slate-400">{item.note}</p>
            )}
            <div className="h-1.5 w-full overflow-hidden rounded-full bg-slate-100">
              <div
                className={`h-full rounded-full ${color}`}
                style={{ width: `${item.share > 0 ? item.share * 100 : 0}%` }}
              />
            </div>
          </li>
        );
      })}
    </ul>
  );
}
