/**
 * A time series of the entities of the run.
 *
 * The chart is the same everywhere: the axis of the time is the time of the
 * run, from **zero** to its end (the first change of an entity may come
 * much later, and an axis that started there would hide the time before
 * it), and a vertical line marks the instant the reader is at.
 *
 * The series is the exact history of the entity: the steps are the instants
 * its value changes, not a sample of the run.
 */
import {
  CartesianGrid,
  Legend,
  Line,
  LineChart,
  ReferenceLine,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { number } from "../../format.ts";
import type { SeriesRow } from "../../monitor/series.ts";

/** One series of the chart. */
export interface SeriesSpec {
  /** The name of the entity. */
  name: string;
  /** The color of the line. */
  color: string;
  /** The mean of the series, drawn as a dashed line. */
  mean?: number;
}

export default function TimeSeriesChart({
  data,
  series,
  endTime,
  clock,
  height = 256,
  yDomain,
  reference,
}: {
  /** The rows of the chart: the instants and the value of each entity. */
  data: SeriesRow[];
  /** The series to draw. */
  series: SeriesSpec[];
  /** When the run ended: the end of the axis of the time. */
  endTime: number;
  /** The instant the reader is at. */
  clock: number;
  /** The height of the chart. */
  height?: number;
  /** The domain of the axis of the values. */
  yDomain?: [number, number];
  /** A horizontal line that says something about the values. */
  reference?: { y: number; color: string; label: string };
}) {
  return (
    <div style={{ height }}>
      <ResponsiveContainer width="100%" height="100%">
        <LineChart data={data} margin={{ top: 8, right: 16, bottom: 8, left: 0 }}>
          <CartesianGrid stroke="#f1f5f9" />
          <XAxis
            dataKey="time"
            type="number"
            domain={[0, endTime]}
            tickFormatter={(value) => String(Math.round(value))}
            fontSize={10}
            minTickGap={40}
          />
          <YAxis
            fontSize={10}
            allowDecimals={false}
            domain={yDomain}
            width={44}
          />
          <Tooltip
            formatter={(value) => number(Number(value))}
            labelFormatter={(value) => `t = ${number(Number(value))}`}
          />
          <Legend wrapperStyle={{ fontSize: 11 }} />
          {series.map((entry) => (
            <ReferenceLine
              key={`media ${entry.name}`}
              y={entry.mean}
              stroke={entry.color}
              strokeDasharray="4 3"
              strokeWidth={1}
            />
          ))}
          {reference !== undefined && (
            <ReferenceLine
              y={reference.y}
              stroke={reference.color}
              strokeDasharray="4 3"
              strokeWidth={1.25}
              label={{
                value: reference.label,
                position: "insideTopRight",
                fill: reference.color,
                fontSize: 10,
              }}
            />
          )}
          <ReferenceLine x={clock} stroke="#0f172a" strokeWidth={1.5} />
          {series.map((entry) => (
            <Line
              key={entry.name}
              type="stepAfter"
              dataKey={entry.name}
              name={entry.name}
              stroke={entry.color}
              strokeWidth={1.75}
              dot={false}
              isAnimationActive={false}
            />
          ))}
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
