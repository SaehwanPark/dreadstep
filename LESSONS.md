# Lessons Learned

Read this file before implementation and again before final review. Record only verified,
recurring traps that are not already obvious from code, tests, or canonical documentation.
Update an existing lesson instead of adding a duplicate. Package ownership lives in
[`ARCHITECTURE.md`](ARCHITECTURE.md), not here.

## Index

- Content and procedural floors: connectivity checks; opt-in desktop startup; keep progression
  at the presentation boundary until core owns it.
- Core outcomes, replay, inventory, reach, ranged cost/ammo, and legal discovery: derive
  terminals once; export from the accepted trace; share predicates between discovery and
  execution.
- Adapter goldens and public commands: refresh snapshots after digest or command-set changes;
  keep protocol/control reconciliation in one slice; player-facing gates follow the active
  showcase crate; reconcile layout spacing changes with screenshot goldens and README text blocks.
- Bevy presentation: keep engine features at the boundary; key ECS mirrors by domain identity;
  snapshot before exclusive projection; never order input from a button set; finalize journals
  before `App::run` consumes the world.
- Terminal presentation: render frames as a pure function so goldens and agents do not need a
  TTY; log stripped frames rather than putting glyphs on MCP.
- Items and environment: opaque ids vs catalog membership; atomic tester transfers; exclusive
  consumable results; typed environmental commands; reuse movement as the trap trigger; keep
  affixes typed and aggregate them at the core equipment boundary.
- Module splits: keep crate-root `pub use` and `pub(crate)` intra-crate fields so adapters
  keep compiling without import-path churn; isolate complex multi-archetype selection policies into cohesive submodule helpers.

## 2026-08-17 — Aggregate closed affixes at the equipment boundary

- Context: Procedural loot needed a first meaningful affix without opening a generic stat system.
- Symptom: Applying a generated modifier in content or a client would let combat evidence diverge
  from equipped state and replay digests.
- Resolution: Store one typed affix on the core item, derive its role with the existing closed
  equipment effect, and aggregate only its matching stat in `Actor` while hashing and projecting the
  complete item record.
- Prevention: Keep affix variants closed, derive bounded procedural magnitudes in content, use
  saturating arithmetic, and add core, protocol, Bevy, and TUI assertions whenever a new affix
  changes item state or presentation.

## 2026-08-17 — Derive procedural consumable potency from the effect mix

- Context: Procedural floors now place an existing healing or ammunition consumable alongside
  generated equipment choices.
- Symptom: A fixed amount would make the consumable's seed/depth identity affect its label and
  effect family but not its tactical value.
- Resolution: Derive a bounded potency from the same deterministic mix used to choose the closed
  effect, then construct the typed `HealingAmount` or `AmmunitionAmount` without widening the
  effect enum or public command/protocol contract.
- Prevention: Keep generated consumable effect and potency in one content helper, test reproducible
  same-effect inputs with different strengths, and retain the core `UseItem` path as the only
  effect application boundary.

## 2026-08-17 — Keep procedural ground loot ordered by authored enemy position

- Context: Seeded floors now distribute more than one generated equipment item onto enemy starting
  positions.
- Symptom: An unordered or client-owned placement list could make ground snapshots and pickup targets
  vary even when the seed and depth were unchanged.
- Resolution: Generate each item in content, transfer it through core `give_item` and `drop_item`,
  and rely on core's row-major stack ordering and opaque identity validation for projections.
- Prevention: Test both authored positions, stack order, distinct identities, and player ownership
  exclusion whenever procedural ground distribution expands.

## 2026-08-17 — Derive procedural loadout roles from shared floor state

- Context: Two generated starter choices should create a meaningful comparison instead of randomly
  repeating the same closed equipment effect.
- Symptom: Deriving each role from its variant-specific mix allowed identical roles for some seeds,
  weakening the intended two-choice decision.
- Resolution: Derive a shared seed/depth role base and offset it by the ordered variant, preserving
  deterministic identity/rarity/affix mixes while guaranteeing distinct closed effects.
- Prevention: Test representative seed/depth ranges for distinct typed effects and keep role policy
  in content rather than duplicating it in adapters.

## 2026-08-17 — Apply procedural rarity progression at the content boundary

- Context: Seeded floors needed stronger item progression as depth increased without adding new
  rarity values or changing authored fixtures.
- Symptom: A pure seed mix could still produce Common items deep in a run, weakening descent reward
  expectations.
- Resolution: Map Common to Magic at depth 3 and deeper after the existing deterministic rarity mix,
  preserving Rare values and the closed Common/Magic/Rare enum.
- Prevention: Test shallow Common retention plus deep inventory and ground rarity floors, and keep
  progression policy in content before projection through adapters.

## 2026-08-17 — Keep procedural affix progression bounded at depth

- Context: Deep floors needed stronger generated equipment without opening a new affix vocabulary.
- Symptom: Seeded 1–2 tiers could still leave a deep generated item at magnitude 1, weakening the
  relationship between descent and build power.
- Resolution: Reuse the existing typed affix amount and clamp the deep-floor tier to magnitude 2,
  while preserving shallow seed variation and core saturating aggregation.
- Prevention: Test shallow tier retention plus every deep inventory and ground affix, and keep the
  depth policy in content rather than adapter or combat code.

## 2026-08-15 — Reconcile layout spacing changes with screenshot goldens and README text blocks

- Context: When adding visual section separators (blank lines) between the message window, dungeon
  rendering, status block, and intent/controls sections in `dreadstep-tui`, plain-text line counts
  in `screenshots/` and `README.md` changed.
- Symptom: Integration tests in `crates/dreadstep-tui/tests/screenshots.rs` failed because committed
  screenshot files and embedded README text blocks had differing blank line counts.
- Cause: TUI layout is verified through exact character-for-character regression goldens and README
  documentation embeddings.
