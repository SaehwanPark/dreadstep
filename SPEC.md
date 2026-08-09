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

### Milestone 2 slice: typed session replay evidence

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Complete the player-facing `get_replay` projection without adding persistence or a transport.
`dreadstep-protocol` owns a typed `ReplayEvidence` value containing the explicit seed, ordered
protocol requests, and core replay digest. `dreadstep-mcp::Session::get_replay` returns that
value as a read-only view over the existing core `ReplayTrace`; it does not execute commands or
reconstruct state.

Acceptance:

- A new session returns replay evidence with its explicit seed, empty request list, and seeded
  digest.
- Accepted requests appear once and in execution order in the replay evidence; rejected requests
  remain absent because core execution and trace recording are unchanged.
- Equivalent sessions with equal seeds and accepted requests return equal replay evidence, while
  a changed seed or accepted request order changes the core digest evidence.
- `ReplayEvidence` exposes protocol-owned requests and digest values without leaking
  `dreadstep_core::ReplayTrace` or introducing serialization.

Verification:

- Focused `cargo test -p dreadstep-mcp --all-targets --all-features --locked` passes.
- `scripts/verify.sh` passes before handoff.

Out of scope:

- MCP transport/runtime registration, replay files, wire serialization, session restore, tester
  tools, authored scenarios, interactive input, and replay playback.

### Milestone 2 slice: player actor inspection

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Complete the player-facing `inspect` projection as a read-only lookup over the versioned world
snapshot. `dreadstep-mcp::Session::inspect` accepts a protocol actor identity and returns the
matching protocol `ActorSnapshot` when present, including dead records retained by core; an
unknown identity returns `None`. The method does not add visibility policy, mutate the world, or
duplicate core rules.

Acceptance:

- Inspecting a known living actor returns its protocol identity, kind, position, hit points, life,
  and ready-time values equal to the current `observe` snapshot.
- Inspecting an unknown actor returns `None` without changing the session snapshot or replay
  evidence.
- A dead actor remains inspectable after a valid combat sequence and reports dead life state and
  zero hit points.
- Inspection uses protocol-owned values and remains a pure read-only adapter projection without
  transport, filesystem, or visibility-policy dependencies.

Verification:

- Focused `cargo test -p dreadstep-mcp --all-targets --all-features --locked` passes.
- `scripts/verify.sh` passes before handoff.

Out of scope:

- MCP transport/runtime registration, hidden-information rules, tester tools, authored scenarios,
  interactive input, and replay playback.

### Milestone 2 slice: named session history accessor

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Align the in-memory player session with the proposal's `get_history` operation name. The new
`Session::get_history` method returns the same protocol-owned accepted request sequence as the
existing `history` projection, preserving execution order and read-only behavior. It is an API
naming boundary only: core trace recording, rejection handling, and replay digest semantics do not
change.

Acceptance:

- `get_history` returns an empty protocol request list for a new session and the exact accepted
  request order after actions.
- Rejected requests do not appear in `get_history`, and calling either history accessor leaves the
  world and replay evidence unchanged.
- Equivalent sessions expose equal `get_history` values without leaking core `ReplayTrace` types.
- The existing `history` accessor remains behaviorally identical for current callers.

Verification:

- Focused `cargo test -p dreadstep-mcp --all-targets --all-features --locked` passes.
- `scripts/verify.sh` passes before handoff.

Out of scope:

- MCP transport/runtime registration, tester tools, replay persistence or playback, authored
  scenarios, interactive input, and changes to command execution.

### Milestone 2 slice: in-memory tester snapshot and restore

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Add the first explicit tester savepoint operation without introducing transport or persistence.
`dreadstep-mcp::Session::snapshot` clones the complete valid session state into a
`SessionSnapshot`, including the fixed seed, world state, and core `ReplayTrace` evidence.
`Session::restore` replaces the current session with that savepoint so subsequent observations,
accepted history, and replay digest continue from the captured point.

Acceptance:

- A savepoint captures the current world and replay state without mutating the session.
- Restoring a savepoint restores the seed, world snapshot, accepted request history, and replay
  digest exactly as they were when captured.
- Executing the same accepted request after restore produces the same protocol output and resulting
  snapshot as the original execution from that savepoint.
- Savepoint and restore remain in-memory tester operations; no filesystem, serialization, transport,
  new gameplay rules, or arbitrary state mutation is introduced.

Verification:

- Focused `cargo test -p dreadstep-mcp --all-targets --all-features --locked` passes.
- `scripts/verify.sh` passes before handoff.

Out of scope:

- MCP transport/runtime registration, persistent replay files, wire serialization, scenario
  authoring, spawn/item/HP mutators, interactive input, and replay playback.

### Milestone 2 slice: tester world inspection accessor

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Align the tester surface with the proposal's `inspect_world` operation name. The new
`Session::inspect_world` method returns the same complete protocol `WorldSnapshot` as the player
`observe` projection, without mutating the session or introducing separate world storage. This
slice names the boundary only; no additional hidden state, visibility policy, or tester mutation
is added.

Acceptance:

- `inspect_world` returns the current complete protocol snapshot for a new or advanced session.
- Equivalent sessions return equal `inspect_world` snapshots, and the accessor does not change
  history, replay digest, or future action results.
- The existing `observe` accessor remains behaviorally identical for player callers.
- The tester accessor remains in-memory and read-only, with no transport, filesystem, serialization,
  or arbitrary state mutation.

Verification:

- Focused `cargo test -p dreadstep-mcp --all-targets --all-features --locked` passes.
- `scripts/verify.sh` passes before handoff.

Out of scope:

- MCP transport/runtime registration, hidden-information rules, scenario or actor mutators,
  persistent replay, wire serialization, interactive input, and replay playback.

### Milestone 2 slice: validated tester actor spawning

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Add one explicit tester mutation through the functional core. `WorldState::spawn` validates and
inserts a living actor using the existing map, identity, terrain, and living-occupancy invariants;
`dreadstep-mcp::Session::spawn` translates protocol values into that core operation and maps
validation failures to protocol-owned world errors. Failed spawns leave the world and replay state
unchanged.

Acceptance:

- A valid spawn adds one living actor at a walkable, unoccupied position with the requested typed
  identity, kind, and hit points; the actor is visible through protocol inspection.
