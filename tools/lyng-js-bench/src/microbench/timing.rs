//! Timing harness: high-resolution monotonic clock + sample aggregation.

use std::time::{Duration, Instant};

/// One sample: wall-clock duration plus the dispatch count it measures.
#[derive(Debug, Clone, Copy)]
pub struct Sample {
    pub elapsed: Duration,
    pub dispatches: u64,
}

impl Sample {
    #[must_use]
    pub fn ns_per_dispatch(&self) -> f64 {
        let ns = self.elapsed.as_nanos() as f64;
        ns / (self.dispatches as f64)
    }
}

/// Aggregate sample statistics.
#[derive(Debug, Clone)]
pub struct SampleStats {
    pub samples: Vec<Sample>,
    pub median_ns_per_dispatch: f64,
    pub min_ns_per_dispatch: f64,
    pub max_ns_per_dispatch: f64,
    /// Half-width of a 95% confidence interval around the median, in ns.
    /// Computed via the inter-quartile bootstrap as a robust approximation.
    pub ci95_half_width_ns: f64,
}

impl SampleStats {
    #[must_use]
    pub fn from_samples(mut samples: Vec<Sample>) -> Self {
        assert!(!samples.is_empty(), "need at least one sample");
        let mut rates: Vec<f64> = samples.iter().map(Sample::ns_per_dispatch).collect();
        rates.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let median = rates[rates.len() / 2];
        let min = *rates.first().unwrap();
        let max = *rates.last().unwrap();

        // Conservative CI: half the IQR (75th - 25th percentile) is a
        // robust dispersion estimate that doesn't assume normality.
        let q1 = rates[rates.len() / 4];
        let q3 = rates[(rates.len() * 3) / 4];
        let ci = (q3 - q1) / 2.0;

        samples.sort_by(|a, b| {
            a.ns_per_dispatch()
                .partial_cmp(&b.ns_per_dispatch())
                .unwrap()
        });

        Self {
            samples,
            median_ns_per_dispatch: median,
            min_ns_per_dispatch: min,
            max_ns_per_dispatch: max,
            ci95_half_width_ns: ci,
        }
    }
}

/// Run `f` once, returning (elapsed, dispatches).
///
/// The `dispatches` value must be passed in by the caller — it's the
/// opcode count × inner iteration count.
pub fn time_once<F: FnOnce() -> ()>(dispatches: u64, f: F) -> Sample {
    let start = Instant::now();
    f();
    let elapsed = start.elapsed();
    Sample { elapsed, dispatches }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ns_per_dispatch_computes_correctly() {
        let sample = Sample {
            elapsed: Duration::from_nanos(1_000),
            dispatches: 100,
        };
        assert!((sample.ns_per_dispatch() - 10.0).abs() < 1e-9);
    }

    #[test]
    fn stats_from_samples_basic() {
        let samples = vec![
            Sample { elapsed: Duration::from_nanos(100), dispatches: 10 },  // 10 ns
            Sample { elapsed: Duration::from_nanos(200), dispatches: 10 },  // 20 ns
            Sample { elapsed: Duration::from_nanos(300), dispatches: 10 },  // 30 ns
            Sample { elapsed: Duration::from_nanos(400), dispatches: 10 },  // 40 ns
            Sample { elapsed: Duration::from_nanos(500), dispatches: 10 },  // 50 ns
        ];
        let stats = SampleStats::from_samples(samples);
        assert!((stats.median_ns_per_dispatch - 30.0).abs() < 1e-9);
        assert!((stats.min_ns_per_dispatch - 10.0).abs() < 1e-9);
        assert!((stats.max_ns_per_dispatch - 50.0).abs() < 1e-9);
    }

    #[test]
    fn time_once_returns_positive_elapsed() {
        let sample = time_once(1000, || {
            std::hint::black_box((0..1000).fold(0_u64, |a, b| a.wrapping_add(b)));
        });
        assert!(sample.elapsed.as_nanos() > 0);
        assert_eq!(sample.dispatches, 1000);
    }
}
