//! Headless execution boundary for Dreadstep.
//!
//! This crate will own command-line, file, telemetry, and process effects while delegating
//! game decisions to `dreadstep-core`. Milestone 0 intentionally provides no executable.

#![forbid(unsafe_code)]
