# Dreadstep Architecture

Last Reviewed: 2026-08-08  
Status: Verified

## Overview

Dreadstep is organized as a functional domain kernel surrounded by explicit adapters. The
kernel decides game outcomes; adapters translate external input into semantic commands and
translate semantic events into presentation, files, telemetry, or transport responses.

Milestone 1 now exposes the first gameplay API in `dreadstep-core`: a typed rectangular map,
actors, movement, melee, and chase commands, semantic movement/blocking/combat/death events,
and an integer ready-time scheduler. The API is still headless and does not provide a runnable
application.

## Package Ownership

| Package | Owns | Must not own |
| --- | --- | --- |
| `dreadstep-core` | Domain state, commands, events, errors, deterministic rules | I/O, Bevy, MCP, authored-file formats |
| `dreadstep-protocol` | Versioned external representations and conversions | Domain decisions or transports |
| `dreadstep-content` | Validation of authored definitions into domain values | Hidden simulation rules |
| `dreadstep-headless` | CLI, files, processes, telemetry, batch execution | Authoritative game behavior |
| `dreadstep-mcp` | Bounded player and tester operations | Arbitrary host access or game truth |
| `dreadstep-bevy` | Input and presentation | Authoritative state or rules |

## Dependency Direction

```text
protocol ----> core <---- content
                  ^
                  |
       +----------+----------+
       |          |          |
    headless     MCP       Bevy
```

The adapter packages may depend on protocol and content as well as core. Core, protocol,
and content must never depend on Bevy or MCP runtime libraries. Bevy currently enables
only its `std` feature so headless Linux checks do not require desktop system libraries.

## Intended Data Flow

```text
external input -> adapter -> core command -> deterministic transition
                                           -> next state + semantic events
semantic events -> adapter -> output, presentation, telemetry, or protocol response
```

State, configuration, seeded randomness, and time inputs should be explicit. Prefer pure
transformations and returned outcomes; allow tightly scoped mutation when it is clearer or
materially more efficient in Rust.

## Current kernel slice

`dreadstep-core` owns the canonical `WorldState`. `GridMap` limits dimensions to the signed
`Position` coordinate domain and treats out-of-bounds and wall tiles as terrain blockers;
living actor occupancy is checked separately so events can distinguish terrain from another
actor. `WorldState::execute` accepts only the living actor at the minimum `ActionTime`, orders
ties by `ActorId`, applies fixed melee damage to adjacent targets, resolves enemy chase steps
with horizontal-axis priority, and advances the acting actor by the fixed action cost. Dead
actor records remain inspectable but are removed from scheduling and movement occupancy. No
wall-clock time or process-global randomness participates in these transitions.

## Constraints

- Core owns canonical semantic commands, events, and domain errors.
- Protocol owns versioning and external representation, not domain semantics.
- Rendering, ECS scheduling, wall-clock time, host randomness, and transport state cannot
  determine authoritative game outcomes.
- `unsafe` code is forbidden at the workspace level. A future exception requires an ADR,
  evidence, and explicit review before changing that policy.
- Public concepts should use typed domain representations rather than strings, boolean
  modes, or unvalidated map-shaped data.
- See `docs/adr/0001-functional-core-and-adapters.md` for the decision rationale.