- Duplicate identities, out-of-bounds positions, blocked tiles, overlapping living actors, and
  zero-hit-point actors are rejected with typed protocol world errors.
- Rejected spawns are atomic: world snapshot, history, and replay digest remain unchanged.
- Core remains authoritative for validation; the MCP adapter performs only typed conversion and
  error projection, with no transport, filesystem, serialization, or other tester mutators.

Verification:

- Focused `cargo test -p dreadstep-core -p dreadstep-protocol -p dreadstep-mcp --all-targets
  --all-features --locked` passes.
- Focused Clippy for the core, protocol, and MCP crates passes with `-D warnings`.
- `scripts/verify.sh` passes before handoff.
- `git diff --check` passes, and the single semantic review reports pass at revision 2.

Out of scope:

- set-HP, item, teleport, scenario-authoring, restore changes, MCP transport/runtime, persistent
  storage, wire serialization, interactive input, and replay playback.

### Milestone 2 slice: validated tester hit-point mutation

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Add the next explicit tester mutation through the functional core. `WorldState::set_hit_points`
updates one existing actor's typed hit points, retains dead actor records for inspection, and
re-anchors an actor revived from zero hit points at the world's current action time so the tester
cannot rewind scheduling. `dreadstep-mcp::Session::set_hp` translates protocol values and maps an
unknown actor to a protocol-owned world error. The mutation is in-memory only and does not record
an accepted player command or alter replay evidence.

Acceptance:

- An existing actor can have its hit points set to any typed `u16` value, including zero; zero
  removes the actor from scheduling and living occupancy while preserving its record and position.
- Reviving a dead actor makes it living at the current world action time without rewinding the
  scheduler. Setting the earliest actor to zero may advance current time to the next surviving
  actor's readiness, but never rewinds it; identity, kind, and position remain unchanged.
- Unknown actor identities and revivals onto tiles occupied by living actors are rejected with
  typed world errors and leave world, history, and replay evidence unchanged.
- Successful tester hit-point mutations leave accepted request history and replay evidence
  unchanged; core remains authoritative for life and scheduling semantics.

Verification:

- Focused `cargo test -p dreadstep-core -p dreadstep-protocol -p dreadstep-mcp --all-targets
  --all-features --locked` passes.
- Focused Clippy for the core, protocol, and MCP crates passes with `-D warnings`.
- `scripts/verify.sh` passes before handoff.
- `git diff --check` passes, and the single semantic review reports pass at revision 1.

Out of scope:

- spawn, item, teleport, scenario-authoring, restore changes, MCP transport/runtime, persistent
  storage, wire serialization, interactive input, and replay playback.

### Milestone 2 slice: typed tester scenario replacement

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Add the first bounded scenario-authoring operation without introducing files or transport. A
protocol-owned `Scenario` describes a rectangular typed map and initial actor records;
`dreadstep-mcp::Session::create_scenario` converts it into the existing core `GridMap` and
`WorldState::new` validators, then atomically replaces the in-memory world. The session seed is
preserved, while accepted player history and replay evidence reset to an empty trace for the new
scenario. Invalid maps or actors leave the previous session unchanged and return typed scenario
errors.

Acceptance:

- A valid protocol scenario replaces the current world with its requested map and living actors;
  core remains authoritative for map dimensions, tile count, actor identity, bounds, terrain,
  occupancy, and starting hit-point validation.
- Scenario replacement preserves the explicit session seed and resets accepted history and replay
  evidence to the empty trace for that seed.
- Invalid map or world data returns typed protocol scenario errors atomically; no prior world,
  history, or replay evidence is lost.
- The operation remains in-memory and tester-only, with no filesystem, serialization, transport,
  item system, teleportation, or replay playback.

Verification:

- Focused `cargo test -p dreadstep-core -p dreadstep-protocol -p dreadstep-mcp --all-targets
  --all-features --locked` passes.
- Focused Clippy for the core, protocol, and MCP crates passes with `-D warnings`.
- `scripts/verify.sh` passes before handoff.
- `git diff --check` passes, and the single semantic review reports pass at revision 1.

Out of scope:

- item, teleport, map mutation after creation, persistence, wire serialization, MCP
  transport/runtime, interactive input, and replay playback.

### Milestone 2 slice: opaque tester item ownership

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Define the smallest canonical item contract needed for tester injection without inventing gameplay
effects. Core owns globally unique `ItemId` values, opaque `ItemDefinitionId` content references,
and an ordered item list on each actor. `WorldState::give_item` validates the target actor and
global item identity; `dreadstep-mcp::Session::give_item` performs typed conversion only. Item
ownership appears in deterministic digests and actor snapshots, while successful tester injection
does not enter player history or replay evidence.

Acceptance:

- A valid typed item can be given to any existing actor record and appears in that actor's stable
  inventory projection and world digest.
- Unknown actors and globally duplicate item identities return typed world errors atomically.
- Item order is insertion order and remains deterministic across equivalent worlds and snapshots.
- The slice defines no item effect, equipment slot, pickup/drop command, identification rule,
  content catalog, transfer operation, or inventory capacity; those remain future contracts.

Verification:

- Focused `cargo test -p dreadstep-core -p dreadstep-protocol -p dreadstep-mcp --all-targets
  --all-features --locked` passes.
- Focused Clippy for the core, protocol, and MCP crates passes with `-D warnings`.
- `scripts/verify.sh` passes before handoff.
- `git diff --check` passes, and the single semantic review reports pass at revision 1.

Out of scope:

- item effects, equipment, pickup/drop/transfer, identification, content catalogs, inventory
  capacity, teleport, persistence, wire serialization, MCP transport/runtime, interactive input,
  and replay playback.

### Milestone 2 slice: validated tester teleport

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Add one explicit tester relocation operation without changing player commands or scheduler rules.
`WorldState::teleport` moves an existing actor record to a typed walkable map position while
preserving identity, life, hit points, inventory, readiness, and current world time. Living actors
cannot enter another living actor's tile; dead records remain non-occupying and may be positioned on
an occupied living tile, matching existing dead-record reuse semantics. The MCP session performs
typed conversion only, and the mutation does not enter accepted player history or replay evidence.

Acceptance:

