# Changelog

All notable contributor- and user-visible project changes are recorded here.

## Unreleased

### Added

- Aspirational concept art reference and a documented pixel-art/audio asset sourcing and
  licensing workflow.
- A six-package Rust workspace with explicit domain and adapter boundaries.
- A repo-local Dreadstep development and review harness.
- Reproducible Rust, formatting, lint, documentation, and CI configuration.
- Operational specification, architecture, contribution, ADR, and lessons documentation.
- The first Milestone 1 rules-kernel slice: typed grid state, actors, movement and blocking
  events, and deterministic integer action scheduling.
- Typed hit points, fixed basic melee attacks, and semantic death events with dead actors
  removed from scheduling and movement occupancy.
- Deterministic enemy chase commands with explicit axis tie-breaking and shared blocking
  events.
- Core-owned replay traces and stable state digests for deterministic regression evidence.
- A deterministic `dreadstep-headless` developer CLI that translates command tokens into the
  core simulation and prints replay seed, events, and final state digest evidence.
- A versioned `dreadstep-protocol` world snapshot projection for deterministic agent
  observation, without adding an MCP transport runtime.
- A versioned protocol action envelope that maps typed agent requests to and from core commands
  without executing them.
- A pure in-memory MCP player session for deterministic start, observe, and act flows without a
  transport runtime.
- Core-owned deterministic legal-action discovery exposed through the in-memory MCP session.
- Accepted-action session history and deterministic replay digest evidence backed by core
  `ReplayTrace`.
- A typed in-memory `get_replay` evidence bundle exposing seed, accepted protocol requests, and
  deterministic digest without persistence or transport serialization.
- Read-only player actor inspection over the versioned snapshot, including retained dead actor
  records and an explicit unknown-actor result.
- Named `get_history` access to the existing protocol-owned accepted-request history projection.
- In-memory tester `snapshot`/`restore` savepoints that preserve world and replay state without
  persistence or transport effects.
- Named tester `inspect_world` access to the existing complete protocol world snapshot.
- Validated tester actor spawning through core with protocol-owned world error projections.
- Validated tester hit-point mutation through core, including dead-record retention and
  scheduler-safe reanimation, with typed protocol error projection.
- Typed in-memory tester scenario replacement backed by core map and world validation, with
  atomic failure and a fresh replay trace for the preserved seed.
- Opaque typed tester item ownership with deterministic actor inventory snapshots and duplicate
  identity validation; gameplay effects and inventory capacity remain deferred.
- Validated tester teleport with typed destination validation, dead-record occupancy semantics, and
  no player-trace effects.
- Minimal local MCP stdio observation with versioned snapshot JSON, `start_run`, and read-only
  `observe` tools.
- Typed MCP player `act` requests with explicit command/event JSON and schemas, structured action
  evidence over stdio, and invalid-params rejection that leaves state and replay unchanged.
- Read-only MCP `legal_actions` discovery over stdio with core-defined deterministic ordering and
  no world, history, or replay mutation.
- Read-only MCP actor inspection over stdio with typed IDs, structured snapshot-or-null results, and
  no world, history, or replay mutation.
- Read-only MCP `get_history` accepted-request evidence over stdio with deterministic ordering,
  rejection omission, and no world, history, or replay mutation.
- Read-only MCP `get_replay` replay evidence over stdio with typed seed, accepted requests,
  deterministic digest, explicit JSON/JSON Schema, and no persistence or playback semantics.
- A deterministic Bevy presentation bridge with immutable map/actor snapshots, keyboard intent
  mapping, core command execution, and replay evidence without enabling desktop platform features.
- A validated authored starter-floor definition in `dreadstep-content` and a Bevy `start_run` path
  that delegates to it while preserving the explicit replay seed.
- Headless Bevy scene synchronization for deterministic map-tile and actor ECS mirrors, including
  stable entity identity, stale-entity removal, and retained dead-record presentation.
- A headless Bevy `PresentationRuntime` resource and `PresentationPlugin` that automatically project
  core-backed snapshots into the scene after each app update without desktop engine features.
- Deterministic headless keyboard dispatch with explicit controlled-actor selection, fixed key
  priority, one-command-per-update consumption, and same-update scene projection.
- A one-shot `PresentationRuntime` feedback buffer for accepted typed event/snapshot evidence, with
  stale-output clearing on rejected commands and no new authoritative state.
- A typed headless `PresentationFocus` projection that mirrors the selected actor's position for
  future camera systems without adding viewport, visibility, or rendering policy.
- A typed headless `SceneFocus` marker that reuses the stable keyed actor entity for future camera
  or selection systems without copying actor state or adding marker visuals.

### Changed

- Replaced the root Bevy starter binary with package shells for Milestone 0.
- Limited Bevy to the presentation package and its minimal standard-library feature set.
