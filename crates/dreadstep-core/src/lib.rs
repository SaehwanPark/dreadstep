//! Deterministic game rules and domain state for Dreadstep.
//!
//! This crate owns semantic commands, events, and domain errors. It stays independent of
//! presentation, transport, storage, wall-clock time, and operating-system services so
//! the same rules can serve tests, headless tools, agents, and human-facing clients.

#![forbid(unsafe_code)]
