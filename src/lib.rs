//! # Audio Utils
//!
//! A collection of efficient utilities for real-time audio processing.
//!
//! This crate provides:
//! - Fast dB/gain conversions via lookup tables
//! - Smooth parameter transitions with drift-free exponential smoothing
//!
//! All implementations are optimised for real-time audio with minimal allocations
//! and predictable performance characteristics.

pub mod tiny_smoother;
pub mod decibels;

pub use tiny_smoother::TinySmoother;
pub use decibels::{db_to_volt, volt_to_db, DbToVolt, VoltToDb};