# Dreadstep Architecture

Last Reviewed: 2026-08-09
Status: Verified

## Overview

Dreadstep is organized as a functional domain kernel surrounded by explicit adapters. The
kernel decides game outcomes; adapters translate external input into semantic commands and
translate semantic events into presentation, files, telemetry, or transport responses.

Milestone 1 now exposes the first gameplay API in `dreadstep-core`: a typed rectangular map,
actors, movement, melee, and chase commands, semantic movement/blocking/combat/death events,
an integer ready-time scheduler, and core-owned replay traces/state digests. The
`dreadstep-headless` adapter now provides a fixed-scenario developer CLI that translates text
arguments into those core commands; it owns parsing and stdout only. `dreadstep-mcp` also provides
a minimal local stdio server for the bounded player tools `start_run`, `observe`, `legal_actions`,
`inspect`, `get_history`, `get_replay`, and typed `act`; no graphical client exists yet.

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
and content must never depend on Bevy or MCP runtime libraries. Bevy currently enables only its
`std` and `keyboard` features, so headless Linux checks do not require desktop system libraries.

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
actor records remain inspectable but are removed from scheduling and movement occupancy. The
stable state digest uses an explicit deterministic byte order and does not use a
process-randomized standard hasher. No wall-clock time or process-global randomness
participates in these transitions.

The headless CLI must not duplicate movement, combat, chase, scheduling, or digest logic. Its
fixed scenario is test data at the adapter boundary; every outcome still comes from
`dreadstep-core::WorldState::execute`.

The first Milestone 2 protocol slice is a read-only `WorldSnapshot` projection. It may expose
stable actor data and core digest evidence, but it must not decide legal actions or mutate the
world; the minimal MCP stdio observation server now packages this projection, while broader
transport and session effects remain later adapter work.

The protocol action envelope is likewise only a typed conversion boundary: it can translate
external request values into canonical core commands and back, but command validation and
execution remain owned by `dreadstep-core::WorldState`.

The first MCP player slice is an in-memory session over those protocol values. It owns session
seed/scenario setup and response shaping only; the minimal stdio server wraps its `start_run`,
`observe`, `legal_actions`, `inspect`, `get_history`, `get_replay`, and typed `act` operations without duplicating
core transition rules. Additional transports and tester operations remain future slices.

Legal-action discovery is a core query, not an MCP policy: `WorldState::legal_commands` decides
which typed commands are currently valid, and the session only maps those commands into protocol
requests without mutating state.

Session history is an adapter-owned view over core `ReplayTrace`: accepted commands are recorded
after successful execution, rejected requests are omitted, and only protocol requests plus the
core digest value cross the MCP boundary.

The `get_replay` projection packages that history, explicit seed, and core trace digest in a
protocol-owned `ReplayEvidence` value. It remains an in-memory read-only view; persistence,
serialization, playback, and transport registration stay outside this slice.

The player `inspect` operation is likewise a read-only lookup over the protocol world snapshot.
It returns one protocol `ActorSnapshot` or no value for an unknown identity, preserves dead actor
records for inspection, and adds no visibility policy or gameplay behavior.

The proposal's `get_history` name maps to the same adapter-owned protocol request projection as
`Session::history`; the alias does not create a second source of truth or alter core trace
recording.

Tester savepoints are explicit in-memory `SessionSnapshot` values containing the session seed,
core world, and core replay trace. `restore` replaces that owned state, so branching and rollback
remain deterministic without exposing arbitrary mutation or adding storage effects.

The tester `inspect_world` name maps to the same protocol `WorldSnapshot` projection as player
`observe`; it is a read-only alias and does not create separate world storage or hidden rules.

Tester spawning crosses the boundary as a typed request to core `WorldState::spawn`. Core validates
identity, map, living occupancy, and hit points; MCP only converts protocol values and projects
typed world errors, preserving atomic failure and one source of game truth.

Tester hit-point mutation crosses the boundary as a typed request to core
`WorldState::set_hit_points`. Core owns dead-record retention, living occupancy, and scheduler-safe
reanimation at the current action time; MCP only converts the request and projects an unknown-actor
world error. Tester mutations remain outside accepted player history and replay evidence.

Tester scenario replacement crosses the boundary as a protocol-owned `Scenario` value. MCP maps
its tiles and actor specs into `GridMap` and `Actor` values, then delegates all map and world
validation to `WorldState::new` before replacing the session. Failed construction is atomic;
successful replacement preserves the seed and starts a fresh in-memory replay trace.