- A valid teleport updates the selected actor's position and digest/snapshot while preserving all
  other actor and scheduler fields.
- Unknown actors, out-of-bounds destinations, blocked terrain, and living-actor overlap return
  typed world errors atomically.
- Dead actor records remain valid teleport targets and do not block living destinations.
- The operation remains an in-memory tester mutation with no player-facing teleport command,
  map mutation, transport, persistence, wire serialization, or replay playback.

Verification:

- Focused `cargo test -p dreadstep-core -p dreadstep-protocol -p dreadstep-mcp --all-targets
  --all-features --locked` passes.
- Focused Clippy for the core, protocol, and MCP crates passes with `-D warnings`.
- `scripts/verify.sh` passes before handoff.
- `git diff --check` passes, and the single semantic review reports pass at revision 1.

Out of scope:

- player teleport commands, action costs, map editing, item effects, equipment, capacity, transfer,
  identification, content catalogs, persistence, wire serialization, MCP transport/runtime,
  interactive input, and replay playback.

### Milestone 2 slice: minimal MCP stdio observation

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Add the first process boundary around the existing in-memory player session. The versioned
`WorldSnapshot` projection gains explicit JSON serialization, and the `dreadstep-mcp` package gains
a local stdio server using the official MCP runtime. It exposes only `start_run(seed)` and the
read-only `observe()` tool; both return structured snapshots, while protocol text remains on stdout
and startup/runtime failures go to stderr. Core rules and session history remain authoritative in
the existing library, and no tester mutation or player action is exposed through transport yet.

Acceptance:

- A stdio MCP client can initialize, discover exactly the bounded observation tools, start a seeded
  fixed scenario, and observe the resulting versioned JSON snapshot.
- Snapshot JSON includes protocol version, current time, next actor, digest, and stable actor/item
  projections with deterministic field names and ordering.
- `observe` does not mutate the session; `start_run` replaces only the in-memory session state and
  does not expose host filesystem, environment, or arbitrary process access.
- The server's stdout is reserved for MCP protocol traffic, and operational failures are reported
  through structured MCP errors/stderr rather than ad-hoc game output.

Verification:

- Focused protocol serialization and MCP server tests pass.
- A subprocess smoke test completes initialize, tools/list, start_run, and observe over stdio.
- Focused Clippy and `scripts/verify.sh` pass before handoff.
- `git diff --check` passes, and the single semantic review reports pass at revision 0.

Out of scope:

- player `act`, legal-actions transport, tester mutations over MCP, replay persistence, transport
  alternatives, hidden information, interactive input, map editing, and gameplay item semantics.

### Milestone 2 slice: typed MCP player actions

- Status: verified
- Started: 2026-08-08
- Completed: 2026-08-08

Extend the minimal stdio server with one typed player `act` tool. Protocol command requests and
semantic events gain explicit JSON/JSON Schema projections, and the server returns structured
`SessionOutput` evidence containing the session seed, event order, and post-action snapshot. Core
continues to validate scheduling, targets, movement, combat, and errors; accepted actions enter the
existing session history/replay trace, while rejected actions become structured MCP invalid-params
errors and do not mutate state.

Acceptance:

- A stdio MCP client can call `act` with an object containing a typed move/wait/attack/chase request
  and receive stable structured seed, event, and snapshot evidence.
- Command requests use explicit tagged JSON variants and protocol-owned typed IDs/directions; event
  values preserve semantic variant data and deterministic order.
- Core command errors cross the process boundary as MCP invalid-params errors; rejected actions leave
  world, history, and replay evidence unchanged.
- The server exposes only `start_run`, `observe`, and `act`; legal-action discovery and tester
  mutations remain outside this wire slice.

Verification:

- Focused protocol JSON/schema, MCP tool, and subprocess tests pass.
- Accepted and rejected action behavior is covered at the in-memory and stdio boundaries.
- Focused Clippy and `scripts/verify.sh` pass before handoff.
- The single semantic review reports pass at revision 1.

Out of scope:

- legal-actions transport, tester mutations over MCP, replay persistence, alternate transports,
  hidden information, interactive input, map editing, and gameplay item semantics.

### Milestone 2 slice: MCP legal-action discovery

- Status: verified
- Started: 2026-08-08
- Completed: 2026-08-08

Expose the existing deterministic `Session::legal_actions` query through the local stdio MCP
server. The `legal_actions` tool takes no arguments and returns the core-selected command list as
typed protocol requests with stable ordering. It is read-only: the scheduler and rules remain in
`dreadstep-core`, and the MCP adapter only projects the result.

Acceptance:

- A stdio MCP client can discover and call `legal_actions` without arguments and receive an array
  of typed move/wait/attack/chase requests in core-defined deterministic order.
- The tool's input and output schemas are explicit (`object` input and array output), and command
  variants retain the same tagged JSON shape used by `act`.
- Calling `legal_actions` does not change the world snapshot, accepted history, or replay evidence.
- The process exposes exactly `start_run`, `observe`, `legal_actions`, and `act`; tester mutations
  and other broader tools remain outside this slice.

Verification:

- Focused MCP tool-schema and ordered subprocess tests pass.
- Direct session and stdio tests prove read-only behavior and deterministic action ordering.
- Focused Clippy and `scripts/verify.sh` pass before handoff.
- The single semantic review reports pass at revision 1.

Out of scope:

- tester mutations over MCP, replay persistence, alternate transports, hidden information,
  interactive input, map editing, and gameplay item semantics.

### Milestone 2 slice: MCP actor inspection

- Status: verified
- Started: 2026-08-08
- Completed: 2026-08-08

Expose the existing read-only `Session::inspect` lookup through the local stdio MCP server. The
`inspect` tool takes a typed actor identity and returns the corresponding protocol `ActorSnapshot`
or `null` when no actor with that identity exists. It preserves the existing information boundary:
the adapter projects already-visible actor data and does not add hidden state, visibility policy, or
gameplay behavior.

Acceptance:

- A stdio MCP client can call `inspect` with an object containing a typed actor ID and receive a
  structured actor snapshot or an explicit absent result.
- The input schema is an object with a protocol-owned actor ID, and the output schema represents
  `ActorSnapshot | null` without ad-hoc strings or IDs.
