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

### Milestone 3 slice: deterministic headless camera anchor

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Add the next human-presentation boundary without enabling a desktop renderer. The Bevy adapter
projects the selected actor's authoritative position into a typed `PresentationCamera` resource
and a single disposable `SceneCamera` ECS component. The anchor follows the existing controlled
actor identity, clears for an unknown actor, and never becomes a second source of world state.

Acceptance:

- `PresentationCamera` exposes one typed controlled `ActorId` and an optional core `Position`;
  the plugin updates it from the current runtime snapshot after keyboard dispatch.
- A headless `SceneCamera` mirror contains only the projected center position. Synchronization is
  deterministic, deduplicates stale camera entities, and preserves the selected camera entity
  while its key remains valid.
- Accepted keyboard movement and controlled-actor changes update the resource and scene anchor in
  the same app update; unknown actors clear the center and scene anchor without mutating runtime,
  replay evidence, or complete keyed tile/actor/item projections.
- Missing runtime, input, or camera resources are safe no-ops. The anchor remains presentation-only
  and adds no windowing, transforms, viewport math, visibility/fog, rendering, transport, or rules.

Verification:

- Focused `cargo test -p dreadstep-bevy --test camera --all-features --locked` covers startup,
  movement, actor changes, unknown actors, duplicate cleanup, and absent-resource no-ops; all nine
  focused tests pass, including complete tile/actor/ground/inventory atomicity and recycled-index
  cleanup.
- All Bevy targets, focused Clippy with `-D warnings`, Cargo docs, formatting, `git diff --check`,
  and `scripts/verify.sh` pass locally.
- Exactly one semantic code reviewer reports PASS on the final revision, and the normal CI matrix
  remains green.

Out of scope:

- Window or camera plugins, transforms, viewport sizing or clamping, interpolation, smoothing,
  rendering, sprites, HUD, visibility/fog rules, input rebinding, transport, and new gameplay.

### Milestone 3 slice: deterministic headless viewport projection

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Extend the headless camera boundary with an explicit viewport size and deterministic map clamping.
`PresentationViewport` owns only the requested tile dimensions and the latest projected origin;
`SceneViewport` mirrors the effective in-map rectangle for future renderers. The projection follows
the authoritative camera center and remains disposable adapter state.

Acceptance:

- `PresentationViewport::new` rejects zero-sized viewports and exposes typed requested dimensions,
  an optional in-map origin, and effective dimensions after map clamping.
- The plugin centers the effective rectangle on the camera anchor, clamps it to the snapshot map,
  and updates both resource and one disposable `SceneViewport` entity deterministically in one app
  update after keyboard dispatch and camera synchronization.
- Viewports larger than the map shrink to the full map. Accepted actor movement updates the
  authoritative runtime, replay, and actor projection through core while moving the viewport;
  selection-only controlled-actor changes update the origin without mutating runtime, replay, or
  complete keyed tile/actor/item mirrors.
- Unknown actors clear the origin and scene viewport. Missing runtime, input, or viewport resources
  are safe no-ops, and duplicate viewport entities are reduced to one retained identity.
- The projection remains headless and adds no window/camera plugins, transforms, interpolation,
  rendering, visibility/fog policy, transport, persistence, or gameplay rules.

Verification:

- Focused `cargo test -p dreadstep-bevy --test viewport --all-features --locked` covers validation,
  startup, movement, edge/oversize clamping, actor changes, unknown actors, duplicate cleanup, and
  absent-resource no-ops; all thirteen focused tests pass.
- All Bevy targets, focused Clippy with `-D warnings`, Cargo docs, formatting, `git diff --check`,
  and `scripts/verify.sh` pass locally.
- Exactly one semantic code reviewer reports PASS on the final revision, and the normal CI matrix
  remains green.

### Milestone 3 slice: deterministic Bevy starter-item run projection

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

The explicit Bevy startup bridge for the verified non-default content scenario is complete.
`PresentationState::start_item_run` and `PresentationRuntime::start_item_run` consume
`dreadstep-content::starter_item_floor()` while preserving the caller's seed; the existing
`start_run` constructors continue to consume the item-free starter floor. The existing plugin
projects the scenario's core-owned inventory items through `SceneInventoryItem` during its normal
headless synchronization.

Acceptance:

- Item-run state/runtime constructors produce the same complete map, actor, inventory, ground,
  scheduler, digest, and empty replay projection as the content scenario with the explicit seed.
- A Bevy `App` with `PresentationPlugin` and item-run runtime creates the complete tile/actor and
  typed inventory-item scene projection, preserving item ID, owner, opaque definition, and
  insertion order; no ground item or duplicate scene entity appears.
- The default `start_run` path remains item-free and unchanged, and both constructors preserve
  core authority, replay behavior, and deterministic snapshots.
- This remains a headless adapter bridge: no windowing, rendering, camera, HUD, item gameplay,
  player/tester commands, persistence, serialization, or transport behavior is introduced.

Verification:

- Focused Bevy startup tests cover item-run state/runtime equality, seed/replay preservation,
  complete tile/actor/inventory/ground scene projection, duplicate-entity cardinality, and default
  item-free startup stability. The focused `start_run` test passes all three tests.
- All Bevy targets, focused Clippy with `-D warnings`, Bevy Cargo docs, formatting,
  `git diff --check`, and `scripts/verify.sh` pass locally.
