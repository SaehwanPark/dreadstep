# Dreadstep Architecture

Last Reviewed: 2026-08-13
Status: Verified

## Overview

Dreadstep is a functional domain kernel surrounded by explicit adapters. The kernel decides
game outcomes; adapters translate external input into semantic commands and translate semantic
events into presentation, files, telemetry, or transport responses.

The current tree exposes a headless CLI, a bounded MCP stdio server, and a feature-gated Bevy
desktop showcase around the same `dreadstep-core` world. Verified capabilities are indexed in
[`SPEC.md`](SPEC.md) Present.

## Package Ownership

| Package | Owns | Must not own |
| --- | --- | --- |
| `dreadstep-core` | Domain state, commands, events, errors, deterministic rules | I/O, Bevy, MCP, authored-file formats |
| `dreadstep-protocol` | Versioned external representations and conversions | Domain decisions or transports |
| `dreadstep-content` | Validation of authored definitions into domain values | Hidden simulation rules |
| `dreadstep-headless` | CLI, files, processes, telemetry, batch execution | Authoritative game behavior |
| `dreadstep-mcp` | Bounded player and tester operations | Arbitrary host access or game truth |
| `dreadstep-bevy` | Headless projection plus optional desktop input, window/render setup, HUD, assets, and journal | Authoritative state or rules |

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
and content must never depend on Bevy or MCP runtime libraries. `dreadstep-bevy` keeps the
headless feature graph minimal; its opt-in `desktop` feature adds Bevy's winit, X11, 2D render,
UI/text, nearest-neighbor image, optional audio playback, and logging capabilities while continuing
to exclude Wayland and `default_platform`.

## Intended Data Flow

```text
external input -> adapter -> core command -> deterministic transition
                                           -> next state + semantic events
semantic events -> adapter -> output, presentation, telemetry, or protocol response
```

Desktop timers, HUD text, asset handles, animation/audio, and JSONL journals are disposable
effects. Only `WorldState::legal_commands` and `WorldState::execute` determine simulation
outcomes. Display-free `--smoke` reuses those helpers without winit or a renderer.

State, configuration, seeded randomness, and time inputs should be explicit. Prefer pure
transformations and returned outcomes; allow tightly scoped mutation when it is clearer or
materially more efficient in Rust.

## Current Invariants

- `dreadstep-core` owns `WorldState` transitions, occupancy, scheduling, combat, inventory,
  environmental tiles, terrain-aware one-use kick-noise hearing, canonical `RunOutcome`, replay traces, and
  the stable state digest.
  The digest uses an explicit deterministic byte order, not a process-randomized hasher.
- Protocol v21 projects those values, including actor status snapshots and typed status events. MCP,
  headless, and Bevy convert types and shape I/O;
  they must not reimplement rules, legal-action policy, or terminal-outcome predicates.
- Bevy ECS mirrors, FOV, enemy-intent, HUD, and desktop session state are presentation-only.
  Missing optional resources are no-ops or recorded fallbacks.
- Content validates authored and generated floors into core values; connectivity checks stay
  at that boundary.
- See [`SPEC.md`](SPEC.md) Present for the current capability summary and
  [`docs/demo.md`](docs/demo.md) for desktop controls.

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
