/**
 * The numbers of the entities of the run, as the charts need them.
 *
 * A chart shows the **instant the reader is at**: the numbers of the
 * replay, never the numbers the engine published for the whole run. A
 * storage that has not been touched yet shows zero units, not the twenty
 * it will hold at the end of the run — a chart that shows the future at
 * the second zero is a chart that lies.
 *
 * The engine's report is the source of two things only: the **names** of the
 * entities (the model declares them) and the **capacity** of a storage (a
 * property of the model, not of the run). The values come from the replay;
 * the ones the replay does not know yet are zero.
 *
 * At the end of the run the two agree: the replay's fidelity check
 * (`checkFidelity`) proves it on every run, so the charts end where the
 * report says.
 */
import {
  chainDerived,
  facilityDerived,
  queueDerived,
  storageDerived,
} from "../../monitor/derived.ts";
import type { Figures } from "../../monitor/stats.ts";
import type { ReportDocument } from "../../types";

/**
 * The time the numbers were measured over.
 *
 * The numbers of the instant were measured over the time the replay has
 * reached, and the published ones over the whole run: each set of numbers
 * is divided by its own time. Dividing the published area by the clock of
 * the replay would say that a queue held three hundred units in the first
 * fifteen seconds of a run that held four.
 */
export interface Elapsed {
  /** The time of the numbers of the instant. */
  live: number;
  /** The time of the published numbers. */
  published: number;
}

/** The numbers of a queue, for the charts. */
export interface QueueNumbers {
  /** The name of the queue. */
  name: string;
  /** The transactions waiting in it right now. */
  content: number;
  /** The most transactions it ever held. */
  max: number;
  /** How many transactions entered it. */
  entries: number;
  /** How many of them waited no time at all. */
  zeroEntries: number;
  /** The time its transactions waited, added up. */
  totalTime: number;
  /** Its content times the time, added up. */
  area: number;
  /** The mean content: `area / elapsed`. */
  meanContent: number;
  /** The mean wait: `totalTime / entries`. */
  meanWait: number;
  /** The share of the entries that waited no time. */
  zeroShare: number;
  /** Whether the replay knows this queue yet. */
  live: boolean;
}

/** The numbers of a facility, for the charts. */
export interface FacilityNumbers {
  /** The name of the facility. */
  name: string;
  /** How many times it was taken. */
  entries: number;
  /** The time it was held, added up. */
  busyTime: number;
  /** Whether it is held right now. */
  busy: boolean;
  /** The transaction that holds it right now, if it is held. */
  owner: number | null;
  /** How many times its holder was interrupted. */
  inter: number;
  /** How many times a transaction could not take it. */
  retry: number;
  /** The utilization: `busyTime / elapsed`. */
  utilization: number;
  /** The mean service: `busyTime / entries`. */
  meanService: number;
  /** Whether the replay knows this facility yet. */
  live: boolean;
}

/** The numbers of a storage, for the charts. */
export interface StorageNumbers {
  /** The name of the storage. */
  name: string;
  /** Its capacity, in units. */
  capacity: number;
  /** The units in use right now. */
  inUse: number;
  /** The most units it ever held. */
  maxUse: number;
  /** How many times units were taken. */
  entries: number;
  /** Its units times the time, added up. */
  area: number;
  /** How many times units did not fit. */
  retry: number;
  /** The mean utilization: `area / (elapsed * capacity)`. */
  meanUtilization: number;
  /** Whether the replay knows this storage yet. */
  live: boolean;
}

/** The numbers of a chain, for the charts. */
export interface ChainNumbers {
  /** The name of the chain. */
  name: string;
  /** The transactions in it right now. */
  content: number;
  /** The most transactions it ever held. */
  max: number;
  /** How many transactions joined it. */
  entries: number;
  /** The time its transactions were linked, added up. */
  totalTime: number;
  /** Its size times the time, added up. */
  area: number;
  /** The mean size: `area / elapsed`. */
  meanSize: number;
  /** Whether the replay knows this chain yet. */
  live: boolean;
}