- Known and unknown inspection calls do not change the world snapshot, history, or replay evidence.
- The process exposes exactly `start_run`, `observe`, `legal_actions`, `act`, and `inspect`; tester
  mutations and other broader tools remain outside this slice.

Verification:

- Focused MCP tool-schema and ordered subprocess tests pass.
- Direct session and stdio tests cover known, unknown, deterministic, and read-only inspection.
- Focused Clippy and `scripts/verify.sh` pass before handoff.
- The single semantic review reports pass at revision 1.

Out of scope:

- tester mutations over MCP, replay persistence, alternate transports, hidden information,
  interactive input, map editing, and gameplay item semantics.

### Milestone 2 slice: MCP accepted history

- Status: verified
- Started: 2026-08-08
- Completed: 2026-08-08

Expose the existing named `Session::get_history` projection through the local stdio MCP server as a
read-only `get_history` tool. It takes no arguments and returns accepted typed protocol requests in
execution order. Core replay recording remains authoritative; MCP only serializes the existing
adapter-owned view and does not expose `ReplayTrace` internals.

Acceptance:

- A stdio MCP client can call `get_history` without arguments and receive an array of accepted
  tagged move/wait/attack/chase requests in execution order.
- The tool has explicit object input and array output schemas using protocol-owned request values.
- A new run returns empty history; accepted actions appear exactly once, rejected actions do not,
  and history calls do not mutate world, history, or replay evidence.
- The process exposes exactly `start_run`, `observe`, `legal_actions`, `inspect`, `act`, and
  `get_history`; tester mutations and other broader tools remain outside this slice.

Verification:

- Focused MCP tool-schema and ordered subprocess tests pass.
- Direct session and stdio tests cover empty, accepted, rejected, ordered, deterministic, and
  read-only history behavior.
- Focused Clippy and `scripts/verify.sh` pass before handoff.
- The single semantic review reports pass at revision 1.

Out of scope:

- replay evidence transport, tester mutations over MCP, replay persistence, alternate transports,
  hidden information, interactive input, map editing, and gameplay item semantics.

### Milestone 2 slice: MCP replay evidence

- Status: verified
- Started: 2026-08-08
- Completed: 2026-08-08

Expose the existing typed `Session::get_replay` projection through the local stdio MCP server as a
read-only `get_replay` tool. `ReplayEvidence` gains explicit JSON/JSON Schema serialization for the
transport, carrying the seed, accepted protocol requests, and deterministic digest without exposing
core `ReplayTrace`, persistence, or playback.

Acceptance:

- A stdio MCP client can call `get_replay` without arguments and receive structured seed, commands,
  and digest evidence.
- A new run returns its explicit seed, an empty command array, and a numeric seeded digest; accepted
  commands appear once in order, while rejected commands do not change replay evidence.
- The tool has explicit object input and output schemas using protocol-owned command requests and
  digest values; equivalent seed/request sequences produce equal JSON evidence.
- The process exposes exactly `start_run`, `observe`, `legal_actions`, `inspect`, `get_history`,
  `get_replay`, and `act`; tester mutations, persistence, and playback remain outside this slice.

Verification:

- Focused protocol JSON/schema and ordered MCP subprocess tests pass.
- Direct session and stdio tests cover empty, accepted, rejected, deterministic, and read-only
  replay evidence behavior.
- Focused Clippy and `scripts/verify.sh` pass before handoff.
- The single semantic review reports pass at revision 1.

Out of scope:

- replay files, playback, tester mutations over MCP, alternate transports, hidden information,
  interactive input, map editing, and gameplay item semantics.

### Milestone 3 slice: deterministic Bevy presentation bridge

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Establish the first human-client boundary without making Bevy presentation state authoritative.
`dreadstep-bevy` will project the core map and actor records into a stable, read-only
`PresentationSnapshot`, translate keyboard intent into canonical core commands for an explicitly
selected actor, and execute those commands through `WorldState`. The bridge remains usable in
headless tests, so adding Bevy windowing or desktop platform features is not required for this
slice.

Acceptance:

- `GridMap` exposes its validated row-major tiles through an immutable accessor; callers cannot
  mutate core map storage.
- `dreadstep-bevy` exposes a typed `PresentationState` that owns a core `WorldState` and explicit
  replay trace, returns deterministic map/actor/time/digest projections, and reports core events
  after accepted commands.
- Keyboard intent maps only the supported cardinal movement and wait keys to canonical
  `dreadstep_core::Command` values for a caller-supplied actor identity; unmapped keys produce no
  command and never mutate state.
- Accepted presentation commands delegate to core scheduling and validation; rejected commands
  leave the bridge world and replay trace unchanged.
- The bridge remains independent of MCP, filesystem, wall-clock time, host randomness, and
  desktop window/audio features.

Verification:

- Focused `cargo test -p dreadstep-core -p dreadstep-bevy --all-targets --all-features --locked`
  covers immutable map projection, deterministic presentation snapshots, keyboard mapping,
  accepted execution, and rejection atomicity.
- Focused Clippy for the changed crates and `scripts/verify.sh` pass before handoff.
- `git diff --check` passes, and exactly one semantic code reviewer reports pass at the final
  revision.

Out of scope:

- Bevy window creation, desktop platform backends, rendering assets, sprites, animations, camera,
  HUD, combat messages, audio, fog of war, content catalogs, and new gameplay rules.

### Milestone 3 slice: shared authored starter floor

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Move the first authored presentation scenario behind the content boundary so human-facing clients
can start the same validated floor without copying map or actor setup. `dreadstep-content` will
construct one deterministic starter floor from typed map and actor definitions, while
`dreadstep-bevy::PresentationState::start_run` delegates to that content constructor and preserves
the explicit run seed in presentation replay evidence. This slice establishes scenario ownership;
it does not add randomness, progression, or rendering.

Acceptance:

- `dreadstep-content` exposes a typed starter-floor constructor that validates its rectangular map
  and living actor records through `dreadstep-core`, returning structured content errors.
- The starter floor has one player, three distinct living enemies, stable row-major terrain, and
  no overlapping or blocked actor placements; equivalent constructions produce equal worlds and
  digests.
- `dreadstep-bevy::PresentationState::start_run(seed)` delegates to the content constructor,
  preserves the supplied seed, and exposes the same deterministic snapshot as direct content
  construction without depending on MCP or filesystem state.
