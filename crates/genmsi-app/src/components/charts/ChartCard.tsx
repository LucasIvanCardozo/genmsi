/**
 * The card the charts live in.
 *
 * Every chart of the section is a card with the same shape: a title, a
 * sentence that says what the chart shows, a badge that says where the
 * numbers come from, the chart itself, and a footnote that says how the
 * numbers were made.
 */
export default function ChartCard({
  title,
  subtitle,
  badge,
  badgeClass,
  footnote,
  children,
}: {
  /** The title of the card. */
  title: string;
  /** The sentence that says what the chart shows. */
  subtitle?: string;
  /** The badge that says where the numbers come from. */
  badge?: string;
  /** The color of the badge. */
  badgeClass?: string;
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
            className={
              badgeClass ??
              "shrink-0 rounded bg-indigo-50 px-1.5 py-0.5 text-[10px] font-semibold text-indigo-700"
            }
          >
            {badge}
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
