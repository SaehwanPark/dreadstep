---
name: test-player
description: Play Dreadstep through its runnable 2D desktop showcases and visually evaluate observable feature behavior, regressions, usability, readability, pacing, feedback, and overall game experience. Use when a player-facing change or milestone needs hands-on visual inspection, exploratory playtesting, acceptance validation, or an evidence-backed feedback pass for a development or review agent. Do not use for headless-only correctness checks, code review, or implementing fixes.
---

# Test Player

Play the game as a player, collect visual and runtime evidence, and return concerns to the
development owner without changing the implementation under test.

## Required Inputs

- the request, target feature, acceptance criteria, or experience question;
- `AGENTS.md` and the current working-tree state;
- `docs/demo.md` for the canonical showcase command, controls, journal contract, and manual
  checklist;
- the relevant `SPEC.md`, `CHANGELOG.md`, diff, and tests when validating a feature;
- a graphical session plus an available UI-control and screenshot capability;
- any seed, scenario, time budget, or handoff path supplied by the coordinating agent.

Discover missing inputs from the repository. Do not guess a binary, control, fixture, or expected
behavior that has a documented source.

## Inspection Modes

Choose one mode before launching:

- **Feature inspection:** Default during early development or for a scoped change. Compare visible
  behavior with explicit acceptance criteria, including feedback, state transitions, and one
  relevant recovery or edge path.
- **Experience evaluation:** Use when asked about usability, readability, pacing, atmosphere,
  learning, decision quality, or fun. Perform a player-blind first pass before reading detailed
  implementation notes, then use repository evidence to investigate observations.
- **Combined:** Perform the experience pass first so implementation knowledge does not bias the
  first-play account, then run the targeted feature checks.

Label qualitative experience judgments as observations, not correctness defects. Prefer specific
moments and player impact over general taste.

## Workflow

### 1. Establish the Test Charter

1. Inspect `git status --short` and preserve all existing changes.
2. Identify the showcase, build, seed, requested behavior, and visible success criteria.
3. Read `docs/demo.md` immediately for feature inspection. For experience evaluation, initially
   read only enough to launch and control the game; defer the detailed spec and diff until after
   the blind pass.
4. Define a bounded route: startup, normal interaction, target behavior, one meaningful edge or
   recovery probe, and clean shutdown. Record any deliberate exclusions.

Use the current command documented in `docs/demo.md`. At present the primary interactive surface
is:

```sh
cargo run -p dreadstep-bevy --features desktop --bin dreadstep -- --seed 7
```

Treat that command as an example, not a second source of truth. If documentation and the runnable
surface disagree, report the mismatch rather than silently inventing a replacement.

### 2. Preflight and Launch

1. Run the documented display-free smoke command when startup risk or the requested change makes
   it useful. Record it only as preflight evidence; it never counts as visual playtesting.
2. Launch the interactive process from a persistent terminal session, retain its output, and wait
   for the game window to appear.
3. Use an available UI-control tool to focus and operate the window, following that tool's own
   instructions. Confirm that input reaches the game before starting the test route.
4. Capture the seed, command, build or commit identity, window state, and initial visual evidence.

If the game cannot build, open, render, or receive input, collect the exact failure and return a
`blocked` report. Do not replace the requested visual pass with source inspection or smoke output.

### 3. Play Through the Surface

1. Use documented player controls and follow the route as a player would. Do not use MCP tester
   mutations, debug state injection, save editing, or code changes to manufacture the target state
   unless the request explicitly requires them.
2. Observe both immediate feedback and the resulting state. Check whether the game communicates
   what happened, why it happened, and what the player can do next.
3. Capture before-and-after screenshots for important transitions when the environment supports
   it. Record exact input sequences for failures and high-impact experience concerns.
4. Exercise the normal path before destructive, losing, or terminal paths. Use restart with the
   same seed when reproducibility matters.
5. In experience mode, note discoverability, control confidence, readability, pacing, decision
   pressure, feedback, recovery, and enjoyment only where the play session provides evidence.

Avoid exhaustive key mashing. Expand beyond the charter only when a concrete observation needs a
small reproduction probe.

### 4. Corroborate and Classify

1. Exit through a documented clean-shutdown path when possible and confirm the process terminates.
2. Inspect only the journal, replay artifact, terminal output, and screenshots produced by this
   run. Use them to corroborate what was visible, not to overwrite the player observation.
3. Repeat a suspected defect once with the same seed and inputs unless repetition risks data,
   hangs the environment, or adds no evidence after a crash.
4. Classify the boundary only when evidence supports it:
   - **simulation/content:** the journal and visible result agree on an incorrect game outcome;
   - **presentation/input:** authoritative evidence is correct but the visible or interactive
     surface is absent, stale, misleading, or uncontrollable;
   - **process/environment:** build, windowing, audio device, asset, or tooling prevents the pass;
   - **unknown:** evidence is insufficient or contradictory.

Never diagnose from appearance alone when the journal can distinguish a presentation mismatch
from a semantic one. Preserve contradictory evidence explicitly.

### 5. Return the Handoff

Return the report through the current task to the coordinating or main agent. Do not implement a
fix, rewrite content, or broaden the specification. Write a durable `_workspace` artifact only
when the coordinator requests one or supplies a handoff path.

Use this structure:

```markdown
## Test-player report

- Verdict: pass | concerns | blocked
- Mode: feature | experience | combined
- Showcase/build: <command and revision or working tree>
- Seed/scenario: <value>
- Coverage: <routes and behaviors exercised>

### Confirmed observations
- <what visibly happened, with screenshot/journal/terminal evidence>

### Concerns
- [blocker|high|medium|low] [defect|experience|environment] <concise title>
  - Reproduce: <exact inputs and starting state>
  - Expected: <documented or clearly labeled player expectation>
  - Actual: <visible result>
  - Evidence: <artifact path, journal record, screenshot, or terminal excerpt>
  - Boundary: <simulation/content|presentation/input|process/environment|unknown>

### Experience notes
- <specific moment, player impact, and confidence; omit for feature-only runs>

### Unverified
- <anything skipped or made uncertain, and why>

### Recommended disposition
- <pass onward, investigate named concern, or rerun after named blocker>
```

Omit empty sections. Report `pass` only when the requested visual route ran and no material concern
was observed. Use `concerns` for completed runs with actionable issues and `blocked` when the
requested visual evidence could not be obtained.

## Severity Guide

- **blocker:** prevents launch, control, progression through the target, or safe continuation;
- **high:** breaks or seriously miscommunicates the requested feature or a core player decision;
- **medium:** creates material friction, ambiguity, inconsistency, or poor recovery;
- **low:** minor polish or consistency issue with limited player impact.

Keep optional ideas separate from concerns unless an observed player problem motivates them.

## Validation Checklist

- Operate a live 2D showcase; do not claim visual coverage from tests, logs, or static art alone.
- Cite the command, seed, route, and working-tree context needed to reproduce the pass.
- Support each defect with visible evidence and, when available, the matching journal or replay
  evidence.
- Distinguish confirmed defects, experience observations, environmental blockers, and unverified
  areas.
- Preserve the implementation under test and disclose any unavoidable test-state mutation.
- Shut down the game process and identify the generated evidence artifacts.

## Stop Conditions

Stop and report instead of improvising when no graphical/control capability is available, the
documented showcase cannot run, the requested state requires an unapproved mutation, evidence
conflicts without a safe discriminator, or continued play risks user data or the host environment.
