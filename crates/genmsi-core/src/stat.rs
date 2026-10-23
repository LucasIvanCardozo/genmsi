//! Entity state and the statistics GPSS reports for it.
//!
//! Every time-dependent statistic is an integral over the run: each entity
//! keeps `area` (content × time) and `last_change`, so averages are exact for
//! piecewise-constant content instead of being sampled at event boundaries.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::xact::XactId;

/// Facility (single-server resource).
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Facility {
    /// Entity name.
    pub name: String,
    /// Number of successful seizures.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub entries: u64,
    /// Time the facility was busy.
    pub busy_time: f64,
    /// Whether a transaction owns the facility.
    pub busy: bool,
    /// Whether the facility is available for seizure.
    pub available: bool,
    /// Transaction that owns the facility.
    #[serde(skip)]
    #[cfg_attr(feature = "ts", ts(skip))]
    pub owner: Option<XactId>,
    /// Owner saved when the facility was preempted.
    #[serde(skip)]
    #[cfg_attr(feature = "ts", ts(skip))]
    pub preempted: Option<Preempted>,
    /// Transactions waiting to seize.
    #[serde(skip)]
    #[cfg_attr(feature = "ts", ts(skip))]
    pub waiters: Vec<XactId>,
    /// Transactions blocked by a GATE test on this facility.
    #[serde(skip)]
    #[cfg_attr(feature = "ts", ts(skip))]
    pub gate_waiters: Vec<XactId>,
    /// Failed attempts reported as RETRY.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub retry: u64,
    /// Interruptions observed.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub inter: u64,
    /// Time the current owner started using the facility.
    #[serde(skip)]
    #[cfg_attr(feature = "ts", ts(skip))]
    pub seized_at: f64,
    /// Integration bookkeeping.
    #[serde(skip)]
    #[cfg_attr(feature = "ts", ts(skip))]
    pub last_change: f64,
}

/// A preempted facility owner waiting to be returned.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Preempted {
    /// Transaction that owned the facility.
    pub xact: XactId,
    /// Remaining service time when it was preempted.
    pub remaining: f64,
}

impl Facility {
    /// Create an available facility.
    pub fn new(name: impl Into<String>, now: f64) -> Self {
        Self {
            name: name.into(),
            entries: 0,
            busy_time: 0.0,
            busy: false,
            available: true,
            owner: None,
            preempted: None,
            waiters: Vec::new(),
            gate_waiters: Vec::new(),
            retry: 0,
            inter: 0,
            seized_at: now,
            last_change: now,
        }
    }

    /// Utilization ratio (`busy_time / elapsed`).
    pub fn utilization(&self, elapsed: f64) -> f64 {
        if elapsed <= 0.0 {
            0.0
        } else {
            self.busy_time / elapsed
        }
    }

    /// Average service time over successful seizures.
    pub fn average_time(&self) -> f64 {
        if self.entries == 0 {
            0.0
        } else {
            self.busy_time / self.entries as f64
        }
    }

    /// Mark the facility busy for a new owner.
    pub fn start_busy(&mut self, now: f64) {
        self.busy = true;
        self.seized_at = now;
        self.last_change = now;
    }

    /// Release the facility, accumulating busy time.
    pub fn stop_busy(&mut self, now: f64) {
        if self.busy {
            self.busy_time += (now - self.seized_at).max(0.0);
        }
        self.busy = false;
        self.owner = None;
    }
}

/// Queue entity.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Queue {
    /// Entity name.
    pub name: String,
    /// Current content, in units.
    ///
    /// A queue counts units, not transactions: the original compiles the
    /// `StorageUse` as `ARRIVE [Queue],[Cantidad]` and its queue holds one
    /// place per unit (`QUEUE qAlmacen,2` puts two places in the queue).
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub content: u64,
    /// Maximum content observed, in units.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub max: u64,
    /// Total enqueues.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub entries: u64,
    /// Enqueues whose wait was zero.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub zero_entries: u64,
    /// Accumulated waiting time.
    pub total_time: f64,
    /// Content × time integral.
    pub area: f64,
    /// Transactions that failed to enter (retries).
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub retry: u64,
    /// Failed admission attempts.
    #[serde(skip)]
    #[cfg_attr(feature = "ts", ts(skip))]
    pub enqueued_at: BTreeMap<XactId, f64>,
    #[serde(skip)]
    #[cfg_attr(feature = "ts", ts(skip))]
    pub last_change: f64,
}

