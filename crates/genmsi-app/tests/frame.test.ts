//! When the picture of the graph is framed again.

import assert from "node:assert/strict";
import { test } from "node:test";
import { decideFrame, type FrameState } from "../src/graph/frame.ts";

/** The state of a picture that was never framed. */
const NONE: FrameState = { framed: null, fitted: 0 };

test("a model that arrives is framed whole", () => {
  const decision = decideFrame(NONE, "/modelo.gmdl|Comedor", 0);
  assert.equal(decision.fit, true);
  assert.equal(decision.state.framed, "/modelo.gmdl|Comedor");
});

test("the reader asks for the picture to be framed", () => {
  const first = decideFrame(NONE, "/modelo.gmdl|Comedor", 0);
  const asked = decideFrame(first.state, "/modelo.gmdl|Comedor", 1);
  assert.equal(asked.fit, true);
  assert.equal(asked.state.fitted, 1);
});

test("a change of the model does not frame the picture again", () => {
  // The reader pressed *Reacomodar*: the picture was framed for the model
  // that was composed.
  const asked = decideFrame(
    { framed: "/modelo.gmdl|Comedor", fitted: 1 },
    "/modelo.gmdl|Comedor",
    1,
  );
  assert.equal(asked.fit, false);
  // The reader moves a block: the model changed, but nobody asked for the
  // picture to be framed. The picture stays where the reader left it, at
  // the zoom he left it.
  const moved = decideFrame(
    { framed: "/modelo.gmdl|Comedor", fitted: 1 },
    "/modelo.gmdl|Comedor",
    1,
  );
  assert.equal(moved.fit, false);
});

test("another model is framed when it arrives", () => {
  const state: FrameState = { framed: "/modelo.gmdl|Comedor", fitted: 1 };
  const decision = decideFrame(state, "/otro.gmdl|Otro", 1);
  assert.equal(decision.fit, true);
  assert.equal(decision.state.framed, "/otro.gmdl|Otro");
});
