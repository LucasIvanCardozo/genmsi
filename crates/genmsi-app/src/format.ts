/**
 * The numbers of the app, the way the original shows them.
 *
 * The report of the CLI prints its numbers with a comma for the decimals, so
 * a number read in the app and the same number read in the CLI say the same
 * thing. Every view that shows a number uses this one.
 */

/** A number the way the original's report shows it: comma decimals. */
export function number(value: number): string {
  if (!Number.isFinite(value)) {
    return "0,000";
  }
  if (Math.abs(value - Math.round(value)) < 1e-9 && Math.abs(value) < 1e12) {
    return String(Math.round(value));
  }
  return value.toFixed(3).replace(".", ",");
}

/**
 * A duration as a literal chronometer: `h:mm:ss`.
 *
 * The report of the CLI prints its times as plain seconds, which is what a
 * table of numbers needs. A clock that the reader watches needs the shape of
 * a clock: the monitor shows the time of the run as `0:39:59`, not as
 * `2399,673`, and the same shape is used for the elapsed wall clock.
 */
export function chronometer(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) {
    return "0:00:00";
  }
  const total = Math.floor(seconds);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const rest = total % 60;
  const pad = (value: number) => value.toString().padStart(2, "0");
  return `${hours}:${pad(minutes)}:${pad(rest)}`;
}