- Invalid content construction is surfaced as a typed error before a presentation state exists;
  no new gameplay rules, random source, or transport contract is introduced.

Verification:

- Focused `cargo test -p dreadstep-content -p dreadstep-bevy --all-targets --all-features --locked`
  covers content validation, starter-floor shape, deterministic digest, and presentation startup.
- Focused Clippy for changed crates and `scripts/verify.sh` pass before handoff.
- `git diff --check` passes, and exactly one semantic code reviewer reports pass at the final
  revision.

Out of scope:

- Bevy windowing, rendering assets, sprites, animation, camera, HUD, audio, fog of war, multiple
  floors, procedural generation, seeded randomness, item content/effects, and new gameplay rules.

### Milestone 3 slice: headless Bevy scene synchronization

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Add the first ECS-facing presentation projection without making ECS state authoritative. A
`dreadstep-bevy` scene synchronizer will materialize map tiles and actor render data as typed Bevy
components from a complete `PresentationSnapshot`, update existing entities deterministically by
tile position and actor identity, and remove stale entities. Core state remains the only source of
truth; scene components are disposable mirrors for later rendering systems.

Acceptance:

- `SceneTile` and `SceneActor` expose typed, immutable presentation data for terrain, actor
  identity/kind, position, life, hit points, and scheduler readiness without exposing mutable core
  storage.
- `sync_scene` creates one entity per projected map tile and actor, preserves entity identity for
  unchanged keys, updates changed actor data after an accepted core command, and removes entities
  absent from a later snapshot.
- Dead actor records remain represented because core snapshots retain them for inspection; scene
  synchronization adds no visibility, movement, or gameplay rules.
- Synchronization is deterministic, headless-testable with a Bevy `World`, and independent of MCP,
  filesystem, wall-clock time, host randomness, windowing, rendering backends, and audio.

Verification:

- Focused `cargo test -p dreadstep-bevy --all-targets --all-features --locked` covers initial
  entity creation, stable entity identity, changed actor projection, stale-entity removal, and
  dead-record retention.
- Focused Clippy and `scripts/verify.sh` pass before handoff.
- `git diff --check` passes, and exactly one semantic code reviewer reports pass at the final
  revision.

Out of scope:

- Bevy windowing/rendering plugins, sprites, textures, camera, animations, HUD, audio, fog of war,
  ECS-driven commands, map generation, and new gameplay rules.

### Milestone 3 slice: headless Bevy application shell

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Own the headless presentation boundary inside a Bevy `App` without making ECS state authoritative.
`dreadstep-bevy` will expose a `PresentationRuntime` resource that contains one
`PresentationState`, plus a `PresentationPlugin` whose update system snapshots that runtime and
keeps the disposable scene mirrors synchronized. Commands continue to enter through the runtime's
explicit core delegation API; the plugin itself only projects state.

Acceptance:

- `PresentationRuntime` preserves the explicit seed, exposes read-only snapshots and replay
  evidence, and delegates accepted or rejected commands to exactly one `PresentationState`.
- `PresentationPlugin` adds only a headless update system and can start a complete authored scene
  in a Bevy `App` without windowing, rendering, audio, desktop platform features, wall-clock time,
  or host randomness.
- Every app update projects the runtime snapshot through `sync_scene`; unchanged scene identities
  remain stable and an accepted runtime command becomes visible after the next update.
- ECS scene components remain disposable mirrors and cannot issue commands or replace core world
  truth; rejected commands leave runtime and scene projections unchanged.

Verification:

- Focused `cargo test -p dreadstep-bevy --all-targets --all-features --locked` covers runtime
  ownership, plugin startup, update synchronization, accepted-command projection, and rejection
  atomicity.
- Focused Clippy and `scripts/verify.sh` pass before handoff.
- `git diff --check` passes, and exactly one semantic code reviewer reports pass at the final
  revision.

Out of scope:

- Bevy window/render plugins, sprites, textures, camera, animation, HUD, audio, fog of war, input
  systems, persistence, transport, ECS-issued commands, and new gameplay rules.

### Milestone 3 slice: deterministic headless keyboard dispatch

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Add the first interactive input effect without adding a desktop client. `PresentationInput` names
the controlled actor, and the existing headless `PresentationPlugin` will read optional Bevy
`ButtonInput<KeyCode>` state, choose at most one supported just-pressed key from a fixed priority
order, delegate its `KeyboardIntent` through `PresentationRuntime`, and synchronize the scene after
that command. Core remains authoritative for scheduling, legality, and all state changes.

Acceptance:

- `PresentationInput` exposes one typed controlled [`dreadstep_core::ActorId`] and no mutable core
  storage; missing input/control/runtime resources are safe no-ops.
- Arrow, WASD, and wait aliases map through the existing `KeyboardIntent` conversion. A fixed
  documented priority (`ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight`, `KeyW`, `KeyS`, `KeyA`,
  `KeyD`, `Enter`, `Space`) chooses one simultaneous key per update, consumes all supported
  just-pressed keys for that update, and never depends on hash-set iteration order.
- The plugin delegates accepted and rejected commands through `PresentationRuntime` before scene
  synchronization; accepted movement appears in the same app update's scene projection, while
  rejected commands leave runtime and complete keyed scene projections unchanged.
- The dispatch remains headless and independent of windowing, event readers, mouse/gamepad input,
  filesystem, wall-clock time, host randomness, transport, and new gameplay rules.

Verification:

- Focused `cargo test -p dreadstep-bevy --all-targets --all-features --locked` covers controlled
  actor ownership, key priority/consumption, accepted projection, rejected atomicity, and absent
  resources.
- Focused Clippy and `scripts/verify.sh` pass before handoff.
- `git diff --check` passes, and exactly one semantic code reviewer reports pass at the final
  revision.

Out of scope:

- Windowing/event readers, mouse/gamepad/text input, rebinding persistence, rendering, audio,
  transport, ECS-issued commands, and new gameplay rules.

### Milestone 3 slice: deterministic presentation feedback buffer

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Expose accepted presentation output as an adapter-owned, one-shot feedback buffer for future HUD
and combat-message systems. `PresentationRuntime` will retain the latest `PresentationOutput` from
an accepted direct or keyboard command, expose read-only inspection, and offer explicit consumption.
Rejected commands clear stale feedback but do not change the authoritative core world, replay trace,
or scene mirrors.