- Resolution: Regenerate screenshot files using `cargo run -p dreadstep-tui -- --capture screenshots/`
  and update embedded text blocks in `README.md` in the same slice.
- Prevention: Whenever frame layout or spacing changes, update the screenshot captures and README
  embeddings together, then verify with `cargo test -p dreadstep-tui` and `scripts/verify.sh`.

## 2026-08-15 — Decompose enemy intent selection into cohesive submodule helpers

- Context: As additional enemy behaviors (Kiter, Brute, Frostcaster, Blocker, Scavenger) were added
  to `preferred_enemy_command`, the inline selection function in `world/mod.rs` exceeded function
  line limits and mixed multiple distinct archetype rules in a single block.
- Symptom: `clippy::too_many_lines` failed on `preferred_enemy_command` during workspace verification.
- Cause: Placing all behavior-specific match arms and candidate legal-action searches in a monolithic
  function.
- Resolution: Extract intent selection into a dedicated `world/intent.rs` submodule with small, focused
  associated helper functions for each behavior policy (`preferred_kiter_command`,
  `preferred_scavenger_command`, `preferred_blocker_command`, etc.).
- Prevention: Keep archetype selection policies isolated in dedicated associated functions within
  `world/intent.rs` when adding new enemy behaviors, preserving exact deterministic priority order and
  testability without bloated functions.

## 2026-08-14 — Player-facing gates follow the active showcase crate

- Context: Skills, `docs/demo.md`, `CONTRIBUTING.md`, `scripts/verify.sh`, and CI all required
  Bevy desktop smoke and visual playtesting, so core-facing work could not defer pixel clients
  without those gates failing.
- Symptom: A new terminal adapter would still leave `verify.sh` and `$test-player` pointed at
  the Bevy window.
- Cause: The player-facing maintenance contract was hardcoded to `dreadstep-bevy` instead of
  naming the active showcase crate.
- Resolution: Make `dreadstep-tui` the default showcase, retarget verify/CI/skills, and keep
  Bevy smoke in `scripts/verify-bevy-desktop.sh` for a later visual-enhancement stage.
- Prevention: When the human client changes, update the skill, demo, contributing, verify, and
  CI paths in the same slice. Do not leave Bevy as an implicit required mapping for core work.

## 2026-08-14 — Terminal frames must be a pure function

- Context: Agents and README captures need to see the client without a graphical session, and
  alternate-screen TUIs often hide frames from terminal transcripts.
- Symptom: A curses-only renderer cannot produce deterministic goldens or agent-readable logs.
- Cause: Layout was treated as a TTY effect instead of a projection of core state.
- Resolution: Build `TextFrame` from session plus UI state, strip colors for goldens/`frame`
  journal records, and print frames on stdout when stdin is not a TTY.
- Prevention: Do not put glyphs on MCP. Do not make screenshot goldens depend on crossterm.

## 2026-08-13 — Keep crate-root re-exports when splitting production modules

- Context: Moving types out of a crate root made sibling modules lose private-field access and
  broke `use dreadstep_*::{...}` characterization tests even though behavior was unchanged.
- Resolution: Re-export the previous public surface from `lib.rs`, mark intra-crate fields and
  helpers `pub(crate)`, and leave integration-test import paths alone.
- Prevention: After a split, compile `--all-targets` before touching golden command order,
  digests, or smoke matrices.

## 2026-08-12 — Test procedural content connectivity at the authored boundary

- Context: A seeded floor generator can look corridor-like while a future partition or gap change
  silently strands walkable cells, creating unreachable content before core sees a semantic problem.
- Resolution: Keep generation in `dreadstep-content`, route the result through
  `StarterFloorDefinition::build`, and exercise a bounded breadth-first reachability check across
  representative seeds and depths.
- Prevention: Whenever procedural terrain claims connected play space, test every generated walkable
  tile from the authored player start; do not move this presentation/content concern into core rules
  or rely on visual inspection alone.

## 2026-08-12 — Keep procedural startup opt-in while smoke fixtures stay exhaustive

- Context: Exposing generated terrain through the desktop launcher can accidentally replace the
  authored item fixture that exercises inventory and environmental command coverage.
- Resolution: Add explicit visible-run `--procedural`/`--depth` selection and preserve the default
  and display-free smoke startup on the item-bearing fixture; carry scenario/depth evidence in the
  journal and restart the selected mode from the same seed.
- Prevention: Treat launcher scenario selection as an adapter concern, keep smoke fixtures stable
  until their coverage is intentionally redesigned, and test both startup branches independently.

## 2026-08-12 — Derive terminal outcomes once at the core boundary

- Context: The desktop showcase inferred victory by counting dead enemies locally while core
  already retained every actor record, so future clients could disagree on terminal semantics.
- Resolution: Add a pure `WorldState::outcome()` projection with explicit player-defeat precedence,
  require at least one enemy for victory, and carry the typed value through protocol and Bevy
  snapshots. Desktop terminal records now consume that projection without changing commands,
  events, scheduling, or replay evidence.
- Prevention: Keep terminal outcome predicates in the deterministic kernel, test empty/enemy/dead
  edge cases and precedence, and make adapters project the value rather than reimplementing it.

## 2026-08-12 — Export replay evidence from the accepted core trace

- Context: Desktop journals contain request, outcome, and presentation records, but reconstructing
  a replay from those diagnostics would risk including rejected commands or adapter-only mutations.
- Resolution: Expose the runtime's read-only accepted command trace and write a small versioned
  desktop artifact from that source, carrying seed, command order, digest, and canonical outcome;
  allocate the file with create-new semantics and record its path in the journal.
- Prevention: Treat replay export as evidence rather than playback, derive command lists only from
  core's accepted trace, preserve order, and test collision suffixes plus journal/artifact parity.

