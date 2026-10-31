/**
 * One number of the run, in a card.
 *
 * The card says which state the number is in, and the state is the one of
 * the vocabulary of the section (`states.ts`): the figure of the instant is
 * painted in indigo, a derived one in amber with its asterisk, a published
 * one in slate, and a figure the replay has not reached yet says "todavía
 * no" with the mark of an empty value — it is not a zero that looks
 * published.
 */
import { DERIVED_MARK, FIGURE_STATES, type FigureState } from "./states.ts";

export default function StatCard({
  label,
  value,
  note,
  state,
}: {
  /** The name of the number. */
  label: string;
  /** The number, already written the way the app writes them. */
  value: React.ReactNode;
  /** The sentence that says what the number is. */
  note?: string;
  /** The state the number is in. */
  state: FigureState;
}) {
  const rule = FIGURE_STATES[state];
  const unknown = state === "unknown";
  return (
    <div className="rounded-lg border border-slate-200 bg-white px-3.5 py-2.5">
      <p className="text-[10px] font-semibold uppercase tracking-wider text-slate-400">
        {label}
        {state === "derived" ? ` ${DERIVED_MARK}` : ""}
      </p>
      <p
        className={[
          "mt-0.5 font-mono text-lg font-semibold leading-tight",
          rule.textClass,
        ].join(" ")}
      >
        {unknown ? (
          <span className="inline-block rounded border border-dashed border-slate-400 px-1 text-sm font-normal text-slate-300">
            {rule.label}
          </span>
        ) : (
          value
        )}
      </p>
      {note !== undefined && (
        <p className="mt-0.5 text-[11px] leading-snug text-slate-500">{note}</p>
      )}
    </div>
  );
}
