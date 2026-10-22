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
 * faster the reader runs the reproduction, the quicker the ball crosses: 200 ms
 * of screen at 1x, 100 ms at 2x, and so on.
 *
 * After landing, the ball stays as a fading mark for `trail` seconds. The
 * trail is measured in **screen** time, because a trail is about legibility:
 * a mark that lasted 0.2 seconds of the run would be invisible at any speed
 * above 1x.
 */

/** How long a ball crosses, in seconds of the run. */
export const FLIGHT_SECONDS = 0.2;

/** How long a ball stays as a fading mark, in seconds of the screen. */
export const TRAIL_SECONDS = 1.5;

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

/** One ball on its way, or the mark it leaves behind. */
export interface Ball {
  /** The middle of the block it comes from. */
  from: Place;
  /** The middle of the block it goes to. */
  to: Place;
  /** How far along it is, from 0 to 1. */
  progress: number;
  /** How visible it is, from 0 to 1. */
  opacity: number;
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
 * A ball is on screen from the moment it leaves its block until its mark has
 * faded. The moves that matter are the ones of the last `trail` seconds and
 * the ones of the next `flight` seconds, so the search finds both ends and
 * the walk between them is the number of balls, not the number of moves.
 */
export function ballsAt(
  moves: Move[],
  times: number[],
  time: number,
  flight: number,
  trail: number,
  placeOf: (block: number) => Place | undefined,
): Ball[] {
  const balls: Ball[] = [];
  if (moves.length === 0) {
    return balls;
  }
  // The moves that have not landed yet are the ones after the clock, up to the
  // end of the flight.
  const end = firstMoveAfter(times, time + flight);
  // And the ones that landed: their mark is still fading.
  const oldest = time - trail;
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
    if (time <= move.time) {
      // The ball is still flying: it leaves `flight` before the instance
      // reaches its destination.
      const progress = flight <= 0 ? 1 : (time - (move.time - flight)) / flight;
      balls.push({
        from,
        to,
        progress: progress < 0 ? 0 : progress > 1 ? 1 : progress,
        opacity: 1,
      });
    } else {
      // The ball has landed: its mark fades.
      const age = time - move.time;
      const opacity = trail <= 0 ? 0 : 1 - age / trail;
      if (opacity <= 0) {
        continue;
      }
      balls.push({ from, to, progress: 1, opacity });
    }
  }
  return balls;
}
