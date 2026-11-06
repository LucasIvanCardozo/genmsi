/**
 * The sampler of the live picture: what the charts show, and how often.
 *
 * The replay publishes its picture sixty times a second, and every publish
 * that the charts hear redraws the whole section. A reader cannot see a
 * change that lasts less than a frame, and on a slow machine that work is
 * what makes the app lag: the simulation keeps running at the same speed,
 * but the section redraws only as often as the reader asked for.
 *
 * The sampler keeps the newest picture and gives it away when the refresh
 * rate says so. The picture that waits is never lost: the one that is
 * waiting when its time comes is the one that is shown, and it is the
 * newest one, never an old one.
 *
 * The logic lives here, without React, so that it can be tested: the
 * runner of the app (`node --test tests/*.test.ts`) has no DOM.
 */
export class Sampler<T> {
  /**
   * How often the value is shown, in milliseconds; zero shows every value
   * at once.
   */
  readonly refreshMs: number;
  /** The value that waits to be shown. */
  private pending: T | null = null;
  /** When the last value was shown. */
  private shownAt = Number.NEGATIVE_INFINITY;
  /** The clock the rate is measured with. */
  private readonly now: () => number;

  /**
   * @param refreshMs how often the value is shown, in milliseconds; zero
   *        shows every value at once.
   * @param now the clock the rate is measured with.
   */
  constructor(refreshMs: number, now: () => number = () => Date.now()) {
    this.refreshMs = refreshMs;
    this.now = now;
  }

  /**
   * A new value arrives: it is shown at once when the rate says so, and it
   * waits otherwise.
   *
   * @return the value to show, or null when it has to wait.
   */
  offer(value: T): T | null {
    this.pending = value;
    return this.due() ? this.take() : null;
  }

  /** Whether the value that waits is due. */
  due(): boolean {
    if (this.pending === null) {
      return false;
    }
    return this.now() - this.shownAt >= this.refreshMs;
  }

  /**
   * Take the value that waits, if there is one.
   *
   * The value that is taken is the last one that was offered: the picture
   * of the instant that is shown is the newest one.
   */
  take(): T | null {
    const value = this.pending;
    this.pending = null;
    if (value !== null) {
      this.shownAt = this.now();
    }
    return value;
  }
}