impl Queue {
    /// Create an empty queue.
    pub fn new(name: impl Into<String>, now: f64) -> Self {
        Self {
            name: name.into(),
            content: 0,
            max: 0,
            entries: 0,
            zero_entries: 0,
            total_time: 0.0,
            area: 0.0,
            retry: 0,
            enqueued_at: BTreeMap::new(),
            last_change: now,
        }
    }

    /// Record an enqueue of `units` at `now`.
    pub fn enqueue(&mut self, xact: XactId, units: u64, now: f64) {
        self.area += self.content as f64 * (now - self.last_change).max(0.0);
        self.last_change = now;
        self.content += units;
        self.max = self.max.max(self.content);
        self.entries += 1;
        self.enqueued_at.insert(xact, now);
    }

    /// Record a dequeue of `units` at `now`, returning the waiting time.
    pub fn dequeue(&mut self, xact: XactId, units: u64, now: f64) -> f64 {
        self.area += self.content as f64 * (now - self.last_change).max(0.0);
        self.last_change = now;
        self.content = self.content.saturating_sub(units);
        let started = self.enqueued_at.remove(&xact).unwrap_or(now);
        let waited = (now - started).max(0.0);
        self.total_time += waited;
        if waited == 0.0 {
            self.zero_entries += 1;
        }
        waited
    }

    /// Average content over `elapsed`.
    pub fn average_content(&self, elapsed: f64) -> f64 {
        if elapsed <= 0.0 {
            0.0
        } else {
            self.area / elapsed
        }
    }

    /// Average waiting time over all enqueues.
    ///
    /// The original divides the queue's integral (its content over the time,
    /// in units) by the number of entries: `AVE.TIME` in its report. The
    /// integral counts the transactions that are still waiting, which the
    /// sum of the waits of the ones that left does not.
    pub fn average_time(&self) -> f64 {
        if self.entries == 0 {
            0.0
        } else {
            self.area / self.entries as f64
        }
    }

    /// Average waiting time excluding zero waits.
    ///
    /// A transaction that did not wait adds nothing to the integral, so the
    /// integral of the entries that waited is the whole one.
    pub fn average_time_nonzero(&self) -> f64 {
        let nonzero = self.entries - self.zero_entries;
        if nonzero == 0 {
            0.0
        } else {
            self.area / nonzero as f64
        }
    }

    /// Close the area integral at `now`.
    pub fn finish(&mut self, now: f64) {
        self.area += self.content as f64 * (now - self.last_change).max(0.0);
        self.last_change = now;
    }
}

/// Storage entity (multi-unit resource).
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Storage {
    /// Entity name.
    pub name: String,
    /// Declared capacity, in units.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub capacity: i64,
    /// Units currently in use.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub in_use: i64,
    /// Units × time integral.
    pub area: f64,
    /// Minimum in use observed.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub min_use: i64,
    /// Maximum in use observed.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub max_use: i64,
    /// Number of ENTER operations completed.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub entries: u64,
    /// Failed ENTER attempts.
    ///
    /// The original never counts a refusal here: its storage rows show
    /// `RETRY` 0 and the transactions that wait in a `DELAY` column.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub retry: u64,
    /// Transactions waiting for the storage.
    ///
    /// This is the original's `DELAY` column: the transactions that are
    /// waiting for units the storage does not have.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub delay: u64,
    /// Transactions waiting for units.
    #[serde(skip)]
    #[cfg_attr(feature = "ts", ts(skip))]
    pub waiters: Vec<XactId>,
    /// Transactions blocked by a GATE test on this storage.
    #[serde(skip)]
    #[cfg_attr(feature = "ts", ts(skip))]
    pub gate_waiters: Vec<XactId>,
    #[serde(skip)]
    #[cfg_attr(feature = "ts", ts(skip))]
    pub last_change: f64,
}

