# Dreadstep Development Harness

## Goal

Provide a portable, direct-first workflow that turns a bounded request into verified code
and synchronized project state without weakening Dreadstep's domain boundaries.

## Architecture Pattern

Use a sequential pipeline with a conditional producer-reviewer gate. One development owner
normally carries context from specification through handoff. A reviewer phase is required
only for explicit review requests or the higher-risk boundaries listed below.

## Roles

| Role | Responsibility | Skill |
| --- | --- | --- |
| Development owner | Scope, spec, tests, implementation, refactor, docs, verification | `$develop-dreadstep` |
| Semantic reviewer | Cross-boundary coherence, evidence, and final disposition | `$review-dreadstep` |
| Test player | Hands-on visual feature inspection and experience feedback through runnable 2D showcases | `$test-player` |

The development owner remains the synthesis and acceptance owner. A reviewer reports
findings but does not silently expand scope or take over implementation. The test player reports
evidence-backed observations and concerns without modifying the surface under test; the development
owner decides their disposition.

## Phase Order

1. **Recover context:** read the request, repo state, spec, architecture, ADRs, tests, and
   lessons. Output a bounded task statement.
2. **Specify behavior:** record observable acceptance, exclusions, and verification in
   `SPEC.md`. Output an active spec item.
3. **Establish red evidence:** add and run a focused failing test or deterministic check.
   Output the command and intended failure.
4. **Implement and refactor:** make the smallest typed change, keep effects at adapters,
   reach green, and improve names or structure only inside the agreed slice.
5. **Reconcile knowledge:** update only affected docs and revisit `LESSONS.md`. Output a
   truthful documentation disposition.
6. **Review and verify:** apply the risk gate, run required checks, and issue `pass`, `fix`,
   or `redo`. Output final evidence and unresolved concerns.

For a player-facing slice with a runnable 2D surface, use `$test-player` when visual inspection is
requested or when acceptance depends on player-visible behavior or experience. Treat its report as
input to the development owner; do not substitute display-free smoke output for its visual pass.

## Review Gate

Require semantic review for changes involving:

- multiple packages or both sides of an integration boundary;
- public types, commands, events, errors, serialization, or versioned formats;
- determinism, scheduling, randomness, replay, snapshots, or state digests;
- MCP player/tester visibility or host-access boundaries;
- dependencies, toolchains, CI, unsafe-code requests, or architecture constraints.

Small local code changes and trivial documentation edits use author review and proportional
checks. Do not create coordination artifacts when they add no inspection or recovery value.

## Durable Handoffs

For nontrivial, resumable, or reviewed tasks, use:

```text
_workspace/<task-slug>/
  00-request.md
  01-spec-and-tests.md
  02-review.md
  99-handoff.md
```

The request records scope and exclusions. Test evidence records the red and green commands.
Review cites the original request and exact changed boundary. The handoff lists changed
files, checks, deviations, unresolved concerns, and the lessons disposition. `_workspace`
is intentionally ignored by Git.

## Failure and Revision Policy

- Retry a transient command once only when the cause is understood and no state is hidden.
- Use `fix` for bounded corrections and `redo` when the approach violates intent or an
  invariant.
- Stop after two targeted revision loops and report unresolved blockers.
- If required evidence is unavailable, label the area unverified rather than inferring a
  successful result.
- If sources of truth conflict, preserve both and request a decision; do not choose silently.

## Delegation and Concurrency

Keep work direct by default. Independent read-only exploration, review angles, or isolated
tests may be delegated when specialization or context isolation has clear value. Never run
parallel writers against overlapping paths or stateful tests against shared mutable
resources. Use isolated checkouts when parallel writes are explicitly authorized.

## Validation Scenarios

The committed cases in `evals/cases.json` cover core development, an adapter boundary, a
trivial near-miss, a public replay contract, and an invariant conflict. Evaluate the
selection boundary and workflow behavior after meaningful skill changes. Store generated
outputs and grading under `_workspace/harness-evals/`; do not train the canonical skill to
one fixture or invent evidence for a failed case.
