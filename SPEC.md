# Dreadstep Specification

This file tracks verified project state. The broader product vision and roadmap live in
[`docs/dreadstep-proposal.md`](docs/dreadstep-proposal.md).

## Past

- The Dreadstep proposal established a deterministic, simulation-first tactical roguelike
  built with Rust, Bevy, and an eventual MCP testing interface.
- The project adopted the MIT license and a Rust 2024 starter package.

### Milestone 0: Project charter and development harness

- Status: verified
- Completed: 2026-08-08

The repository provides a portable development/review harness, a compiling six-package
Cargo workspace, explicit domain boundaries, contributor guidance, and reproducible
verification without requiring the long-form proposal to act as operational state.

Verification:

- `scripts/verify.sh` passes on Linux without desktop Wayland, X11, or audio packages.
- Cargo metadata reports the six declared workspace packages and no root package.
- `dreadstep-core`, `dreadstep-protocol`, and `dreadstep-content` have no Bevy or MCP
  dependencies.
- Rustfmt and EditorConfig require spaces with an indentation and tab width of 2.
- The repo-local development and review skills pass structural validation.
- CI performs full Linux verification plus native Apple Silicon macOS and Windows checks.
- README, architecture, contribution, changelog, lesson, ADR, and agent guidance agree
  with the verified repository state.

Evidence:

- `scripts/verify.sh` passed on Linux/WSL2 with Rust 1.97.1.
- The skill-creator validator accepted both repo-local skills.
- Cargo metadata reported exactly six workspace members and no root package.
- The minimal Bevy feature graph contained no audio, default-platform, Wayland, or X11
  feature.

Out of scope:

- gameplay domain types or rules;
- runnable headless, MCP, or Bevy clients;
- content, replay, or wire schemas;
- `rmcp`, rendering, windowing, input, or audio dependencies;
- release packaging or deployment.

## Present

### Milestone 1 slice: deterministic grid movement and scheduling

- Status: verified
- Started: 2026-08-08
- Completed: 2026-08-08

The rules kernel begins with a typed rectangular grid, actor identity and position, terrain
and actor blocking, and an integer action scheduler. A command addressed to the scheduled
actor either moves it to an unoccupied floor tile or reports why the move was blocked; both
outcomes consume the same deterministic movement action cost. The scheduler orders actors by
ready time and then actor identity, so the same initial state and command sequence produce the
same events and state.

Acceptance:

- `dreadstep-core` exposes typed map, actor, position, command, event, and scheduling values.
- Invalid map dimensions (including dimensions outside the signed position range), tile
  data, duplicate actor identities, overlapping actors, and out-of-bounds or blocking
  movement are rejected with structured errors or events.
- A scheduled actor can move or wait; each action advances its ready time by the fixed
  movement cost and the next scheduled actor is observable.
- Unit tests cover successful movement, terrain blocking, actor blocking, deterministic
  scheduler ordering, and command rejection for an unscheduled actor.
- Core remains independent of Bevy, MCP, filesystem, wall-clock time, and host randomness.

Verification:

- Focused `cargo test -p dreadstep-core` passes with the slice tests.
- `scripts/verify.sh` passes before handoff.

Out of scope:

- HP changes, melee combat, death, enemy chase behavior, seeded randomness, replay schemas,
  and the developer CLI; each is a later Milestone 1 slice with its own acceptance evidence.

### Milestone 1 slice: basic melee combat and death

- Status: verified
- Started: 2026-08-08
- Completed: 2026-08-08

Extend the deterministic core with typed hit points and a fixed basic melee attack. A
scheduled living actor may attack one adjacent living actor, reducing its hit points by the
fixed melee damage and emitting semantic attack evidence. Reaching zero hit points emits a
death event and removes the actor from scheduling and movement occupancy while retaining the
dead actor record for inspection.

Acceptance:

- Actors expose typed hit points and living/dead state; worlds reject actors that start dead.
- A basic melee command requires an adjacent living target and consumes the same standard
  action cost as movement and waiting.