## 2026-08-12 — Enforce inventory capacity at every ownership ingress

- Context: Item ownership enters through tester give/transfer/pickup operations as well as the
  scheduled player pickup command; checking only one path would let adapters disagree about a full
  inventory.
- Resolution: Keep one fixed four-item capacity on the core actor, validate before mutation in all
  ownership ingress paths, hide full pickup from legal discovery, and carry typed overflow errors
  plus capacity metadata through protocol/MCP snapshots.
- Prevention: Whenever inventory limits change, test accepted boundary count, every ingress rejection
  atomically, and legal-action filtering before updating adapter schemas or UI projections.

## 2026-08-12 — Player inventory actions need an item-bearing public fixture

- Context: The default MCP/headless fixed scenario is intentionally item-free, so a new player item
  command cannot be proven through public boundaries by changing only the command enum.
- Resolution: Keep `start_run` item-free, add an explicit authored item-run entry point for MCP, and
  give the headless developer fixture one stable item; both public paths now exercise accepted drop
  behavior without weakening the tester-only mutation boundary.
- Prevention: For player inventory commands, provide a deterministic item-bearing fixture and test
  legal discovery plus wire/CLI acceptance before declaring adapter parity.

## 2026-08-12 — Public commands require versioned boundary and control reconciliation

- Context: Adding a player reload action changed the core command/event/error set and competed
  with the existing desktop restart key.
- Resolution: Bump the protocol version, carry the new typed variants through every exhaustive
  adapter, and move restart to `Shift+R` while keeping plain `R` legal-action driven. Reuse the
  existing optional audio family rather than inventing a new media contract.
- Prevention: Treat public enum additions and input bindings as one cross-boundary slice; test
  JSON shape, accepted/rejected replay evidence, smoke coverage, and the visible control text
  together.

## 2026-08-13 — Intent projections must reuse the actor driver's policy

- Context: The scheduled enemy exposes several legal movement and combat commands before its
  preferred driver action, so presenting the first vector entry would disagree with the desktop
  enemy driver.
- Resolution: Project the exact core command selected by the same deterministic
  attack-then-ranged-then-chase-then-wait preference used by the driver, with a first-legal fallback
  for future behavior families.
- Prevention: Keep intent projections read-only, test actor/target identity and replay stability,
  exercise both melee and clear-ranged targets, and update the presentation policy whenever the
  driver selection policy changes.

## 2026-08-12 — Legal discovery and execution must share reach predicates

- Context: Adding a second melee range can make a target appear legal while direct execution still
  applies the old adjacent-only rule, or permit a command that discovery omitted.
- Resolution: Keep the typed actor reach and one Manhattan-distance predicate at the core boundary,
  then use it from both legal-action discovery and `Attack` validation; adapters only project the
  resulting value and command evidence.
- Prevention: Test default and explicit reach through both paths, include the reach in the digest
  and snapshots, and preserve atomic rejection/replay evidence for out-of-range requests.

## Keep desktop engine features at the presentation boundary

- Context: The initial root package depended on Bevy 0.19 with all default features.
- Symptom: `cargo clippy` on a headless Linux/WSL2 environment failed while building
  `wayland-sys` because the `wayland-client` system package was unavailable.
- Cause: Bevy's default platform features pulled windowing, Wayland, X11, input, and audio
  dependencies into every repository check before any project code was analyzed.
- Resolution: Move Bevy into `dreadstep-bevy`, disable default features, and enable only
  `std` until a presentation milestone needs a reviewed feature set. Later headless Sprite API
  work enabled only Bevy's `bevy_sprite` feature; its image/mesh/camera support remains usable
  without adding render plugins or desktop backends. The later optional audio boundary enables
  Bevy audio only for desktop and requires `pkg-config` plus `libasound2-dev` on Linux; CI installs
  that prerequisite before the workspace verification. A representative Bevy 0.19 package with
  this configuration passed Clippy on the same environment.
- Prevention: Keep engine dependencies out of core, protocol, and content; inspect enabled
  features before adding presentation capabilities; document platform packages and install them in
  CI before verifying the headless Linux workflow.

## Enable Bevy input modules explicitly when default features are disabled

- Context: The first human-client bridge needs keyboard `KeyCode` values while the workspace keeps
  Bevy desktop backends disabled for headless verification.
- Symptom: With `default-features = false` and only `std`, the Bevy input module's keyboard types
  are not available to the presentation adapter.
- Cause: Bevy gates keyboard support behind its optional `keyboard` feature; `std` does not enable
  optional input modules.
- Resolution: Enable Bevy's `keyboard` feature alongside the existing `std` feature in the
  workspace dependency. The bridge then compiles and tests without enabling `default_platform`,
  Wayland, X11, audio, or window backends.
- Prevention: Add the narrowest Bevy feature required by a presentation slice, inspect
  `cargo tree -e features`, and keep `scripts/check-repository.sh` guarding desktop features.

## Key ECS mirrors by stable domain identity

- Context: The headless Bevy scene bridge mirrors map tiles and actors before rendering exists.
- Symptom: Rebuilding scene entities by allocation order can move a sprite or leave stale entities
  when a snapshot changes; hash-map iteration can also make replacement order unstable.
- Cause: Bevy `Entity` values are presentation handles, not domain identity, and unordered cleanup
  does not preserve a deterministic allocation sequence.
- Resolution: Key tile mirrors by `(x, y)` and actor mirrors by core `ActorId` using ordered maps and
  sets; core snapshots remain authoritative and dead records are intentionally retained.
- Prevention: Treat scene components as disposable projections, choose explicit stable keys for
  updates/removal, and test repeated synchronization plus stale and dead-record cases headlessly.

