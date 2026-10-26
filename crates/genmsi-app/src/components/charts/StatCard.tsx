/**
 * One number of the run, in a card.
 *
 * The card says which number it shows: the live one (the value the replay
 * knows at this instant) is in indigo, the published one (the value the
 * engine published for the finished run) is in slate. The reader must not
 * mistake the end of a run for the state of the run.
 */
export default function StatCard({
  label,
  value,
  note,
  live,
}: {
  /** The name of the number. */
  label: string;
  /** The number, already written the way the app writes them. */
  value: string;
  /** The sentence that says what the number is. */
  note?: string;
  /** Whether the number is the one of the instant the reader is at. */
  live?: boolean;
}) {
  return (
    <div className="rounded-lg border border-slate-200 bg-white px-3.5 py-2.5">
      <p className="text-[10px] font-semibold uppercase tracking-wider text-slate-400">
        {label}
      </p>
      <p
        className={[
          "mt-0.5 font-mono text-lg font-semibold leading-tight",
          live === true ? "text-indigo-700" : "text-slate-800",
        ].join(" ")}
      >
        {value}
      </p>
      {note !== undefined && (
        <p className="mt-0.5 text-[11px] leading-snug text-slate-500">{note}</p>
      )}
    </div>
  );
}
