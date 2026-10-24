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
 * A ball travels **with the transaction**: it leaves the block the
 * transaction came from when the transaction enters it, and it arrives when
 * the transaction leaves it. The flight of a move is the time the
 * transaction spent in that block, so the ball crosses the link in the time
 * the transaction was really there.
 *
 * A transaction **born** in that block was not there at all: it appears and
 * leaves in the same instant. Its ball leaves the block the whole flight
 * before the instance reaches the next one, so every ball crosses in the
 * same time and the illusion holds: when the ball arrives, the instance
 * passes.
 *
 * The flight is bounded on both sides: a transaction that passes through a
 * block in no time still shows a ball (`MIN_FLIGHT`), and a transaction that
 * waits an hour in a queue does not make the ball crawl (`FLIGHT_SECONDS`).
 *
 * The ball arrives at the instant of the move: the flight is the time
 * *before* the arrival, not after it. So the ball of a move is on screen
 * while the clock is between `move.time - flight` and `move.time`, and the
 * picture says what the run says.
 */

/** The shortest a ball flies, in seconds of the run: a ball is always seen. */
export const MIN_FLIGHT = 0.3;

/** The longest a ball flies, in seconds of the run: a ball never crawls. */
export const FLIGHT_SECONDS = 2;

/** The radius of a ball, in the canvas' own units. */
export const BALL_RADIUS = 5;

/** The share of the flight over which the ball shrinks as it arrives. */
export const FADE_SHARE = 0.2;

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
  /** The seconds the transaction spent in the block it came from. */
  flight: number;
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
  /** The seconds its ball flies, within the bounds of the module. */
  flight: number;
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
 * The index of the first move that happens at or after `time`.
 *
 * The move of a ball that is on the way happens *after* the instant the
 * ball is looked at: the ball arrives with the transaction. This search
 * finds where those moves begin.
 */
export function firstMoveAtOrAfter(times: number[], time: number): number {
  let low = 0;
  let high = times.length;
  while (low < high) {
    const middle = (low + high) >>> 1;
    if (times[middle] < time) {
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
 * A ball is on screen while it flies: it left the block the transaction
 * came from `flight` seconds before the instance reached its destination,
 * so the moves that matter are the ones that happen in the next seconds of
 * the run. The searches find where those moves begin and end, so the cost
 * is the number of balls and not the number of moves.
 *
 * The flight is the one of each move, within the bounds of the module: a
 * ball that would take no time at all still flies `MIN_FLIGHT` seconds, and
 * one that would take an hour flies at most `FLIGHT_SECONDS`. The last
 * argument is the longest a ball may fly: the default is the bound of the
 * module, and a smaller one makes the balls cross the links faster.
 */
export function travellersAt(
  moves: Move[],
  times: number[],
  time: number,
  maxFlight: number = FLIGHT_SECONDS,
): Traveller[] {
  const travellers: Traveller[] = [];
  if (moves.length === 0) {
    return travellers;
  }
  // The longest a ball may fly: the ceiling of every flight, and the window
  // the search looks ahead.
  const ceiling = Math.min(Math.max(maxFlight, MIN_FLIGHT), FLIGHT_SECONDS);
  // The ball of a move that happens at `t` flies while the clock is between
  // `t - flight` and `t`: the move that matters happens in the next
  // `ceiling` seconds of the run, and not before the clock is at it.
  const start = firstMoveAtOrAfter(times, time);
  const end = firstMoveAfter(times, time + ceiling);
  for (let index = start; index < end; index += 1) {
    const move = moves[index];
    const flight = flightOf(move, ceiling);
    const progress = (time - (move.time - flight)) / flight;
    if (progress < 0) {
      // The ball has not left yet: the transaction is still in the block it
      // came from.
      continue;
    }
    travellers.push({
      from: move.from,
      to: move.to,
      xact: move.xact,
      progress: progress > 1 ? 1 : progress,
      flight,
    });
  }
  return travellers;
}

/**
 * The flight of a move, within the bounds of the module.
 *
 * The flight of the move is the time the transaction spent in the block it
 * came from: the ball leaves the block that long before the instance reaches
 * the next one, so the reader sees the transaction travel. A transaction
 * born in that block has no time in it, and its move carries the whole
 * flight (`FLIGHT_SECONDS`), so every ball crosses in the same time. A
 * transaction that passed through a block in no time still shows a ball
 * (`MIN_FLIGHT`).
 */
function flightOf(move: Move, ceiling: number): number {
  const raw = Number.isFinite(move.flight) ? move.flight : 0;
  if (raw < MIN_FLIGHT) {
    return MIN_FLIGHT;
  }
  if (raw > ceiling) {
    return ceiling;
  }
  return raw;
}

/**
 * How much of the ball's size is left at a given progress.
 *
 * The ball shrinks over the last fifth of its flight, so it vanishes as it
 * reaches the block instead of sitting on its edge. At the start it is
 * whole: the reader must see the ball leave the block the transaction came
 * from.
 */
export function ballFade(progress: number): number {
  const at = progress < 0 ? 0 : progress > 1 ? 1 : progress;
  if (at <= 1 - FADE_SHARE) {
    return 1;
  }
  return (1 - at) / FADE_SHARE;
}