- Exactly one semantic code reviewer reports PASS on implementation revision `2df4d35`, and Linux,
  Apple Silicon macOS, and Windows CI are green for that revision.

Out of scope:

- Changing content scenario definitions, default starter contents, core/protocol/MCP APIs, item
  effects, equipment, capacity, identification, gameplay commands, ground placement, windowing,
  rendering, camera policy, HUD, persistence, serialization, transport, and UI.

### Milestone 4 slice: deterministic authored starter-item scenario

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

The reusable, non-default starter-floor scenario is complete. `starter_item_floor_definition()`
binds the existing starter item catalog and declares a small deterministic set of opaque item
instances in actor inventory order; `starter_item_floor()` builds that definition into a core world.
The existing `starter_floor()` scenario remains item-free and unchanged.

Acceptance:

- The item scenario uses the shared authored map and actors, binds the starter catalog, and
  preserves complete item identity/definition data plus declaration order within each inventory.
- `starter_item_floor_definition()` and `starter_item_floor()` are repeatable and produce equal
  core worlds/digests with empty ground stacks; the default starter scenario remains equal to its
  item-free definition and has no inventory items.
- The scenario delegates actor/item identity validation to `StarterFloorDefinition::build` and
  core; no catalog is stored in `WorldState` and no item effects or gameplay rules are inferred.
- The helper remains content-owned and independent of Bevy, MCP, transport, persistence,
  serialization, rendering, and player/tester item commands.

Verification:

- Focused content tests assert complete item data, interleaved declaration order, repeatability,
  item-free ground state, exact canonical map/actor/scheduler reuse, and preservation of the
  default item-free scenario.
- `cargo test -p dreadstep-content --test starter_items --all-features --locked` passes all five
  focused tests; `cargo test -p dreadstep-content --all-targets --all-features --locked` passes
  all ten content tests. Focused Clippy with `-D warnings`, content Cargo docs, formatting,
  `git diff --check`, and `scripts/verify.sh` pass locally.
- Exactly one semantic code reviewer reports PASS on implementation revision `e9f5810`, and
  Linux, Apple Silicon macOS, and Windows CI are green for that revision.

Out of scope:

- Changing the default starter scenario, item effects, equipment, capacity, identification,
  player/tester commands, ground placement, protocol/MCP/Bevy APIs, persistence, serialization,
  rendering, and UI.

### Milestone 4 slice: deterministic catalog-bound starter item placements

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

The authored starter-floor item placement binding is complete. `StarterFloorDefinition` validates
an explicit content-owned `ItemCatalogDefinition` and every placement's opaque `ItemDefinitionId`
before constructing the core world, while the catalog remains content authoring data and never
enters `WorldState`.

Acceptance:

- `StarterFloorDefinition::with_item_catalog` binds an explicit authored catalog, and `build`
  rejects duplicate definition IDs with the existing typed catalog error.
- Every authored placement must reference a catalog member; an unknown definition returns a typed
  content error before map/world construction, while valid placement order and complete item data
  remain unchanged.
- The default item-free starter floor remains deterministic and valid with its empty catalog, and
  the catalog is not copied into core snapshots, digests, protocol, MCP, or Bevy state.
- Core remains authoritative for actor/item identity, inventory order, digest state, and mutation;
  no item effects, capacity, equipment, commands, persistence, or transport behavior is added.

Verification:

- Focused `cargo test -p dreadstep-content --test starter_items --all-features --locked` covers
  catalog-bound success/order, catalog-order independence, duplicate catalog IDs, unknown
  definitions, and default stability; all four focused tests pass.
- `cargo test -p dreadstep-content --all-targets --all-features --locked` passes all nine content
  tests. Focused Clippy with `-D warnings`, content Cargo docs, `git diff --check`, and the full
  `scripts/verify.sh` suite pass locally.
- Exactly one semantic code reviewer reports PASS on implementation revision `1bec9fa`, and Linux,
  Apple Silicon macOS, and Windows CI are green for that revision.

Out of scope:

- Changing default starter-floor contents, core/protocol/MCP/Bevy APIs, item effects, capacity,
  equipment, identification, player/tester commands, ground state, persistence, serialization,
  rendering, or UI.

### Milestone 4 slice: deterministic authored starter-floor item placements

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

The smallest content-to-core bridge for authored opaque item instances is complete without
changing the default starter scenario. `StarterFloorDefinition` accepts an ordered list of typed
`StarterItemPlacement` values, and its validated build delegates each placement to core's existing
`WorldState::give_item` operation. Item definition references remain opaque; catalog membership,
item effects, capacity, player commands, and ground placement are not inferred here.

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
  interleaved ordered placements, repeatability, both item-free default constructors, unknown
  actors, and duplicate identities; all three focused tests pass.
- `cargo test -p dreadstep-content --all-targets --all-features --locked` passes all eight content
  tests. Focused Clippy with `-D warnings`, content Cargo docs, `git diff --check`, and the full
  `scripts/verify.sh` suite pass locally.
- Exactly one semantic code reviewer reports PASS on implementation revision `8b5d4d9`, and Linux,
  Apple Silicon macOS, and Windows CI are green for that revision.

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
- Protocol version 3 exposes ground-item snapshots and optional equipped-item identity without
  inventing rules, and MCP delegates the
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
typed ground-miss error and reuses the version-3 ground snapshot; MCP exposes only the in-memory
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

