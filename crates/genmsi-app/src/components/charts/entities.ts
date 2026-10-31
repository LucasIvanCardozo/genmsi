/**
 * The numbers of the entities of the run, as the charts need them.
 *
 * A chart shows the **instant the reader is at**: the numbers of the
 * replay, never the numbers the engine published for the whole run. A
 * storage that has not been touched yet carries zero units, not the twenty
 * it will hold at the end of the run — a chart that shows the future at
 * the second zero is a chart that lies. The cards do not show that zero:
 * an entity the replay has not reached says "todavía no".
 *
 * The engine's report is the source of three things: the **names** of the
 * entities (the model declares them), the **capacity** of a storage (a
 * property of the model, not of the run) and the **retries** of a facility
 * (the replay never emits one, so only the engine can say how many there
 * were). The other values come from the replay.
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
  /** How many transactions left the queue. */
  departures: number;
  /** The time its transactions waited, added up. */
  totalTime: number;
  /** Its content times the time, added up. */
  area: number;
  /** The mean content: `area / elapsed`. */
  meanContent: number;
  /**
   * The wait of the engine, its `AVE.TIME`: `area / entries`.
   *
   * It is a different number than the mean of the departures, and the
   * section shows both with their own name.
   */
  engineMeanWait: number;
  /**
   * The mean wait of the transactions that left: `totalTime / departures`.
   */
  departureMeanWait: number;
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
  /**
   * The retry the engine published for the whole run, or null when the
   * report does not know the facility.
   *
   * The figure of the instant is always zero: the replay never emits a
   * facility retry. The number the engine counts is the one that is worth
   * showing, and it is a number of the finished run.
   */
  publishedRetry: number | null;
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
  /**
   * The mean time of the engine (`AVE.TIME`): `area / entries`.
   */
  meanTime: number;
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
        departures: figure?.departures ?? 0,
      },
      elapsed.live,
    );
    return {
      name,
      content: figure?.content ?? 0,
      max: figure?.max ?? 0,
      entries: figure?.entries ?? 0,
      zeroEntries: figure?.zeroEntries ?? 0,
      departures: figure?.departures ?? 0,
      totalTime: figure?.totalTime ?? 0,
      area: figure?.area ?? 0,
      meanContent: derived.meanContent,
      engineMeanWait: derived.engineMeanWait,
      departureMeanWait: derived.departureMeanWait,
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
      publishedRetry: report.entities.facilities[name]?.retry ?? null,
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
    const derived = chainDerived(
      { area: figure?.area ?? 0, entries: figure?.entries ?? 0 },
      elapsed.live,
    );
    return {
      name,
      content: figure?.content ?? 0,
      max: figure?.max ?? 0,
      entries: figure?.entries ?? 0,
      totalTime: figure?.totalTime ?? 0,
      area: figure?.area ?? 0,
      meanSize: derived.meanSize,
      meanTime: derived.meanTime,
      live: figure !== undefined,
    };
  });
}

/**
 * The pressure of a resource: how much of its limit it uses.
 *
 * The bars of the card that ranks the resources used to be normalized by
 * the biggest bar of their own panel: a resource at 3 % drew a full bar
 * and one at 98 % drew the same one. Each bar now has an **absolute
 * scale**:
 *
 * - a facility: the share of the time up to the clock it was held;
 * - a queue: the mean content over the maximum the queue ever held;
 * - a storage: the share of its capacity it used on average.
 */
export interface ResourcePressure {
  /** The name of the resource. */
  name: string;
  /** The figure, in its own unit: a share of a hundred or units. */
  value: number;
  /** The share of the scale the bar takes, from 0 to 1. */
  share: number;
  /** Whether the figure is a percentage of a hundred. */
  percent: boolean;
  /** Whether the replay knows this resource yet. */
  live: boolean;
}

/** The pressure of the resources of the run, one family at a time. */
export interface ResourcePressures {
  /** The facilities: the share of the time up to the clock they were held. */
  facilities: ResourcePressure[];
  /** The queues: the mean content over the maximum they reached. */
  queues: ResourcePressure[];
  /** The storages: the share of their capacity they used on average. */
  storages: ResourcePressure[];
}

/**
 * The pressure of every resource of the run.
 *
 * The three families do not share a scale, and the card does not pretend
 * that they do: each one is measured against its own limit.
 */
export function resourcePressuresOf(
  facilities: FacilityNumbers[],
  queues: QueueNumbers[],
  storages: StorageNumbers[],
): ResourcePressures {
  return {
    facilities: facilities.map((facility) => ({
      name: facility.name,
      value: facility.utilization,
      share: facility.live ? facility.utilization : 0,
      percent: true,
      live: facility.live,
    })),
    queues: queues.map((queue) => ({
      name: queue.name,
      value: queue.meanContent,
      share:
        queue.live && queue.max > 0 ? queue.meanContent / queue.max : 0,
      percent: false,
      live: queue.live,
    })),
    storages: storages.map((storage) => ({
      name: storage.name,
      value: storage.meanUtilization,
      share: storage.live ? storage.meanUtilization : 0,
      percent: true,
      live: storage.live,
    })),
  };
}