impl Storage {
    /// Create a storage with `capacity` units.
    pub fn new(name: impl Into<String>, capacity: i64, now: f64) -> Self {
        Self {
            name: name.into(),
            capacity,
            in_use: 0,
            area: 0.0,
            min_use: 0,
            max_use: 0,
            entries: 0,
            retry: 0,
            delay: 0,
            waiters: Vec::new(),
            gate_waiters: Vec::new(),
            last_change: now,
        }
    }

    /// Remaining free units.
    pub fn remaining(&self) -> i64 {
        (self.capacity - self.in_use).max(0)
    }

    /// Take `units` if they fit; returns whether the operation succeeded.
    pub fn try_enter(&mut self, units: i64, now: f64) -> bool {
        if units <= self.remaining() {
            self.area += self.in_use as f64 * (now - self.last_change).max(0.0);
            self.last_change = now;
            self.in_use += units;
            self.max_use = self.max_use.max(self.in_use);
            self.entries += 1;
            true
        } else {
            false
        }
    }

    /// Release `units`.
    pub fn leave(&mut self, units: i64, now: f64) {
        self.area += self.in_use as f64 * (now - self.last_change).max(0.0);
        self.last_change = now;
        self.in_use = (self.in_use - units).max(0);
        self.min_use = self.min_use.min(self.in_use);
    }

    /// Whether a GATE "empty"/"not empty" test passes.
    pub fn is_empty(&self) -> bool {
        self.in_use == 0
    }

    /// Average content over `elapsed`.
    pub fn average_content(&self, elapsed: f64) -> f64 {
        if elapsed <= 0.0 {
            0.0
        } else {
            self.area / elapsed
        }
    }

    /// Utilization ratio.
    pub fn utilization(&self, elapsed: f64) -> f64 {
        if elapsed <= 0.0 || self.capacity <= 0 {
            0.0
        } else {
            self.area / (self.capacity as f64 * elapsed)
        }
    }

    /// Close the area integral at `now`.
    pub fn finish(&mut self, now: f64) {
        self.area += self.in_use as f64 * (now - self.last_change).max(0.0);
        self.last_change = now;
        // The original's `DELAY` column: the transactions that are waiting
        // for the storage when the run ends.
        self.delay = self.waiters.len() as u64;
    }
}

/// User chain entity.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Chain {
    /// Entity name.
    pub name: String,
    /// Current chain size.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub content: u64,
    /// Maximum size observed.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub max: u64,
    /// Total LINK operations.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub entries: u64,
    /// Accumulated time in the chain.
    pub total_time: f64,
    /// Size × time integral.
    pub area: f64,
    /// Members in chain order.
    #[serde(skip)]
    #[cfg_attr(feature = "ts", ts(skip))]
    pub members: Vec<XactId>,
    /// The indicator of the chain.
    ///
    /// The original's chain has an indicator that is off until the first
    /// transaction reaches it: that one goes on to the destination of the
    /// `LINK` statement instead of joining the chain. The indicator is not
    /// part of the report.
    #[serde(skip)]
    #[cfg_attr(feature = "ts", ts(skip))]
    pub indicator: bool,
    #[serde(skip)]
    #[cfg_attr(feature = "ts", ts(skip))]
    pub linked_at: BTreeMap<XactId, f64>,
    #[serde(skip)]
    #[cfg_attr(feature = "ts", ts(skip))]
    pub last_change: f64,
}

impl Chain {
    /// Create an empty chain.
    pub fn new(name: impl Into<String>, now: f64) -> Self {
        Self {
            name: name.into(),
            content: 0,
            max: 0,
            entries: 0,
            total_time: 0.0,
            area: 0.0,
            members: Vec::new(),
            linked_at: BTreeMap::new(),
            indicator: false,
            last_change: now,
        }
    }