### Milestone 3 slice: deterministic headless HUD status projection

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Project the selected actor's authoritative status into a typed `PresentationHud` resource for a
future HUD without introducing text, layout, rendering, or UI policy. The resource keeps the
controlled actor identity and optional core kind, position, hit points, and scheduler readiness;
unknown actors clear the optional values rather than inventing presentation data.

Acceptance:

- `PresentationHud::new` stores one typed controlled `ActorId` and starts with no projected actor;
  getters expose the actor identity and optional typed status values.
- The plugin refreshes the resource from the current runtime snapshot after keyboard dispatch and
  scene synchronization, so startup, accepted movement, and controlled-actor selection are visible
  in the same app update.
- Selection-only actor changes update the HUD without mutating runtime, replay evidence, or complete
  keyed tile/actor/ground/inventory scene mirrors; accepted movement updates the authoritative core
  state and the HUD together.
- Unknown actors clear all optional status values. Missing runtime, input, or HUD resources are
  safe no-ops that preserve existing HUD state where a projection cannot be refreshed.
- The projection remains headless and typed: no strings, formatting, widgets, textures, windowing,
  rendering, animation, audio, persistence, transport, or gameplay rules are introduced.

Verification:

- Focused `cargo test -p dreadstep-bevy --test hud --all-features --locked` covers startup,
  movement, selection-only changes, unknown actors, and absent-resource preservation with complete
  non-empty scene atomicity; all seven focused tests pass.
- All Bevy targets, focused Clippy with `-D warnings`, Cargo docs, formatting, `git diff --check`,
  and `scripts/verify.sh` pass locally.
- Exactly one semantic code reviewer reports PASS on final implementation revision `3431d20`; the
  docs-only closeout is reviewed separately, and Linux, Apple Silicon macOS, and Windows CI remain
  green.

Out of scope:

- HUD widgets, text/localization, health-bar styling, inventory panels, event/combat messages,
  sprites, animations, audio, windowing, rendering, persistence, transport, and gameplay rules.

### Milestone 3 slice: deterministic headless event-message evidence

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Project core semantic events into a typed `PresentationMessages` resource for future HUD and combat
message systems. The adapter preserves event order and typed payloads without formatting strings,
choosing localization, or adding another source of gameplay truth.

Acceptance:

- `PresentationMessage` covers every current core event (`Moved`, `MovementBlocked`, `Waited`,
  `Attacked`, and `Died`) with typed actor, position, block-reason, damage, and hit-point data.
- `PresentationMessages` mirrors the latest runtime output in deterministic event order and clears
  stale messages when a rejected command produces no output; it remains read-only evidence and
  cannot issue commands or mutate core state.
- Accepted movement, blocked movement, waiting, attack, and death outputs map to the expected
  typed message sequence in the same app update after keyboard or direct runtime dispatch.
- Missing runtime or message resources are safe no-ops that preserve existing message evidence;
  no output remains an empty message list.
- The projection remains headless and presentation-only: no strings, localization, text layout,
  widgets, sprites, animation, audio, persistence, transport, or gameplay rules are introduced.

Verification:

- Focused `cargo test -p dreadstep-bevy --test messages --all-features --locked` covers every core
  event mapping, stale-output clearing, and absent-resource preservation; all eight focused tests
  pass.
- All Bevy targets, focused Clippy with `-D warnings`, Cargo docs, formatting, `git diff --check`,
  and `scripts/verify.sh` pass locally.
- Exactly one semantic code reviewer reports PASS on final implementation revision `51424ae`; the
  docs-only closeout is reviewed separately, and Linux, Apple Silicon macOS, and Windows CI remain
  green.

Out of scope:

- Text/localization, text layout, widgets, sprites, animation, audio, windowing, rendering,
  persistence, transport, and gameplay rules.

### Milestone 3 slice: deterministic headless audio-cue evidence

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Project current core events into a typed `PresentationAudioCues` resource as a placeholder boundary
for a future audio player. The adapter preserves event order and typed identities without loading
assets, playing sounds, formatting text, or adding another source of gameplay truth.

Acceptance:

- `PresentationAudioCue` covers every current core event (`Moved`, `MovementBlocked`, `Waited`,
  `Attacked`, and `Died`) with typed actor/target and block-reason data where the event carries it.
- `PresentationAudioCues` mirrors the latest runtime output in deterministic event order and clears
  stale cues when a rejected command produces no output; it remains read-only evidence and cannot
  issue commands or mutate core state.
- Accepted movement, blocked movement, waiting, attack, and death outputs map to the expected cue
  sequence in the same app update after keyboard or direct runtime dispatch.
- Missing runtime or cue resources are safe no-ops that preserve existing cue evidence; no output
  remains an empty cue list.
- The projection remains headless and placeholder-only: no audio assets, asset handles, playback,
  audio dependencies, strings, localization, widgets, sprites, animation, persistence, transport,
  or gameplay rules are introduced.

Verification:

- Focused `cargo test -p dreadstep-bevy --test audio_cues --all-features --locked` covers every core
  event mapping, deterministic order, stale-output clearing, and absent-resource preservation; all
  seven focused tests pass.
- All Bevy targets, focused Clippy with `-D warnings`, Cargo docs, formatting, repository checks,
  `git diff --check`, and `scripts/verify.sh` pass locally.
- Exactly one semantic code reviewer reports PASS on final implementation revision `81e444a`; the
  docs-only closeout is reviewed separately, and Linux, Apple Silicon macOS, and Windows CI remain
  green.

Out of scope:

