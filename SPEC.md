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

## Future

### Deferred item gameplay semantics

The opaque ownership slice intentionally does not define item effects, equipment, identification,
capacity, transfer, or content catalogs. These contracts must be specified in core before adding
gameplay-facing item commands or richer tester operations.

### Milestone 1: Rules kernel

Implement the deterministic headless simulation described in the proposal: typed world
state, commands and events, seeded randomness, movement, blocking, combat, scheduling,
replay evidence, and a developer CLI.
