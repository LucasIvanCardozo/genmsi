/**
 * The resources that squeeze the run.
 *
 * Three panels, one per family of resources: the utilization of the
 * facilities, the mean content of the queues and the mean utilization of
 * the storages.
 *
 * Every bar has an **absolute scale**: the utilization of a facility is
 * the share of the time up to the clock that it was held, the mean
 * utilization of a storage is the share of its capacity, and the mean
 * content of a queue is measured against the maximum that queue reached.
 * The bars used to be normalized by the biggest bar of their own panel, so
 * a resource at 3 % drew the same bar as one at 98 %.
 *
 * The families do not share a scale, so the card does not name a winner:
 * a percentage of the time and a number of units are not the same thing,
 * and comparing them says nothing.
 */
import { decimals } from "../../format.ts";
import type { Figures } from "../../monitor/stats.ts";
import type { ReportDocument } from "../../types";
import BarList, { type BarItem } from "./BarList.tsx";
import {
  facilitiesOf,
  queuesOf,
  resourcePressuresOf,
  storagesOf,
  type Elapsed,
  type ResourcePressure,
} from "./entities.ts";

/** How many resources a panel shows. */
const TOP = 5;

/** A panel of the ranking: a title, its scale and its resources. */
function Panel({
  title,
  scale,
  items,
  color,
  more,
}: {
  title: string;
  scale: string;
  items: BarItem[];
  color: string;
  more: number;
}) {
  if (items.length === 0) {
    return null;
  }
  const shown = items.slice(0, TOP);
  return (
    <div className="min-w-0 flex-1">
      <h4 className="mb-0.5 text-[11px] font-semibold text-slate-700">{title}</h4>
      <p className="mb-2 text-[10px] leading-snug text-slate-400">{scale}</p>
      <BarList items={shown} color={color} />
      {more > 0 && (
        <p className="mt-1.5 text-[10px] text-slate-400">
          y {more} más, con menos barra
        </p>
      )}
    </div>
  );
}

/** The bar of a resource, with the figure of the instant next to it. */
function itemOf(pressure: ResourcePressure): BarItem {
  return {
    name: pressure.name,
    value: pressure.percent
      ? `${decimals(pressure.value * 100)} %`
      : `${decimals(pressure.value)} u`,
    share: pressure.share,
    // The ratio of the engine's numbers is a derived figure: it is drawn
    // in the color of the derived state, not in the color of the series.
    state: pressure.live ? "derived" : "unknown",
  };
}

/** The items of a panel, the biggest first. */
function itemsOf(pressures: ResourcePressure[]): BarItem[] {
  // The ranking is by the figure the reader sees: the queue with the
  // biggest mean content is the first one, not the one whose bar is the
  // longest.
  return [...pressures]
    .sort((a, b) => b.value - a.value)
    .map(itemOf);
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
  const pressures = resourcePressuresOf(
    facilitiesOf(report, live, elapsed),
    queuesOf(report, live, elapsed),
    storagesOf(report, live, elapsed),
  );

  const facilityItems = itemsOf(pressures.facilities);
  const queueItems = itemsOf(pressures.queues);
  const storageItems = itemsOf(pressures.storages);

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-4 sm:flex-row">
        <Panel
          title="Utilización de las facilities"
          scale="barra: el porcentaje del tiempo hasta el reloj que estuvo ocupada"
          items={facilityItems}
          color="bg-indigo-500"
          more={facilityItems.length - TOP}
        />
        <Panel
          title="Contenido medio de las colas"
          scale="barra: el contenido medio sobre el máximo que alcanzó la cola"
          items={queueItems}
          color="bg-teal-600"
          more={queueItems.length - TOP}
        />
        <Panel
          title="Utilización media de los almacenes"
          scale="barra: el porcentaje de su capacidad que usó en promedio"
          items={storageItems}
          color="bg-orange-500"
          more={storageItems.length - TOP}
        />
      </div>
    </div>
  );
}
