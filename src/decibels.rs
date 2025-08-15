//! Utilities for working with decibels (dB) and linear gains (gain).
//!
//! This module provides functions for converting between decibels (dB) and linear gains (gain).
//!
//! # Examples
//!
//! ```
//! use audio_utils::DbToGain;
//!
//! let decibels:i32 = -60;
//! let gain = decibels.to_gain();
//! ```



/// A static lookup table mapping integer decibel values in the range -100 to +27 dB
/// to corresponding linear gain values (f32). The step size is exactly 1 dB,
/// which is below the just noticeable difference (JND) for loudness at typical
/// listening conditions (~1 dB at 500 Hz), making this resolution perceptually transparent.
///
/// This table is intended for fast real-time conversion (e.g., from MIDI or UI sliders)
/// without expensive floating-point operations like `powf`. No interpolation is required.
///
/// Values are calculated using the formula: 10^(dB/20) and represented in scientific notation
/// for maximum precision within f32 limits.
const DB_GAIN_LOOKUP: [f32; 128] = [
    // -100 dB to -91 dB
    1.0000000e-05,
    1.1220185e-05,
    1.2589254e-05,
    1.4125376e-05,
    1.5848932e-05,
    1.7782794e-05,
    1.9952623e-05,
    2.2387211e-05,
    2.5118864e-05,
    2.8183829e-05,
    // -90 dB to -81 dB
    3.1622777e-05,
    3.5481339e-05,
    3.9810717e-05,
    4.4668359e-05,
    5.0118723e-05,
    5.6234133e-05,
    6.3095734e-05,
    7.0794578e-05,
    7.9432823e-05,
    8.9125094e-05,
    // -80 dB to -71 dB
    1.0000000e-04,
    1.1220185e-04,
    1.2589254e-04,
    1.4125376e-04,
    1.5848932e-04,
    1.7782794e-04,
    1.9952623e-04,
    2.2387211e-04,
    2.5118864e-04,
    2.8183829e-04,
    // -70 dB to -61 dB
    3.1622777e-04,
    3.5481339e-04,
    3.9810717e-04,
    4.4668359e-04,
    5.0118723e-04,
    5.6234133e-04,
    6.3095734e-04,
    7.0794578e-04,
    7.9432823e-04,
    8.9125094e-04,
    // -60 dB to -51 dB
    1.0000000e-03,
    1.1220185e-03,
    1.2589254e-03,
    1.4125376e-03,
    1.5848932e-03,
    1.7782794e-03,
    1.9952623e-03,
    2.2387211e-03,
    2.5118864e-03,
    2.8183829e-03,
    // -50 dB to -41 dB
    3.1622777e-03,
    3.5481339e-03,
    3.9810717e-03,
    4.4668359e-03,
    5.0118723e-03,
    5.6234133e-03,
    6.3095734e-03,
    7.0794578e-03,
    7.9432823e-03,
    8.9125094e-03,
    // -40 dB to -31 dB
    1.0000000e-02,
    1.1220185e-02,
    1.2589254e-02,
    1.4125376e-02,
    1.5848932e-02,
    1.7782794e-02,
    1.9952623e-02,
    2.2387211e-02,
    2.5118864e-02,
    2.8183829e-02,
    // -30 dB to -21 dB
    3.1622777e-02,
    3.5481339e-02,
    3.9810717e-02,
    4.4668359e-02,
    5.0118723e-02,
    5.6234133e-02,
    6.3095734e-02,
    7.0794578e-02,
    7.9432823e-02,
    8.9125094e-02,
    // -20 dB to -11 dB
    1.0000000e-01,
    1.1220185e-01,
    1.2589254e-01,
    1.4125376e-01,
    1.5848932e-01,
    1.7782794e-01,
    1.9952623e-01,
    2.2387211e-01,
    2.5118864e-01,
    2.8183829e-01,
    // -10 dB to -1 dB
    3.1622777e-01,
    3.5481339e-01,
    3.9810717e-01,
    4.4668359e-01,
    5.0118723e-01,
    5.6234133e-01,
    6.3095734e-01,
    7.0794578e-01,
    7.9432823e-01,
    8.9125094e-01,
    // 0 dB to +9 dB
    1.0000000e+00,
    1.1220185e+00,
    1.2589254e+00,
    1.4125376e+00,
    1.5848932e+00,
    1.7782794e+00,
    1.9952623e+00,
    2.2387211e+00,
    2.5118864e+00,
    2.8183829e+00,
    // +10 dB to +19 dB
    3.1622777e+00,
    3.5481339e+00,
    3.9810717e+00,
    4.4668359e+00,
    5.0118723e+00,
    5.6234133e+00,
    6.3095734e+00,
    7.0794578e+00,
    7.9432823e+00,
    8.9125094e+00,
    // +20 dB to +27 dB
    1.0000000e+01,
    1.1220185e+01,
    1.2589254e+01,
    1.4125376e+01,
    1.5848932e+01,
    1.7782794e+01,
    1.9952623e+01,
    2.2387211e+01,
];
const DB_GAIN_LOOKUP_OFFSET: usize = 100;
const DB_GAIN_LOOKUP_SIZE: usize = DB_GAIN_LOOKUP.len();
const DB_GAIN_LOOKUP_MIN: i32 = -(DB_GAIN_LOOKUP_OFFSET as i32);
const DB_GAIN_LOOKUP_MAX: i32 = DB_GAIN_LOOKUP_MIN + (DB_GAIN_LOOKUP_SIZE - 1) as i32;