Bevy entity allocation indexes are recyclable and their numeric representation is not a durable
identity. When a singleton projection must retain its existing ECS entity across duplicate cleanup,
keep the retained `Entity` in adapter-owned state, use full `Entity` ordering only to choose among
untracked duplicates, and test a lower-index recycle explicitly.

## Snapshot a resource before exclusive ECS projection

- Context: The Bevy application shell must read the authoritative runtime and mutate the ECS world
  in one headless update system.
- Symptom: Borrowing a runtime resource while passing the same `World` to scene synchronization
  would violate Rust's aliasing rules and tempt an adapter to duplicate or expose simulation state.
- Cause: Bevy's exclusive system gives one mutable `World`, so a resource reference cannot remain
  live while the synchronizer mutates entities in that world.
- Resolution: Clone the small immutable `PresentationSnapshot` first, end the resource borrow, then
  call `sync_scene` with the snapshot. Keep command submission explicit on `PresentationRuntime`.
- Prevention: Treat app systems as orchestration around core projections; snapshot authoritative
  resources before ECS mutation and never make scene components a second command/state store.

## Never derive input order from a button set

- Context: The headless app shell accepts Bevy `ButtonInput<KeyCode>` while preserving replayable
  command order.
- Symptom: Iterating pressed keys directly can choose different commands because the input resource
  stores keys in a hash set; simultaneous user input then becomes platform/process dependent.
- Cause: Hash-set iteration order is intentionally unspecified and cannot serve as gameplay or replay
  ordering evidence.
- Resolution: Scan a documented fixed key-priority array, issue at most one command per update, and
  consume all supported just-pressed keys for that frame before delegating to core.
- Prevention: Treat input collection as unordered observation; define ordering at the adapter
  boundary and test simultaneous keys plus next-update consumption headlessly.

## Clear stale adapter feedback on rejection

- Context: Future HUD and message systems need the latest accepted event/snapshot without reading ECS
  mirrors or duplicating core state.
- Symptom: Leaving the previous output pending after a rejected command can make consumers display a
  successful event as if it belonged to the rejected action.
- Cause: Adapter feedback is temporal evidence, while core rejection is intentionally atomic and
  produces no new `PresentationOutput`.
- Resolution: Clear the runtime's optional output before every command; publish only accepted output
  and expose explicit one-shot consumption through `take_output`.
- Prevention: Keep feedback owned by the adapter, tie it to accepted command results only, and test
  startup emptiness, exact accepted evidence, one-shot reads, and rejection clearing.

## Keep focus projections separate from camera policy

- Context: A future camera needs the selected actor's latest position before windowing or rendering
  exists.
- Symptom: Storing camera transforms or inventing visibility rules in the focus resource would make
  presentation policy look like simulation state and complicate later clients.
- Cause: Actor position is core truth, while viewport, smoothing, interpolation, and fog are client
  decisions with different lifecycles.
- Resolution: Store only the typed controlled actor and optional projected core position; update it
  after authoritative dispatch/scene sync and use `None` for an unknown identity.
- Prevention: Treat focus as a disposable read-only projection, keep camera math in a later boundary,
  and test actor changes plus unknown-resource behavior headlessly. Guard missing runtime separately
  from an unknown actor: without an authoritative snapshot, preserve the last focus projection;
  with a present snapshot, map only the unknown identity to `None`. For atomicity evidence, compare
  replay digest and complete keyed tile/actor projections rather than entity counts alone. When
  extending focus into ECS, attach only a marker to the already keyed actor entity, guard input,
  focus, and runtime independently, clear stale markers only with an authoritative snapshot, and
  remove old markers before inserting the new target.

## Carry optional visibility through the render-node boundary

- Context: The desktop showcase applies a later sprite-styling system after the headless projection
  system, while the same retained render nodes are inspected by headless tests.
- Symptom: Setting Bevy `Visibility` only in the projection system was overwritten by desktop style
  initialization, making an apparently correct fog-of-war projection render distant nodes anyway.
- Cause: ECS component state was being treated as the source of a presentation decision that had to
  survive multiple downstream systems.
- Resolution: Compute visibility from typed scene-entry positions, store the derived bit on
  `SceneRenderNode`, and let both generic attachment and desktop styling consume that node metadata.
  An inactive optional projection is an explicit no-op, preserving the fully visible headless default.
- Prevention: Carry cross-system presentation decisions in a typed projection boundary, keep retained
  node identity independent of visibility, and test both hidden rendering and restoration after the
  optional authority disappears.

## Keep opaque item identity separate from content membership

- Context: Tester item ownership now has typed opaque `ItemDefinitionId` references, while future
  authored content needs deterministic known-definition membership.
- Symptom: Letting adapters or `WorldState` invent catalog entries would duplicate content truth or
  silently turn an authoring list into gameplay validation.
- Cause: Core owns item instances, ownership, digests, and snapshots; content owns authored
  definition membership, and the two lifecycles do not have the same authority or timing.
- Resolution: Keep an ordered `ItemCatalogDefinition` and validated immutable `ItemCatalog` in
  `dreadstep-content`; reject duplicate IDs there and expose only read-only membership. Do not
  inject the catalog into `WorldState` or add effects, equipment, or player commands.
- Prevention: Treat catalogs as authoring data, use typed opaque IDs and deterministic declaration
  order, and require a later core contract before adding gameplay semantics or richer operations.

Authored item instances now follow the same boundary: `StarterFloorDefinition` may carry ordered
`StarterItemPlacement` values, but construction delegates each one to `WorldState::give_item`.
This preserves core-owned actor/ItemId validation and inventory/digest updates without coupling
the independent definition catalog to runtime item behavior; keep the default starter floor empty
until a content decision explicitly adds instances. When a floor binds a catalog, validate catalog
duplicates and placement membership before constructing the map/world, but never store the catalog
in core state. When a reusable fixture is needed, expose it as a separate explicit content helper
and test its complete inventory/digest projection; do not silently populate the default scenario.