Acceptance:

- A fresh runtime has no pending output; accepted commands publish exact typed core events and the
  post-command snapshot/digest, regardless of whether the command came from direct API or keyboard
  dispatch.
- `output()` never exposes mutable core/ECS storage, and `take_output()` transfers a pending output
  exactly once; no wall-clock expiry or unordered event source exists.
- Rejected commands clear stale output while preserving runtime snapshot, replay digest, and
  complete keyed scene projections; feedback is evidence only and cannot issue commands.
- The buffer remains headless and independent of HUD widgets, text layout, animations, persistence,
  transport, windowing, audio, and new gameplay rules.

Verification:

- Focused `cargo test -p dreadstep-bevy --all-targets --all-features --locked` covers startup
  emptiness, accepted keyboard/direct output, exact event/snapshot evidence, one-shot consumption,
  and rejection clearing/atomicity.
- Focused Clippy and `scripts/verify.sh` pass before handoff.
- `git diff --check` passes, and exactly one semantic code reviewer reports pass at the final
  revision.

Out of scope:

- HUD, messages, animations, output persistence, replay-file formats, ECS authority, transport,
  windowing, audio, and new gameplay rules.

### Milestone 3 slice: typed headless presentation focus projection

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Add a small presentation-only projection for future camera systems. `PresentationFocus` will name
one controlled actor and mirror that actor's latest position from the authoritative runtime after
keyboard dispatch and scene synchronization. Unknown actors produce an explicit `None` position;
the resource does not decide visibility, camera policy, or gameplay and does not duplicate actor
records.

Acceptance:

- `PresentationFocus` exposes a typed actor identity and optional position without mutable core/ECS
  storage; missing runtime, input, or focus resources are safe no-ops.
- The plugin updates focus after an accepted keyboard command in the same app update, and changing
  the controlled actor updates both identity and position deterministically.
- An unknown actor yields `None` without fabricating coordinates or mutating runtime, replay, or
  complete keyed scene mirrors; dead-record visibility semantics remain core-owned.
- The projection remains headless and independent of camera entities/transforms, windowing,
  rendering, viewport math, interpolation, smoothing, visibility/fog, transport, and new rules.

Verification:

- Focused `cargo test -p dreadstep-bevy --test focus --all-features --locked` covers startup,
  accepted movement, controlled-actor changes, unknown actors, and independent absent-resource
  no-ops; the seven focused tests pass.
- `cargo test -p dreadstep-bevy --all-targets --all-features --locked` passes all Bevy targets,
  and Linux, Apple Silicon macOS, and Windows CI are green for the reviewed revision.
- Focused Clippy and `scripts/verify.sh` pass before handoff.
- `git diff --check` passes, and exactly one semantic code reviewer reports pass at the final
  revision (`5671d78`).

Out of scope:

- Camera entities/transforms, viewport policy, rendering, windowing, interpolation, smoothing,
  visibility/fog rules, transport, input rebinding, and new gameplay rules.

### Milestone 3 slice: deterministic headless scene-focus marker

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Project the selected actor onto the existing keyed `SceneActor` entity with a typed `SceneFocus`
marker. The marker is a disposable ECS identity projection for future camera or selection systems;
it stores no actor position or gameplay state and does not decide visual styling or visibility.

Acceptance:

- `SceneFocus` is a marker-only component attached to at most one keyed `SceneActor` entity when
  runtime, input, and focus resources are present; absent resources are safe no-ops.
- The plugin performs dispatch, complete scene synchronization, focus synchronization, then marker
  synchronization in one deterministic update; accepted movement preserves the focused entity's
  identity while updating its existing `SceneActor` projection.
- Changing the controlled actor moves the marker to that actor's existing keyed entity without
  duplicate markers; an unknown actor clears stale markers while preserving runtime snapshot,
  replay digest, and complete keyed tile/actor projections.
- The projection remains headless and independent of camera transforms, viewport policy, marker
  visuals, rendering, windowing, visibility/fog, transport, and new gameplay rules.

Verification:

- Focused `cargo test -p dreadstep-bevy --test scene_focus --all-features --locked` covers startup,
  accepted movement, actor changes, unknown actors, stable identity, and independent
  absent-resource no-ops; all seven focused tests pass.
- `cargo test -p dreadstep-bevy --all-targets --all-features --locked` passes all Bevy targets,
  and Linux, Apple Silicon macOS, and Windows CI are green for the reviewed revision.
- Focused Clippy and `scripts/verify.sh` pass before handoff.
- `git diff --check` passes, and exactly one semantic code reviewer reports pass at the final
  revision (`c8ba7a1`).

Out of scope:

- Camera entities/transforms, viewport policy, marker styling, rendering, windowing,
  visibility/fog rules, transport, input rebinding, and new gameplay rules.

### Milestone 3 slice: deterministic headless ground-item scene projection

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Extend the disposable Bevy scene mirror with complete core-owned ground-item records. The
`PresentationSnapshot` exposes stable row-major ground stacks, and `sync_scene` projects each
opaque item as a typed `SceneGroundItem` keyed by globally unique item identity. Core remains
authoritative for item ownership, stack order, and digest state; this slice only mirrors data for
future presentation systems.

Acceptance:

- `PresentationSnapshot::ground_items()` exposes complete immutable ground stacks, including
  position, `ItemId`, and `ItemDefinitionId`, in core-provided deterministic order.
- `SceneGroundItem` carries typed item identity, definition reference, position, and zero-based stack
  order without mutable core storage, gameplay effects, or presentation policy.
- `sync_scene` creates, updates, and deduplicates item entities deterministically by `ItemId`,
  preserves entity identity for unchanged ground items, and removes items absent from later
  snapshots (including picked-up items).
- Existing keyed tile and actor mirrors remain complete and unchanged by item projection; scene
  synchronization never mutates runtime state, replay evidence, or core world truth.
- The projection remains headless and independent of rendering, camera, HUD, transport, desktop
  platform features, persistence, and new gameplay rules.

Verification:

- Focused `cargo test -p dreadstep-bevy --test scene_sync --all-features --locked` passes all seven
  ordered projection, identity-update, duplicate-cleanup, stale-removal, and mirror-preservation
  tests.
