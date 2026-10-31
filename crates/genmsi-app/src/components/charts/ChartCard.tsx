/**
 * The card the charts live in.
 *
 * Every chart of the section is a card with the same shape: a title, a
 * sentence that says what the chart shows, a badge that says where the
 * numbers come from, the chart itself, and a footnote that says how the
 * numbers were made.
 *
 * The badge is a **state** of the vocabulary of the section (`states.ts`):
 * the card says where its numbers come from with the same words and the
 * same colors that the legend explains, and no card writes a badge of its
 * own.
 */
import { FIGURE_STATES, badgeClassOf, type FigureState } from "./states.ts";

export default function ChartCard({
  title,
  subtitle,
  badge,
  badgeLabel,
  footnote,
  children,
}: {
  /** The title of the card. */
  title: string;
  /** The sentence that says what the chart shows. */
  subtitle?: string;
  /** The state the numbers of the card are in. */
  badge?: FigureState;
  /**
   * The word of the badge, when the card mixes states and cannot say one.
   *
   * The color is still the one of the state: the word says what the card
   * holds and the color says how to read it.
   */
  badgeLabel?: string;
  /** The sentence that says how the numbers were made. */
  footnote?: string;
  /** The chart. */
  children: React.ReactNode;
}) {
  return (
    <section className="rounded-lg border border-slate-200 bg-white p-4">
      <header className="mb-2 flex flex-wrap items-start justify-between gap-2">
        <div className="min-w-0">
          <h3 className="text-sm font-semibold text-slate-800">{title}</h3>
          {subtitle !== undefined && (
            <p className="mt-0.5 text-[11px] leading-snug text-slate-500">{subtitle}</p>
          )}
        </div>
        {badge !== undefined && (
          <span
            className={`shrink-0 rounded px-1.5 py-0.5 text-[10px] font-semibold ${badgeClassOf(badge)}`}
          >
            {badgeLabel ?? FIGURE_STATES[badge].label}
          </span>
        )}
      </header>
      {children}
      {footnote !== undefined && (
        <p className="mt-2 text-[11px] leading-snug text-slate-400">{footnote}</p>
      )}
    </section>
  );
}
