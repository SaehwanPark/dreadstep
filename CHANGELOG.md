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

### Changed

- Replaced the root Bevy starter binary with package shells for Milestone 0.
- Limited Bevy to the presentation package and its minimal standard-library feature set.