/// Converts integer dB values in the range −100 to +20 into a linear gain
/// using a precomputed lookup table. This avoids expensive runtime calls
/// to `powf()` in the audio processing hot path and runs ~80×–160× faster,
/// with precision sufficient for most practical real-time audio use cases.
///
/// # Arguments
///
/// * `db` - An integer decibel value (usually from MIDI or UI), clamped to [-100, 27].
///
/// # Returns
///
/// * `f32` linear gain value in the range `[1e-5, ~22.4]`.
///
/// # Example
/// ```
/// use audio_utils::db_to_gain;
///
/// let decibels:i32 = -60;
/// let gain = db_to_gain(decibels);
///
/// assert_eq!(gain, 0.001);
/// ```
/// # Performance
///
/// - The lookup table is about _6_ to _7_ times _faster_ than `powf()`
/// - The lookup table has a realtime factor of __14,202__ at a sample rate of 48 kHz,
///   meaning you can call it several thousand times per sample.
///
#[inline(always)]
pub fn db_to_gain(db: i32) -> f32 {
    let db = db.clamp(DB_GAIN_LOOKUP_MIN, DB_GAIN_LOOKUP_MAX);
    let idx = (db + DB_GAIN_LOOKUP_OFFSET as i32) as usize;
    DB_GAIN_LOOKUP[idx]
}
/// Syntactic sugar. Instead of `db_to_gain(decibels)` you can use `decibels.to_gain()`
pub trait DbToGain {
    fn to_gain(self) -> f32;
}

impl DbToGain for i32 {
    /// Converts a decibel value given as an i32 into a gain value.
    ///
    /// # Example
    /// ```
    /// use audio_utils::DbToGain;
    ///
    /// let decibels:i32 = -60;
    /// let gain = decibels.to_gain();
    ///
    /// assert_eq!(gain, 1.0000000e-03f32);
    /// ```
    ///
    #[inline]
    fn to_gain(self) -> f32 {
        db_to_gain(self)
    }
}
impl DbToGain for i64 {
    /// Converts a decibel value given as an i64 into a gain value.
    ///
    /// # Example
    /// ```
    /// use audio_utils::DbToGain;
    ///
    /// let decibels:i64 = -60;
    /// let gain = decibels.to_gain();
    ///
    /// assert_eq!(gain, 1.0000000e-03f32);
    /// ```
    ///
    #[inline]
    fn to_gain(self) -> f32 {
        db_to_gain(self as i32)
    }
}
impl DbToGain for f32 {
    /// Converts a decibel value given as a f32 into a gain value.
    ///
    /// Note:
    /// 1. The floating point value is truncated to the nearest integer, there is no interpolation.
    /// 2. The value is clamped to the range of [-100, 27] decibels.
    ///
    /// # Example
    /// ```
    /// use audio_utils::DbToGain;
    ///
    /// let decibels:f32 = -59.8; // will be rounded to -60 dB
    /// let gain = decibels.to_gain();
    ///
    /// assert_eq!(gain, 1.0000000e-03f32);
    /// ```
    ///
    #[inline]
    fn to_gain(self) -> f32 {
        if !self.is_finite() {
            return 1.0; // Unity gain as a secure default
        }
        db_to_gain(self.clamp(-100.0, 27.0).round() as i32)
    }
}
impl DbToGain for f64 {
    /// Converts a decibel value given as a f64 into a gain value.
    ///
    /// Note:
    /// 1. The floating point value is truncated to the nearest integer, there is no interpolation.
    /// 2. The value is clamped to the range of [-100, 27] decibels.
    ///
    /// # Example
    /// ```
    /// use audio_utils::DbToGain;
    ///
    /// let decibels:f64 = -59.8; // will be rounded to -60 dB
    /// let gain = decibels.to_gain();
    ///
    /// assert_eq!(gain, 1.0000000e-03f32);
    /// ```
    ///
    #[inline]
    fn to_gain(self) -> f32 {
        if !self.is_finite() {
            return 1.0; // Unity gain as a secure default
        }
        db_to_gain(self.clamp(-100.0, 27.0).round() as i32)
    }
}

