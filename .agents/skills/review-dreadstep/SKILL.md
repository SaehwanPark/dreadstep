---
name: review-dreadstep
description: Review Dreadstep diffs, plans, or handoff artifacts for concrete domain correctness, determinism, dependency direction, functional-core boundaries, type safety, test evidence, comments, specifications, contributor experience, and lesson capture. Use after nontrivial changes or when a Dreadstep review is requested explicitly.
---

# Review Dreadstep

Evaluate whether a change satisfies its original contract and preserves the project's
domain boundaries. Review semantic coherence, not only whether commands pass.

## Required Inputs

- the original request and active `SPEC.md` item;
- the diff and affected implementation, tests, callers, and public contracts;
- `ARCHITECTURE.md`, applicable ADRs, `LESSONS.md`, and verification evidence;
- any `_workspace/<task-slug>/` handoff files created for the task.

## Workflow

1. Read the request and source specification before reading the producer's conclusion.
2. Compare both sides of every changed boundary:
   - core semantic types against protocol representations;
   - content definitions against typed runtime values;
   - adapter inputs and outputs against core commands and events;
   - documented state transitions against executable transitions;
   - comments and Rustdoc against signatures, failures, and tests.
3. Verify domain state and effects are explicit, invalid states are constrained by types,
   recoverable errors are structured, and mutation is local and justified.
4. Verify the test failed for the intended reason before implementation and now exercises
   success, failure, absence, boundary, and determinism cases proportional to the change.
5. Check names, functions, modules, comments, and docs for clarity to contributors from
   different Rust, game-development, and agent-tooling backgrounds.
6. Reconcile `SPEC.md`, `ARCHITECTURE.md`, `CHANGELOG.md`, ADRs, and `LESSONS.md` with the
   behavior supported by evidence. Do not accept speculative lessons or completion claims.
7. For player-facing changes, verify the runnable desktop executable, JSONL state/event mapping,
   smoke command/event coverage, and `docs/demo.md` matrix; ensure a new visible variant cannot
   silently bypass those gates.
8. Run the narrowest commands needed to verify findings and classify the result.

## Status Contract

- `pass`: no blocking issue; the change is ready to hand off.
- `fix`: bounded corrections are required and cheaper than redoing the change.
- `redo`: the approach violates the requested direction or a durable invariant.

Report findings by severity with exact evidence and the smallest safe correction. Do not
raise style preferences that are neither enforced nor material to maintainability.

## Outputs

- review status and revision count;
- blocking findings, follow-up improvements, and unverified areas kept separate;
- commands run and evidence inspected;
- documentation and lesson disposition.

## Stop Conditions

Stop and preserve uncertainty when required evidence is missing, sources of truth conflict,
or two targeted revision loops do not reach `pass`.