Opaque tester item ownership crosses the boundary as typed `ItemId` and `ItemDefinitionId` values.
Core owns global identity uniqueness, ordered actor inventories, digest inclusion, and snapshot
projection; MCP only converts the request to `WorldState::give_item`. Effects, equipment, capacity,
transfer, and content catalogs remain outside this slice so no adapter invents item truth.

Validated tester teleport crosses the boundary as a typed actor identity and destination position.
Core owns bounds, terrain, living occupancy, and preservation of scheduler/inventory state; MCP only
converts the request and projects typed world errors. Dead actor records remain non-occupying, and
the mutation does not enter player history or replay evidence.

The minimal MCP stdio slice adds a process adapter around the existing session. The adapter owns
`rmcp` transport setup, tool schemas, and versioned JSON serialization for the bounded player tools
`start_run`, `observe`, `legal_actions`, `inspect`, `get_history`, `get_replay`, and typed `act`;
session and core
remain authoritative for seeded state and world truth. Stdout is reserved for MCP protocol traffic,
and tester mutations remain library-only.

The first Milestone 3 Bevy slice is a deterministic presentation bridge. `GridMap::tiles` gives
the adapter an immutable row-major terrain projection, while `dreadstep-bevy::PresentationState`
owns a core world and replay trace and exposes map/actor/time/digest snapshots. Its keyboard intent
mapping produces canonical core movement and wait commands for an explicit actor, and accepted
commands delegate to `WorldState::execute`; rejected commands are not recorded. The bridge is
headless-testable and enables only Bevy's keyboard feature, so windowing, rendering, assets, and
audio remain later presentation slices.

The shared authored starter-floor slice adds `dreadstep-content::StarterFloorDefinition` and its
validated `starter_floor` constructor. Content owns the row-major map and initial actor records,
then delegates all dimension, terrain, identity, occupancy, and life validation to core.
`PresentationState::start_run` consumes that constructor and preserves the caller's seed; MCP and
future clients may choose their own adapter scenarios without making content or presentation
state authoritative over the rules kernel.

The headless scene-synchronization slice projects a complete `PresentationSnapshot` into disposable
`SceneTile` and `SceneActor` ECS components. The synchronizer keys entities by stable map position
and `ActorId`, preserves identity across updates, removes stale or duplicate keys deterministically,
and mirrors actor position, life, hit points, and scheduler readiness. Dead actor records remain
represented because core snapshots retain them. ECS data is a render mirror only; it cannot issue
commands or replace `WorldState` as game truth.

The headless application-shell slice adds `PresentationRuntime` as the sole Bevy resource owning a
`PresentationState`, and `PresentationPlugin` as an exclusive update system that clones a runtime
snapshot before calling `sync_scene`. This keeps the core-backed resource borrow separate from ECS
mutation while making startup and post-command projection automatic for a Bevy `App`. The plugin
still enables no window, rendering, audio, or desktop platform features; command submission remains
an explicit runtime API and never originates from scene components. Its keyboard-dispatch extension
uses an explicit `PresentationInput` actor and fixed key priority, consumes one frame's supported
just-pressed keys deterministically, delegates through core, and projects before the update ends.

The typed MCP player-action slice extends that same process boundary with JSON command requests and
structured `SessionOutput` event/snapshot evidence. MCP maps invalid command results to protocol
errors only; core still owns scheduling, target validation, semantic events, and replay recording.
Tester mutations remain outside the process wire contract.

The legal-action MCP slice exposes `Session::legal_actions` as a no-argument, read-only tool. Core
selects the scheduled actor and deterministic command order; the MCP adapter only serializes the
typed protocol request array. A legal-action call cannot mutate world, history, or replay state.

The actor-inspection MCP slice exposes `Session::inspect` through a typed actor-ID parameter. The
session performs the existing snapshot lookup and returns an `ActorSnapshot` or `None`; MCP only
serializes that visible projection and does not invent hidden-information or visibility rules.

The accepted-history MCP slice exposes `Session::get_history` as a no-argument, read-only array of
protocol requests. The session remains the adapter-owned view over core replay recording; MCP does
not expose `ReplayTrace` internals or add a second history source.

The replay-evidence MCP slice exposes `Session::get_replay` as a no-argument, read-only structured
`ReplayEvidence` value. Protocol owns its JSON/JSON Schema projection; the MCP adapter does not add
persistence or playback semantics and core remains authoritative for the digest.

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