- Audio assets, asset handles, playback, audio backends, text/localization, text layout, widgets,
  sprites, animation, windowing, rendering, persistence, transport, and gameplay rules.

### Milestone 3 slice: deterministic headless sprite-role metadata

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Attach a typed `SceneSpriteRole` to each existing headless Bevy scene mirror so a future renderer
can classify terrain, living actors, retained dead records, and item entities without inspecting
untyped ECS layout. The role is disposable metadata derived from core-owned scene components; it
does not select assets or become a second source of presentation state.

Acceptance:

- `SceneSpriteRole` classifies every synchronized mirror as `Terrain`, `Player`, `Enemy`,
  `DeadActor`, `GroundItem`, or `InventoryItem`.
- `sync_scene` inserts or refreshes exactly one role alongside each keyed `SceneTile`, `SceneActor`,
  `SceneGroundItem`, and `SceneInventoryItem`; actor roles follow authoritative kind/life data and
  stale entities are still removed by the existing stable keys.
- Role metadata preserves existing scene entity identity and complete typed tile/actor/item values;
  it cannot issue commands or mutate core state.
- Missing or unsynchronized scene data is not invented; no role is projected without the existing
  scene mirror, and no output adds hidden-information or visibility policy.
- The projection remains headless and metadata-only: no textures, asset handles, materials,
  transforms, window/render plugins, animation, audio, persistence, transport, or gameplay rules
  are introduced.

Verification:

- Focused `cargo test -p dreadstep-bevy --test sprite_roles --all-features --locked` covers every
  role variant, item mirrors, stable identity, stale cleanup, and actor role refresh; all five
  focused tests pass.
- All Bevy targets, focused Clippy with `-D warnings`, Cargo docs, formatting, repository checks,
  `git diff --check`, and `scripts/verify.sh` pass locally.
- Exactly one semantic code reviewer reports PASS on final implementation revision `4fad448`; the
  docs-only closeout is reviewed separately, and Linux, Apple Silicon macOS, and Windows CI remain
  green.

Out of scope:

- Text/localization, textures, asset handles, materials, transforms, window/render plugins,
  animation, audio, persistence, transport, and gameplay rules.

### Milestone 3 slice: deterministic headless animation-cue evidence

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Project current core events into a typed `PresentationAnimationCues` resource as a placeholder
signal for future movement and combat animation. The adapter preserves event order and typed
payloads without timers, interpolation, assets, or another source of gameplay truth.

Acceptance:

- `PresentationAnimationCue` covers every current core event (`Moved`, `MovementBlocked`, `Waited`,
  `Attacked`, and `Died`) with typed actor, position, block-reason, damage, and hit-point data where
  the event carries it.
- `PresentationAnimationCues` mirrors the latest runtime output in deterministic event order and
  clears stale cues when a rejected command produces no output; it remains read-only evidence and
  cannot issue commands or mutate core state.
- Accepted movement, blocked movement, waiting, attack, and death outputs map to the expected cue
  sequence in the same app update after keyboard or direct runtime dispatch.
- Missing runtime or cue resources are safe no-ops that preserve existing cue evidence; no output
  remains an empty cue list.
- The projection remains headless and signal-only: no timers, interpolation, animation state
  machine, textures, asset handles, materials, transforms, window/render plugins, audio playback,
  persistence, transport, or gameplay rules are introduced.

Verification:

- Focused `cargo test -p dreadstep-bevy --test animation_cues --all-features --locked` covers every
  core event mapping, deterministic order, stale-output clearing, and absent-resource preservation;
  all seven focused tests pass.
- All Bevy targets, focused Clippy with `-D warnings`, Cargo docs, formatting, repository checks,
  `git diff --check`, and `scripts/verify.sh` pass locally.
- Exactly one semantic code reviewer reports PASS on final implementation revision `5fbbfe6`; the
  docs-only closeout is reviewed separately, and Linux, Apple Silicon macOS, and Windows CI remain
  green.

Out of scope:

- Timers, interpolation, animation state machines, textures, asset handles, materials, transforms,
  window/render plugins, audio playback, persistence, transport, and gameplay rules.

### Milestone 3 slice: deterministic headless window request

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Define a typed `PresentationWindow` request with logical dimensions and an integer pixel scale for a
future desktop client. The headless adapter validates checked physical dimensions without creating
an OS window, enabling desktop features, or adding another source of presentation state.

Acceptance:

- `PresentationWindow::new` accepts nonzero logical width/height and pixel scale, exposes each typed
  value, and computes checked physical width/height deterministically.
- Zero dimensions/scale and physical-size multiplication overflow are rejected without panic or
  partial configuration.
- The request is ordinary typed configuration that can be inserted as a Bevy resource but does not
  create a window, process OS events, choose a default resolution, or mutate runtime/scene state.
- The projection remains headless and configuration-only: no window/render plugins, desktop
  backends, transforms, textures, assets, audio, timers, persistence, transport, or gameplay rules
  are introduced.

Verification:

- Focused `cargo test -p dreadstep-bevy --test window_request --all-features --locked` covers valid
  dimensions, deterministic physical sizes, zero values, overflow, and resource equality; all four
  focused tests pass.
- All Bevy targets, focused Clippy with `-D warnings`, Cargo docs, formatting, repository checks,
  `git diff --check`, and `scripts/verify.sh` pass locally.
- Exactly one semantic code reviewer reports PASS on final implementation revision `d864a32`; the
  docs-only closeout is reviewed separately, and Linux, Apple Silicon macOS, and Windows CI remain
  green.