The Bevy adapter follows the same distinction: `start_run` remains the stable item-free default,
while `start_item_run` explicitly opts into the non-default content fixture. Verify both the
state/runtime snapshot and the complete typed `SceneInventoryItem` projection so a convenience
constructor cannot silently alter the default client path or lose owner/order data.

## Keep tester item transfer atomic and outside player replay

- Context: Opaque item ownership now needs a deterministic tester operation to move an existing item
  between actor records without inventing item effects.
- Symptom: Mutating source and target inventories in separate unchecked borrows could partially move
  an item, reorder remaining items, or accidentally record a tester mutation as player history.
- Cause: Core owns both inventories and replay trace, while protocol/MCP only project a tester effect;
  dead records are retained actor identities but are not scheduler participants.
- Resolution: Validate both actor identities and source ownership first, treat same-actor transfer as
  an idempotent no-op, remove from source preserving relative order, append unchanged data to target,
  and keep the operation outside `ReplayTrace`; map only the typed core error at adapter boundaries.
- Prevention: Test digest/order and complete rejection snapshots at core, map every new world error in
  protocol, and assert MCP history/replay invariants for both accepted and rejected transfers.

## Keep ground item identity and projection ordered

- Context: Tester item drop/pickup now moves opaque instances between actor inventories and stable
  map-position ground stacks without defining player interactions or gameplay effects.
- Symptom: A separate unordered ground store or position iteration could permit duplicate identities,
  make snapshots/digests vary by process, or lose item order when multiple items share a tile.
- Cause: Core owns one global item-identity invariant across inventories and ground stacks, while
  adapters need a complete deterministic projection for inspection and typed pickup failures.
- Resolution: Keep stacks in core, order positions row-major and items by append order, validate
  ownership before both mutations, remove empty stacks, include ground items in the world digest, and
  project them read-only through protocol/MCP without player replay/history. For the headless Bevy
  mirror, snapshot complete stacks and actor inventories before ECS mutation, key scene entities by
  globally unique `ItemId`, preserve typed definition/position/owner/order data, update retained
  identities after core-authoritative transfers, and remove stale items before spawning new keys.
- Prevention: Test source/stack order, round-trip item data, empty-stack cleanup, row-major
  projection, duplicate give-after-drop, dead-source behavior, typed ground misses, accepted/rejected
  tester replay invariants, complete Bevy item data, retained scene identities, duplicate cleanup,
  picked-up stale removal, inventory owner/order updates, and inventory stale removal alongside
  complete tile/actor/ground projections.

## Refresh every adapter golden after a digest schema change

- Context: The equipment contract added optional identity bytes to the state digest and new command
  codes to the replay digest, intentionally changing both deterministic namespaces to `V2`.
- Symptom: Core, protocol, and presentation tests passed locally while Linux CI failed in the
  headless CLI's exact rendered-output golden, which still contained the pre-change digest value.
- Cause: Digest values are consumed by adapter tests as observable evidence; searching only core
  assertions misses hard-coded values in downstream CLI or integration fixtures.
- Resolution: Refresh the headless golden from the observed `scripts/verify.sh` failure, then rerun
  the focused headless suite and full workspace verification. The corrected expectation is tracked
  in `crates/dreadstep-headless/src/lib.rs`.
- Prevention: Whenever digest bytes or namespace changes, search the full workspace for every old
  golden value, run `scripts/verify.sh`, and isolate new hash-field tests from action-time or trace-
  length changes so identity bytes are proven directly.

## Refresh legal-command expectations when adding a command variant

- Context: The single-item consumption slice added `UseItem`, and the ranged-combat slice added
  `RangedAttack`, to core legal-action discovery while preserving deterministic ordering.
- Symptom: Focused new tests passed while an older exact legal-action vector or adapter smoke
  coverage remained stale after a new command variant.
- Cause: Adding a public command changes every deterministic discovery projection, not only the
  new command's own tests; the stale assertion was outside the edited slice.
- Resolution: Add each new command at its documented deterministic target position, update every
  exact vector and downstream command-coverage expectation, and rerun the full workspace
  verification. Core emits exactly one melee or ranged command per living target in stable
  `ActorId` order; both reuse the existing attack event/cue family at the presentation boundary.
- Prevention: Whenever a command variant changes legal discovery, search all workspace tests and
  adapters for exact command vectors, smoke matrices, JSON mappings, and hard-coded adapter output;
  update those expectations deliberately, and require the full cross-platform verification before
  handoff.

## Keep ranged visibility pure and shared

- Context: The ranged-combat follow-up adds wall blocking without adding projectile state or a
  presentation dependency.
- Symptom: Validating line of sight only during execution would advertise commands that later fail;
  adding separate adapter checks would also allow protocol/MCP and desktop projections to diverge.
- Cause: Legal discovery and command execution are independent core entry points, and diagonal grid
  rays have no existing interpolation rule.
- Resolution: Use one core predicate for both paths: only same-row or same-column distance-2..=3
  rays with walkable interior cells are visible; blocked or diagonal requests return a typed error
  atomically. Adapters translate that error and retain existing attack cues.
- Prevention: Keep visibility predicates in the functional core, specify diagonal behavior before
  implementation, and test legal filtering plus rejected state/replay evidence together.

## Keep command cost selection in core execution

- Context: The ranged-cost slice makes ranged attacks consume two scheduler ticks while all other
  commands retain the standard cost.
- Symptom: Applying a global scheduler increment or changing only the desktop/MCP projection would
  make ready-time and next-actor evidence disagree across adapters.
- Cause: Action timing is authoritative core state, while legal discovery has to account for the
  selected command's overflow independently of standard-cost commands.
- Resolution: Select `ActionCost::RANGED` from the core command before execution, guard that same
  cost in legal discovery, and leave protocol command shape and presentation cues unchanged.