- Successful attacks emit attacker, target, damage, and remaining-hit-point evidence.
- Attacks that reach zero hit points emit a death event; dead actors are not selected by the
  scheduler and do not block movement.
- Structured command errors cover unknown, dead, self, and out-of-range targets.
- Core remains independent of Bevy, MCP, filesystem, wall-clock time, and host randomness.

Verification:

- Focused `cargo test -p dreadstep-core` covers attack success, death, scheduler removal,
  occupancy removal, and invalid target cases.
- `scripts/verify.sh` passes before handoff.

Out of scope:

- Variable weapons or damage, armor or resistances, status effects, enemy chase behavior,
  seeded randomness, replay schemas, and the developer CLI.

### Milestone 1 slice: deterministic enemy chase

- Status: verified
- Started: 2026-08-08
- Completed: 2026-08-08

Add a deterministic chase command for enemy actors. The command selects one cardinal step
toward a living target using horizontal-axis priority when both axes differ, then reuses the
same terrain and living-actor blocking rules as movement. A blocked chase still consumes the
standard action and emits the existing blocking event; invalid actor or target roles return
structured errors.

Acceptance:

- Only living enemy actors may issue a chase command, and the target must be a distinct living
  actor in the world.
- Chase direction is deterministic: horizontal movement wins a diagonal tie, with east/west
  selected from the target's relative position and north/south used when columns align.
- Successful chase emits the normal movement event and consumes the standard action cost.
- Terrain and living-actor blocking emit the normal blocking event and consume the action;
  dead actors do not block chase movement.
- Structured command errors cover a player chase, self-target, unknown target, and dead target.
- Core remains independent of Bevy, MCP, filesystem, wall-clock time, and host randomness.

Verification:

- Focused `cargo test -p dreadstep-core` covers diagonal tie-breaking, movement, blocking, and
  invalid chase cases.
- `scripts/verify.sh` passes before handoff.

Out of scope:

- Pathfinding around obstacles, multiple-step planning, ranged behavior, enemy archetypes,
  seeded randomness, replay schemas, and the developer CLI.

### Milestone 1 slice: deterministic replay evidence

- Status: verified
- Started: 2026-08-08
- Completed: 2026-08-08

Add core-owned replay evidence without introducing a wire format or external effects. A replay
trace records an explicit seed and ordered semantic commands, and a stable state digest covers
the map, living/dead actor state, positions, hit points, ready times, and current action time.
The digest and trace identity use a documented deterministic algorithm rather than a
process-randomized standard hasher.

Acceptance:

- `ReplayTrace` exposes a seed, ordered commands, append behavior, and a deterministic trace
  digest; command order and seed changes affect the digest.
- `WorldState::digest` returns the same `StateDigest` for identical initial state and command
  sequences across independently constructed worlds.
- The state digest includes terrain, actor identity/kind/life, position, hit points, ready
  time, and current action time so meaningful state changes alter evidence.
- Replay evidence remains core-only and does not claim to be a serialized protocol, cryptographic
  integrity check, or complete replay runner.
- Core remains independent of Bevy, MCP, filesystem, wall-clock time, and host randomness.

Verification:

- Focused `cargo test -p dreadstep-core` covers trace ordering/seed sensitivity and equivalent
  state digests after movement and combat transitions.
- `scripts/verify.sh` passes before handoff.

Out of scope:

- Serialized replay files, protocol versioning, RNG implementation, replay playback/CLI,
  scenario storage, and cryptographic hashes.

### Milestone 1 slice: deterministic headless CLI

- Status: verified
- Started: 2026-08-08
- Completed: 2026-08-08

Add a small `dreadstep-headless` executable that demonstrates the adapter boundary without
owning game rules. It accepts an explicit seed and ordered semantic command tokens for a fixed
developer scenario, translates them into `dreadstep-core::Command` values, executes them, and
prints the seed, event debug output, and final state digest. Invalid arguments and rejected
core commands return structured errors and a non-success process result.

