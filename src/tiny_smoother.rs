//! A one-pole IIR filter for smooth parameter transitions in audio applications.
//!
//! `TinySmoother` provides exponential smoothing with excellent numerical stability and
//! performance. It processes values using internal f64 precision while maintaining
//!  an f32 interface, ensuring zero drift even over extended periods.
//!
//! # Performance
//! Benchmarks show a __~4000x real-time factor__ on modern CPUs, allowing thousands
//! of parallel instances in typical audio workloads.
pub struct TinySmoother {
    last_value: f64,
    start_value: f32,
    beta: f64,
}

impl Default for TinySmoother {
    /// Creates a smoother with ~10ms half-life at common audio sample rates.
    ///
    /// The default configuration reaches 50% of the target value after approximately
    /// 500 samples, which corresponds to ~10ms at 48 kHz or ~11ms at 44.1 kHz.
    ///
    /// The default configuration starts at 0.0 (silence).
    fn default() -> TinySmoother {
        // Beta calculation for 500-sample half-life:
        // At sample n=500, we want output = 0.5 * target
        // This gives us: beta = e^(-ln(2)/500)
        let beta = (-2.0_f64.ln() / 500.0).exp();
        TinySmoother::new(beta, 0.0)
    }
}

impl TinySmoother {
    /// Creates a smoother with a custom smoothing coefficient.
    ///
    /// # Parameters
    /// * `beta` - Smoothing coefficient in range (0.0, 1.0)
    ///   - Values near 0.0 give instant response (no smoothing)
    ///   - Values near 1.0 give slower smoothing
    ///   - Values equal or greater 1.0 are instable
    ///
    /// To calculate beta for a specific time constant:
    /// * For 50% at n samples: `beta = e^(-ln(2)/n)`
    /// * For 63.2% at n samples: `beta = e^(-1/n)`
    ///
    /// * `start_value` - the value, the smoother should start from when reset (usually 0.0 or 1.0)
    pub fn new(beta: f64, start_value: f32) -> TinySmoother {
        assert!(beta >= 0.0 && beta < 1.0,
                "Beta must be in range [0.0, 1.0), got {}", beta);
        assert!(start_value.is_finite(),
                "Start value must be finite, got {}", start_value);
        TinySmoother {
            last_value: start_value as f64,
            beta,
            start_value,
        }
    }

    /// Processes the next target value with exponential smoothing.
    ///
    /// The filter converges smoothly toward the target using an error-feedback
    /// approach that guarantees numerical stability. Once the target is reached,
    /// the output remains exactly at the target value without drift.
    ///
    /// # Example
    /// ```
    /// use audio_utils::TinySmoother;
    ///
    /// let mut smoother = TinySmoother::default();
    /// let smoothed = smoother.next(1.0);  // Start transition to 1.0
    /// ```
    pub fn next(&mut self, target: f32) -> f32 {
        if !target.is_finite() {
            return self.last_value as f32;
        }
        let target = target as f64;
        let new_value = target - self.beta * (target - self.last_value);
        self.last_value = new_value;
        new_value as f32
    }
    /// Resets the internal value of the smoother to its initial starting value.
    ///
    ///
    /// # Example
    /// ```
    /// use audio_utils::TinySmoother;
    ///
    /// let mut smoother = TinySmoother::default();
    /// // let it run for 500 samples
    /// for _ in 0..500 {
    ///     smoother.next(1.0);
    /// }
    /// // now the value should be close to 0.5
    /// assert!( smoother.next(1.0) > 0.499);
    ///
    /// smoother.reset();
    /// // after reset, the value should be close to 0.0
    /// assert!(smoother.next(1.0) < 0.01);
    /// ```
    ///
    /// # Notes
    /// - Ensure that `start_value` is properly set before calling this method, as it
    ///   directly determines the reset value.
    pub fn reset(&mut self) {
        self.last_value = self.start_value as f64;
    }
}

//--- Tests ---------------------------------------------------------------------------------------
//
#[cfg(test)]
mod tests {
    use super::*;

    // smoother -------------
    #[test]
    fn smoother_reaches_half_target_within_500_samples() {
        let mut tiny_smoother = TinySmoother::default();
        // start at 0.0
        let start = tiny_smoother.next(0.0);
        assert_eq!(start, 0.0);

        // target 1.0 for 500 samples
        let target = 1.0f32;
        for _ in 0..500 {
            let _value = tiny_smoother.next(target);
        }

        // now the value should be close to 0.5
        let value = tiny_smoother.next(target);
        assert!(value > 0.499 && value < 0.501);
        println!("value = {value}.")
    }

    #[test]
    fn smoother_does_not_drift_when_target_is_reached() {
        let mut tiny_smoother = TinySmoother::default();
        const TARGET: f32 = 1.0;
        const SAMPLE_RATE: usize = 48_000;
        const TEST_DURATION_MINUTES: usize = 15;

        // wait until 99% of the target is reached.
        let mut value = 0.0;
        let samples_to_target_count = (0..)
            .map(|_| tiny_smoother.next(TARGET))
            .position(|value| value >= 0.99)
            .unwrap();

        println!(
            "Target reached after {} samples ({:.1} ms at 48kHz)",
            samples_to_target_count,
            samples_to_target_count as f64 * 1000.0 / SAMPLE_RATE as f64
        );

        // Start time measurement.
        let start = std::time::Instant::now();

        // let it run for fifteen minutes and check every second for drift.
        let mut max_drift = 0.0f32;
        for second in 0..TEST_DURATION_MINUTES * 60 {
            for _ in 0..SAMPLE_RATE {
                value = tiny_smoother.next(TARGET);
            }

            // Drift-Check
            let drift = (value - TARGET).abs();
            max_drift = max_drift.max(drift);
            assert!(
                drift < 0.01,
                "Drift detected after {} seconds: value={:.17}, drift={:e}",
                second + 1,
                value,
                drift
            );
        }

        // End time measurement.
        let elapsed = start.elapsed();
        let elapsed_micros = elapsed.as_micros();
        let simulated_micros = (TEST_DURATION_MINUTES * 60 * 1_000_000) as u128;
        let realtime_factor = simulated_micros as f64 / elapsed_micros as f64;

        println!(
            "Final value after {} minutes: {:.17}",
            TEST_DURATION_MINUTES, value
        );
        println!("Maximum drift from target: {:e}", max_drift);
        println!(
            "Performance: {} minutes audio processed in {:.3} ms",
            TEST_DURATION_MINUTES,
            elapsed.as_secs_f64() * 1000.0
        );
        println!(
            "Realtime factor: {:.0}x (could run ~{:.0} smoother in parallel)",
            realtime_factor, realtime_factor
        );
    }

    #[test]
    fn smoother_can_be_reset() {
        let mut smoother = TinySmoother::default();
        // let it run for 500 samples
        for _ in 0..500 {
            smoother.next(1.0);
        }
        // now the value should be close to 0.5
         assert!( smoother.next(1.0) > 0.499);

        smoother.reset();
        // after reset, the value should be close to 0.0
        assert!(smoother.next(1.0) < 0.01);
    }
}