Out of scope:

- OS windows, platform event loops, desktop backends, rendering, transforms, textures, assets,
  audio, timers, persistence, transport, and gameplay rules.

### Milestone 3 slice: deterministic scene pixel placement

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Define a typed, headless placement boundary between core map coordinates and a future renderer.
`PresentationTileSize` accepts a caller-selected logical tile extent without choosing between the
proposal's 24×24 and 32×32 candidates. When present, the Bevy presentation plugin projects checked
logical-pixel origins through `ScenePixelPosition` on existing terrain, actor, and ground-item
mirrors. The projection remains disposable metadata and never becomes a second source of scene or
simulation truth.

Acceptance:

- `PresentationTileSize::new` rejects zero width or height and exposes validated dimensions.
- Coordinate conversion rejects negative positions and checked multiplication overflow; valid
  positions map deterministically to logical-pixel origins on both axes.
- Terrain, actor, and ground-item mirrors receive refreshed `ScenePixelPosition` values after
  synchronization, and retained keyed actor entities keep their identity after accepted movement.
- Missing tile-size configuration leaves the existing headless scene unchanged and adds no pixel
  metadata; inventory items remain unplaced because they have no map coordinate.
- Missing runtime authority preserves complete existing keyed scene values and pixel metadata
  without deriving a new scene state.
- The boundary adds no Bevy `Transform`, textures, asset handles, window/render plugins, audio,
  timers, interpolation, visibility policy, persistence, transport, or gameplay rules.

Verification:

- Focused `cargo test -p dreadstep-bevy --test tile_layout --all-features --locked` passes all six
  tests covering valid/invalid configuration, both arithmetic axes, all terrain origins,
  actor/ground placement, inventory exclusion, retained identity, and authority/resource absence.
- All Bevy targets, focused Clippy with `-D warnings`, Cargo docs, formatting, repository checks,
  `git diff --check`, and `scripts/verify.sh` pass locally.
- Exactly one semantic reviewer reports PASS on final reviewed revision `c38e6a9`; the initial
  implementation commit is `d877179`, and the test-only evidence correction is the final reviewed
  revision. Linux, Apple Silicon macOS, and Windows CI are green.

Out of scope:

- Tile-size/asset selection, OS windows, platform event loops, desktop backends, rendering,
  transforms, textures, asset handles, audio, timers, interpolation, persistence, transport, and
  gameplay rules.

### Milestone 3 slice: presentation asset evaluation

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Evaluate representative original/generated and free reusable candidates before enabling a real
renderer. Keep all binaries local-only while recording source, creator, license, attribution, and
modification status in tracked [`docs/presentation/asset-evaluation.md`](docs/presentation/asset-evaluation.md).
The generated sheets remain visual direction only; tile-size selection was deferred to the separately
verified native-sample slice below. The audio result remains bounded: generated timing/UI and Kenney
CC0 UI cues are evaluated, while dungeon combat/movement/item cue sourcing is explicitly deferred.

Acceptance:

- Unconstrained original/generated contact sheets are retained locally as visual-direction evidence;
  native/normalized tile-size evidence is verified separately below.
- A free reusable pixel-art candidate and a free reusable audio candidate are retained locally with
  factual source, creator, license, attribution, modification, and SHA-256 records.
- A generated/original audio cue candidate is evaluated for timing and level, without claiming a
  distribution license or integrating it into Bevy playback.
- The mixed pixel-art fallback and unresolved dungeon audio sourcing are recorded rather than
  silently treated as complete.
- Root/crate-local `assets/`, `art/`, and `audio/` binaries remain ignored; tracked concept art and
  root `screenshots/` remain visible; no binary is loaded by code or committed.

Verification:

- `git check-ignore --no-index` covers local candidates and `git ls-files` confirms no candidate
  binary is tracked; repository checks, `git diff --check`, and full `scripts/verify.sh` pass.
- Exactly one semantic reviewer reports PASS on final reviewed revision `a3b7cc0` for the initial
  asset-evaluation record; PR #57 merged as `c98b5a9` with Linux, Apple Silicon macOS, and Windows
  CI green. Native tile-size evidence is verified separately below.

### Milestone 3 slice: native tile-size evidence

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Resolve the tile-size decision with exact nearest-neighbor samples from the official 16×16 CC0
source. Keep the archive and output sheets under ignored `art/`, record source IDs, method,
dimensions, hashes, and comparison in [`docs/presentation/tile-samples.md`](docs/presentation/tile-samples.md),
and leave production asset loading deferred. Twelve named source tiles produce exact 24×24 and
32×32 4×3 sheets; the comparison supports 32×32 as the provisional logical working size while
retaining 24×24 as a valid typed option.

Verification:

- Exact source/archive/output hashes and dimensions, independent pixel/order checks, repository
  checks, `git diff --check`, and full `scripts/verify.sh` pass.
- Exactly one semantic reviewer reports PASS on final reviewed implementation/evidence revision
  `837c91b` (implementation `c0d1cfc`); Linux, Apple Silicon macOS, and Windows CI are green.
- Local media remains ignored and tracked concept-art/screenshot exceptions remain visible; no
  production asset loading or runtime code is introduced.

### Milestone 3 slice: reversible headless-to-renderer spike

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Consume the verified 32×32 placement and sprite-role decisions at a reversible Bevy presentation
boundary. The spike must make the mapping from keyed `Scene*` mirrors to render-ready metadata
observable without creating a second source of truth, while keeping all local media ignored and
avoiding production asset loading until the boundary is proven.