Acceptance:

- The binary runs without Bevy and accepts `--seed <u64>` plus a comma-separated `--commands`
  value for movement, waiting, melee, and chase commands.
- Parsing is deterministic and rejects missing, duplicate, malformed, or unknown arguments and
  command tokens without panicking.
- The fixed scenario is explicit in headless code; command execution delegates all outcomes to
  `dreadstep-core`, and output includes the supplied seed and final `StateDigest` value.
- Unit tests cover parsing success/failure and an end-to-end command sequence; a subprocess
  smoke test proves the binary exits successfully for a valid run.
- The adapter owns process/stdout effects only; it does not add authoritative game behavior or
  Bevy/MCP dependencies.

Verification:

- Focused `cargo test -p dreadstep-headless --all-targets --all-features --locked` passes.
- `scripts/verify.sh` passes before handoff.

Out of scope:

- Interactive input, authored scenario files, serialized replay output, CLI subcommands,
  terminal rendering, and production content configuration.

### Milestone 2 slice: versioned agent observation

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Establish the first agent-facing protocol projection without adding a transport runtime. The
core exposes deterministic actor iteration, and `dreadstep-protocol` converts a
`WorldState` into a versioned snapshot containing the current time, next scheduled actor,
stable digest, and complete actor state. The projection preserves dead actor records for
inspection while making no gameplay decisions.

Acceptance:

- `WorldState` exposes actors in stable `ActorId` order without exposing mutable storage.
- `dreadstep-protocol` exposes a versioned `WorldSnapshot` and typed `ActorSnapshot` values for
  actor identity, kind, life, position, hit points, and ready time.
- Snapshot conversion includes the core `StateDigest`, current action time, and next actor; two
  equivalent worlds produce equal snapshots and meaningful state changes alter the snapshot.
- Conversion is a pure boundary projection and does not add rules, I/O, serialization
  dependencies, MCP runtime dependencies, or Bevy dependencies.
- Focused protocol tests cover stable ordering, equivalent snapshots, and dead-actor
  inspection after a core transition.

Verification:

- Focused `cargo test -p dreadstep-protocol --all-targets --all-features --locked` passes.
- `scripts/verify.sh` passes before handoff.

Out of scope:

- MCP transport/runtime registration, sessions, `start_run`, `legal_actions`, `act`, tester
  tools, wire serialization, and authored scenarios.

### Milestone 2 slice: versioned agent action requests

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Add the protocol-owned action envelope that a future MCP session can accept. The envelope
represents movement, waiting, melee, and chase with protocol-owned actor identities and
directions, and provides explicit conversion to and from the canonical core command. It does
not execute commands, enumerate legal actions, or introduce a transport.

Acceptance:

- `dreadstep-protocol` exposes a typed `CommandRequest` and protocol `Direction` without public
  fields that leak core representations.
- Every supported request maps exactly to one `dreadstep-core::Command`, and core commands can
  be projected back without changing actor, target, or direction values.
- Conversion is deterministic, side-effect free, and does not apply scheduling, validation, or
  gameplay rules; core remains the authority for command acceptance.
- Focused protocol tests cover all four request variants and round-trip mapping.

Verification:

- Focused `cargo test -p dreadstep-protocol --all-targets --all-features --locked` passes.
- `scripts/verify.sh` passes before handoff.

Out of scope:

- MCP transport/runtime registration, sessions, `start_run`, `legal_actions`, `act`, tester
  tools, wire serialization, authored scenarios, and command validation beyond representation
  conversion.

### Milestone 2 slice: in-memory MCP player session

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Add the first usable player-facing MCP boundary without depending on an MCP transport runtime.
`dreadstep-mcp` owns an explicit in-memory session with a seed and fixed developer scenario;
`start_run` constructs it, `observe` returns the versioned protocol snapshot, and `act` maps a
protocol request into one core command and returns protocol event evidence plus the new snapshot.
Core remains authoritative for scheduling, legality, and all state transitions.

