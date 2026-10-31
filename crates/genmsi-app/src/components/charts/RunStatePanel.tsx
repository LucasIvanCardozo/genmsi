/**
 * The state of the run, in the panel that does not scroll away.
 *
 * The panel gathers what describes the run next to the charts that describe
 * the instant: what the run is, where the clock is, why it ended, what
 * governs the model, what the model had stored and how the figures of the
 * section are to be read.
 *
 * The numbers come from two places and each one says which: the report of
 * the engine (the seed, the end of the run, the transactions that were
 * waiting for units when it ended) and the instant of the reader (the
 * switches and the savevalues). The colors are the ones of the vocabulary
 * of the states (`states.ts`): no figure of the panel invents its own.
 */
import { useMemo } from "react";
import { decimals, number } from "../../format.ts";
import type { Instant } from "../../monitor/stats.ts";
import type { ReportDocument } from "../../types";
import StateLegend from "./StateLegend.tsx";
import { runStateOf } from "./runState.ts";
import { FIGURE_STATES } from "./states.ts";

/** The word the panel says when the instant is not known. */
const NO_DATA = "Sin datos.";

/** A row of the panel: a name and its figure. */
function Row({
  label,
  value,
  valueClass,
}: {
  /** The name of the thing. */
  label: string;
  /** The figure. */
  value: React.ReactNode;
  /** The color of the figure. */
  valueClass?: string;
}) {
  return (
    <li className="flex items-baseline justify-between gap-3 border-b border-slate-100 py-1.5 text-[11.5px] text-slate-700 last:border-b-0">
      <span>{label}</span>
      <span className={`text-right ${valueClass ?? "font-mono text-slate-800"}`}>
        {value}
      </span>
    </li>
  );
}

/** The heading of a block of the panel. */
function Heading({ children }: { children: React.ReactNode }) {
  return (
    <h3 className="text-[10px] font-bold uppercase tracking-wider text-slate-500">
      {children}
    </h3>
  );
}

