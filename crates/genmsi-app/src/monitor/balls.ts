/**
 * The balls that cross the links of the model.
 *
 * A ball is one transaction moving from one block of the model to another.
 * The link that joins the two blocks is the one that draws it: the ball
 * travels over the very curve the reader sees, and the graph knows where
 * that curve is.
 *
 * This module only says *which* balls are on the way and *how far* along
 * they are. Where they are drawn is the link's business.
 *
 * ## The life of a ball
 *
 * A move is stated by the run at the instant the instance reaches its
 * destination. The ball leaves `FLIGHT_SECONDS` of the run before that and
 * lands exactly on it: the ball arrives at the block at the very moment the
 * instance does, so the picture says what the run says.
 *
 * The flight is a fixed stretch of the **run** (`FLIGHT_SECONDS`), so the
 * faster the reader runs the reproduction, the quicker the ball crosses: two
 * seconds of screen at 1x, one at 2x, and so on.
 */

/** How long a ball crosses, in seconds of the run. */
export const FLIGHT_SECONDS = 2;

/** The radius of a ball, in the canvas' own units. */
export const BALL_RADIUS = 5;

/** One move of a transaction between two blocks of the model. */
export interface Move {
  /** The block the instance came from. */
  from: number;
  /** The block the instance went to. */
  to: number;
  /** The time of the run at which the instance reached its destination. */
  time: number;
  /** The transaction that moved. */
  xact: number;
}

/** One transaction on its way from one block to another. */
export interface Traveller {
  /** The block the transaction comes from. */
  from: number;
  /** The block the transaction goes to. */
  to: number;
  /** The transaction that travels. */
  xact: number;
  /** How far along it is, from 0 to 1. */
  progress: number;
}

/**
 * The index of the first move that happened after `time`.
 *
 * The moves are ordered by the time they happened, so the answer is a binary
 * search: a run of 33 thousand moves is not walked to find the ones of a
 * moment.
 */
export function firstMoveAfter(times: number[], time: number): number {
  let low = 0;
  let high = times.length;
  while (low < high) {
    const middle = (low + high) >>> 1;
    if (times[middle] <= time) {
      low = middle + 1;
    } else {
      high = middle;
    }
  }
  return low;
}

/**
 * The transactions that are on their way at `time`.
 *
 * A ball is on screen while it flies: it left `flight` before the instance
 * reached its destination, so the moves that matter are the ones of the last
 * `flight` seconds of the run. The search finds where to start and the walk
 * goes backwards from there, so the cost is the number of balls and not the
 * number of moves.
 */
export function travellersAt(
  moves: Move[],
  times: number[],
  time: number,
  flight: number,
): Traveller[] {
  const travellers: Traveller[] = [];
  if (moves.length === 0) {
    return travellers;
  }
  // The moves whose ball is on screen: the ball of a move that happens at `t`
  // flies while the clock is between `t - flight` and `t`, so the moves of
  // the next `flight` seconds are the ones that have not landed yet.
  const end = firstMoveAfter(times, time + flight);
  const oldest = time - flight;
  for (let index = end - 1; index >= 0; index -= 1) {
    const move = moves[index];
    if (move.time < oldest) {
      break;
    }
    const progress = flight <= 0 ? 1 : (time - (move.time - flight)) / flight;
    travellers.push({
      from: move.from,
      to: move.to,
      xact: move.xact,
      progress: progress < 0 ? 0 : progress > 1 ? 1 : progress,
    });
  }
  return travellers;
}
