//! Bevy presentation adapter for Dreadstep.
//!
//! This crate will translate human input into core commands and semantic events into
//! presentation. Rendering and ECS state must never become authoritative game state.
//! Milestone 0 enables only Bevy's standard-library support and provides no executable.

#![forbid(unsafe_code)]
