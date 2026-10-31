/**
 * The run in numbers.
 *
 * The card answers the first question a reader asks about a run: how much
 * happened, how much of it is still inside the model and how fast it went.
 *
 * The clock, the end and the identity of the run are not here: they are in
 * the panel of the state of the run, which does not scroll away. What is
 * left is a mix of states, and each figure says which one it is: the
 * transactions of the instant, the events the engine counted, the ratios
 * the app derives from the numbers of the engine and the population the
 * live layer published with the clock.
 *
 * The ratios are marked with an asterisk and their formula is in the foot
 * of the card: they are not numbers the engine published.
 */
import { decimals, number } from "../../format.ts";
import type { Instant } from "../../monitor/stats.ts";
import type { ReportDocument } from "../../types";
import type { QueueNumbers } from "./entities.ts";
import { busiestQueueOf } from "./runState.ts";
import { FIGURE_STATES, type FigureState } from "./states.ts";

/**
 * The foot of the card: what the asterisks mean.
 *
 * The divisor of the ratios is the clock of the instant —the time the
 * reader is at— and not the length of the run: a mean content that divides
 * by the whole run says what the queue will hold, not what it holds.
 */
export const RUN_SUMMARY_FOOTNOTE =
  "Caudal * = transacciones ÷ tiempo hasta el reloj · contenido medio * = área ÷ tiempo hasta el reloj · espera media * = área ÷ entradas (la del motor).";

/**
 * One figure of the card.
 *
 * It follows the rules of the `StatCard` of the section (the label, the
 * figure in the mono and the note under it) and takes the color of its
 * state from the vocabulary of the states: the `StatCard` only knew two
 * colors, and the section now has four.
 */
function Stat({
  label,
  value,
  note,
  state,
}: {
  /** The name of the figure. */
  label: string;
  /** The figure, already written the way the app writes them. */
  value: React.ReactNode;
  /** The sentence that says what the figure is. */
  note?: string;
  /** The state the figure is in. */
  state: FigureState;
}) {
  return (
    <div className="rounded-lg border border-slate-200 bg-white px-3.5 py-2.5">
      <p className="text-[10px] font-semibold uppercase tracking-wider text-slate-400">
        {label}
      </p>
      <p
        className={[
          "mt-0.5 font-mono text-lg font-semibold leading-tight",
          FIGURE_STATES[state].textClass,
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

/** A figure with its unit written smaller, the way the artboard writes it. */
function measure(value: string, unit: string) {
  return (
    <>
      {value}
      <span className="text-xs font-normal"> {unit}</span>
    </>
  );
}

export default function RunSummary({
  report,
  instant,
  generated,
  queues,
}: {
  /** The report of the run. */
  report: ReportDocument;
  /**
   * The instant the reader is at: the clock, the figures of that second
   * and the time the figures were measured over.
   */
  instant: Instant;
  /** How many transactions were created up to the clock. */
  generated: number;
  /** The queues of the run, with the numbers of the instant. */
  queues: QueueNumbers[];
}) {
  // The time the figures of the instant were measured over: the clock, not
  // the length of the run.
  const elapsed = instant.live;
  const known = instant.known;
  // The queue the card speaks of: the one that holds the most.
  const queue = busiestQueueOf(queues);

  // The ratios are only numbers when the clock and the figures are of the
  // same instant: without that pair there is nothing to divide.
  const throughput = known && elapsed > 0 ? generated / elapsed : null;
  const meanContent = known && queue !== null ? queue.meanContent : null;
  const meanWait = known && queue !== null ? queue.engineMeanWait : null;

  return (
    <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-6">
      <Stat
        label="Transacciones"
        value={number(generated)}
        note={`de ${number(report.transactions)} en la corrida`}
        state="instant"
      />
      <Stat
        label="Eventos"
        value={number(report.events)}
        note="los cuenta el motor"
        state="published"
      />
      <Stat
        label="Caudal *"
        value={
          throughput === null
            ? "Sin datos."
            : measure(decimals(throughput, 2), "trans/s")
        }
        note={
          elapsed > 0
            ? `transacciones ÷ ${decimals(elapsed)} s`
            : "transacciones ÷ tiempo hasta el reloj"
        }
        state="derived"
      />
      <Stat
        label="Población"
        value={
          instant.population !== null ? number(instant.population) : "Sin datos."
        }
        note="dentro del modelo ahora"
        state={instant.population !== null ? "instant" : "unknown"}
      />
      <Stat
        label="Contenido medio de las colas *"
        value={meanContent === null ? "Sin datos." : measure(decimals(meanContent), "u")}
        note={
          queue === null
            ? "el modelo no tiene colas"
            : `${queue.name}, área ÷ ${decimals(elapsed)} s`
        }
        state="derived"
      />
      <Stat
        label="Espera media *"
        value={meanWait === null ? "Sin datos." : measure(decimals(meanWait), "s")}
        note={queue === null ? "el modelo no tiene colas" : `${queue.name}, área ÷ entradas`}
        state="derived"
      />
    </div>
  );
}
