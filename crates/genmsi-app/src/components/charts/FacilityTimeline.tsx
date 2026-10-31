/**
 * The occupancy of the facilities over the time of the run.
 *
 * Every facility is a track: the grey is the time it was free and the
 * color is the time it was held. The line that marks the instant the
 * reader is at crosses all of them.
 *
 * The chart is drawn with the layout of the page and not with the chart
 * library: a track is a row of colored pieces, and the library would draw
 * it as a line of ones and zeros, which says less and takes more.
 */
import { useMemo } from "react";
import { number } from "../../format.ts";
import { segmentsOf } from "../../monitor/series.ts";
import type { ReplayIndex } from "../../monitor/replay.ts";
import type { FacilityNumbers } from "./entities.ts";
import { colorOf } from "./palette.ts";

export default function FacilityTimeline({
  monitor,
  facilities,
  clock,
}: {
  /** The index of the replay: the instants the facilities change. */
  monitor: ReplayIndex;
  /** The facilities of the run, with their numbers. */
  facilities: FacilityNumbers[];
  /** The instant the reader is at. */
  clock: number;
}) {
  const rows = useMemo(
    () =>
      facilities.map((facility, index) => ({
        facility,
        color: colorOf(index),
        segments: segmentsOf(
          monitor.facilityPoints.get(facility.name) ?? [],
          clock,
        ),
      })),
    [facilities, monitor, clock],
  );

  const span = Math.max(monitor.endTime, 1);
  const ticks = [0, 0.25, 0.5, 0.75, 1].map((share) => Math.round(span * share));
  const share = Math.min(1, Math.max(0, clock / span));

  return (
    <div className="relative">
      <div className="flex flex-col gap-2">
        {rows.map((row) => (
          <div key={row.facility.name} className="flex items-center gap-3">
            <span className="w-24 shrink-0 truncate text-[11px] text-slate-700">
              {row.facility.name}
            </span>
            <div className="relative h-5 flex-1 overflow-hidden rounded bg-slate-100">
              {row.segments.map((segment, index) =>
                segment.busy ? (
                  <div
                    key={index}
                    className="absolute top-0 h-full"
                    style={{
                      left: `${(segment.start / span) * 100}%`,
                      width: `${((segment.end - segment.start) / span) * 100}%`,
                      background: row.color,
                    }}
                  />
                ) : null,
              )}
            </div>
            <span className="w-12 shrink-0 text-right font-mono text-[11px] text-indigo-700">
              {Math.round(row.facility.utilization * 100)} %
            </span>
          </div>
        ))}
      </div>
      {/* The instant the reader is at: the line that crosses the tracks.
          It is drawn with the same widths and gaps as the rows, so it falls
          exactly on the point of the track it marks — a line placed with
          arithmetic drifts from the track it belongs to. */}
      <div className="pointer-events-none absolute inset-0 flex gap-3">
        <span className="w-24 shrink-0" />
        <div className="relative flex-1">
          <div
            className="absolute top-0 h-full w-px bg-slate-900"
            style={{ left: `${share * 100}%` }}
          />
        </div>
        <span className="w-12 shrink-0" />
      </div>
      <div className="mt-1 flex gap-3">
        <span className="w-24 shrink-0" />
        <div className="relative flex-1">
          {ticks.map((tick) => (
            <span
              key={tick}
              className="absolute -translate-x-1/2 font-mono text-[10px] text-slate-400"
              style={{ left: `${(tick / span) * 100}%` }}
            >
              {number(tick)}
            </span>
          ))}
        </div>
        <span className="w-12 shrink-0" />
      </div>
    </div>
  );
}
