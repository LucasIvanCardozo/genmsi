/**
 * The styles of the chrome of the app: the header and the toolbar.
 *
 * They live together because they are the same thing seen from two sides:
 * one row that says which model is open, and one row that holds the
 * commands. Both are dense on purpose — a desktop tool gives its space to
 * the work, not to its own furniture.
 */

/** A button with an icon only. */
export const iconButton =
  "flex size-[26px] items-center justify-center rounded-md border border-slate-300 bg-white text-slate-600 transition hover:bg-slate-50 disabled:cursor-not-allowed disabled:text-slate-300 disabled:hover:bg-white";

/** A button with an icon and a label. */
export const textButton =
  "flex h-[26px] items-center gap-1.5 rounded-md border border-slate-300 bg-white px-2 text-[11.5px] font-medium text-slate-700 transition hover:bg-slate-50 disabled:cursor-not-allowed disabled:text-slate-300 disabled:hover:bg-white";

/** The label of a group of commands. */
export const groupLabel =
  "text-[9px] font-semibold uppercase tracking-[0.09em] text-slate-400";

/** The rule between two groups of commands. */
export const groupRule = "h-4 w-px bg-slate-200";

/**
 * A number of the next run.
 *
 * The arrows of the browser are hidden: in a row this dense they eat the
 * width of the seed, which is a ten-digit number that has to be read whole.
 */
export const numberField =
  "h-6 rounded-md border border-slate-300 bg-white px-1.5 text-[11px] font-mono text-slate-800 outline-none [appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none focus:border-slate-500 focus:ring-2 focus:ring-slate-200";
