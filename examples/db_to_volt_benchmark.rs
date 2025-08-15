use audio_utils::decibels::db_to_volt;
use rand::{Rng, SeedableRng, rngs::SmallRng};
use std::hint::black_box;
use std::time::{Duration, Instant};

/// Benchmark domain: align with your LUT's coverage.
const MIN_DB: i32 = -100;
const MAX_DB: i32 = 20;

/// Audio "realtime" budgets in nanoseconds per sample.
const NS_PER_SAMPLE_48K: f64 = 20_833.333_333_333_332;
const NS_PER_SAMPLE_96K: f64 = 10_416.666_666_666_666;

/// Deterministically shuffled list of all dB values.
/// Intent: avoid monotonic access patterns and branch prediction artifacts.
fn mixed_db_values() -> Vec<i32> {
    let mut v: Vec<i32> = (MIN_DB..=MAX_DB).collect();
    let mut rng = SmallRng::seed_from_u64(0xDEC1_BA5E_u64); // valid hex, fixed seed
    for i in (1..v.len()).rev() {
        let j = rng.random_range(0..=i);
        v.swap(i, j);
    }
    v
}

/// Reference implementation without LUT.
#[inline(always)]
fn powf_ref(db: i32) -> f32 {
    10.0_f32.powf(db as f32 / 20.0)
}

/// Measure average time per call (ns/op) by running `iters` iterations over a mixed input set.
/// The closure `f` is the function under test. We perform a short warmup and take the best of a few runs
/// to reduce incidental noise (scheduler, turbo, cache state).
fn measure_per_call<F: Fn(i32) -> f32>(name: &str, xs: &[i32], iters: usize, f: F) -> f64 {
    // Warmup: touch inputs and produce a value to keep the optimizer honest.
    let mut acc = 0.0f32;
    for &db in xs.iter().take(64) {
        acc += black_box(f(black_box(db)));
    }
    black_box(acc);

    // Multiple runs, take the best duration as a robust lower bound.
    let runs = 5;
    let mut best: Duration = Duration::from_secs(u64::MAX);
    let mut idx = 0usize;

    for _ in 0..runs {
        let start = Instant::now();
        for _ in 0..iters {
            // Rotate through mixed inputs to avoid trivial predictability.
            idx = if idx + 1 == xs.len() { 0 } else { idx + 1 };
            let db = unsafe { *xs.get_unchecked(idx) };
            let y = f(black_box(db));
            black_box(y);
        }
        let dt = start.elapsed();
        if dt < best {
            best = dt;
        }
    }

    let ns_total = best.as_secs_f64() * 1e9;
    let ns_per_op = ns_total / (iters as f64);
    println!(
        "{name}: best-of-{runs}  total={ns_total:.3} ns  iters={iters}  ⇒  {ns_per_op:.3} ns/op"
    );
    ns_per_op
}

/// Measure batch-sweep time: process the whole input set once per iteration, repeated `sweeps` times.
/// Returns ns/op averaged over all calls inside the sweeps.
/// Useful to model block-based processing and amortize loop overhead.
fn measure_batch_sweep<F: Fn(i32) -> f32>(name: &str, xs: &[i32], sweeps: usize, f: F) -> f64 {
    // Warmup
    let mut acc = 0.0f32;
    for &db in xs.iter().take(64) {
        acc += black_box(f(black_box(db)));
    }
    black_box(acc);

    let runs = 3;
    let mut best: Duration = Duration::from_secs(u64::MAX);

    for _ in 0..runs {
        let start = Instant::now();
        let mut sum = 0.0f32;
        for _ in 0..sweeps {
            for &db in xs {
                sum += black_box(f(black_box(db)));
            }
        }
        black_box(sum);
        let dt = start.elapsed();
        if dt < best {
            best = dt;
        }
    }

    let total_calls = (xs.len() * sweeps) as f64;
    let ns_per_op = best.as_secs_f64() * 1e9 / total_calls;
    println!(
        "{name}: best-of-{runs}  sweeps={sweeps}  total_calls={total_calls:.0}  ⇒  {ns_per_op:.3} ns/op"
    );
    ns_per_op
}

/// Print speedup and realtime headroom for the given per-call ns/op numbers.
fn summarize(speed_lut: f64, speed_powf: f64) {
    let speedup = speed_powf / speed_lut;
    let rt48_lut = NS_PER_SAMPLE_48K / speed_lut;
    let rt48_powf = NS_PER_SAMPLE_48K / speed_powf;
    let rt96_lut = NS_PER_SAMPLE_96K / speed_lut;
    let rt96_powf = NS_PER_SAMPLE_96K / speed_powf;

    println!();
    println!("=== Summary (per-call) ===");
    println!(
        "LUT:  {:.3} ns/op   @48k {:.0}×   @96k {:.0}×",
        speed_lut, rt48_lut, rt96_lut
    );
    println!(
        "powf: {:.3} ns/op   @48k {:.0}×   @96k {:.0}×",
        speed_powf, rt48_powf, rt96_powf
    );
    println!("Speedup (LUT /powf): {:.2}×", speedup);
}

fn main() {
    // Prepare inputs once.
    let xs = mixed_db_values();

    // Choose iteration counts large enough to reduce timing noise.
    // Adjust if your machine is very fast/slow to keep runs under ~1–2 seconds.
    let per_call_iters = 20_000_000usize;
    let batch_sweeps = 200_000usize / xs.len().max(1); // ~200k total calls as a baseline

    println!("db_to_gain summary using Instant timing");
    println!("Range: [{}, {}] dB, count = {}", MIN_DB, MAX_DB, xs.len());
    println!();

    // Per-call
    let lut_pc = measure_per_call("LUT per-call", &xs, per_call_iters, db_to_volt);
    let pow_pc = measure_per_call("powf per-call", &xs, per_call_iters, powf_ref);

    // Batch
    let lut_bs = measure_batch_sweep("LUT batch-sweep", &xs, batch_sweeps, db_to_volt);
    let pow_bs = measure_batch_sweep("powf batch-sweep", &xs, batch_sweeps, powf_ref);

    // Summaries
    summarize(lut_pc, pow_pc);

    println!();
    println!("=== Summary (batch-sweep) ===");
    let speedup_bs = pow_bs / lut_bs;
    println!("LUT:  {:.3} ns/op", lut_bs);
    println!("powf: {:.3} ns/op", pow_bs);
    println!("Speedup (LUT /powf): {:.2}×", speedup_bs);
}
