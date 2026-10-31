/**
 * The metrics the app derives from the numbers the engine publishes.
 *
 * A **derived** metric is a ratio of two published numbers: the area of a
 * queue and the time of the run, the time a facility was busy and the time
 * of the run. The engine does not publish them, so the app computes them —
 * and every place that shows one says so.
 *
 * Nothing here invents a number: each function takes the numbers the report
 * (or the replay) carries and divides them. The arithmetic is the engine's
 * own definition of each metric, written out in `crates/genmsi-core/src/stat.rs`:
 *
 * - the **mean content** of a queue is its area over the elapsed time
 *   (`Queue::area`), the same integral the engine closes at the end of the
 *   run;
 * - the **wait of the engine** (`AVE.TIME`) is the area of the queue over its
 *   entries: the integral counts the transactions that are still waiting,
 *   which the sum of the waits of the ones that left does not
 *   (`crates/genmsi-core/src/stat.rs:211-223`);
 * - the **mean of the departures** is the accumulated wait over the
 *   transactions that left: it is the mean of the ones that are already
 *   gone, and it is not the mean wait of the queue;
 * - the **utilization** of a facility is the time it was busy over the
 *   elapsed time (`Facility::utilization`);
 * - the **mean service** of a facility is the time it was busy over the
 *   seizures that took it;
 * - the **mean utilization** of a storage is its area over the elapsed time
 *   and its capacity;
 * - the **mean size** of a chain is its area over the elapsed time, and its
 *   **mean time** (`AVE.TIME`) is its area over its entries;
 * - the **mean** and the **deviation** of a table come from the sum and the
 *   sum of squares the engine publishes (`Table::sum`, `Table::sum_squares`).
 */

/** The numbers of a queue that the metrics are derived from. */
export interface QueueSample {
  /** How many transactions entered the queue. */
  entries: number;
  /** How many of them waited no time at all. */
  zeroEntries: number;
  /** The time its transactions waited, added up. */
  totalTime: number;
  /** Its content times the time, added up. */
  area: number;
  /**
   * How many transactions left the queue.
   *
   * It is not the entries: the transactions that are still waiting entered
   * the queue too, and their wait is not in the sum of the waits.
   */
  departures: number;
}

/** The metrics derived from a queue. */
export interface QueueDerived {
  /** The mean content: `area / elapsed`. */
  meanContent: number;
  /**
   * The wait of the engine, its `AVE.TIME`: `area / entries`.
   *
   * It is not the mean wait above: the integral of the queue counts the
   * transactions that are still waiting in it, and the sum of the waits
   * does not. The engine publishes this one, and the app shows both so
   * that the two numbers never carry the same name.
   */
  engineMeanWait: number;
  /**
   * The mean wait of the transactions that left: `totalTime / departures`.
   *
   * It is the mean of the ones that are already gone, and it says nothing
   * about the ones that are still waiting: that is what the mean wait of
   * the engine says.
   */
  departureMeanWait: number;
  /** The share of the entries that waited no time: `zeroEntries / entries`. */
  zeroShare: number;
}

/**
 * The metrics of a queue.
 *
 * A run of no time divides by nothing: every metric of a run that did not
 * last is zero.
 */
export function queueDerived(queue: QueueSample, elapsed: number): QueueDerived {
  return {
    meanContent: elapsed > 0 ? queue.area / elapsed : 0,
    engineMeanWait: queue.entries > 0 ? queue.area / queue.entries : 0,
    departureMeanWait:
      queue.departures > 0 ? queue.totalTime / queue.departures : 0,
    zeroShare: queue.entries > 0 ? queue.zeroEntries / queue.entries : 0,
  };
}

/** The numbers of a facility that the metrics are derived from. */
export interface FacilitySample {
  /** How many times it was taken. */
  entries: number;
  /** The time it was held, added up. */
  busyTime: number;
}

/** The metrics derived from a facility. */
export interface FacilityDerived {
  /** The utilization: `busyTime / elapsed`. */
  utilization: number;
  /** The mean service: `busyTime / entries`. */
  meanService: number;
}

/** The metrics of a facility. */
export function facilityDerived(facility: FacilitySample, elapsed: number): FacilityDerived {
  return {
    utilization: elapsed > 0 ? facility.busyTime / elapsed : 0,
    meanService: facility.entries > 0 ? facility.busyTime / facility.entries : 0,
  };
}