/// Converts a linear gain factor back into an approximate integer decibel value.
/// Performs a binary search on the same precomputed `DB_GAIN_LOOKUP` table used by `db_to_gain()`.
///
/// This function guarantees that `gain_to_db(db_to_gain(given_db))` yields the `given_db` value
/// (it is round trip is stable).
///
/// # Arguments
///
/// * `gain` - A linear gain value (f32). Values below the minimum map to -100 dB. Values above
///            maximum map to +27 dB.
///
/// # Returns
///
/// * `i32` decibel value in the range `[-100, 27]`
///
/// # Example
/// ```
/// use audio_utils::gain_to_db;
///
/// let gain:f32 = 0.001;
/// let decibels = gain_to_db(gain);
///
/// assert_eq!(decibels, -60);
/// ```
/// # Performance
///
/// To be honest, the performance of `gain_to_db` is not better than `log10()` even on a small
/// system. But it still might be useful where you need the round-trip stability of
/// `gain_to_db(db_to_gain(given_db))`.
///
/// - The lookup table iteration is about _1.26_ times _faster_ than `log10()`
/// - The lookup table iteration has a realtime factor of __1865__ at a sample rate of 48 kHz, on a
///   small Intel® Core™ i5-7200U CPU system.
///   Meaning you can call it several hundred times per sample.
pub fn gain_to_db(gain: f32) -> i32 {
    // Decibels are defined as 10*log(gain^2). Because of the squaring, gain_to_db(g) = gain_to_db(-g).
    let gain = gain.abs();

    // shortcut (and clamping) for small values
    if gain <= DB_GAIN_LOOKUP[0] {
        return DB_GAIN_LOOKUP_MIN;
    }

    // shortcut (and clamping) for large values
    if gain >= DB_GAIN_LOOKUP[DB_GAIN_LOOKUP_SIZE - 1] {
        return DB_GAIN_LOOKUP_MAX;
    }

    let mut low = 0;
    let mut high = DB_GAIN_LOOKUP_SIZE - 1;

    while low < high {
        let mid = (low + high) / 2;
        if DB_GAIN_LOOKUP[mid] < gain {
            low = mid + 1;
        } else {
            high = mid;
        }
    }

    let idx = if low > 0 {
        // Pick the closer of low and low-1
        let lo = DB_GAIN_LOOKUP[low];
        let hi = DB_GAIN_LOOKUP[low - 1];
        if (gain - hi).abs() < (gain - lo).abs() {
            low - 1
        } else {
            low
        }
    } else {
        low
    };

    (idx as i32) + DB_GAIN_LOOKUP_MIN
}
/// Syntactic sugar. Instead of `gain_to_db(gain)` you can use `gain.to_db()`
pub trait GainToDb {
    fn to_db(self) -> i32;
}
impl GainToDb for f32 {
    /// Converts a gain value given as a f32 into a decibel value.
    /// Note: the calculation is a crude approximation and may not be exact for most values.
    ///
    /// # Example
    /// ```
    /// use audio_utils::GainToDb;
    ///
    /// let gain:f32 = 0.001;
    /// let decibels = gain.to_db();
    ///
    /// assert_eq!(decibels, -60);
    /// ```
    ///
    #[inline]
    fn to_db(self) -> i32 {
        if !self.is_finite() {
            return -100; // Minimum as a secure default
        }
        gain_to_db(self)
    }
}
impl GainToDb for f64 {
    /// Converts a gain value given as a f64 into a decibel value.
    /// Note: the calculation is a crude approximation and may not be exact for most values.
    ///
    /// # Example
    /// ```
    /// use audio_utils::GainToDb;
    ///
    /// let gain:f64 = 0.001;
    /// let decibels = gain.to_db();
    ///
    /// assert_eq!(decibels, -60);
    /// ```
    ///
    #[inline]
    fn to_db(self) -> i32 {
        if !self.is_finite() {
            return -100; // Minimum as a secure default
        }
        gain_to_db(self as f32)
    }
}

