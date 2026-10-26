/**
 * The colors of the series.
 *
 * Every series of a chart has its own color: two queues in the same color
 * are two lines the reader cannot tell apart. The colors are the ones the
 * app already uses (the indigo of the report, the teal of the facilities,
 * the orange of the accumulated curve) and they are dark enough to be read
 * on the white of a card.
 */
export const SERIES_COLORS = [
  "#6366f1",
  "#0d9488",
  "#ea580c",
  "#7c3aed",
  "#e11d48",
  "#0284c7",
  "#d97706",
  "#059669",
  "#9333ea",
  "#db2777",
] as const;

/** The color of a series, by its place in the order. */
export function colorOf(index: number): string {
  return SERIES_COLORS[index % SERIES_COLORS.length];
}