/** The numbers of a storage that the metrics are derived from. */
export interface StorageSample {
  /** The capacity of the storage, in units. */
  capacity: number;
  /** Its units times the time, added up. */
  area: number;
}

/** The metrics derived from a storage. */
export interface StorageDerived {
  /** The mean utilization: `area / (elapsed * capacity)`. */
  meanUtilization: number;
}

/**
 * The metrics of a storage.
 *
 * The capacity is a property of the model, not of the run: the report
 * publishes it and the replay does not carry it, so the caller gives the
 * capacity of the report and the area of the instant.
 */
export function storageDerived(storage: StorageSample, elapsed: number): StorageDerived {
  return {
    meanUtilization:
      elapsed > 0 && storage.capacity > 0
        ? storage.area / (elapsed * storage.capacity)
        : 0,
  };
}

/** The numbers of a chain that the metrics are derived from. */
export interface ChainSample {
  /** Its size times the time, added up. */
  area: number;
  /** How many transactions joined it. */
  entries: number;
}

/** The metrics derived from a chain. */
export interface ChainDerived {
  /** The mean size: `area / elapsed`. */
  meanSize: number;
  /**
   * The mean time of the engine (`AVE.TIME`): `area / entries`.
   *
   * It is the integral of the size of the chain over the transactions that
   * joined it, the same number the report of the engine prints.
   */
  meanTime: number;
}

/** The metrics of a chain. */
export function chainDerived(chain: ChainSample, elapsed: number): ChainDerived {
  return {
    meanSize: elapsed > 0 ? chain.area / elapsed : 0,
    meanTime: chain.entries > 0 ? chain.area / chain.entries : 0,
  };
}

/** The numbers of the values that did not fit in any class of a table. */
export interface OverflowSample {
  /** How many values did not fit. */
  overflow: number;
  /** The sum of the values that did not fit. */
  sum: number;
  /** The sum of the squares of the values that did not fit. */
  sumSquares: number;
}

/** The metrics derived from the overflow of a table. */
export interface OverflowDerived {
  /** The mean of the values that did not fit: `sum / overflow`. */
  mean: number;
  /** The deviation of the values that did not fit. */
  stdDev: number;
}

/**
 * The metrics of the values that did not fit in any class of a table.
 *
 * The engine publishes the sum and the sum of the squares of the values
 * that overflowed (`Table::overflow_sum`, `Table::overflow_sum_squares`), and
 * the report of the CLI prints their mean and their deviation. The app
 * derives them the same way, so the two say the same thing.
 */
export function overflowDerived(overflow: OverflowSample): OverflowDerived {
  if (overflow.overflow <= 0) {
    return { mean: 0, stdDev: 0 };
  }
  const mean = overflow.sum / overflow.overflow;
  const variance = Math.max(
    0,
    overflow.sumSquares / overflow.overflow - mean * mean,
  );
  return { mean, stdDev: Math.sqrt(variance) };
}

/** The numbers of a table that the metrics are derived from. */
export interface TableSample {
  /** How many values were tabulated. */
  samples: number;
  /** The sum of the values. */
  sum: number;
  /** The sum of the squares of the values. */
  sumSquares: number;
  /** How many values fell beyond the last interval. */
  overflow: number;
}

/** The metrics derived from a table. */
export interface TableDerived {
  /** The mean: `sum / samples`. */
  mean: number;
  /** The standard deviation, from the sum of squares. */
  stdDev: number;
  /** The share of the values that did not fit in an interval. */
  overflowRate: number;
}

/**
 * The metrics of a table.
 *
 * The deviation is the square root of the variance, and the variance comes
 * from the sum of squares the engine publishes: `E[x²] - E[x]²`. A sum of
 * squares that is a hair below the square of the mean (rounding) would give
 * a negative variance, so it is floored at zero.
 */
export function tableDerived(table: TableSample): TableDerived {
  if (table.samples <= 0) {
    return { mean: 0, stdDev: 0, overflowRate: 0 };
  }
  const mean = table.sum / table.samples;
  const variance = Math.max(0, table.sumSquares / table.samples - mean * mean);
  return {
    mean,
    stdDev: Math.sqrt(variance),
    overflowRate: table.overflow / table.samples,
  };
}