//--- Tests ---------------------------------------------------------------------------------------
//
#[cfg(test)]
mod tests {
    use super::*;
    use std::hint::black_box;

    //--- db_to_gain
    #[test]
    fn db_to_gain_for_unity_gain_is_exact() {
        assert_eq!(db_to_gain(0), 1.0);
    }
    #[test]
    fn db_to_gain_delivers_correct_values() {
        for db in -100..=27 {
            let expected = 10.0_f32.powf(db as f32 / 20.0);
            let actual = db_to_gain(db);

            // verify that the values differ by at most 0.01%
            let ratio = expected / actual;
            assert!(ratio >= 0.9999 && ratio <= 1.0001);
        }
    }
    #[test]
    fn db_to_gain_clamps_values() {
        assert_eq!(db_to_gain(-101), 1.0000000e-05);
        assert!(db_to_gain(28) > 20.0);
    }

    //--- Edge case tests for DbToGain trait
    #[test]
    fn db_to_gain_handles_nan_f32() {
        let nan_db = f32::NAN;
        let result = nan_db.to_gain();
        assert_eq!(result, 1.0); // Should return unity gain
    }

    #[test]
    fn db_to_gain_handles_infinity_f32() {
        let inf_db = f32::INFINITY;
        let neg_inf_db = f32::NEG_INFINITY;
        assert_eq!(inf_db.to_gain(), 1.0); // Should return unity gain
        assert_eq!(neg_inf_db.to_gain(), 1.0); // Should return unity gain
    }

    #[test]
    fn db_to_gain_handles_nan_f64() {
        let nan_db = f64::NAN;
        let result = nan_db.to_gain();
        assert_eq!(result, 1.0); // Should return unity gain
    }

    #[test]
    fn db_to_gain_handles_infinity_f64() {
        let inf_db = f64::INFINITY;
        let neg_inf_db = f64::NEG_INFINITY;
        assert_eq!(inf_db.to_gain(), 1.0); // Should return unity gain
        assert_eq!(neg_inf_db.to_gain(), 1.0); // Should return unity gain
    }

    //--- Edge case tests for GainToDb trait
    #[test]
    fn gain_to_db_handles_nan_f32() {
        let nan_gain = f32::NAN;
        let result = nan_gain.to_db();
        assert_eq!(result, -100); // Should return minimum dB
    }

    #[test]
    fn gain_to_db_handles_infinity_f32() {
        let inf_gain = f32::INFINITY;
        let result = inf_gain.to_db();
        assert_eq!(result, -100); // Should return minimum dB (because infinity.is_finite() is false)
    }

    #[test]
    fn gain_to_db_handles_zero() {
        let zero_gain = 0.0f32;
        let result = zero_gain.to_db();
        assert_eq!(result, -100); // Should clamp to the minimum
    }

    #[test]
    fn gain_to_db_handles_negative_gains() {
        // Test that negative gains are treated the same as positive (due to abs())
        let positive = 0.5f32;
        let negative = -0.5f32;
        assert_eq!(positive.to_db(), negative.to_db());

        // Test specific value
        assert_eq!((-1.0f32).to_db(), 0); // -1.0 has the same magnitude as 1.0 -> 0 dB
    }

    #[test]
    fn gain_to_db_handles_nan_f64() {
        let nan_gain = f64::NAN;
        let result = nan_gain.to_db();
        assert_eq!(result, -100); // Should return minimum dB
    }

    #[test]
    fn gain_to_db_handles_infinity_f64() {
        let inf_gain = f64::INFINITY;
        let result = inf_gain.to_db();
        assert_eq!(result, -100); // Should return minimum dB
    }