    /// Average content over `elapsed`.
    pub fn average_content(&self, elapsed: f64) -> f64 {
        if elapsed <= 0.0 {
            0.0
        } else {
            self.area / elapsed
        }
    }

    /// Average time spent in the chain.
    ///
    /// The original divides the chain's integral (its size over the time) by
    /// the number of entries: `AVE.TIME` in its report. The integral counts
    /// the transactions that are still waiting, which the sum of the waits of
    /// the ones that left does not.
    pub fn average_time(&self) -> f64 {
        if self.entries == 0 {
            0.0
        } else {
            self.area / self.entries as f64
        }
    }

    /// Close the area integral at `now`.
    pub fn finish(&mut self, now: f64) {
        self.area += self.content as f64 * (now - self.last_change).max(0.0);
        self.last_change = now;
    }
}

/// Histogram recorded by TABULATE.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Table {
    /// Entity name.
    pub name: String,
    /// Upper bound of the first interval.
    pub first: f64,
    /// Interval width.
    pub width: f64,
    /// Number of intervals.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub intervals: usize,
    /// Recorded values.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub samples: u64,
    /// Sum of recorded values.
    pub sum: f64,
    /// Sum of squared values.
    pub sum_squares: f64,
    /// Largest value recorded.
    pub max_value: f64,
    /// Frequency per interval.
    ///
    /// The last one is the class of the values that do not fit in any
    /// other: the original keeps it apart (`>   a  80`).
    #[cfg_attr(feature = "ts", ts(as = "Vec<f64>"))]
    pub frequencies: Vec<u64>,
    /// Values beyond the last interval.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub overflow: u64,
    /// Sum of the values that did not fit.
    pub overflow_sum: f64,
    /// Sum of the squares of the values that did not fit.
    pub overflow_sum_squares: f64,
}

impl Table {
    /// Create a table definition.
    pub fn new(name: impl Into<String>, first: f64, width: f64, intervals: usize) -> Self {
        Self {
            name: name.into(),
            first,
            width: if width <= 0.0 { 1.0 } else { width },
            intervals: intervals.max(1),
            samples: 0,
            sum: 0.0,
            sum_squares: 0.0,
            max_value: 0.0,
            frequencies: vec![0; intervals.max(1)],
            overflow: 0,
            overflow_sum: 0.0,
            overflow_sum_squares: 0.0,
        }
    }

    /// Record one observation.
    pub fn tabulate(&mut self, value: f64) {
        self.samples += 1;
        self.sum += value;
        self.sum_squares += value * value;
        self.max_value = self.max_value.max(value);
        let index = if value <= self.first {
            0
        } else {
            (((value - self.first) / self.width).ceil() as usize).min(self.intervals - 1)
        };
        if index == self.intervals - 1 && self.intervals > 1 {
            // The last class is the one of the values that do not fit: the
            // original keeps them apart and counts them in their own
            // statistics (`MEDIA OVERFLOW`).
            self.overflow += 1;
            self.overflow_sum += value;
            self.overflow_sum_squares += value * value;
        }
        self.frequencies[index] += 1;
    }

    /// The mean of the values that did not fit in any class.
    pub fn overflow_mean(&self) -> f64 {
        if self.overflow == 0 {
            0.0
        } else {
            self.overflow_sum / self.overflow as f64
        }
    }

    /// The standard deviation of the values that did not fit in any class.
    pub fn overflow_stddev(&self) -> f64 {
        if self.overflow == 0 {
            return 0.0;
        }
        let n = self.overflow as f64;
        let variance = (self.overflow_sum_squares / n) - self.overflow_mean().powi(2);
        variance.max(0.0).sqrt()
    }

    /// Mean of recorded values.
    pub fn mean(&self) -> f64 {
        if self.samples == 0 {
            0.0
        } else {
            self.sum / self.samples as f64
        }
    }