- `cargo test -p dreadstep-bevy --all-targets --all-features --locked`, focused Clippy, Cargo docs,
  `git diff --check`, and `scripts/verify.sh` pass.
- Exactly one semantic code reviewer reports PASS on implementation revision `4f24029`, and Linux,
  Apple Silicon macOS, and Windows CI are green for that revision.

Out of scope:

- Player pickup/drop commands, item effects, equipment, capacity, identification, rendering,
  sprites, camera policy, HUD, visibility, persistence, transport, and new gameplay rules.

### Milestone 3 slice: deterministic headless inventory-item scene projection

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Extend the disposable Bevy scene mirror with complete core-owned actor inventory records. A typed
`SceneInventoryItem` carries each item's global identity, owner actor, opaque definition reference,
and insertion order; `sync_scene` updates those entities deterministically from the immutable actor
projections. Core remains authoritative for ownership and order, and this slice adds no item gameplay
or UI policy.

Acceptance:

- `SceneInventoryItem` carries typed `ItemId`, owner `ActorId`, `ItemDefinitionId`, and zero-based
  inventory order without mutable core storage, gameplay effects, or presentation policy.
- `sync_scene` creates, updates, and deduplicates inventory-item entities deterministically by global
  `ItemId`, preserves entity identity when an item remains projected, and removes stale items absent
  from later snapshots.
- Owner and order changes from a core-authoritative tester transfer update the retained item entity;
  complete keyed tile, actor, and ground-item mirrors remain unchanged.
- The projection remains headless and independent of rendering, camera, HUD, transport, desktop
  platform features, persistence, and new gameplay rules.

Verification:

- Focused `cargo test -p dreadstep-bevy --test scene_sync --all-features --locked` passes all nine
  inventory, transfer, duplicate-cleanup, stale-removal, and complete-mirror tests.
- `cargo test -p dreadstep-bevy --all-targets --all-features --locked`, focused Clippy, Cargo docs,
  `git diff --check`, and `scripts/verify.sh` pass.
- Exactly one semantic code reviewer reports PASS on implementation revision `c1057c7`, and Linux,
  Apple Silicon macOS, and Windows CI are green for that revision.

Out of scope:

- Player inventory commands, item effects, equipment, capacity, identification, rendering, sprites,
  camera policy, HUD widgets, visibility, persistence, transport, and new gameplay rules.

## Future

### Milestone 4 slice: deterministic authored starter-floor item placements

- Status: active
- Started: 2026-08-09

Add the smallest content-to-core bridge for authored opaque item instances without changing the
default starter scenario. `StarterFloorDefinition` will accept an ordered list of typed
`StarterItemPlacement` values, and its validated build will delegate each placement to core's
existing `WorldState::give_item` operation. Item definition references remain opaque; catalog
membership, item effects, capacity, player commands, and ground placement are not inferred here.

Acceptance:

- `StarterItemPlacement` carries a typed target `ActorId` and complete core `Item`; placement
  declaration order becomes the target actor's deterministic inventory order.
- A floor with valid placements builds an equivalent core world with those items, while the
  existing default `starter_floor_definition()` remains item-free and deterministic.
- Unknown target actors and duplicate item identities return typed `ContentError::World` before
  the built world is returned; no partial `WorldState` escapes a failed content build.
- The content boundary remains independent of Bevy, MCP, transport, persistence, item effects,
  equipment, capacity, identification, ground stacks, and player-facing item commands. Core still
  owns item identity, inventory state, digest inclusion, and all mutation rules.

Verification:

- Focused `cargo test -p dreadstep-content --test starter_items --all-features --locked` covers
  ordered valid placements, default-floor stability, unknown actors, and duplicate identities.
- Existing core item tests plus `cargo test -p dreadstep-content --all-targets --all-features
  --locked`, focused Clippy, Cargo docs, `git diff --check`, and `scripts/verify.sh` pass before
  handoff.
- Exactly one semantic code reviewer must review the cross-boundary implementation, and Linux,
  Apple Silicon macOS, and Windows CI must be green for the reviewed revision before closeout.

Out of scope:

- Changing the default starter-floor contents, item-definition catalog membership checks, item
  effects, equipment, capacity, identification, pickup/drop/transfer commands, ground placement,
  player replay/history, protocol/MCP operations, persistence, serialization, rendering, or UI.

### Milestone 4 slice: deterministic content item-definition catalog

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Add a content-owned catalog of opaque `ItemDefinitionId` values as the smallest foundation for
future item authoring. `ItemCatalogDefinition` preserves authored order and validates globally
duplicate definition identities before producing an immutable `ItemCatalog`; core still treats
those IDs as opaque references and no catalog data enters `WorldState` automatically.

Acceptance:

- A valid catalog preserves declaration order, exposes read-only deterministic definitions, and
  answers known/unknown membership without hash iteration or mutation.
- Duplicate definition IDs return a typed `ContentError` before a catalog is constructed; repeated
  construction of the starter catalog produces equal ordered values.
- Content owns only definition membership. Item instances, ownership, digests, and snapshots remain
  core-owned; no effects, equipment, identification, capacity, transfer, or player commands are
  introduced.
- The catalog remains independent of Bevy, MCP, transport, filesystem authoring, serialization,
  procedural generation, and presentation policy.

Verification:

- Focused `cargo test -p dreadstep-content --test item_catalog --all-features --locked` covers
  stable starter order, known/unknown lookup, repeatability, and duplicate rejection; all three
  focused tests pass.
- `cargo test -p dreadstep-content --all-targets --all-features --locked`, focused Clippy,
  `cargo doc`, and `scripts/verify.sh` pass; Linux, Apple Silicon macOS, and Windows CI are green
  for the reviewed revision.
- `git diff --check` passes, and exactly one semantic code reviewer reports pass at the final
  revision (`655b042`).

Out of scope:

- Item effects, equipment, consumables, affixes, rarity, identification, transfer, capacity,
  pickup/drop/use commands, protocol/MCP operations, persistence, serialization, and UI.

### Deferred item gameplay semantics

The opaque ownership slice and content catalog foundation intentionally do not define item effects,
equipment, identification, capacity, or gameplay-facing item commands. Tester-only transfer, drop,
and pickup are specified separately below; richer player operations still require an explicit core
contract.

