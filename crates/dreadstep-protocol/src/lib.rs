//! Versioned external representations for Dreadstep domain concepts.
//!
//! Protocol types translate between stable wire or replay formats and the semantic types
//! owned by `dreadstep-core`. Transport-specific behavior belongs in its adapter crate,
//! not here.

#![forbid(unsafe_code)]
