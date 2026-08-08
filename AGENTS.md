# Repository Agent Guide

## What

- Dreadstep is a deterministic tactical roguelike whose Rust domain model serves tests,
  headless tools, MCP agents, and a future Bevy client.
- `dreadstep-core` owns semantic game truth. Protocol, content, headless, MCP, and Bevy are
  boundaries around it; see `ARCHITECTURE.md`.

## Why

- Simulation-first boundaries keep gameplay reproducible and testable without rendering.
- Specs, tests, types, and verified lessons preserve decisions across contributors and
  agent sessions.

## How

- Before changing behavior, read `SPEC.md`, `ARCHITECTURE.md`, applicable ADRs, tests, and
  `LESSONS.md`.
- Use spaces only with an indentation and tab width of 2.
- Develop through `.agents/skills/develop-dreadstep/SKILL.md`; use the review skill when
  its risk gate applies. Coordination details live in `docs/harness/dreadstep/team-spec.md`.
- Run `scripts/verify.sh` before handing off a nontrivial change.
- Reconcile affected specs, architecture, changelog entries, and verified lessons with the
  implementation evidence.
