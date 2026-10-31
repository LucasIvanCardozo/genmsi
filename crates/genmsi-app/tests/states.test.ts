//! The states a figure of the section can be in.
//!
//! The section had three meanings for one color and a fourth state that
//! nobody could name: the violet was "derived", "of the instant" and
//! "origin: replay" at the same time, and a number the replay had not
//! reached yet was painted in slate, the color of "published".
//!
//! This module is the one place where the four states live, with the words
//! of the chosen artboard and the color of each one. A card that shows a
//! figure takes its state from here: no card invents its own.

import assert from "node:assert/strict";
import { test } from "node:test";
import {
  FIGURE_STATES,
  FIGURE_STATE_ORDER,
  type FigureState,
} from "../src/components/charts/states.ts";

test("the section has four states and no more", () => {
  assert.deepEqual(FIGURE_STATE_ORDER, ["instant", "derived", "published", "unknown"]);
  assert.equal(Object.keys(FIGURE_STATES).length, 4);
});

test("the state of the instant is the indigo one", () => {
  const state = FIGURE_STATES.instant;
  assert.equal(state.label, "del instante");
  assert.equal(state.title, "Del instante");
  assert.equal(state.textClass, "text-indigo-700");
  assert.match(state.explanation, /segundo que marca el reloj/);
});

test("the derived state is the amber one, with its asterisk", () => {
  const state = FIGURE_STATES.derived;
  assert.equal(state.label, "derivada *");
  assert.equal(state.title, "Derivada *");
  assert.equal(state.textClass, "text-amber-700");
  assert.match(state.explanation, /ratio/);
});

test("the published state is the slate one", () => {
  const state = FIGURE_STATES.published;
  assert.equal(state.label, "publicada");
  assert.equal(state.title, "Publicada");
  assert.equal(state.textClass, "text-slate-500");
  assert.match(state.explanation, /corrida completa/);
});

test("the state that is not known yet is the pale one, with the mark of an empty value", () => {
  const state = FIGURE_STATES.unknown;
  assert.equal(state.label, "todavía no");
  assert.equal(state.title, "Todavía no");
  assert.equal(state.textClass, "text-slate-300");
  assert.match(state.markClass, /dashed/, "the mark of an empty value");
  assert.match(state.explanation, /no es un cero/);
});

test("every state has a color, a badge and a mark", () => {
  for (const id of FIGURE_STATE_ORDER) {
    const state: FigureState = id;
    assert.ok(state.length > 0, `${state} has a label`);
    assert.ok(FIGURE_STATES[state].textClass.length > 0);
    assert.ok(FIGURE_STATES[state].badgeClass.length > 0);
    assert.ok(FIGURE_STATES[state].markClass.length > 0);
  }
});