Acceptance:

- `PresentationRenderProjection` exposes an ordered read-only `SceneRenderEntry` slice containing
  complete keyed terrain, actor, ground-item, and inventory metadata after one app update, with
  `ScenePixelPosition` only on terrain, actors, and ground items because inventory remains unplaced;
  each entry receives the role derived for its typed mirror, including independent roles when kinds
  share one ECS entity.
- The boundary consumes a caller-selected 32×32 `PresentationTileSize` without mutating core,
  replay/history, or existing headless scene mirrors.
- Missing runtime, missing configuration, stale keyed entities, and recycled Bevy entity indices
  preserve or clear the render-ready projection deterministically according to authority rules.
- The spike introduces no production texture/asset handles, transforms, window/render plugins,
  animation timers, audio playback, visibility policy, persistence, transport, or gameplay rules.
- Local media remains ignored and tracked concept-art/screenshot exceptions remain visible.

Verification:

- Focused `cargo test -p dreadstep-bevy --test render_projection --all-features --locked` proves
  complete entries, identity/authority behavior, 32×32 placement, inventory exclusion, duplicate
  ordering, and read-only runtime state; the focused suite passes 10/10, and all Bevy targets,
  Clippy, docs, repository checks, and `scripts/verify.sh` pass.
- Exactly one semantic reviewer reports PASS on final implementation/evidence revision `7cb1647`
  (implementation `c5d29a0`, with bounded corrections in `0ae06a8`, `1beb836`, and `7cb1647`);
  the docs-only closeout is reviewed separately, and Linux, Apple Silicon macOS, and Windows CI are
  green.
- The ignored render-boundary evidence records the intended red contract failure, focused 10/10,
  all-target Clippy/docs/repository/full verification, and the reviewer/CI gate.

Out of scope:

- OS windows, platform event loops, desktop backends, rendering plugins, transforms, textures,
  asset handles/loading, audio playback, animation timers/interpolation, visibility policy,
  persistence, transport, and gameplay rules.

### Milestone 4 preparation slice: deterministic single-slot item equipment

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

The first gameplay-facing item semantic is now verified without item effects: a scheduled living
actor may equip one owned opaque item instance or unequip its current reference. Equipment remains
an optional `ItemId` pointing into the actor's ordered inventory, so core owns one item store and
content catalogs remain authoring-only.

Acceptance:

- Typed `Command::Equip` and `Command::Unequip` validate actor scheduling, life, ownership, and
  current equipment; replacement emits deterministic unequip-before-equip events, while unequip
  clears only the optional reference.
- Accepted equipment actions consume one standard action and enter replay history; world digests and
  snapshots include the optional equipped identity. Rejected commands preserve complete world,
  replay, digest, and snapshot state.
- Tester drop/transfer reject equipped items atomically rather than silently clearing the slot.
- Protocol command/event/error/snapshot mappings and MCP action/history/replay evidence preserve the
  typed optional equipment field; Bevy `SceneActor` mirrors the same field without a second store.
- No item effects, stat modifiers, consumables, capacity, additional slots, rendering, assets,
  windowing, audio, persistence, transport, or dependencies are introduced.

Verification:

- Focused core equipment tests pass 5/5, protocol JSON tests pass 5/5, MCP equipment tests pass
  3/3, and Bevy equipment projection tests pass 2/2. The tests isolate equipped identity in both
  state and replay digests, prove tester atomicity, round-trip every equipment command/event, and
  retain actor/inventory entities across replacement and unequip cues.
- All workspace targets, all-target Clippy with `-D warnings`, warning-denied workspace docs,
  repository checks, `git diff --check`, and `scripts/verify.sh` pass.
- Exactly one semantic reviewer reports PASS on final implementation/evidence revision `7bc400a`
  (implementation `2a7d814`, bounded CI-golden correction `27dcae5`); this docs-only closeout is
  reviewed separately, and Linux, Apple Silicon macOS, and Windows CI are green.
- The ignored equipment evidence records the initial red compile target, the observed Linux digest
  golden correction, focused counts, full verification, and reviewer/CI gate.

Out of scope:

- Item effects, stat modifiers, consumables, capacity, additional slots, rendering, assets,
  windowing, audio playback, persistence, transport, and dependencies.

### Milestone 4 preparation slice: deterministic single-item consumption

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

The deterministic item-use boundary is now verified: a scheduled living actor may consume one
owned, unequipped opaque item instance. Consumption removes that instance and emits a typed event;
it does not infer an effect, alter stats, or introduce a second item store.

Acceptance:

- Typed `Command::UseItem` validates actor scheduling, life, ownership, and equipment, while
  rejected commands preserve complete world and digest state.
- An accepted consumption advances one standard action, removes exactly the requested inventory
  instance, emits `ItemConsumed`, and records the command in replay history/digest evidence.
- Legal-action discovery includes each owned unequipped item in deterministic inventory order and
  excludes the currently equipped item.
- Protocol JSON, MCP action/history/replay/snapshot projections, and Bevy scene/message/audio/
  animation projections preserve the typed command/event and remove only the stale inventory
  mirror; retained actor and remaining-item identities stay stable.
- No item effects, stat modifiers, capacity, identification, additional slots, rendering, assets,
  audio playback, windowing, persistence, transport, or dependencies are introduced.

