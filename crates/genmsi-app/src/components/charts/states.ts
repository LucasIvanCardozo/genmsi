/**
 * The states a figure of the section can be in.
 *
 * The section had three meanings for one color: the violet was "derived",
 * "of the instant" and "origin: replay" at the same time. Worse, a figure
 * the replay had not reached yet was painted in slate, the color of
 * "published", so a number nobody knew looked like a number the engine had
 * published.
 *
 * This module is the **one place** where the states live: the four of them,
 * the word each one is written with and the color each one is painted in.
 * A card that shows a figure takes its state and its color from here, so
 * no card invents its own.
 *
 * The words are the ones of the chosen artboard
 * (`design/graficos-verdad/03-panel-lateral-estado.html`), textual:
 *
 * | Estado      | Se lee                                                        | Color        |
 * |-------------|---------------------------------------------------------------|--------------|
 * | del instante| medido en el segundo que marca el reloj                     | indigo-700   |
 * | derivada *  | un ratio del motor; la fórmula está en el pie de su tarjeta  | amber-700    |
 * | publicada   | lo que el motor publicó para la corrida completa             | slate-500    |
 * | todavía no  | el replay todavía no la alcanzó: no es un cero, es que no se sabe | slate-300  |
 */

/** The four states a figure of the section can be in. */
export type FigureState = "instant" | "derived" | "published" | "unknown";

/** What a state is called, what it means and how it is painted. */
export interface FigureStateRule {
  /** The name of the state. */
  id: FigureState;
  /**
   * The word the section writes, in lower case: the badge of a card.
   */
  label: string;
  /**
   * The same word at the head of a line: the legend.
   */
  title: string;
  /**
   * What the state means, in one sentence. The legend says it and nothing
   * else says it again.
   */
  explanation: string;
  /** The color of a figure of this state. */
  textClass: string;
  /** The badge that carries this state. */
  badgeClass: string;
  /** The mark of the legend. */
  markClass: string;
  /**
   * The color of the state as a stroke, for the SVG of the charts: the
   * dashed line of a mean is drawn in the color of the derived state, the
   * one of the clock in the color of the instant.
   */
  stroke: string;
}

/**
 * The four states, with their words and their colors.
 *
 * The colors are the ones of the artboard and of the design system: the
 * indigo of the accent (`#4338ca`), the amber of the attention (`#b45309`)
 * and the slates of the neutrals. The section does not paint a figure in
 * a color that is not one of these.
 */
export const FIGURE_STATES: Readonly<Record<FigureState, FigureStateRule>> = {
  instant: {
    id: "instant",
    label: "del instante",
    title: "Del instante",
    explanation: "Medido en el segundo que marca el reloj.",
    textClass: "text-indigo-700",
    badgeClass: "bg-indigo-50 text-indigo-700",
    markClass: "bg-indigo-700",
    stroke: "#4338ca",
  },
  derived: {
    id: "derived",
    label: "derivada *",
    title: "Derivada *",
    explanation: "Un ratio del motor; la fórmula está en el pie de su tarjeta.",
    textClass: "text-amber-700",
    badgeClass: "bg-amber-50 text-amber-700",
    markClass: "bg-amber-700",
    stroke: "#b45309",
  },
  published: {
    id: "published",
    label: "publicada",
    title: "Publicada",
    explanation: "Lo que el motor publicó para la corrida completa.",
    textClass: "text-slate-500",
    badgeClass: "bg-slate-100 text-slate-600",
    markClass: "bg-slate-500",
    stroke: "#64748b",
  },
  unknown: {
    id: "unknown",
    label: "todavía no",
    title: "Todavía no",
    explanation:
      "El replay todavía no la alcanzó: no es un cero, es que no se sabe.",
    textClass: "text-slate-300",
    badgeClass: "border border-dashed border-slate-400 bg-white text-slate-500",
    markClass: "border border-dashed border-slate-500 bg-white",
    stroke: "#cbd5e1",
  },
};

/**
 * The order the states are told in: the ones that are numbers first, and
 * the one that is not a number last.
 */
export const FIGURE_STATE_ORDER: readonly FigureState[] = [
  "instant",
  "derived",
  "published",
  "unknown",
];

/**
 * The mark that says a figure is a ratio of the engine's numbers.
 *
 * It is written next to the name of the figure (in a column header, in the
 * label of a card) and the formula of the ratio is written in the foot of
 * the card that shows it.
 */
export const DERIVED_MARK = "*";

/** The color of a figure of this state. */
export function textClassOf(state: FigureState): string {
  return FIGURE_STATES[state].textClass;
}

/** The badge that carries this state. */
export function badgeClassOf(state: FigureState): string {
  return FIGURE_STATES[state].badgeClass;
}

/** The mark of the legend of this state. */
export function markClassOf(state: FigureState): string {
  return FIGURE_STATES[state].markClass;
}

/** The color of this state as a stroke, for the SVG of the charts. */
export function strokeOf(state: FigureState): string {
  return FIGURE_STATES[state].stroke;
}
