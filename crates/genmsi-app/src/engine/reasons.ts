/**
 * The reasons the engine gives when it refuses to run a model.
 *
 * The engine writes in English and the app in Spanish: the reason is a fact
 * about the model, and the app has to say it in its own words and, above
 * all, say what the reader has to do about it.
 *
 * The table is open: a new reason of the engine is a new entry here, and a
 * reason the app does not know is not a broken app — it is the plain
 * failure, with the engine's own message, which is never hidden.
 */

/** Why the engine refused to run the model. */
export type RunReason = "never-finishes" | "unknown";

/** One reason the app knows, and what it tells the reader about it. */
interface ReasonRule {
  /** The name of the reason. */
  reason: RunReason;
  /**
   * The marks of the engine's message that name this reason.
   *
   * They are short and distinctive on purpose: the message of the engine
   * can grow —it can name the block, or add a hint— and the reason has to
   * survive that.
   */
  marks: readonly string[];
  /** What the reader has to do about it, in the words of the app. */
  advice: string;
}

/** The reasons the app knows. */
const REASONS: readonly ReasonRule[] = [
  {
    reason: "never-finishes",
    marks: [
      "would never finish",
      "no terminate block with operand a and no clock block",
    ],
    advice:
      "Este modelo no puede terminar: no tiene un bloque «Reloj» (Clock) ni un " +
      "«Terminar» (Terminate) con su operando A, y sin una de las dos la corrida " +
      "no tendría fin. El motor lo rechaza, así que para simularlo hace falta " +
      "agregar un bloque «Reloj» con su «Frecuencia», o poner el operando A " +
      "(«Unidades») del bloque «Terminar». El campo «Hasta» de la barra es el " +
      "tope de la corrida (3600 por defecto) y acota las corridas que sí terminan.",
  },
];

/**
 * The reason the engine's message names, if the app knows it.
 *
 * A message that names no known reason is the reason `unknown`: the app
 * does not invent a reason, it shows the failure of the engine as it is.
 */
export function reasonOf(message: string): RunReason {
  const text = message.toLowerCase();
  for (const rule of REASONS) {
    if (rule.marks.some((mark) => text.includes(mark))) {
      return rule.reason;
    }
  }
  return "unknown";
}

/** What the app tells the reader about a reason, if it knows it. */
export function adviceOf(reason: RunReason): string | null {
  return REASONS.find((rule) => rule.reason === reason)?.advice ?? null;
}

/** What the app tells the reader about the failure the engine reported. */
export function adviceFor(message: string): string | null {
  return adviceOf(reasonOf(message));
}

/**
 * How a run ended, in the words of the app.
 *
 * The engine names the reason in English (`terminated`, `no_events`,
 * `time_limit`, `cancelled`) and the report shows it raw. A reason the app
 * does not know is shown as it comes: the app does not hide what it does
 * not understand.
 */
const END_REASONS: Readonly<Record<string, string>> = {
  terminated: "terminada",
  no_events: "sin eventos: la cadena de eventos se agotó",
  time_limit: "por el tope de tiempo",
  cancelled: "cancelada",
};

/** How a run ended, in the words of the app. */
export function endReasonText(reason: string): string {
  return END_REASONS[reason] ?? reason;
}
