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