Verification:

- Focused core consumption tests pass 4/4; protocol command/event/JSON tests pass 2/1/5; MCP
  consumption tests pass 2/2; and Bevy consumption projection tests pass 1/1. The evidence covers
  accepted action-time transition, unscheduled/dead/unknown/equipped atomic rejection, legal order,
  replay identity, consumed-identity reuse, complete snapshot/history evidence, stale inventory
  cleanup, retained scene identity, and all three ordered typed cue projections.
- All workspace tests, all-target Clippy with `-D warnings`, warning-denied workspace docs,
  repository checks, `git diff --check`, and full `scripts/verify.sh` pass.
- Exactly one semantic reviewer reports PASS on final reviewed evidence revision `148a6b2`
  (implementation `f270006`, legal-command correction `946afa4`); Linux, Apple Silicon macOS,
  and Windows CI are green. The docs-only closeout is reviewed separately.
- The ignored consumption evidence records the initial red compile target, observed Linux stale
  legal-order failure and correction, focused counts, full verification, reviewer/CI gate, merge,
  and branch cleanup.

Out of scope:

- Item effects, stat modifiers, capacity, identification, additional slots, rendering, assets,
  windowing, audio playback, persistence, transport, and dependencies.

### Milestone 3 slice: typed sprite-key projection

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Extend the verified headless render boundary with a closed typed content selector for each complete
render entry. Terrain keys retain the typed tile, actor keys distinguish living player/enemy and
dead records, and item keys retain opaque definition identity. The projection remains read-only and
does not load or commit presentation media.

Acceptance:

- `SceneSpriteKey` exhaustively maps terrain, player, enemy, dead-actor, ground-item, and
  inventory-item families without string paths or catalog copies.
- `PresentationSpriteProjection` preserves the ordered complete `SceneRenderEntry`, retained ECS
  entity, typed role, and optional pixel metadata; inventory entries remain unplaced.
- Accepted scene updates refresh stale keys and role changes while retaining keyed identity;
  missing runtime authority preserves the prior sprite projection, and missing projection resources
  are safe no-ops.
- The slice adds no texture handles, asset loading, render plugins, transforms, windowing, audio
  playback, committed binaries, gameplay rules, persistence, transport, or dependencies.

Verification:

- Focused `cargo test -p dreadstep-bevy --test sprite_keys --locked` passes all four tests covering
  exhaustive ordered keys, complete render-entry retention, authoritative actor-key derivation at
  the public entry boundary, dead-role refresh with retained entity identity, missing-runtime and
  upstream-resource preservation, inventory-unplaced metadata, and no-op destination guards.
- All Bevy targets, warning-denied Clippy/docs, formatting, repository checks, `git diff --check`,
  and `scripts/verify.sh` pass locally.
- Exactly one semantic reviewer reports PASS on final implementation/evidence revision `2ccfa5b`
  (initial implementation `c997025`); Linux, Apple Silicon macOS, and Windows CI are green. The
  docs-only closeout is reviewed separately.

Out of scope:

- Texture handles, asset loading, render plugins, transforms, windowing, audio playback, committed
  binaries, gameplay rules, persistence, transport, and dependencies.

### Milestone 3 slice: deterministic render-command plan

- Status: verified
- Started: 2026-08-09
- Completed: 2026-08-09

Derive an ordered, read-only draw-command plan from the verified sprite projection. Each command
retains the complete typed entry, stable ECS identity, sprite key, optional map placement, and a
deterministic layer/order value that a later renderer can consume without becoming simulation
authority.

Acceptance:

- `PresentationRenderCommandPlan` exposes a read-only ordered slice of commands derived from
  `PresentationSpriteProjection`; commands preserve complete render values and stable keyed entity
  identity.
- `SceneRenderLayer` classifies terrain, ground items, actors, and inventory metadata with explicit
  deterministic ordering, while original source order remains available for stable same-layer
  handling.
- Map-backed commands preserve checked `ScenePixelPosition` values and inventory commands remain
  unplaced; accepted updates refresh dead roles and stale entries without mutating runtime, replay,
  or core state.
- Missing runtime authority preserves the prior plan, and missing source or destination resources
  are safe no-ops.
- The slice adds no texture handles, asset loading, render plugins, transforms, windowing, audio
  playback, committed binaries, gameplay rules, persistence, transport, or dependencies.

Verification:

- Focused `cargo test -p dreadstep-bevy --test render_command_plan --locked` passes all four tests
  covering complete-entry retention, layer/source-order mapping, exact map placement and inventory
  exclusion, retained dead-actor identity with stale-key removal, and independent authority/resource
  guards.
- All Bevy targets, warning-denied Clippy/docs, formatting, repository checks, `git diff --check`,
  and `scripts/verify.sh` pass locally.
- Exactly one semantic reviewer reports PASS on final reviewed evidence/docs revision `4840b5d`
  (initial implementation `8ff0b00`, behavior correction `a650755`); Linux, Apple Silicon macOS,
  and Windows CI are green. PR #65 merged as `7ca9a24`; this docs-only closeout is reviewed
  separately.

Out of scope:

- Texture handles, asset loading, render plugins, transforms, windowing, audio playback, committed
  binaries, gameplay rules, persistence, transport, and dependencies.

### Milestone 3 slice: deterministic placeholder render-node bootstrap

- Status: verified
- Started: 2026-08-10
- Completed: 2026-08-10