- Prevention: Whenever one command gets a distinct cost, test both accepted ready-time transitions
  and near-overflow legal filtering; do not encode timing in adapters or replay-only metadata.

## Consume ranged resources only after semantic acceptance

- Context: The ranged-ammunition slice adds three default shots without introducing reload or pickup
  commands.
- Symptom: Decrementing ammo before target and line-of-sight validation would make rejected actions
  mutate state and diverge from the accepted history/replay contract.
- Cause: Scheduler cost and resource mutation are core-owned effects that must follow all semantic
  validation branches.
- Resolution: Filter zero-ammo ranged commands during legal discovery, reject direct zero-ammo
  requests with a typed error, and decrement exactly once only after `ranged_attack` succeeds.
- Prevention: For every finite command resource, assert accepted digest/snapshot changes, empty
  action filtering, and atomic rejection across all preconditions.

## Keep walkability separate from ranged visibility

- Context: Cover is intentionally walkable but blocks an interior ranged ray.
- Symptom: Reusing `is_walkable` for line-of-sight checks would make cover behave like floor and
  silently advertise an action that should reject.
- Cause: Movement occupancy and ranged visibility are different terrain policies.
- Resolution: Keep `Tile::is_walkable` permissive for floor and cover, and use the explicit
  `blocks_ranged_line_of_sight` predicate for cardinal ray interiors.
- Prevention: Test cover placement/movement separately from legal ranged filtering and preserve the
  existing atomic no-line-of-sight error contract.

## Keep enemy legal intent and diagnostic driving bounded

- Context: The enemy-melee slice reuses the existing `Attack` command for adjacent living targets
  while preserving `Chase` for distant targets and complete desktop smoke coverage.
- Symptom: Changing only the presentation driver would diverge from MCP/core legal actions; allowing
  every smoke enemy attack would eventually kill the diagnostic player before later command/event
  coverage completed.
- Cause: Core legal discovery is authoritative, while the smoke path is a finite evidence scenario
  rather than a full player-death loop.
- Resolution: Share one core adjacent-attack predicate and one Bevy attack-before-ranged-before-
  chase selector; keep the visible driver fully authoritative, but let the smoke helper choose
  legal `Wait` after a low-health threshold for either damaging attack so its deterministic coverage
  can finish.
- Prevention: Test adjacent and distant legal ordering at the core boundary, test the exact Bevy
  selector, and treat smoke-only safety guards as diagnostic policy rather than simulation rules.

## Keep item effects on instances and report capped results

- Context: The first authored healing consumable extends the previously opaque item-ownership
  contract without making adapters infer gameplay from definition IDs.
- Symptom: Applying healing in content, MCP, or Bevy would duplicate rules and could disagree on
  maximum hit points, while reporting only “consumed” would hide the actual capped recovery.
- Cause: Core owns actor hit points, item instances, scheduling, and digest state; content authors
  the effect, and adapters only translate evidence around the core transition.
- Resolution: Store an optional typed effect on each core item instance, keep `Item::new` as the
  explicit no-effect constructor, retain an actor maximum-hit-point value, clamp healing in core,
  and carry the actual amount plus remaining HP as optional `ItemConsumed` evidence. Keep the
  protocol version and desktop diagnostic event mapping synchronized.
- Prevention: Test full-health, partial, capped, no-effect, and rejected uses at core; assert the
  authored content fixture; map optional evidence through protocol/MCP/Bevy; and search adapter
  JSON/golden tests whenever state-digest or event schemas change.

## Keep mutually exclusive consumable results explicit

- Context: The ammunition consumable extends `UseItem` beyond healing while preserving one typed
  `ItemConsumed` event and the existing presentation cue families.
- Symptom: Adding a second effect-specific payload without an explicit shape would force adapters
  to infer the effect or report contradictory healing and ammunition values.
- Cause: Core owns the item effect and actor resource limits; the event is the shared semantic
  boundary for all accepted item uses.
- Resolution: Add a typed ammunition amount/result, cap restoration in core, and carry optional
  healing and ammunition results as mutually exclusive event fields. Keep the opaque snapshot and
  no-effect constructor unchanged while synchronizing protocol v14 and desktop JSON/text output.
- Prevention: Test authored and hand-built ammunition items at partial and full capacity, assert
  no-effect/healing compatibility, and keep event/schema constructors exhaustive across all adapters.

## Keep environmental mutation behind a typed command

- Context: The first Living Dungeon preparation slice adds a closed door without introducing a
  generalized interaction framework.
- Symptom: Letting adapters flip map tiles directly would make presentation or tester setup a
  second source of game truth and would omit action timing, replay, and event evidence.
- Cause: Terrain state and scheduled transitions belong to `WorldState`, while desktop smoke and
  MCP scenarios only need fixture setup around that authority.
- Resolution: Add `Tile::Door`, a checked core tile mutation primitive, and one adjacent scheduled
  `Interact` predicate shared by legal discovery and execution. Open only after all validation,
  emit `DoorOpened`, and keep fixture-only setup out of replay history.
- Prevention: Test blocked movement/line of sight, successful standard-cost opening, deterministic
  legal ordering and digest participation, and atomic rejection for every invalid target before
  adding locks, closing, traps, or procedural generation.

## Reuse movement as the trap trigger boundary

- Context: The next Living Dungeon slice adds a one-shot floor trap without adding another command
  or a generalized environmental interaction framework.
- Symptom: Implementing trap effects only in player movement would make enemy chase behavior diverge,
  while emitting damage before movement would leave adapters unable to explain the actor's position.
- Cause: `Move` and `Chase` share one core movement transition, and entering a trap is a consequence
  of successful movement rather than a separate intent.