/** The names of the entities of the model: the report and the replay. */
function namesOf(
  published: Record<string, unknown>,
  live: Record<string, unknown> | undefined,
): string[] {
  return [
    ...new Set([...Object.keys(published), ...Object.keys(live ?? {})]),
  ].sort((a, b) => a.localeCompare(b));
}

/** The queues of the run, with the numbers of the instant. */
export function queuesOf(
  report: ReportDocument,
  live: Figures | null,
  elapsed: Elapsed,
): QueueNumbers[] {
  return namesOf(report.entities.queues, live?.queues).map((name) => {
    const figure = live?.queues[name];
    const derived = queueDerived(
      {
        entries: figure?.entries ?? 0,
        zeroEntries: figure?.zeroEntries ?? 0,
        totalTime: figure?.totalTime ?? 0,
        area: figure?.area ?? 0,
      },
      elapsed.live,
    );
    return {
      name,
      content: figure?.content ?? 0,
      max: figure?.max ?? 0,
      entries: figure?.entries ?? 0,
      zeroEntries: figure?.zeroEntries ?? 0,
      totalTime: figure?.totalTime ?? 0,
      area: figure?.area ?? 0,
      meanContent: derived.meanContent,
      meanWait: derived.meanWait,
      zeroShare: derived.zeroShare,
      live: figure !== undefined,
    };
  });
}

/** The facilities of the run, with the numbers of the instant. */
export function facilitiesOf(
  report: ReportDocument,
  live: Figures | null,
  elapsed: Elapsed,
): FacilityNumbers[] {
  return namesOf(report.entities.facilities, live?.facilities).map((name) => {
    const figure = live?.facilities[name];
    const derived = facilityDerived(
      { entries: figure?.entries ?? 0, busyTime: figure?.busyTime ?? 0 },
      elapsed.live,
    );
    return {
      name,
      entries: figure?.entries ?? 0,
      busyTime: figure?.busyTime ?? 0,
      busy: figure?.busy ?? false,
      owner: figure?.owner ?? null,
      inter: figure?.inter ?? 0,
      retry: figure?.retry ?? 0,
      utilization: derived.utilization,
      meanService: derived.meanService,
      live: figure !== undefined,
    };
  });
}

/** The storages of the run, with the numbers of the instant. */
export function storagesOf(
  report: ReportDocument,
  live: Figures | null,
  elapsed: Elapsed,
): StorageNumbers[] {
  return namesOf(report.entities.storages, live?.storages).map((name) => {
    const figure = live?.storages[name];
    // The capacity is a property of the model: the report publishes it and
    // the replay does not carry it, so the live figures borrow it.
    const capacity = report.entities.storages[name]?.capacity ?? 0;
    const derived = storageDerived(
      { capacity, area: figure?.area ?? 0 },
      elapsed.live,
    );
    return {
      name,
      capacity,
      inUse: figure?.inUse ?? 0,
      maxUse: figure?.maxUse ?? 0,
      entries: figure?.entries ?? 0,
      area: figure?.area ?? 0,
      retry: figure?.retry ?? 0,
      meanUtilization: derived.meanUtilization,
      live: figure !== undefined,
    };
  });
}

/** The chains of the run, with the numbers of the instant. */
export function chainsOf(
  report: ReportDocument,
  live: Figures | null,
  elapsed: Elapsed,
): ChainNumbers[] {
  return namesOf(report.entities.chains, live?.chains).map((name) => {
    const figure = live?.chains[name];
    const derived = chainDerived({ area: figure?.area ?? 0 }, elapsed.live);
    return {
      name,
      content: figure?.content ?? 0,
      max: figure?.max ?? 0,
      entries: figure?.entries ?? 0,
      totalTime: figure?.totalTime ?? 0,
      area: figure?.area ?? 0,
      meanSize: derived.meanSize,
      live: figure !== undefined,
    };
  });
}