    /// Population standard deviation of recorded values.
    pub fn stddev(&self) -> f64 {
        if self.samples == 0 {
            return 0.0;
        }
        let n = self.samples as f64;
        let variance = (self.sum_squares / n) - self.mean().powi(2);
        variance.max(0.0).sqrt()
    }
}

/// All entities of a run.
#[derive(Debug, Clone, Default, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Entities {
    /// Facilities by name.
    pub facilities: BTreeMap<String, Facility>,
    /// Storages by name.
    pub storages: BTreeMap<String, Storage>,
    /// Queues by name.
    pub queues: BTreeMap<String, Queue>,
    /// User chains by name.
    pub chains: BTreeMap<String, Chain>,
    /// Logic switches by name.
    pub logic: BTreeMap<String, bool>,
    /// Savevalues by name.
    pub savevalues: BTreeMap<String, f64>,
    /// Tables by name.
    pub tables: BTreeMap<String, Table>,
    /// Queue tables by name.
    pub qtables: BTreeMap<String, Table>,
}

impl Entities {
    /// Facility, created on first reference (GPSS autodefinition).
    pub fn facility(&mut self, name: &str, now: f64) -> &mut Facility {
        self.facilities
            .entry(name.to_string())
            .or_insert_with(|| Facility::new(name, now))
    }

    /// Storage, created with `capacity` units on first reference.
    pub fn storage(&mut self, name: &str, capacity: i64, now: f64) -> &mut Storage {
        self.storages
            .entry(name.to_string())
            .or_insert_with(|| Storage::new(name, capacity, now))
    }

    /// Queue, created on first reference.
    pub fn queue(&mut self, name: &str, now: f64) -> &mut Queue {
        self.queues
            .entry(name.to_string())
            .or_insert_with(|| Queue::new(name, now))
    }

    /// Chain, created on first reference.
    pub fn chain(&mut self, name: &str, now: f64) -> &mut Chain {
        self.chains
            .entry(name.to_string())
            .or_insert_with(|| Chain::new(name, now))
    }

    /// Logic switch, created reset on first reference.
    pub fn logic(&mut self, name: &str) -> &mut bool {
        self.logic.entry(name.to_string()).or_insert(false)
    }

    /// Savevalue, created at zero on first reference.
    pub fn savevalue(&mut self, name: &str) -> &mut f64 {
        self.savevalues.entry(name.to_string()).or_insert(0.0)
    }