- Resolution: Keep `Tile::Trap` walkable and line-of-sight transparent, emit `Moved` first, consume
  the tile, apply fixed damage, then emit `TrapTriggered` and optional `Died` in one deterministic
  event list. Keep fixture placement outside replay evidence and map the event exhaustively at
  protocol/MCP/Bevy/desktop boundaries.
- Prevention: Test chase reuse, one-shot consumption, lethal ordering, standard timing, and atomic
  blocked movement before adding hidden traps, disarming, rearming, or trap archetypes.

## Keep breakable terrain separate from generic interaction

- Context: The next Living Dungeon preparation slice adds one deterministic destructible tile without
  introducing tool stats, durability, or a generalized interaction framework.
- Symptom: Letting an adapter flip a breakable tile directly, or routing it through an underspecified
  `Interact`, would hide action timing and make replay/event evidence ambiguous.
- Cause: Breaking is a distinct scheduled intent with a narrower target contract than door opening;
  terrain mutation still belongs to the core transition boundary.
- Resolution: Keep `Tile::Breakable` blocking and sight-blocking, add a dedicated adjacent `Break`
  command, validate every target before mutation, emit `BreakableBroken`, and map the command/event/
  error exhaustively at each boundary. Fixture-only placement remains outside replay history.
- Prevention: Test movement/sight blocking, legal-action ordering, standard timing, replay/state
  participation, and atomic same-position/non-adjacent/out-of-bounds/non-breakable/already-broken
  rejection before adding damage, tools, multi-hit durability, procedural placement, or noise.

## Keep noise evidence ahead of noise behavior

- Context: A kick-open-door slice introduces the proposal's noise vocabulary without committing to
  an enemy investigation policy or propagation algorithm.
- Symptom: Making a door kick mutate enemy intent immediately would couple a local environmental verb
  to an unfinished global AI rule and make smoke behavior depend on hidden range semantics.
- Cause: `NoiseCreated` is semantic event evidence, while propagation and attraction are separate
  gameplay policies that need their own deterministic state and acceptance contract.
- Resolution: Emit `DoorOpened` first and fixed-radius `NoiseCreated` second from the core kick
  transition; map both events exhaustively and defer propagation/attraction explicitly.
- Prevention: Test ordered event evidence, standard timing, replay participation, and atomic target
  rejection before adding noise state, falloff, enemy interest, or interaction composition.

## Keep kick-noise investigation one-use and core-owned

- Context: The kick-open-door slice intentionally stopped at ordered `NoiseCreated` evidence, but
  the next tactical consequence needs to remain deterministic and bounded while broader sound
  propagation is unresolved.
- Symptom: Letting Bevy infer an enemy's attraction or retaining a global sound field would create
  adapter-owned AI state and make replay/snapshot behavior depend on presentation timing.
- Resolution: Core arms one optional hearing position directly on each living enemy within the
  fixed radius, advertises `Investigate` between ranged and chase priorities, reuses deterministic
  movement, and clears the target after one accepted attempt (including blocked movement). Digest,
  protocol snapshot, and replay evidence include the optional target.
- Prevention: Keep generic sound sources, falloff, persistence, wall-aware propagation, and hearing
  archetypes out of this preparation slice; share the exact legal/execution predicate and test
  replacement, distance boundary, dead actors, stale targets, and blocked steps.

## Keep bounded noise propagation terrain-aware and deterministic

- Context: The one-use kick-noise target was useful tactically, but direct Manhattan eligibility
  allowed a wall, closed door, or breakable to transmit sound as if it were open floor.
- Symptom: An enemy could investigate a kick through blocking terrain, while adding a global sound
  field would introduce unnecessary lifetime and source-policy decisions.
- Cause: The source has a fixed radius and a single origin, so its next bounded step is a local
  reachability query rather than persistent world state or adapter-owned hearing.
- Resolution: Expand from the opened door through walkable cells only with a fixed radius-three
  North/South/West/East BFS. Ignore actor occupancy, preserve one source position, and arm living
  enemies only when their cells are reached.
- Prevention: Keep traversal order explicit, test exact boundaries, occlusion, short detours, and
  actor occupancy, and defer falloff, persistence, multiple sources, and hearing archetypes.

## Preserve reversible terrain identity in the core

- Context: The one-way door interaction was sufficient for the first environment slice, but later
  combat positioning needs an opened doorway to remain distinguishable from ordinary floor.
- Symptom: Converting an opened door directly to `Floor` made a future close action impossible and
  forced adapters to invent presentation-only door state.
- Cause: Terrain identity, walkability, line-of-sight, occupancy checks, action timing, and replay
  evidence all belong to the core transition boundary.
- Resolution: Add a distinct `OpenDoor` tile with the same walkability and transparency semantics as
  an open passage. Keep `Interact` and `Kick` as the existing open verbs, add a typed adjacent
  `Close` command, reject occupied doorways atomically, and hash/snapshot the distinct terrain.
- Prevention: Preserve reversible authored terrain states before introducing locks, keys, durability,
  diagonal closing, or adapter-owned visual state; test open/close identity, occupancy, legal order,
  digest/replay participation, and boundary mappings together.

## Keep floor progression at the presentation boundary until core state exists

- Context: The seeded procedural corridor now needs a small visible multi-floor loop without
  prematurely adding persistent progression state to the simulation or agent contracts.
- Symptom: Letting desktop input mutate a depth field in isolation could create a second source of
  progression truth, while replacing the authored smoke fixture would remove exhaustive coverage.
- Cause: The generator already accepts explicit seed/depth inputs and the core outcome projection
  identifies a canonical victory, but no core floor-history or transition contract exists yet.
- Resolution: Permit only procedural visible `Victory` to accept `N`; start the next floor through
  `PresentationRuntime::start_procedural_run` with the same seed and checked incremented depth,
  reset disposable presentation/replay state, and record `floor_advanced`. Keep item startup,
  smoke, protocol/MCP scenarios, and future core progression unchanged.
