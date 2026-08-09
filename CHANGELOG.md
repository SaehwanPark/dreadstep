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

### Changed

- Replaced the root Bevy starter binary with package shells for Milestone 0.
- Limited Bevy to the presentation package and its minimal standard-library feature set.
