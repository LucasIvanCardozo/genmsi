//! When the picture of the graph is framed again.
//!
//! The picture is framed when a model arrives and when the reader asks for
//! it: the *Reacomodar* button and the *Encuadrar* button ask for it. A
//! change of the model does **not** frame it again: the picture would jump
//! away from under the reader while he moves a block, and the zoom would go
//! back to the smallest one, which is what the reader complained about.

/** The state the framing keeps between one picture and the next. */
export interface FrameState {
  /** The model the picture was framed for. */
  readonly framed: string | null;
  /** The request the picture answered last. */
  readonly fitted: number;
}

/** What to do with the picture of the graph. */
export interface FrameDecision {
  /** Whether the picture has to be framed now. */
  readonly fit: boolean;
  /** The state the framing keeps after the decision. */
  readonly state: FrameState;
}

/**
 * Whether the picture of the graph has to be framed again.
 *
 * @param state the state the framing keeps
 * @param key the model that is on the canvas
 * @param request the counter that asks for the picture to be framed
 */
export function decideFrame(
  state: FrameState,
  key: string,
  request: number,
): FrameDecision {
  if (state.framed !== key) {
    // A model arrived: the picture is framed for it, whole.
    return { fit: true, state: { framed: key, fitted: request } };
  }
  if (request !== state.fitted) {
    // The reader asked for the picture to be framed again.
    return { fit: true, state: { framed: state.framed, fitted: request } };
  }
  // The model changed but nobody asked: the picture stays where the reader
  // left it, at the zoom he left it.
  return { fit: false, state };
}