### Milestone 4 slice: deterministic tester item transfer

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Add an in-memory tester mutation that transfers one existing opaque item instance between actor
inventories. Core validates source ownership and actor identities, preserves item identity and
relative ordering, appends to the target, and treats same-actor transfer as an idempotent no-op.
Protocol maps the typed world error and MCP exposes the operation through `Session`; it remains
outside player history/replay and has no stdio tool or gameplay effects.

Acceptance:

- A successful cross-actor transfer changes the deterministic world digest, removes the item from
  the source without reordering its remaining items, and appends the unchanged item to the target.
- Unknown source/target actors and a source that does not own the item return typed errors before
  mutation; same-actor transfer of an owned item succeeds without changing world state.
- Dead actor records remain valid transfer endpoints; accepted tester transfers do not enter player
  history or replay evidence, while rejected transfers preserve world, history, and replay exactly.
- Core remains authoritative for ownership and item data; protocol/MCP only convert and expose the
  tester operation, with no player command, stdio registration, or item gameplay semantics.

Verification:

- Focused core, protocol, and MCP item-transfer tests cover success/order/digest, same-actor
  idempotence, dead records, typed rejection/atomicity, and replay/history preservation.
- `cargo test -p dreadstep-core --test item_transfer --all-features --locked`,
  `cargo test -p dreadstep-protocol --test item_transfer --all-features --locked`,
  `cargo test -p dreadstep-protocol --test world_error --all-features --locked`, and
  `cargo test -p dreadstep-mcp --test tester_item_transfer --all-features --locked` pass.
- Focused Clippy, Cargo docs, `git diff --check`, and `scripts/verify.sh` pass.
- Exactly one semantic code reviewer reports pass on the implementation revision (`921a227`), and
  Linux, Apple Silicon macOS, and Windows CI are green for that revision.

Out of scope:

- Item effects, equipment, consumables, affixes, rarity, identification, capacity, pickup/drop/use,
  player commands, stdio/MCP transport registration, persistence, and UI.

### Milestone 4 slice: deterministic tester item drop

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Add an in-memory tester mutation that drops one owned opaque item instance at its actor's current
map position. Core owns the ground-item records and deterministic stack ordering; protocol and MCP
project the new read-only ground-item snapshot and expose only the tester mutation. This slice does
not define pickup, item effects, equipment, capacity, or a player command.

Acceptance:

- A successful drop removes the unchanged item from the actor inventory while preserving the
  relative order of remaining items, appends it to the ordered ground stack at the actor's current
  position, and changes the deterministic world digest.
- Ground stacks are keyed by stable map position and projected in row-major position order; item
  order within each stack is insertion order. Dead actor records remain valid drop sources at their
  retained position.
- Unknown actors and source items not owned return typed errors before mutation. Duplicate item
  identity checks for `give_item` include ground items, so the global instance invariant remains
  atomic and explicit.
- Protocol version 2 exposes ground-item snapshots without inventing rules, and MCP delegates the
  tester drop without recording player history or replay evidence. Core remains authoritative.

Verification:

- Focused core drop tests cover ordered stacks, source-order preservation, dead sources, typed
  rejection/atomicity, and digest changes; protocol snapshot tests cover row-major projection and
  complete item data; MCP tests cover accepted/rejected history and replay invariants.
- `cargo test -p dreadstep-core --test item_drop --all-features --locked`,
  `cargo test -p dreadstep-protocol --test item_drop --all-features --locked`, and
  `cargo test -p dreadstep-mcp --test tester_item_drop --all-features --locked` pass.
- Focused Clippy, Cargo docs, `git diff --check`, and `scripts/verify.sh` pass; exactly one
  semantic code reviewer reports pass on implementation revision `4255a58`; Linux, Apple Silicon
  macOS, and Windows CI are green for that revision.

Out of scope:

- Pickup, item effects, equipment, consumables, affixes, rarity, identification, capacity,
  player commands, stdio/MCP transport registration, persistence, serialization, and UI.

### Milestone 4 slice: deterministic tester item pickup

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Add the inverse tester mutation for an opaque item on the ground. Core validates an existing actor
and the requested item in that actor's current ground stack, removes the item while preserving the
remaining stack order, and appends it unchanged to the actor's ordered inventory. Protocol maps a
typed ground-miss error and reuses the version-2 ground snapshot; MCP exposes only the in-memory
tester mutation. No player command or gameplay effect is introduced.

Acceptance:

- A successful pickup changes the deterministic world digest, removes the unchanged item from the
  actor's current ground stack, removes an empty stack, and appends the item to the actor inventory
  without reordering existing inventory items.
- Unknown actors and items absent from the actor's current ground stack return typed errors before
  mutation; dead actor records remain valid sources at their retained position.
- Core remains authoritative for the global item identity and position/order invariants. Protocol
  projects the typed error and complete ground-item snapshot; MCP leaves player history and replay
  evidence unchanged for accepted and rejected tester pickups.

Verification:

- Focused core pickup tests cover stack-order preservation, inventory append order, stack cleanup,
  dead sources, digest changes, and typed atomic rejection; protocol tests cover exhaustive error
  mapping; MCP tests cover complete snapshot plus history/replay invariants.
- `cargo test -p dreadstep-core --test item_pickup --all-features --locked`,
  `cargo test -p dreadstep-protocol --test item_pickup --all-features --locked`,
  `cargo test -p dreadstep-protocol --test world_error --all-features --locked`, and
  `cargo test -p dreadstep-mcp --test tester_item_pickup --all-features --locked` pass.
- Focused Clippy, Cargo docs, `git diff --check`, and `scripts/verify.sh` pass; exactly one
  semantic code reviewer reports pass on implementation revision `d7dac61`; Linux, Apple Silicon
  macOS, and Windows CI are green for that revision.

Out of scope:

- Player pickup commands, item effects, equipment, consumables, affixes, rarity, identification,
  capacity, stdio/MCP transport registration, persistence, serialization, and UI.

### Milestone 1: Rules kernel

Implement the deterministic headless simulation described in the proposal: typed world
state, commands and events, seeded randomness, movement, blocking, combat, scheduling,
replay evidence, and a developer CLI.
