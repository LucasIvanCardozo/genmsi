//! The loop of the proposal, on the side of the person.
//!
//! The agent proposes a model in the staging area of the app and the app
//! adopts it in memory — dirty and undoable, so the person sees it on the
//! canvas without anything being written. The person decides: accept it or
//! discard it, with a note that travels with the rejection.
//!
//! The decisions of this loop are pure: they are in one module, and this
//! file pins them. Four rules are pinned here: the app adopts a proposal
//! that waits only when it is not on the canvas, the band says what the
//! agent proposed, a proposal that is not on the canvas is said to be so
//! (and is not offered to be accepted), and the note of the reader travels
//! with the rejection.

import assert from "node:assert/strict";
import { test } from "node:test";
import {
  noteForRejection,
  proposalBand,
  samePending,
  shouldAdopt,
  verdictText,
  waitsForSave,
  type ProposalBand,
} from "../src/state/proposal.ts";
import type { ProposalBody, ProposalDocument } from "../src/types/protocol.ts";

/**
 * The words of the band.
 *
 * A band that says nothing has no words: the empty text is what the
 * reader would read.
 */
function textOf(band: ProposalBand): string {
  return band.kind === "none" ? "" : band.text;
}

/**
 * A proposal of an agent that waits in the staging area.
 *
 * The document is the one of the core: `genmsi.proposal/1`.
 */
function waiting(id = "1700000000000-ab", summary: string | null = "el bus del banco"): ProposalDocument {
  return {
    schema: "genmsi.proposal/1",
    id,
    model: `/home/lucas/.local/share/genmsi/staging/${id}.gmdl`,
    target: "/home/lucas/modelo.gmdl",
    summary,
    state: "pending",
    note: null,
  };
}

/** The proposal the session holds: the one the person has to decide. */
function held(
  state: ProposalBody["state"] = "pending",
  summary: string | null = "el bus del banco",
): ProposalBody {
  return { id: "1700000000000-ab", state, summary, note: null };
}

test("a proposal that waits and is not on the canvas is adopted", () => {
  assert.equal(
    shouldAdopt({ pending: waiting(), onCanvas: false, adopted: null, dirty: false }),
    true,
    "the proposal that waits has to be put on the canvas",
  );
});

test("a proposal that is already on the canvas is not adopted again", () => {
  assert.equal(
    shouldAdopt({ pending: waiting(), onCanvas: true, adopted: null, dirty: false }),
    false,
    "the app adopted it already: adopting it again would be the loop",
  );
});

test("there is no proposal: there is nothing to adopt", () => {
  assert.equal(
    shouldAdopt({ pending: null, onCanvas: false, adopted: null, dirty: false }),
    false,
    "the app adopted a proposal that does not exist",
  );
});

test("a proposal the person undid is not put on the canvas behind their back", () => {
  assert.equal(
    shouldAdopt({
      pending: waiting(),
      onCanvas: false,
      adopted: "1700000000000-ab",
      dirty: false,
    }),
    false,
    "the app fought the undo of the reader",
  );
  // Another proposal is a different one: it waits to be adopted.
  assert.equal(
    shouldAdopt({
      pending: waiting("200"),
      onCanvas: false,
      adopted: "1700000000000-ab",
      dirty: false,
    }),
    true,
    "the app did not adopt a proposal it had never adopted",
  );
});

test("a reader with changes that are not saved keeps them: the proposal waits", () => {
  assert.equal(
    shouldAdopt({ pending: waiting(), onCanvas: false, adopted: null, dirty: true }),
    false,
    "the app stepped on the work of the person",
  );
});

test("the band says what the agent proposed", () => {
  const band = proposalBand({ proposal: held(), onCanvas: true, pending: waiting() });
  assert.equal(band.kind, "onCanvas", "the band is not the one of the proposal");
  assert.match(
    textOf(band),
    /el bus del banco/,
    "the band does not say what the agent proposed",
  );
  // A proposal without a summary is still said: the band does not go
  // silent.
  const plain = proposalBand({
    proposal: held("pending", null),
    onCanvas: true,
    pending: waiting("1700000000000-ab", null),
  });
  assert.equal(plain.kind, "onCanvas");
  assert.match(
    textOf(plain),
    /El agente propuso un modelo/,
    "the band does not say that a model was proposed",
  );
});

test("a proposal that is not on the canvas is said to be so, and is not offered", () => {
  const band = proposalBand({ proposal: held(), onCanvas: false, pending: waiting() });
  assert.equal(band.kind, "offCanvas", "the band does not say that the proposal is not there");
  assert.match(
    textOf(band),
    /ya no está en el lienzo/,
    "the band does not say that the proposal left the canvas",
  );
  assert.doesNotMatch(
    textOf(band),
    /Aceptar/,
    "the band offers to accept a proposal that is not there",
  );
});

test("a proposal that waits for the save is said to be waiting", () => {
  const band = proposalBand({ proposal: null, onCanvas: false, pending: waiting() });
  assert.equal(band.kind, "waiting", "the band does not say that a proposal waits");
  assert.match(
    textOf(band),
    /esperando/,
    "the band does not say that a proposal waits",
  );
});

test("a proposal that was decided leaves no band", () => {
  assert.equal(
    proposalBand({ proposal: held("accepted"), onCanvas: false, pending: null }).kind,
    "none",
    "the band of a decided proposal is still there",
  );
  assert.equal(
    proposalBand({ proposal: held("rejected"), onCanvas: false, pending: null }).kind,
    "none",
    "the band of a decided proposal is still there",
  );
});

test("the note travels with the rejection", () => {
  assert.equal(noteForRejection("  el bus no cierra  "), "el bus no cierra");
  assert.equal(noteForRejection("   "), null, "a blank note is a note");
  assert.equal(
    verdictText("rejected", noteForRejection("el bus no cierra")),
    "Se descartó la propuesta del agente: «el bus no cierra».",
    "the note does not travel with the rejection",
  );
  assert.equal(
    verdictText("rejected", null),
    "Se descartó la propuesta del agente.",
    "a rejection without a note says something else",
  );
  assert.equal(
    verdictText("accepted", null),
    "Se aceptó la propuesta del agente.",
    "the acceptance says something else",
  );
});

test("the refusal of the engine that waits for the save is known", () => {
  assert.equal(
    waitsForSave("the model has changes that are not saved: the proposal waits until they are saved"),
    true,
    "the app does not know that the proposal waits for the save",
  );
  assert.equal(waitsForSave("the proposal is about another model: it stays pending"), false);
});

test("two answers of the staging area that say the same thing are one answer", () => {
  assert.equal(samePending(waiting(), waiting()), true);
  assert.equal(samePending(waiting(), waiting("200")), false);
  assert.equal(samePending(waiting(), null), false);
  assert.equal(samePending(null, null), true);
});
