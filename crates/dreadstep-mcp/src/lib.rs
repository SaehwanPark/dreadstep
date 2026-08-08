//! Model Context Protocol adapter boundary for Dreadstep.
//!
//! This crate will translate explicit player and tester operations into project-owned
//! semantic commands. It must not become a generic shell, filesystem escape hatch, or
//! hidden source of game truth. Milestone 0 intentionally adds no MCP runtime dependency.

#![forbid(unsafe_code)]
