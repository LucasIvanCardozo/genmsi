/**
 * The state of the run, as the panel shows it.
 *
 * The panel gathers everything that describes the run: what it is, where the
 * clock is, what governs the model and what the model had stored. It is
 * the part of the section that does not change with the scroll.
 *
 * The logic lives here, without React, so it can be tested: the runner of
 * the app (`node --test tests/*.test.ts`) has no DOM, and a panel that
 * computes its own numbers could not be proved.
 *
 * Two rules govern the numbers of the panel:
 *
 * - **the clock and the figures are of the same instant**. The instant
 *   comes from the live layer (`useInstant`), and when it is not known no
 *   figure of it is shown: the panel says "Sin datos.";
 * - **what the replay has not reached yet is not a zero**. A switch or a
 *   savevalue the report knows and the replay has not touched says
 *   "todavía no".
 */
import { endReasonText } from "../../engine/reasons.ts";
import type { Instant } from "../../monitor/stats.ts";
import type { ReportDocument } from "../../types";
import type { QueueNumbers } from "./entities.ts";

/** A switch of the model and what the instant knows of it. */
export interface SwitchState {
  /** The name of the switch. */
  name: string;
  /**
   * Whether the switch is set, or null when the replay has not reached it
   * yet: it is not known, not off.
   */
  on: boolean | null;
}

/** A savevalue of the model and what the instant knows of it. */
export interface ValueState {
  /** The name of the savevalue. */
  name: string;
  /**
   * What it holds, or null when the replay has not reached it yet: it is
   * not known, not zero.
   */
  value: number | null;
}

/** The state of the run, as the panel shows it. */
export interface RunState {
  /** The title of the model. */
  title: string;
  /** The seed of the run: the run is only reproducible with it. */
  seed: number;
  /** The clock: the instant the reader is at. */
  clock: number;
  /**
   * Whether the clock and the figures are of the same instant. When they
   * are not, no figure of the instant is shown.
   */
  known: boolean;
  /** How long the run lasted, in seconds. */
  duration: number;
  /** Why the run ended, in the words of the app. */
  endReason: string;
  /** The transactions of the instant. */
  transactions: number;
  /** The transactions of the whole run, published by the engine. */
  publishedTransactions: number;
  /**
   * The transactions that were still waiting for units when the run ended:
   * the run ended blocked.
   */
  waiting: number;
  /** The storages that were holding transactions when the run ended. */
  waitingStorages: string[];
  /** The switches of the model. */
  switches: SwitchState[];
  /** The savevalues of the model. */
  values: ValueState[];
}

/** What the panel needs to know about the run. */
export interface RunStateInput {
  /** The report of the run. */
  report: ReportDocument;
  /** The instant the reader is at: the clock and the figures of it. */
  instant: Instant;
  /** How many transactions were created up to the clock. */
  generated: number;
}

/**
 * The state of the run, from the report and the instant.
 *
 * The report carries what the engine published (the seed, the title, the
 * end of the run and the transactions that were waiting for units when it
 * ended); the instant carries what the replay knows at this second (the
 * switches and the savevalues).
 */
export function runStateOf({
  report,
  instant,
  generated,
}: RunStateInput): RunState {
  const figures = instant.figures;
  const switches: SwitchState[] = figures === null
    ? []
    : namesOf(report.entities.logic, figures.logic).map((name) => ({
        name,
        on: name in figures.logic ? figures.logic[name] : null,
      }));
  const values: ValueState[] = figures === null
    ? []
    : namesOf(report.entities.savevalues, figures.values).map((name) => ({
        name,
        value: name in figures.values ? figures.values[name] : null,
      }));
  return {
    title: report.title,
    seed: report.seed,
    clock: instant.clock,
    known: instant.known,
    duration: report.end_time - report.start_time,
    endReason: endReasonText(report.end_reason),
    transactions: generated,
    publishedTransactions: report.transactions,
    waiting: waitingOf(report),
    waitingStorages: waitingStoragesOf(report),
    switches,
    values,
  };
}

/**
 * The transactions that were waiting for units when the run ended.
 *
 * The `delay` of a storage is the engine's own count of the transactions
 * that were waiting for units it did not have (`stat.rs`), published with
 * the report: it is a number of the finished run, not of the instant.
 */
function waitingOf(report: ReportDocument): number {
  let total = 0;
  for (const storage of Object.values(report.entities.storages ?? {})) {
    total += storage.delay ?? 0;
  }
  return total;
}

/** The storages that were holding transactions when the run ended. */
function waitingStoragesOf(report: ReportDocument): string[] {
  return Object.values(report.entities.storages ?? {})
    .filter((storage) => (storage.delay ?? 0) > 0)
    .map((storage) => storage.name)
    .sort((a, b) => a.localeCompare(b));
}

/**
 * The names of the entities of the model: the published ones and the ones
 * the replay knows, sorted so that the panel always reads the same way.
 */
function namesOf(
  published: Record<string, unknown> | undefined,
  live: Record<string, unknown> | undefined,
): string[] {
  return [
    ...new Set([...Object.keys(published ?? {}), ...Object.keys(live ?? {})]),
  ].sort((a, b) => a.localeCompare(b));
}

/**
 * The queue the summary speaks of: the one that holds the most.
 *
 * The cards of the section speak of one queue at a time, and the summary
 * speaks of the busiest one, so that the number of the card and the number
 * of the table are of the same queue.
 */
export function busiestQueueOf(queues: QueueNumbers[]): QueueNumbers | null {
  let busiest: QueueNumbers | null = null;
  for (const queue of queues) {
    if (
      busiest === null ||
      queue.meanContent > busiest.meanContent ||
      (queue.meanContent === busiest.meanContent && queue.content > busiest.content)
    ) {
      busiest = queue;
    }
  }
  return busiest;
}