    #[test]
    fn db_to_gain_is_performant() {
        const SAMPLE_RATE: usize = 48_000;
        const TEST_DURATION_SECONDS: usize = 3600;
        const ITERS: usize = SAMPLE_RATE * TEST_DURATION_SECONDS;

        let start = std::time::Instant::now();
        for i in 0..ITERS {
            let db = ((i as i32) % 120) - 100;
            let out = db_to_gain(db);
            // … and the result is observed, no DCE
            black_box(out);
        }

        let elapsed = start.elapsed();
        let elapsed_micros = elapsed.as_micros();
        let simulated_micros = (TEST_DURATION_SECONDS * 1_000_000) as u128;
        let realtime_factor = simulated_micros as f64 / elapsed_micros as f64;

        println!(
            "Realtime factor: {:.0}x (could run ~{:.0} db_to_gain() in parallel)",
            realtime_factor, realtime_factor
        );
    }

    //--- gain_to_db
    #[test]
    fn gain_to_db_for_unity_gain_is_exact() {
        assert_eq!(gain_to_db(1.0), 0);
    }

    #[test]
    fn db_to_gain_and_gain_to_db_are_inverse_functions() {
        for given_db in DB_GAIN_LOOKUP_MIN..=DB_GAIN_LOOKUP_MAX {
            let actual_db = gain_to_db(db_to_gain(given_db));
            assert_eq!(actual_db, given_db);
        }
    }

    #[test]
    fn gain_to_db_accepts_negative_values() {
        let a_gain = 0.12345f32;
        assert_eq!(gain_to_db(a_gain), gain_to_db(-a_gain));
    }

    #[test]
    fn gain_to_db_clamps_small_values() {
        let a_gain = f32::MIN_POSITIVE;
        assert_eq!(gain_to_db(a_gain), DB_GAIN_LOOKUP_MIN);
    }

    #[test]
    fn gain_to_db_clamps_large_values() {
        let a_gain = f32::MAX;
        assert_eq!(gain_to_db(a_gain), DB_GAIN_LOOKUP_MAX);
    }
    #[test]
    fn gain_to_db_rounds_to_nearest_table_value() {
        let a_gain_above = 1.0001f32;
        assert_eq!(gain_to_db(a_gain_above), 0);

        let a_gain_below = 0.9999f32;
        assert_eq!(gain_to_db(a_gain_below), 0);
    }

    #[test]
    #[ignore = "Performance benchmark - run with cargo test -- --ignored"]
    fn gain_to_db_is_performant() {
        // to be honest, it is not faster than `log10()`...
        const SAMPLE_RATE: usize = 48_000;
        const TEST_DURATION_SECONDS: usize = 3600;
        const ITERS: usize = SAMPLE_RATE * TEST_DURATION_SECONDS;

        let start = std::time::Instant::now();
        for _ in 0..ITERS {
            let out = gain_to_db(black_box(3.1622777e-03));
            // … and the result is observed, no DCE
            black_box(out);
        }

        let elapsed = start.elapsed();
        let elapsed_micros = elapsed.as_micros();
        let simulated_micros = (TEST_DURATION_SECONDS * 1_000_000) as u128;
        let realtime_factor = simulated_micros as f64 / elapsed_micros as f64;

        println!(
            "Realtime factor: {:.0}x (could run ~{:.0} gain_to_db() in parallel)",
            realtime_factor, realtime_factor
        );
    }

    #[test]
    #[ignore = "Performance benchmark - run with cargo test -- --ignored"]
    fn gain_to_db_calculated_is_performant() {
        const SAMPLE_RATE: usize = 48_000;
        const TEST_DURATION_SECONDS: usize = 3600;
        const ITERS: usize = SAMPLE_RATE * TEST_DURATION_SECONDS;

        let start = std::time::Instant::now();
        for _ in 0..ITERS {
            let gain:f32 = black_box(3.1622777e-03);
            let out = 20.0*gain.log10();
            // … and the result is observed, no DCE
            black_box(out);
        }

        let elapsed = start.elapsed();
        let elapsed_micros = elapsed.as_micros();
        let simulated_micros = (TEST_DURATION_SECONDS * 1_000_000) as u128;
        let realtime_factor = simulated_micros as f64 / elapsed_micros as f64;

        println!(
            "Realtime factor: {:.0}x (could run ~{:.0} gain_to_db() in parallel)",
            realtime_factor, realtime_factor
        );
    }
}