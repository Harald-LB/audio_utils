# audio_utils

A small collection of utilities for real-time audio processing in Rust.

## Overview

This crate provides a few simple helpers that are useful when developing audio plugins or real-time audio applications:

- **Fast dB/voltage conversions** - Lookup table-based conversions between decibels and linear voltage ratios
- **Parameter smoothing** - A numerically stable exponential smoother for glitch-free parameter changes

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
audio_utils = { git = "https://github.com/Harald-LB/audio_utils"}
```

### Quick Examples

```rust
use audio_utils::{DbToVolt, TinySmoother};

// Convert decibels to linear voltage ratio
let db = -20_i32;
let volt = db.to_volt();  // ~0.1

// Smooth parameter changes
let mut smoother = TinySmoother::default();
let smoothed = smoother.next(1.0);  // Gradually approaches 1.0
```

## Features

### dB/Voltage Conversion

Provides fast conversion between decibels and linear values using precomputed lookup tables. 
This avoids expensive `powf()` calls in the audio processing path.

```rust
use audio_utils::{db_to_volt, volt_to_db};

let volt = db_to_volt(-60);  // 0.001
let db = volt_to_db(0.1);    // -20
```

The conversion functions guarantee round-trip stability: `volt_to_db(db_to_volt(x)) == x`

### Parameter Smoothing

`TinySmoother` implements a one-pole IIR filter for smooth parameter transitions without zipper noise. 
It uses internal f64 precision to prevent numerical drift over extended periods.

```rust
use audio_utils::TinySmoother;

// Default: ~10ms half-life at 48kHz
let mut smoother = TinySmoother::default();

// Custom smoothing coefficient
let mut smoother = TinySmoother::new(0.99, 0.0);
```

## Example

A simple gain plugin example is included in `examples/tiny_gain_plug/` showing how to use these 
utilities with the [NIH-plug](https://github.com/robbert-vdh/nih-plug) framework.

## Performance

The utilities are designed for real-time audio processing:
- dB/voltage conversion: ~7× faster than `powf()`
- Parameter smoothing: ~4000× real-time factor on modern CPUs

All functions are allocation-free and suitable for use in audio processing callbacks.

## Status

This is a personal collection of utilities that has proven useful in various audio projects. 
The API is simple and unlikely to change significantly, but the crate is still considered work in progress.

## License

Licensed under the MIT licence ([LICENSE-MIT](./LICENSE-MIT.txt) or http://opensource.org/licenses/MIT)
