/**
 * The loop of the proposal: what the app does with the model an agent
 * leaves for the person.
 *
 * The agent writes a model in the staging directory of the app and
 * describes it in the `genmsi.proposal/1` document. The app adopts that
 * model **in memory** — dirty and undoable, so the person sees it on the
 * canvas without anything being written — and the person decides: accept
 * it or discard it, with a note that travels with the rejection.
 *
 * The decisions of this module are **pure**: when the app has to adopt a
 * proposal, what the band of the proposal says, and what travels with a
 * rejection. The store asks this module and does what it says; the module
 * knows nothing of the engine, of React or of the clock, so it can be
 * pinned by a test.
 */

import type { ProposalBody, ProposalDocument } from "../types/protocol";

/**
 * What the band of the proposal says.
 *
 * `none` is the ordinary case: there is no proposal on the table.
 */
export type ProposalBand =
  | { kind: "none" }
  | { kind: "waiting"; id: string; text: string }
  | { kind: "onCanvas"; id: string; text: string }
  | { kind: "offCanvas"; id: string; text: string };

/**
 * Whether the app has to put the proposal that waits on the canvas.
 *
 * The rule is **state-driven**: the app adopts when there is a proposal
 * that waits and the proposal of the session is not on the canvas. It is
 * not a timer and it is not a loop:
 *
 * - a proposal that is already on the canvas is not adopted again;
 * - a proposal the app already put on the canvas is not put back behind
 *   the back of the reader, who may have undone the adoption on purpose;
 * - a reader with changes that are not saved keeps them: the engine
 *   refuses the adoption, so the app does not even ask. The proposal
 *   waits for the save, and the band says so.
 */
export function shouldAdopt(input: {
  /** The proposal that waits in the staging area. */
  pending: ProposalDocument | null;
  /** Whether the proposal of the session is on the canvas. */
  onCanvas: boolean;
  /** The identity of the proposal the app already put on the canvas. */
  adopted: string | null;
  /** Whether the reader has changes that are not saved. */
  dirty: boolean;
}): boolean {
  const { pending, onCanvas, adopted, dirty } = input;
  if (pending === null) {
    // There is no proposal: there is nothing to do.
    return false;
  }
  if (onCanvas) {
    // The proposal is on the canvas already: adopting it again would be
    // the loop.
    return false;
  }
  if (adopted !== null && pending.id === adopted) {
    // The app put this proposal on the canvas and the reader undid it:
    // the app does not fight the reader. The band says that the proposal
    // is not there, and the reader can adopt it again if they want it.
    return false;
  }
  if (dirty) {
    // The work of the person is not stepped on: the proposal waits for
    // the save, and the band says that it waits.
    return false;
  }
  return true;
}

/**
 * What the band of the proposal says.
 *
 * The band is the only place where the proposal is said: it is where the
 * app already asks the reader about the model that does not compile.
 *
 * - `onCanvas`: the proposal is on the canvas and waits for a decision.
 * - `offCanvas`: the proposal is the one of the session but it is not on
 *   the canvas —the reader undid the adoption— so there is nothing to
 *   accept and the band says so.
 * - `waiting`: a proposal waits in the staging area and the app could not
 *   put it on the canvas yet, because the reader has changes that are not
 *   saved.
 */
export function proposalBand(input: {
  /** The proposal the session holds. */
  proposal: ProposalBody | null;
  /** Whether the proposal of the session is on the canvas. */
  onCanvas: boolean;
  /** The proposal that waits in the staging area. */
  pending: ProposalDocument | null;
}): ProposalBand {
  const { proposal, onCanvas, pending } = input;
  if (proposal !== null && proposal !== undefined) {
    if (proposal.state !== "pending") {
      // The person already decided: the verdict is said in the notice of
      // the app, and the band of the proposal is gone.
      return { kind: "none" };
    }
    const summary = summaryOf(proposal.summary);
    if (onCanvas) {
      return {
        kind: "onCanvas",
        id: proposal.id,
        text:
          summary === null
            ? "El agente propuso un modelo. ¿Lo aceptás o lo descartás?"
            : `El agente propuso «${summary}». ¿Lo aceptás o lo descartás?`,
      };
    }
    return {
      kind: "offCanvas",
      id: proposal.id,
      text:
        summary === null
          ? "La propuesta del agente ya no está en el lienzo: la deshaciste. " +
            "Volvé a adoptarla para decidir."
          : `La propuesta «${summary}» del agente ya no está en el lienzo: ` +
            "la deshaciste. Volvé a adoptarla para decidir.",
    };
  }
  if (pending !== null && pending !== undefined) {
    const summary = summaryOf(pending.summary);
    return {
      kind: "waiting",
      id: pending.id,
      text:
        summary === null
          ? "Hay una propuesta del agente esperando: guardá el modelo para verla en el lienzo."
          : `Hay una propuesta del agente esperando («${summary}»): ` +
            "guardá el modelo para verla en el lienzo.",
    };
  }
  return { kind: "none" };
}

/**
 * The note that travels with the rejection.
 *
 * The note is optional: what the reader did not write is no note at all,
 * and the agent reads nothing instead of an empty line.
 */
export function noteForRejection(note: string): string | null {
  const text = note.trim();
  return text === "" ? null : text;
}

/**
 * What the app says when the person decided about the proposal.
 *
 * The note is said back: the reader wrote it for the agent, and seeing it
 * says that it travelled.
 */
export function verdictText(
  state: "accepted" | "rejected",
  note: string | null,
): string {
  if (state === "accepted") {
    return "Se aceptó la propuesta del agente.";
  }
  return note === null
    ? "Se descartó la propuesta del agente."
    : `Se descartó la propuesta del agente: «${note}».`;
}

/**
 * Whether the engine refused the adoption because the reader has changes
 * that are not saved.
 *
 * The engine writes in English and the app in Spanish: the marker of the
 * message is the only thing the two share, and it is what tells the app
 * that the proposal is waiting for the save instead of having failed.
 */
export function waitsForSave(message: string): boolean {
  return message.includes("changes that are not saved");
}

/**
 * Whether two answers of the staging area are the same proposal.
 *
 * The app asks every half second: two answers that say the same thing
 * are one answer, and the app does not redraw for nothing.
 */
export function samePending(
  a: ProposalDocument | null,
  b: ProposalDocument | null,
): boolean {
  if (a === null || a === undefined || b === null || b === undefined) {
    return (a === null || a === undefined) && (b === null || b === undefined);
  }
  return a.id === b.id && a.state === b.state && a.summary === b.summary;
}

/**
 * The summary of a proposal, or nothing when it has none.
 *
 * A summary that is only spaces is no summary: the band says what the
 * agent proposed, and a blank line says nothing.
 */
function summaryOf(summary: string | null | undefined): string | null {
  const text = summary?.trim() ?? "";
  return text === "" ? null : text;
}
