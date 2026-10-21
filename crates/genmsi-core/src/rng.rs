//! Seeded random streams (decision D5).
//!
//! A run is fully determined by its base seed: every random stream is derived
//! from `(seed, stream)` with a splitmix64 mix, so adding a new consumer of
//! randomness never changes the numbers another consumer draws. Streams `1..`
//! are the GPSS `RNn` streams (`FunctionExpon`/`VariableA` seeds); streams
//! `100_000 + block_id` belong to a block's own draws (`GENERATE`, `ADVANCE`,
//! `SEIZE`, ...).

use std::collections::HashMap;

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;

/// First stream id reserved for block-local draws.
pub const BLOCK_STREAM_BASE: u32 = 100_000;

/// Deterministic 64-bit mixer (splitmix64 finalizer).
pub fn mix_u64(seed: u64, stream: u32) -> u64 {
    let mut z = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add((stream as u64).wrapping_mul(0xD1B5_4A32_D192_ED03));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// One generator per stream, created on first use.
pub struct Streams {
    seed: u64,
    generators: HashMap<u32, ChaCha12Rng>,
}

impl Streams {
    /// Create the streams for a run seeded with `seed`.
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            generators: HashMap::new(),
        }
    }

    /// Base seed of the run.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Uniform draw in `[0, 1)` from `stream`.
    pub fn uniform(&mut self, stream: u32) -> f64 {
        let generator = self
            .generators
            .entry(stream)
            .or_insert_with(|| ChaCha12Rng::seed_from_u64(mix_u64(self.seed, stream)));
        generator.random::<f64>()
    }

    /// Uniform draw in `[low, high)`; equal bounds return that value.
    pub fn uniform_between(&mut self, stream: u32, low: f64, high: f64) -> f64 {
        if high <= low {
            return low;
        }
        low + (high - low) * self.uniform(stream)
    }

    /// Uniform integer in `[low, high]` inclusive.
    pub fn uniform_integer(&mut self, stream: u32, low: i64, high: i64) -> i64 {
        if high <= low {
            return low;
        }
        let span = (high - low + 1) as f64;
        let offset = (self.uniform(stream) * span).floor() as i64;
        (low + offset).min(high)
    }

    /// Draw from a cumulative probability table (values in `[0, 1]`, last is 1).
    pub fn cumulative_index(&mut self, stream: u32, cumulative: &[f64]) -> usize {
        let draw = self.uniform(stream);
        for (index, bound) in cumulative.iter().enumerate() {
            if draw < *bound {
                return index;
            }
        }
        cumulative.len().saturating_sub(1)
    }

    /// Standard exponential variate with mean `mean`.
    pub fn exponential(&mut self, stream: u32, mean: f64) -> f64 {
        let draw = self.uniform(stream).max(f64::MIN_POSITIVE);
        -mean * (1.0 - draw).max(f64::MIN_POSITIVE).ln()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Streams::new(42);
        let mut b = Streams::new(42);
        for _ in 0..10 {
            assert_eq!(a.uniform(1), b.uniform(1));
        }
    }

    #[test]
    fn different_streams_do_not_couple() {
        let mut streams = Streams::new(7);
        let from_one: Vec<f64> = (0..5).map(|_| streams.uniform(1)).collect();
        let from_two: Vec<f64> = (0..5).map(|_| streams.uniform(2)).collect();
        assert_ne!(from_one, from_two);
    }

    #[test]
    fn adding_a_stream_does_not_change_another() {
        let mut alone = Streams::new(99);
        let first: Vec<f64> = (0..3).map(|_| alone.uniform(5)).collect();

        let mut mixed = Streams::new(99);
        let _unused = mixed.uniform(1234);
        let second: Vec<f64> = (0..3).map(|_| mixed.uniform(5)).collect();
        assert_eq!(first, second);
    }

    #[test]
    fn uniform_stays_in_range() {
        let mut streams = Streams::new(1);
        for _ in 0..1000 {
            let value = streams.uniform_between(3, 10.0, 20.0);
            assert!((10.0..20.0).contains(&value), "{value}");
            let integer = streams.uniform_integer(3, 1, 6);
            assert!((1..=6).contains(&integer), "{integer}");
        }
        assert_eq!(streams.uniform_between(3, 5.0, 5.0), 5.0);
        assert_eq!(streams.uniform_integer(3, 4, 4), 4);
    }

    #[test]
    fn exponential_has_roughly_the_requested_mean() {
        let mut streams = Streams::new(2024);
        let draws = 20_000;
        let total: f64 = (0..draws)
            .map(|_| streams.exponential(2, 12.0))
            .sum();
        let mean = total / draws as f64;
        assert!((mean - 12.0).abs() < 0.6, "mean was {mean}");
    }

    #[test]
    fn cumulative_index_respects_percentages() {
        let mut streams = Streams::new(5);
        let cumulative = [0.3, 0.8, 1.0];
        let mut counts = [0usize; 3];
        for _ in 0..10_000 {
            counts[streams.cumulative_index(9, &cumulative)] += 1;
        }
        let first = counts[0] as f64 / 10_000.0;
        assert!((first - 0.3).abs() < 0.03, "first bucket was {first}");
        assert_eq!(counts.iter().sum::<usize>(), 10_000);
    }
}
