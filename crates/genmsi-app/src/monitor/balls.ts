/**
 * The balls that cross the links of the monitor's canvas.
 *
 * A ball is one transaction moving from one block of the model to another.
 * The component is deliberately dumb: it receives the moves, the clock and the
 * place of every block, and it draws the ones that are on screen. It holds no
 * animation state of its own, so the reader can scrub the clock and the
 * picture is always the picture of that instant.
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

/** The place of a block of the model, in the canvas' units. */
export interface Place {
  /** The middle of the block. */
  x: number;
  /** The middle of the block. */
  y: number;
}

/** One move of a transaction between two blocks of the model. */
export interface Move {
  /** The block the instance came from. */
  from: number;
  /** The block the instance went to. */
  to: number;
  /** The time of the run at which the instance reached its destination. */
  time: number;
}

/** One ball on its way. */
export interface Ball {
  /** The middle of the block it comes from. */
  from: Place;
  /** The middle of the block it goes to. */
  to: Place;
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
 * The balls that are on screen at `time`.
 *
 * A ball is on screen while it flies: it left `flight` before the instance
 * reached its destination, so the moves that matter are the ones of the last
 * `flight` seconds of the run. The search finds where to start and the walk
 * goes backwards from there, so the cost is the number of balls and not the
 * number of moves.
 */
export function ballsAt(
  moves: Move[],
  times: number[],
  time: number,
  flight: number,
  placeOf: (block: number) => Place | undefined,
): Ball[] {
  const balls: Ball[] = [];
  if (moves.length === 0) {
    return balls;
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
    const from = placeOf(move.from);
    const to = placeOf(move.to);
    if (from === undefined || to === undefined) {
      continue;
    }
    const progress = flight <= 0 ? 1 : (time - (move.time - flight)) / flight;
    balls.push({
      from,
      to,
      progress: progress < 0 ? 0 : progress > 1 ? 1 : progress,
    });
  }
  return balls;
}
