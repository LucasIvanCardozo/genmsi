/**
 * The resources that squeeze the run.
 *
 * Three rankings, one per family of resources: the utilization of the
 * facilities, the mean content of the queues and the mean utilization of
 * the storages. Every number is a ratio of the engine's own numbers, and
 * the card says so.
 */
import { decimals } from "../../format.ts";
import type { Figures } from "../../monitor/stats.ts";
import type { ReportDocument } from "../../types";
import BarList, { type BarItem } from "./BarList.tsx";
import { facilitiesOf, queuesOf, storagesOf, type Elapsed } from "./entities.ts";

/** How many resources a panel shows. */
const TOP = 5;

/** A panel of the ranking: a title and its resources. */
function Panel({
  title,
  items,
  color,
}: {
  title: string;
  items: BarItem[];
  color: string;
}) {
  if (items.length === 0) {
    return null;
  }
  const shown = items.slice(0, TOP);
  const more = items.length - shown.length;
  return (
    <div className="min-w-0 flex-1">
      <h4 className="mb-2 text-[11px] font-semibold text-slate-700">{title}</h4>
      <BarList items={shown} color={color} />
      {more > 0 && (
        <p className="mt-1.5 text-[10px] text-slate-400">
          y {more} más, con menos carga
        </p>
      )}
    </div>
  );
}

/** The share of the bar each resource takes: the biggest one fills it. */
function sharesOf(values: number[]): number[] {
  const biggest = Math.max(...values, 0);
  if (biggest <= 0) {
    return values.map(() => 0);
  }
  return values.map((value) => value / biggest);
}

export default function ResourceRanking({
  report,
  live,
  elapsed,
}: {
  /** The report of the run. */
  report: ReportDocument;
  /** The figures of the entities at the instant the reader is at. */
  live: Figures | null;
  /** The time the numbers were measured over. */
  elapsed: Elapsed;
}) {
  const facilities = facilitiesOf(report, live, elapsed);
  const queues = queuesOf(report, live, elapsed);
  const storages = storagesOf(report, live, elapsed);

  const utilizations = facilities.map((facility) => facility.utilization);
  const contents = queues.map((queue) => queue.meanContent);
  const storageUtilizations = storages.map((storage) => storage.meanUtilization);

  const facilityItems: BarItem[] = facilities
    .map((facility, index) => ({
      name: facility.name,
      value: `${Math.round(utilizations[index] * 100)} %`,
      share: sharesOf(utilizations)[index],
    }))
    .sort((a, b) => b.share - a.share);
  const queueItems: BarItem[] = queues
    .map((queue, index) => ({
      name: queue.name,
      value: decimals(contents[index]),
      share: sharesOf(contents)[index],
    }))
    .sort((a, b) => b.share - a.share);
  const storageItems: BarItem[] = storages
    .map((storage, index) => ({
      name: storage.name,
      value: `${Math.round(storageUtilizations[index] * 100)} %`,
      share: sharesOf(storageUtilizations)[index],
    }))
    .sort((a, b) => b.share - a.share);

  const busiest = [...facilityItems, ...queueItems, ...storageItems].sort(
    (a, b) => b.share - a.share,
  )[0];

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-4 sm:flex-row">
        <Panel
          title="Utilización de facilities"
          items={facilityItems}
          color="bg-indigo-500"
        />
        <Panel
          title="Contenido medio de colas"
          items={queueItems}
          color="bg-teal-600"
        />
        <Panel
          title="Utilización media de almacenes"
          items={storageItems}
          color="bg-orange-500"
        />
      </div>
      {busiest !== undefined && (
        <p className="text-[11px] text-slate-400">
          El recurso que más aprieta es el que más se acerca al 100 % o el que
          más cola carga:{" "}
          <span className="font-medium text-slate-600">{busiest.name}</span> (
          {busiest.value}).
        </p>
      )}
    </div>
  );
}