Reconcile stable ECS placeholder nodes from the verified typed render-command plan. This is the
first renderer-facing node boundary: node entities retain source mirror identity, typed key, layer,
source order, placement, and a placeholder family while actual Bevy render plugins and production
media remain deferred.

Acceptance:

- `PresentationRenderNodeProjection` exposes ordered read-only node entries derived from
  `PresentationRenderCommandPlan`; each node has a stable ECS entity independent of the source
  mirror entity and retains the complete typed command metadata.
- Role changes such as living enemy to retained dead actor update the same source/layer node entity;
  stale commands are removed, and co-located source mirrors remain independently representable.
- Missing runtime or command-plan resources preserve existing nodes, and a missing destination
  projection is a safe no-op.
- Placeholder families are typed metadata only; no production `Sprite`, texture handle, asset
  loading, OS window, render plugin, transform, audio playback, animation, gameplay rule,
  persistence, transport, dependency, or committed media binary is introduced.

Verification:

- Focused `cargo test -p dreadstep-bevy --test render_bootstrap --locked` passes all six tests
  covering ordered placeholder nodes, complete command metadata, stable source/layer identity,
  stale inventory cleanup with despawn, co-located source mirrors, and independent authority/resource
  guards.
- All Bevy targets, warning-denied Clippy/docs, formatting, repository checks, `git diff --check`,
  and `scripts/verify.sh` pass locally.
- Exactly one semantic reviewer reports PASS on final evidence revision `ab9b576` (initial
  implementation `0915b70`); Linux, Apple Silicon macOS, and Windows CI are green. PR #67 merged
  as `595529f`; this docs-only closeout is reviewed separately.

Out of scope:

- Production Sprite components, texture handles, asset loading, render plugins, OS windows,
  transforms, audio playback, animation, gameplay rules, persistence, transport, dependencies, and
  committed media binaries.

## Present

### Milestone 3 slice: validated local-only presentation asset manifest

- Status: active
- Started: 2026-08-10

Define a metadata-only asset boundary for the verified placeholder render nodes. The manifest
must use validated relative repository paths, cover every typed placeholder family exactly once,
and join references to the ordered node projection without reading files or creating Bevy asset
handles. Pixel-art and audio binaries remain local-only and ignored by Git; tracked provenance and
licensing records remain outside ignored media directories.

Acceptance:

- `PresentationAssetReference` accepts non-empty relative paths and rejects traversal, empty
  segments, absolute paths, platform prefixes, backslashes, and NUL bytes without filesystem I/O.
- `PresentationAssetManifest` requires exactly one reference for each terrain, player, enemy,
  dead-actor, ground-item, and inventory-item placeholder family.
- `PresentationRenderAssetProjection` joins every ordered `SceneRenderNodeEntry` to its typed
  reference while retaining node identity, command metadata, and inventory-unplaced semantics.
- Manifest refresh changes only references; missing runtime, node projection, manifest, or
  destination resources preserve the prior asset projection as a safe no-op.
- No asset handles, file loading, render/audio plugins, transforms, windows, gameplay rules,
  dependencies, or committed media binaries are introduced.

Verification target:

- Focused `cargo test -p dreadstep-bevy --test presentation_asset_manifest --locked` passes all
  four tests covering complete joins, path/manifest validation, identity-preserving refresh, and
  independent missing-resource guards.
- All Bevy targets, warning-denied Clippy/docs, formatting, repository checks, `git diff --check`,
  and `scripts/verify.sh` pass locally; anchored media ignore checks keep local binaries ignored
  while the tracked concept-art and future screenshot exceptions remain visible.

### Deferred item gameplay semantics

The opaque ownership slice and content catalog foundation intentionally do not define item effects,
identification, capacity, or richer gameplay-facing item commands beyond the completed equipment
and single-item consumption preparations above. Tester-only transfer, drop, and pickup are verified
separately in their completed slices above; effects and richer player operations still require an
explicit core contract.

## Future

### Remaining roadmap milestones

The completed slices above cover the rules kernel, agent interfaces, and the deterministic
headless presentation boundary currently implemented in the repository. The proposal still
defines these future product milestones; each needs its own bounded acceptance slice before it
can move into `Past`:

- Milestone 3 — First Visible Dreadstep: windowing, rendering, sprites, animation, simple HUD
  widgets, event/combat messages, keyboard presentation, audio placeholders, and fog of war.
- Milestone 4 — Tactical Combat: richer player verbs and systemic combat interactions beyond the
  verified single-item consumption and single-slot equipment preparations and tester item
  operations.
- Milestone 5 — The Living Dungeon: procedural floors, enemy archetypes, environmental state,
  and floor progression.
- Milestone 6 — Loot and Build Formation: curated item progression, identification, and build
  choices.
- Milestone 7 — Vertical Slice: opening-to-victory run, mature presentation, music, polished
  combat feedback, boss, death, victory, save/quit, and replay export.
- Milestone 8 — Agent QA and Balance Laboratory: scenario agents, behavioral agents, and balance
  experiments.
- Milestone 9 — Content Alpha: broader content, authored scenarios, and coherent production
  direction.
- Milestone 10 — Human-Centered Alpha: structured human playtesting for fun, clarity, pacing,
  feel, hierarchy, and audio feedback.
- Milestone 11 — Beta / Release Candidate: stability, accessibility, performance, and release
  hardening.
- Milestone 12 — Dreadstep 1.0: final content, presentation, documentation, and release quality.