Acceptance:

- `dreadstep-mcp` exposes deterministic `Session::start_run`, `observe`, and `act` operations
  with no `rmcp`, filesystem, wall-clock, or Bevy dependency.
- Session output includes the supplied seed, typed protocol events, and a fresh `WorldSnapshot`
  after each accepted action.
- Core command rejection is returned as a structured session error with no partial output or
  state mutation; accepted actions delegate entirely to `WorldState::execute`.
- Protocol owns event representations for movement, blocking, waiting, attack, and death,
  including typed actor, position, damage, hit-point, and block-reason values.
- Focused MCP tests cover start/observe, an accepted action, protocol event mapping, and a
  rejected unscheduled actor command.

Verification:

- Focused `cargo test -p dreadstep-mcp --all-targets --all-features --locked` passes.
- `scripts/verify.sh` passes before handoff.

Out of scope:

- MCP transport/runtime registration, legal-action enumeration, tester tools, session restore,
  wire serialization, authored scenarios, and interactive input.

### Milestone 2 slice: deterministic legal-action discovery

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Expose the core's deterministic legal command set through `Session::legal_actions`. Core owns
the actor-role, target, adjacency, and scheduling rules; the MCP adapter only converts the
resulting canonical commands into protocol requests. Blocked movement remains discoverable as an
accepted movement request, while invalid targets and wrong actor roles are excluded.

Acceptance:

- `WorldState::legal_commands` returns an ordered, read-only command list for the scheduled living
  actor, or an empty list when no actor can act.
- Enumeration includes four cardinal moves and wait for a living scheduled actor, plus only
  valid adjacent attacks for players or valid living targets for enemy chase.
- Enumeration order is deterministic: cardinal directions follow the core direction order,
  wait follows movement, and target commands follow stable `ActorId` order.
- `dreadstep-mcp::Session::legal_actions` returns protocol-owned requests with no world mutation
  or duplicated legality logic.
- Focused core/MCP tests cover initial player actions, enemy chase actions after scheduling, and
  deterministic equality across equivalent sessions.

Verification:

- Focused `cargo test -p dreadstep-core -p dreadstep-mcp --all-targets --all-features --locked`
  passes.
- `scripts/verify.sh` passes before handoff.

Out of scope:

- MCP transport/runtime registration, tester tools, session restore, wire serialization, authored
  scenarios, interactive input, and AI policy over the legal-action list.

### Milestone 2 slice: session history and replay evidence

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Complete the first player session's replay-facing evidence. `dreadstep-mcp::Session` records each
accepted protocol request in core-owned `ReplayTrace` order, exposes protocol history values, and
returns the deterministic replay digest. Rejected requests are not recorded; no replay file or
transport serialization is introduced.

Acceptance:

- A new session has empty history and a replay digest derived from its explicit seed.
- Every accepted `act` appends exactly one converted core command in execution order; rejected
  actions leave history and replay digest unchanged.
- `Session::history` returns protocol-owned requests, and `Session::replay_digest` returns the
  core trace identity without leaking core trace types.
- Equivalent sessions with the same seed and accepted request sequence produce equal history and
  replay digest values; changing seed or order changes the digest through core replay evidence.
- Focused MCP tests cover empty/accepted/rejected history and deterministic replay equality.

Verification:

- Focused `cargo test -p dreadstep-mcp --all-targets --all-features --locked` passes.
- `scripts/verify.sh` passes before handoff.

Out of scope:

- MCP transport/runtime registration, replay files, wire serialization, session restore, tester
  tools, authored scenarios, interactive input, and replay playback.

## Future

### Milestone 1: Rules kernel

Implement the deterministic headless simulation described in the proposal: typed world
state, commands and events, seeded randomness, movement, blocking, combat, scheduling,
replay evidence, and a developer CLI.
