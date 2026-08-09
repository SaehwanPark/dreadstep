# Changelog

All notable contributor- and user-visible project changes are recorded here.

## Unreleased

### Added

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
  `observe` tools; broader actions and tester mutations remain library-only.

### Changed

- Replaced the root Bevy starter binary with package shells for Milestone 0.
- Limited Bevy to the presentation package and its minimal standard-library feature set.
