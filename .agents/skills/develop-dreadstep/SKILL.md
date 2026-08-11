---
name: develop-dreadstep
description: Develop or refactor nontrivial Dreadstep Rust code, domain rules, adapters, tests, tooling, architecture, and contributor documentation. Use when a repository change needs spec-first TDD, functional-core boundaries, typed domain modeling, deterministic behavior, careful comments, cross-document synchronization, or verified lesson capture. Do not use for isolated typo or formatting-only edits, or for generic Rust questions unrelated to Dreadstep.
---

# Develop Dreadstep

Deliver the smallest complete change while keeping the game model deterministic,
testable, and independent of presentation and transport effects.

## Required Inputs

- the user request and current diff;
- `AGENTS.md`, `SPEC.md`, `ARCHITECTURE.md`, and `LESSONS.md`;
- applicable ADRs, proposal sections, callers, and tests;
- the verification commands in `CONTRIBUTING.md`.

## Workflow

1. Recover repository state and read `LESSONS.md` before proposing or editing behavior.
2. Classify the change by owning domain and adapter boundary. Stop if the requested owner
   conflicts with `ARCHITECTURE.md` or an ADR.
3. Add or update a concise `SPEC.md` Present item with observable behavior, exclusions,
   and verification.
4. Add the narrowest failing behavioral test or deterministic check. Run it and confirm
   the failure describes the missing behavior rather than broken setup.
5. Implement only enough to pass:
   - keep domain transformations pure when practical;
   - pass state, time, randomness, and configuration explicitly;
   - use Rust structs, newtypes, enums, `Option`, and typed `Result` errors;
   - keep filesystem, process, logging, MCP, and Bevy effects in adapters;
   - allow localized mutation when it is clearer or measurably cheaper.
6. Refactor under passing tests. Prefer domain-revealing names and cohesive functions over
   speculative abstractions or dense combinator chains.
7. Add comments only for rationale, rules, invariants, contracts, ownership, or safety.
   Define project-specific terms for contributors who do not share the author's context.
8. Re-read `LESSONS.md`, update an existing lesson or add a verified recurring lesson,
   and reconcile affected specs, architecture, ADRs, and changelog entries.
9. For player-facing changes, update the runnable `dreadstep-bevy` desktop path, its JSONL
   journal mapping, display-free smoke coverage, and `docs/demo.md`; record an explicit
   `SPEC.md` deferral when the change is intentionally outside the showcase.
10. Run `scripts/verify.sh` or a documented proportional subset for a trivial docs-only
   change. Review the diff for scope and semantic coherence.

## Domain Routing

- `dreadstep-core`: semantic state, commands, events, errors, and deterministic rules.
- `dreadstep-protocol`: versioned external representations and conversion to core types.
- `dreadstep-content`: validated authored data converted into typed domain values.
- `dreadstep-headless`: CLI, file, process, telemetry, and batch-run effects.
- `dreadstep-mcp`: bounded player/tester operations; never arbitrary host access.
- `dreadstep-bevy`: human input and presentation; never authoritative game rules.

## Review Gate

Use `$review-dreadstep` after changes involving multiple crates, public contracts,
determinism, RNG, replay, serialization, MCP trust boundaries, dependencies, unsafe-code
requests, or toolchain configuration. A low-risk local change may use author review.

For a nontrivial or resumable task, follow the handoff contract in
`docs/harness/dreadstep/team-spec.md`. Do not create `_workspace` files for trivial work.

## Outputs

- implementation and behavioral evidence;
- synchronized operational documentation;
- a handoff listing files changed, checks run, deviations, unresolved concerns, and the
  `LESSONS.md` disposition.

## Stop Conditions

Stop instead of improvising when the request conflicts with a documented invariant,
requires an unapproved public contract or dependency, cannot produce truthful tests or
documentation, or exceeds two review revision loops.