- Prevention: Test victory-only and procedural-only guards, exact seed/depth runtime parity, empty
  replay reset, journal evidence, depth overflow/content faults, and item/smoke no-op behavior before
  introducing floor history, loot progression, or agent-facing scenario selection.

## Show scenario context from the adapter session, not inferred world state

- Context: Procedural and authored starter runs share the same HUD, while only the former carries a
  meaningful depth that changes on visible floor advancement.
- Symptom: Inferring the active scenario from map dimensions or journal records makes the HUD brittle
  and can hide whether a restart or floor transition occurred.
- Cause: Scenario selection and authored depth are already explicit desktop-session inputs; core
  snapshots intentionally do not own presentation scenario metadata.
- Resolution: Render a plain-language scenario/depth line directly from the session fields and keep
  the JSONL/journal schema unchanged. Procedural sessions show the current depth; item sessions show
  the stable fixture name.
- Prevention: Unit-test both labels, refresh them after restart/advancement, and do not derive
  presentation context from map shape or diagnostic paths.

## Finalize Bevy diagnostics before `App::run` consumes the world

- Context: The desktop showcase must export replay evidence and a final shutdown record for
  window-close, Escape, Ctrl-C, victory, defeat, and fault paths.
- Symptom: Reading `PresentationRuntime` from `app.world()` after `App::run()` reports a missing
  resource because Bevy replaces the caller's app with an empty world when its runner returns.
- Cause: Cleanup evidence was scheduled after the runner boundary even though the authoritative
  runtime and session only exist inside the consumed app world.
- Resolution: Observe `AppExit` in a finalizer system, export replay/shutdown records while resources
  are present, and communicate only a small error report through an external `Arc<Mutex<...>>`.
  Ctrl-C remains a clean diagnostic shutdown; export or journal failures retain the existing fault
  path.
- Prevention: Test finalization with an in-world `AppExit` message and exercise Ctrl-C on an unlocked
  desktop host before claiming the visible shutdown checklist complete.

## Keep equipment effects closed and authored

- Context: The first item-derived combat decision needs a real mechanical effect without becoming a
  generic stat or affix system.
- Symptom: Treating every item as consumable would silently destroy a weapon, while hashing only
  effective reach would lose the actor's authored fallback after unequip.
- Resolution: Keep equipment effects closed and separate from consumables, reject `UseItem`
  atomically for equipment, and hash both base reach and each authored effect alongside the item
  identity. Resolve a damage bonus in core's melee attack path so `Attacked` evidence and target
  hit points cannot diverge from the equipped item.
- Prevention: Preserve exact equip/unequip timing and replacement events, expose every closed effect
  in protocol snapshots and adapter diagnostics, and defer weapon classes, armor, affixes,
  durability, and identification until their own bounded contracts exist.

## Keep authored equipment bonuses on the attack boundary

- Context: Adding a melee-damage item extends the first equipment-derived effect without opening a
  generic stat or affix system.
- Symptom: Applying a bonus in content, MCP, or presentation would let the visible combat result
  disagree with core hit points, replay digests, or `Attacked` event evidence.
- Cause: Core owns equipped item identity, combat damage, state digests, and event ordering; adapters
  only project the closed effect and resulting evidence.
- Resolution: Store one typed `MeleeDamage` equipment effect on the item instance, calculate the
  effective melee damage only while that item is equipped, and map the effect through protocol,
  TUI, Bevy, and MCP fixtures. Bump the protocol and state-digest schema tags together.
- Prevention: Test equipped melee and ranged damage plus reduction bonuses across attacks and
  one-shot traps,
  unequipped fallback, target hit points, actual event damage, digest participation,
  non-consumable rejection, adapter snapshot mappings, and deterministic selected-versus-equipped
  labels before adding additional weapon stats or slots.

## Keep throw effects closed and status accounting actor-specific

- Context: The first player-facing throwable reuses Chilled without opening a generic projectile
  or effect system.
- Symptom: A throw applies status to its target while the thrower may also be Chilled, so broad
  event matching can accidentally treat the target's application as a refresh for the thrower and
  skip the thrower's scheduler tick or expiry.
- Cause: `WorldState::execute` owns action-cost/status accounting, while `ItemThrown` and
  `StatusApplied` are shared semantic evidence for two different actors in one accepted command.
- Resolution: Keep one closed `ThrowableEffect::Chill`, validate the player/item/target/ray in core,
  and identify status refreshes by the command actor identity before consuming that actor's prior
  status. Emit throw evidence before the target application and keep the target's refresh separate.
- Prevention: Test a Chilled thrower and target together, assert the thrower pays the extra tick and
  the target retains two actions, and defer splash, misses, projectile simulation, and generic
  throw rules until each has its own typed contract.

## Keep enemy archetype selection core-owned and spatially bounded

- Context: The first retreating enemy should create a tactical difference without introducing a
  general behavior tree, pathfinding, or adapter-owned AI policy.
- Symptom: Letting Bevy infer a Kiter from proximity can diverge from MCP/headless legal actions,
  while an unconstrained retreat may oscillate or choose a process-dependent tile.
- Cause: Enemy behavior is semantic actor state, and movement legality already knows terrain and
  living occupancy; presentation should only select the exact command the core advertises.
- Resolution: Store a closed `EnemyBehavior` on the actor, advertise `Retreat` only for a scheduled
  Kiter with an adjacent living target, and choose the farthest unoccupied walkable cardinal tile
  with explicit North/South/West/East tie order. Keep the standard `Moved`/`MovementBlocked`
  evidence and action cost.
- Prevention: Hash and snapshot behavior identities, test atomic no-escape/non-adjacent/dead-target
  rejections, and keep factions, pathfinding, memory, exact-range preferences, and group tactics
  out of the first archetype slice.