    /// Close every time integral at `now`.
    pub fn finish(&mut self, now: f64) {
        for queue in self.queues.values_mut() {
            queue.finish(now);
        }
        for storage in self.storages.values_mut() {
            storage.finish(now);
        }
        for chain in self.chains.values_mut() {
            chain.finish(now);
        }
        for facility in self.facilities.values_mut() {
            if facility.busy {
                facility.busy_time += (now - facility.seized_at).max(0.0);
                facility.seized_at = now;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_integral_matches_hand_computation() {
        let mut queue = Queue::new("q", 0.0);
        queue.enqueue(1, 1, 0.0);
        queue.enqueue(2, 1, 1.0);
        queue.dequeue(1, 1, 2.0);
        queue.dequeue(2, 1, 4.0);
        queue.finish(4.0);
        // Content was 1 for [0,1), 2 for [1,2), 1 for [2,4).
        assert!((queue.area - (1.0 + 2.0 + 2.0)).abs() < 1e-9, "{}", queue.area);
        assert_eq!(queue.entries, 2);
        assert_eq!(queue.max, 2);
        assert_eq!(queue.zero_entries, 0);
        assert!((queue.average_time() - 2.5).abs() < 1e-9);
        assert!((queue.average_content(4.0) - 1.25).abs() < 1e-9);
    }

    #[test]
    fn a_queue_counts_units_and_not_transactions() {
        let mut queue = Queue::new("q", 0.0);
        queue.enqueue(1, 2, 0.0);
        queue.enqueue(2, 2, 1.0);
        // Two transactions of two units each: the queue holds four places.
        assert_eq!(queue.content, 4);
        assert_eq!(queue.max, 4);
        assert_eq!(queue.entries, 2);
        queue.dequeue(1, 2, 2.0);
        assert_eq!(queue.content, 2);
        queue.finish(2.0);
        // The integral counts the units: four places for [0,1), two for [1,2).
        assert!((queue.area - 6.0).abs() < 1e-9, "{}", queue.area);
    }

    #[test]
    fn the_average_wait_is_the_integral_over_the_entries() {
        // The original divides the queue's integral by the number of
        // entries: a transaction that is still waiting counts in the
        // integral and in the entries.
        let mut queue = Queue::new("q", 0.0);
        queue.enqueue(1, 1, 0.0);
        queue.enqueue(2, 1, 1.0);
        queue.dequeue(1, 1, 2.0);
        queue.finish(4.0);
        // Content was 1 for [0,1), 2 for [1,2), 1 for [2,4).
        assert!((queue.area - 5.0).abs() < 1e-9, "{}", queue.area);
        assert!((queue.average_time() - 2.5).abs() < 1e-9);
        assert!((queue.average_time_nonzero() - 2.5).abs() < 1e-9);
    }

    #[test]
    fn zero_waits_are_counted_separately() {
        let mut queue = Queue::new("q", 0.0);
        queue.enqueue(1, 1, 0.0);
        queue.dequeue(1, 1, 0.0);
        queue.enqueue(2, 1, 0.0);
        queue.dequeue(2, 1, 5.0);
        assert_eq!(queue.entries, 2);
        assert_eq!(queue.zero_entries, 1);
        assert!((queue.average_time_nonzero() - 5.0).abs() < 1e-9);
    }

    #[test]
    fn storage_tracks_min_max_and_utilization() {
        let mut storage = Storage::new("s", 10, 0.0);
        assert!(storage.try_enter(4, 0.0));
        assert!(storage.try_enter(5, 2.0));
        assert!(!storage.try_enter(3, 4.0));
        storage.leave(7, 6.0);
        storage.finish(10.0);
        assert_eq!(storage.max_use, 9);
        assert_eq!(storage.min_use, 0);
        assert_eq!(storage.entries, 2);
        // The original's storage row counts no refusal: its `RETRY` column
        // stays at zero and the transactions that wait are in `DELAY`.
        assert_eq!(storage.retry, 0);
        assert!((storage.average_content(10.0) - (4.0 * 2.0 + 9.0 * 4.0 + 2.0 * 4.0) / 10.0).abs() < 1e-9);
        assert!((storage.utilization(10.0) - storage.average_content(10.0) / 10.0).abs() < 1e-9);
    }

    #[test]
    fn storage_never_goes_negative() {
        let mut storage = Storage::new("s", 3, 0.0);
        storage.leave(5, 1.0);
        assert_eq!(storage.in_use, 0);
    }

    #[test]
    fn facility_accumulates_busy_time() {
        let mut facility = Facility::new("f", 0.0);
        facility.entries += 1;
        facility.start_busy(1.0);
        facility.stop_busy(4.0);
        assert_eq!(facility.entries, 1);
        assert!((facility.busy_time - 3.0).abs() < 1e-9);
        assert!((facility.utilization(6.0) - 0.5).abs() < 1e-9);
        assert!((facility.average_time() - 3.0).abs() < 1e-9);
        assert!(!facility.busy);
        assert_eq!(facility.owner, None);
    }

    #[test]
    fn table_histogram_and_moments() {
        let mut table = Table::new("t", 10.0, 10.0, 3);
        for value in [5.0, 10.0, 11.0, 25.0, 45.0] {
            table.tabulate(value);
        }
        // 5 and 10 fall in the first interval, 11 in the second, 25 in the third,
        // 45 is clamped into the last interval.
        assert_eq!(table.frequencies, vec![2, 1, 2]);
        assert_eq!(table.samples, 5);
        assert!((table.mean() - 19.2).abs() < 1e-9);
        assert!(table.stddev() > 0.0);
        assert_eq!(table.max_value, 45.0);
    }
}