export default function RunStatePanel({
  report,
  instant,
  generated,
}: {
  /** The report of the run. */
  report: ReportDocument;
  /** The instant the reader is at: the clock and the figures of it. */
  instant: Instant;
  /** How many transactions were created up to the clock. */
  generated: number;
}) {
  const state = useMemo(
    () => runStateOf({ report, instant, generated }),
    [report, instant, generated],
  );

  return (
    <section className="rounded-lg border border-slate-200 bg-white p-4">
      <header>
        <h2 className="text-sm font-semibold text-slate-800">Estado de la corrida</h2>
        <p className="mt-0.5 text-[11px] leading-snug text-slate-500">
          La mitad del panel que no cambia con el scroll: qué corrida es,
          dónde está el reloj, qué gobierna el modelo y cómo se leen las
          cifras.
        </p>
      </header>

      {/* ——— la corrida ——— */}
      <section className="mt-4 border-t border-slate-200 pt-3.5">
        <Heading>La corrida</Heading>
        <p className="mt-1.5 text-sm font-semibold leading-snug text-slate-800">
          {state.title}
        </p>
        <ul className="mt-1.5">
          <Row
            label="Semilla"
            value={number(state.seed)}
            valueClass={`font-mono ${FIGURE_STATES.published.textClass}`}
          />
          <Row
            label="Reloj"
            value={`${decimals(state.clock)} s`}
            valueClass={`font-mono ${FIGURE_STATES.instant.textClass}`}
          />
          <Row
            label="Fin de la corrida"
            value={`${decimals(state.duration)} s`}
            valueClass={`font-mono ${FIGURE_STATES.published.textClass}`}
          />
          <Row label="Motivo del fin" value={state.endReason} valueClass="text-slate-700" />
          <Row
            label="Transacciones"
            value={
              <>
                <span className={FIGURE_STATES.instant.textClass}>
                  {number(state.transactions)}
                </span>
                {" / "}
                <span className={FIGURE_STATES.published.textClass}>
                  {number(state.publishedTransactions)}
                </span>
              </>
            }
          />
        </ul>

        {/* The waiting of the end: a published number, and the box is the
            alarm of a run that ended blocked. The alarm is of the error
            family of the app: in this section the amber means "derived". */}
        <div
          className={
            state.waiting > 0
              ? "mt-2.5 rounded-md border border-rose-300 bg-rose-50 px-3 py-2.5"
              : "mt-2.5 rounded-md border border-slate-200 bg-slate-50 px-3 py-2.5"
          }
        >
          <p
            className={
              state.waiting > 0
                ? "text-[10px] font-semibold uppercase tracking-wider text-rose-700"
                : "text-[10px] font-semibold uppercase tracking-wider text-slate-500"
            }
          >
            Esperando al final
          </p>
          <p
            className={`mt-0.5 font-mono text-2xl font-semibold leading-tight ${FIGURE_STATES.published.textClass}`}
          >
            {number(state.waiting)}
          </p>
          <p className="mt-1 text-[11px] leading-snug text-slate-600">
            {waitingText(state.waiting, state.waitingStorages)}
          </p>
        </div>
      </section>

      {/* ——— los switches del modelo ——— */}
      <section className="mt-4 border-t border-slate-200 pt-3.5">
        <Heading>Switches del modelo</Heading>
        {!state.known ? (
          <p className="mt-1.5 text-[11px] text-slate-500">{NO_DATA}</p>
        ) : state.switches.length === 0 ? (
          <p className="mt-1.5 text-[11px] text-slate-500">
            El modelo no tiene switches.
          </p>
        ) : (
          <ul className="mt-1.5">
            {state.switches.map((entry) => (
              <Row
                key={entry.name}
                label={entry.name}
                value={
                  <span
                    className={
                      entry.on === null
                        ? `inline-flex items-center rounded px-2 py-0.5 text-[11px] font-medium ${FIGURE_STATES.unknown.badgeClass}`
                        : `inline-flex items-center rounded px-2 py-0.5 text-[11px] font-semibold ${
                            entry.on
                              ? FIGURE_STATES.instant.badgeClass
                              : FIGURE_STATES.published.badgeClass
                          }`
                    }
                  >
                    {entry.on === null ? "todavía no" : entry.on ? "encendido" : "apagado"}
                  </span>
                }
                valueClass=""
              />
            ))}
          </ul>
        )}
      </section>

      {/* ——— los valores guardados ——— */}
      <section className="mt-4 border-t border-slate-200 pt-3.5">
        <Heading>Valores guardados</Heading>
        {!state.known ? (
          <p className="mt-1.5 text-[11px] text-slate-500">{NO_DATA}</p>
        ) : state.values.length === 0 ? (
          <p className="mt-1.5 text-[11px] text-slate-500">
            El modelo no guarda valores.
          </p>
        ) : (
          <ul className="mt-1.5">
            {state.values.map((entry) => (
              <Row
                key={entry.name}
                label={entry.name}
                value={
                  entry.value === null ? (
                    <span
                      className={`inline-flex items-center rounded px-2 py-0.5 text-[11px] font-medium ${FIGURE_STATES.unknown.badgeClass}`}
                    >
                      todavía no
                    </span>
                  ) : (
                    <span className={`font-mono ${FIGURE_STATES.instant.textClass}`}>
                      {number(entry.value)}
                    </span>
                  )
                }
                valueClass=""
              />
            ))}
          </ul>
        )}
      </section>

      {/* ——— la leyenda ——— */}
      <section className="mt-4 border-t border-slate-200 pt-3.5">
        <StateLegend />
      </section>
    </section>
  );
}

/**
 * What the panel says about the transactions that were waiting at the end.
 *
 * A run that ended with nobody waiting says so plainly: a zero that looks
 * like a problem is not one.
 */
function waitingText(waiting: number, storages: string[]): string {
  if (waiting <= 0) {
    return "nadie quedó esperando: la corrida terminó sin transacciones en espera.";
  }
  const one = waiting === 1;
  const where =
    storages.length === 1
      ? storages[0]
      : `${storages.slice(0, -1).join(", ")} y ${storages[storages.length - 1]}`;
  return one
    ? `una transacción seguía esperando unidades de ${where} cuando la corrida terminó: terminó bloqueada.`
    : `${number(waiting)} transacciones seguían esperando unidades de ${where} cuando la corrida terminó: terminó bloqueada.`;
}
