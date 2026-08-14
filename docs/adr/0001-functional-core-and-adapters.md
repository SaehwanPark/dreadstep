# ADR 0001: Functional Core and Explicit Adapters

- Status: accepted
- Date: 2026-08-08

## Context

Dreadstep must support deterministic tests, replay evidence, headless tools, MCP agents,
a terminal client, and a Bevy client without allowing any one interface to become the source of
game truth.
The original proposal places simulation before presentation but leaves the ownership of
semantic commands and events implicit between core and protocol.

## Decision

`dreadstep-core` owns canonical domain state, commands, events, errors, and deterministic
transitions. `dreadstep-protocol` owns versioned external representations and conversion at
the boundary; it depends on core rather than defining the rules core consumes.

Domain code prefers explicit state and pure transformations. Seeded randomness, time, and
configuration enter through arguments or typed state. Local mutation is permitted when it
keeps Rust ownership clearer or avoids a demonstrated cost without exposing hidden state.

Headless, MCP, TUI, and Bevy packages own effects. They may translate external input into core
commands and core events into external output, but they cannot decide authoritative game
outcomes. Content is validated into typed values supported by core and cannot add hidden
rules.

## Consequences

- Core can compile and test without Bevy, MCP, terminal runtimes, platform services, or
  authored-file formats.
- Protocol versioning can evolve without making wire representation the domain model.
- Adapter integration tests must compare both sides of each boundary.
- Some translations are explicit even when sharing one representation would require less
  code; the separation protects determinism and trust boundaries.
- No gameplay types are introduced by this Milestone 0 decision.
