# Changelog

All notable contributor- and user-visible project changes are recorded here.

## Unreleased

### Added

- Added the authored Brute enemy behavior. A scheduled Brute breaks the existing adjacent
  Breakable that blocks its deterministic horizontal-first chase step, then resumes ordinary
  pursuit. Core intent, protocol v25 behavior snapshots, MCP, Bevy, and desktop smoke evidence
  remain synchronized; door breaking, damage, durability, and pathfinding remain deferred.

- Added the authored Frostcaster enemy behavior. A scheduled Frostcaster casts the existing
  Chilled status along a clear cardinal distance-2..=3 ray instead of ranged attack, consuming no
  ammunition and paying ranged timing. Protocol v26, MCP, headless, Bevy, and desktop smoke map
  the typed `CastChill`/`ChillCast` contract; damage, splash, cooldowns, and spell resources remain
  deferred.

- Added reclosable doors as a bounded Living Dungeon interaction. `Interact` and `Kick` now
  preserve an opened door as transparent, walkable `OpenDoor` terrain; a scheduled living actor
  may use typed `Close` on an adjacent unoccupied OpenDoor to restore blocking `Door` terrain with
  standard timing and `DoorClosed` evidence. Protocol v25, MCP, headless, Bevy/desktop controls,
  and display-free smoke map the same core-owned transition. The authored item showcase now places
  a closed door beside the player for the documented controls. Locks, keys, durability, diagonal
  closing, and automatic enemy door-closing remain deferred.

- Added the bounded authored Kiter enemy behavior. A scheduled Kiter adjacent to a living target
  advertises and executes typed `Retreat` before attack/ranged/investigate/chase fallbacks, choosing
  the farthest unoccupied walkable cardinal tile with stable N/S/W/E ties. Protocol v23 snapshots
  the closed behavior identity; MCP, headless, Bevy/desktop intent, and replay/digest coverage stay
  core-backed. Group tactics, pathfinding, exact-range preferences, and behavior memory remain
  deferred.

- Added the bounded player Frost Flask throw: authored item 104 (definition 5) is a closed
  `ThrowableEffect::Chill` that a player may throw at a living target on a clear cardinal
  distance-2..=3 ray. The accepted command consumes the flask, emits typed `ItemThrown` then
  `StatusApplied` evidence, and applies or refreshes the existing two-action `Chilled` status.
  Protocol v23, content, MCP, headless, Bevy/desktop HUD and journal mappings, and display-free
  smoke coverage are synchronized; splash, misses, projectile simulation, and generic throw
  effects remain deferred.

- Added a one-shot authored `ChillTrap` that applies a two-action `Chilled` status. Chilled actors
  pay one extra scheduler tick for each affected accepted action, with typed status events and
  digest/snapshot projection.

### Changed

- Split oversized core, protocol, and Bevy production sources into cohesive modules with
  crate-root re-exports, added an 800-line CI budget (excluding `*tests.rs` characterization
  suites), and compacted SPEC/ARCHITECTURE/README so verified slice history lives in this
  changelog rather than duplicated writeups. Workspace version remains
  `0.0.0`.

### Added

- Added the first equipment-derived mechanical effect: authored item 103 (definition 4) is a
  non-consumable reach weapon. Equipping it raises effective melee reach to at least two while
  preserving the existing equipment slot, replacement events, and standard action timing;
  unequipping restores the actor's authored base reach. Protocol v20 projects the closed effect,
  direct `UseItem` rejects it with typed `ItemNotConsumable`, and the desktop item fixture/smoke
  now equips 103 while consuming healing item 101 separately. Damage, weapon classes, armor,
  affixes, durability, identification, and randomized loot remain deferred.

- Added deterministic kick-noise enemy investigation: a successful `Kick` arms a one-use,
  terrain-aware radius-3 hearing target on each eligible living enemy; the scheduled enemy now
  exposes and may consume an `Investigate` command between ranged attack and chase, moving with
  the existing deterministic horizontal-first step and clearing hearing even on a blocked step.
  Core digest/replay, protocol v19 snapshots and errors, headless parsing, and Bevy intent/journal
  mappings are synchronized; persistent sound fields, falloff, multiple sources, and hearing
  archetypes remain deferred.

- Added deterministic terrain-aware kick-noise propagation: the fixed radius-three source now
  expands through walkable cells in stable North/South/West/East breadth-first order. Walls,
  closed doors, and breakables occlude the source while actor occupancy does not; the existing
  one-use `Investigate` target, event ordering, protocol v19, and adapter mappings remain intact.

- Added deterministic enemy ranged intent: scheduled enemies now expose clear-cardinal
  `RangedAttack` candidates at distance 2–3 when ammunition and schedule capacity permit, while
  preserving adjacent melee-first and chase fallback ordering. The shared Bevy intent projection
  and desktop driver reuse that policy, the existing two-tick/ammunition/event evidence remains
  authoritative, and display-free smoke now asserts an enemy-driver ranged command; ranged enemy
  archetypes and richer AI remain deferred.

- Added opt-in desktop procedural-run selection: visible runs accept `--procedural --depth <u32>`,
  journal the generated scenario/depth, and preserve that choice across same-seed restart. Default
  and display-free smoke startup remain on the authored item fixture. Procedural visible victory can
  advance with `N` to the next depth using the same seed and a fresh presentation/replay trace;
  the HUD names the active scenario/depth; protocol/MCP selection and procedural loot remain deferred.

- Fixed visible desktop Ctrl-C finalization so replay export and `shutdown` journal evidence flush
  before Bevy consumes the app world; Escape, window-close, and runtime-fault paths now share the
  same pre-exit finalizer.

- Added contextual terminal HUD guidance: procedural victory names the next depth and `N`, while
  authored victory and defeat point to `Shift+R`; maximum-depth victory avoids overflow and gives a
restart-only recovery message.

- Contextualized the controls panel so authored item runs do not advertise the procedural-only `N`
  action.

- Added a deterministic seeded corridor-floor content preparation slice: `procedural_floor` and
  `procedural_floor_definition` generate a validated 13×9 floor with three single-gap wall
  partitions, stable actor placement, small depth-scaled enemy durability, and a tested guarantee
  that every generated walkable tile is reachable from the player. Existing starter floors,
  protocol/MCP contracts, replay evidence, and the desktop fixture remain unchanged while floor
  progression and renderer selection stay deferred.

- Verified deterministic kick-open doors with noise evidence: a scheduled adjacent `Kick` opens a
  closed `Door` with standard action cost, ordered `DoorOpened` then fixed-radius `NoiseCreated`
  evidence, and atomic rejection. Protocol v18, MCP, headless, Bevy/desktop mappings, `K` control,
  and display-free smoke/journal coverage are synchronized; durability and generic interactions
  remain deferred.

- Verified deterministic adjacent breakable terrain: blocking `Breakable` tiles stop movement and
  ranged sight until a scheduled `Break` changes one adjacent tile to `Floor` and emits typed
  `BreakableBroken` evidence with standard action cost and atomic rejection. Protocol v18, MCP,
  headless, Bevy/desktop mappings, `B` control, and display-free smoke/journal coverage are
  synchronized; damage/tool stats, durability, procedural placement, and persistent noise fields remain
  deferred.

- Verified deterministic one-shot floor traps: walkable `Trap` terrain is consumed when a scheduled
  `Move` or enemy `Chase` enters it, emitting ordered `Moved`, `TrapTriggered`, and lethal `Died`
  evidence with fixed one-point damage. Protocol v18, MCP, Bevy/desktop mappings, and display-free
  smoke/journal coverage are synchronized; discovery, disarming, rearming, and procedural placement
  remain deferred.

- Verified deterministic adjacent door interaction: closed `Door` terrain blocks movement and ranged
  sight until a scheduled `Interact` opens it with standard action cost and typed `DoorOpened`
  evidence. Protocol v18, MCP tester scenarios, headless parsing, Bevy/desktop `I` control, and
  display-free smoke/journal coverage are synchronized; lock/key, closing, and procedural floors
  remain deferred.

- Verified deterministic item effects now include authored item `101` as a three-point healing
  consumable and item `102` as a two-round ammunition consumable. Core caps both results at actor
  limits and reports mutually exclusive optional evidence in `ItemConsumed`; protocol v18, MCP,
  Bevy, desktop journal, and smoke evidence preserve the same result while richer item effects
  remain deferred.

- A verified fixed four-item inventory capacity: core, protocol v12, MCP tester operations, and
  player pickup legal-action discovery reject overflow atomically while actor snapshots expose the
  capacity; item effects, stacking, and upgrades remain deferred.
- A verified deterministic desktop replay-evidence export: each smoke or visible run writes a
  create-new version-1 `*.replay.json` artifact with seed, accepted command order, replay digest,
  and canonical outcome, plus a matching journal record; playback and save/load remain deferred.
- A verified canonical run-outcome projection: core derives deterministic in-progress, defeat, and
  victory states with player-death precedence; protocol v11, MCP/headless evidence, and Bevy
  terminal handling consume the same projection without adding persistence or replay playback.
- A verified deterministic melee-reach preparation slice: typed actor reach with a one-tile default,
  explicit extended-reach scenarios, and core/protocol/MCP/Bevy parity without weapon classes or
  new presentation controls.
- A verified deterministic player reload action that restores the fixed three-shot ranged-ammo
  capacity with typed core/protocol/MCP/headless evidence, protocol v9, `R` desktop control, and
  exhaustive journal/smoke coverage; ammo pickups and weapon capacities remain deferred.
- A verified scheduled player-facing item drop slice: protocol v10 command/event/error plumbing,
  deterministic inventory-to-ground ordering, MCP item-run setup, headless CLI coverage, and `X`
  desktop control; item effects and capacity upgrades remain deferred.
- A verified deterministic enemy melee-intent slice: adjacent scheduled enemies now use the existing
  fixed-damage `Attack` before fallback `Chase`, with core/MCP legal-action evidence and Bevy intent,
  desktop-driver, and smoke coverage; ranged enemy AI remains deferred.
- A verified desktop player-defeat terminal preparation: existing core death events now have a
  presentation-only defeat status and `terminal_defeat` journal record, with restart preserved and
  no new simulation or protocol state.
- A deterministic walkable `Cover` terrain variant that blocks interior ranged line of sight,
  with protocol v7 scenario conversion and unchanged damage/presentation behavior.
- A finite three-shot ranged ammunition resource with typed empty-ammo rejection, legal-action
  filtering, protocol v6 snapshot evidence, and unchanged desktop/audio fallback behavior.
- A deterministic two-tick scheduler cost for clear-cardinal ranged attacks, with legal-action
  overflow filtering and MCP ready-time/replay evidence; other command costs remain unchanged.
- A deterministic cardinal line-of-sight guard for scheduled ranged attacks, with typed no-line-of-
  sight rejection, legal-action filtering, protocol v5 conversion, and MCP atomicity evidence;
  cover, ammunition, and projectile presentation remain deferred.
- A deterministic scheduled `RangedAttack` command for living actors at Manhattan distance 2–3,
  with typed core/protocol/MCP/headless evidence, replay participation, desktop `G` targeting,
  and display-free smoke coverage; line of sight, cover, ammunition, weapon effects, varied action
  costs, and enemy ranged AI remain deferred.

- A scheduled player-facing `Pickup` command that preserves deterministic ground-stack and inventory
  order, emits typed protocol/MCP evidence, and binds the desktop `P` control plus display-free smoke
  coverage; effects, capacity, and the drop command remain deferred.
- A bounded desktop animation pulse driven by new typed event-cue batches, with no movement
  interpolation, production media, audio playback, or simulation timing changes.
- An optional desktop audio-cue adapter that routes distinct typed batches through the validated
  local manifest and requests non-looping playback only for existing local files; missing media stays
  a safe fallback and no production audio is committed.
- A reproducible local-only CC0 art preparation script that validates the recorded Kenney archive
  and installs six nearest-neighbor showcase source tiles without tracking media binaries.
- A deterministic tactical HUD summary for health, turn ownership, enemy pressure, and optional
  field-of-view state, without simulation or media boundary changes.
- An optional deterministic presentation field-of-view projection with radius-bounded cardinal
  traversal, readable wall boundaries, retained-but-hidden out-of-view render nodes, and a radius-3
  desktop showcase configuration; core, protocol/MCP snapshots, and journal evidence remain full.
- A feature-gated `dreadstep` desktop showcase with one primary 2D window, deterministic human
  controls, enemy chase driving, inventory/combat/death/HUD presentation, optional per-family art
  fallback, display-free smoke coverage, and create-new flushed JSONL diagnostic journals.
- Aspirational concept art reference and a documented pixel-art/audio asset sourcing and
  licensing workflow.
- A six-package Rust workspace with explicit domain and adapter boundaries.
- A repo-local Dreadstep development and review harness.
- Reproducible Rust, formatting, lint, documentation, and CI configuration.
- Operational specification, architecture, contribution, ADR, and lessons documentation.
- The first Milestone 1 rules-kernel slice: typed grid state, actors, movement and blocking
  events, and deterministic integer action scheduling.
- Typed hit points, fixed basic melee attacks, and semantic death events with dead actors
  removed from scheduling and movement occupancy.
- Deterministic enemy chase commands with explicit axis tie-breaking and shared blocking
  events.
- Core-owned replay traces and stable state digests for deterministic regression evidence.
- A deterministic `dreadstep-headless` developer CLI that translates command tokens into the
  core simulation and prints replay seed, events, and final state digest evidence.
- A versioned `dreadstep-protocol` world snapshot projection for deterministic agent
  observation, without adding an MCP transport runtime.
- A versioned protocol action envelope that maps typed agent requests to and from core commands
  without executing them.
- A pure in-memory MCP player session for deterministic start, observe, and act flows without a
  transport runtime.
- Core-owned deterministic legal-action discovery exposed through the in-memory MCP session.
- Accepted-action session history and deterministic replay digest evidence backed by core
  `ReplayTrace`.
- A typed in-memory `get_replay` evidence bundle exposing seed, accepted protocol requests, and
  deterministic digest without persistence or transport serialization.
- Read-only player actor inspection over the versioned snapshot, including retained dead actor
  records and an explicit unknown-actor result.
- Named `get_history` access to the existing protocol-owned accepted-request history projection.
- In-memory tester `snapshot`/`restore` savepoints that preserve world and replay state without
  persistence or transport effects.
- Named tester `inspect_world` access to the existing complete protocol world snapshot.
- Validated tester actor spawning through core with protocol-owned world error projections.
- Validated tester hit-point mutation through core, including dead-record retention and
  scheduler-safe reanimation, with typed protocol error projection.
- Typed in-memory tester scenario replacement backed by core map and world validation, with
  atomic failure and a fresh replay trace for the preserved seed.
- Opaque typed tester item ownership with deterministic actor inventory snapshots and duplicate
  identity validation; gameplay effects and inventory capacity remain deferred.
- Validated tester teleport with typed destination validation, dead-record occupancy semantics, and
  no player-trace effects.
- Minimal local MCP stdio observation with versioned snapshot JSON, `start_run`, and read-only
  `observe` tools.
- Typed MCP player `act` requests with explicit command/event JSON and schemas, structured action
  evidence over stdio, and invalid-params rejection that leaves state and replay unchanged.
- Read-only MCP `legal_actions` discovery over stdio with core-defined deterministic ordering and
  no world, history, or replay mutation.
- Read-only MCP actor inspection over stdio with typed IDs, structured snapshot-or-null results, and
  no world, history, or replay mutation.
- Read-only MCP `get_history` accepted-request evidence over stdio with deterministic ordering,
  rejection omission, and no world, history, or replay mutation.
- Read-only MCP `get_replay` replay evidence over stdio with typed seed, accepted requests,
  deterministic digest, explicit JSON/JSON Schema, and no persistence or playback semantics.
- A deterministic Bevy presentation bridge with immutable map/actor snapshots, keyboard intent
  mapping, core command execution, and replay evidence without enabling desktop platform features.
- A validated authored starter-floor definition in `dreadstep-content` and a Bevy `start_run` path
  that delegates to it while preserving the explicit replay seed.
- Headless Bevy scene synchronization for deterministic map-tile and actor ECS mirrors, including
  stable entity identity, stale-entity removal, and retained dead-record presentation.
- A headless Bevy `PresentationRuntime` resource and `PresentationPlugin` that automatically project
  core-backed snapshots into the scene after each app update without desktop engine features.
- Deterministic headless keyboard dispatch with explicit controlled-actor selection, fixed key
  priority, one-command-per-update consumption, and same-update scene projection.
- A one-shot `PresentationRuntime` feedback buffer for accepted typed event/snapshot evidence, with
  stale-output clearing on rejected commands and no new authoritative state.
- A typed headless `PresentationFocus` projection that mirrors the selected actor's position for
  future camera systems without adding viewport, visibility, or rendering policy.
- A typed headless `SceneFocus` marker that reuses the stable keyed actor entity for future camera
  or selection systems without copying actor state or adding marker visuals.
- A deterministic headless `PresentationCamera` resource and `SceneCamera` ECS projection that
  mirror the selected actor's authoritative center, retain one camera entity, and clear unknown
  anchors without adding viewport or rendering policy.
- A deterministic headless `PresentationViewport` request and `SceneViewport` ECS projection that
  clamp tile rectangles to the map around the camera anchor without adding visibility or rendering
  policy.
- A typed headless `PresentationHud` status projection for controlled actor kind, position, hit
  points, and scheduler readiness without adding widgets, text, or rendering policy.
- A deterministic typed `PresentationMessages` projection for every current core event, preserving
  event order and clearing stale rejected-command evidence without adding text, audio, or gameplay
  policy.
- A deterministic typed `PresentationAudioCues` placeholder projection for every current core event,
  preserving order and stale-rejection clearing without assets, playback, or an audio backend.
- Typed `SceneSpriteRole` metadata alongside headless scene mirrors for terrain, living actors, dead
  records, and item entities, without textures, assets, or rendering plugins.
- A typed `PresentationAnimationCues` placeholder projection for movement and combat event order,
  with stale-rejection clearing and no timers, interpolation, assets, or rendering backend.
- A validated typed `PresentationWindow` request for logical dimensions, integer pixel scale, and
  checked physical dimensions without creating an OS window or enabling desktop features.
- Caller-selected `PresentationTileSize` and checked `ScenePixelPosition` metadata for terrain,
  actors, and ground items; transforms, assets, and rendering remain deferred after the verified
  tile-size experiment.
- A tracked Milestone 3 asset-evaluation record with local-only generated/CC0 pixel-art and audio
  candidates, exact nearest-neighbor 24×24/32×32 samples, a provisional 32×32 working scale, and
  an open decision for dungeon cue sourcing; no binary is loaded or committed.
- A verified typed ordered `PresentationRenderProjection` over keyed Bevy scene mirrors, preserving
  complete values and per-kind sprite roles/checked placement while keeping pixel positions off
  unplaced inventory items; no render features, textures, transforms, asset loading, or playback
  are added.
- A verified typed `SceneSpriteKey`/`PresentationSpriteProjection` boundary over complete render
  entries, retaining terrain/actor/item selectors, ECS identity, roles, and placement metadata
  without loading assets, enabling rendering, or committing media binaries; actual rendering and
  media remain deferred.
- A verified `PresentationRenderCommandPlan` boundary that derives deterministic terrain, ground,
  actor, and inventory draw layers plus source order and optional placement from sprite entries,
  without loading assets or enabling render plugins; actual rendering remains deferred.
- A verified `PresentationRenderNodeProjection` bootstrap that reconciles stable ECS placeholder
  nodes from typed render commands while keeping Sprite components, render plugins, windows, assets,
  animation, audio, and media deferred; actual rendering remains future work.
- A verified metadata-only `PresentationAssetManifest` and
  `PresentationRenderAssetProjection` boundary that validates one anchored local-media reference per
  placeholder family and joins it to stable node metadata without file loading, asset handles, or
  committed pixel-art/audio binaries.
- A verified metadata-only `PresentationAudioAssetManifest` and
  `PresentationAudioAssetProjection` boundary that exhaustively binds eight typed cue families to
  validated local `audio/` references while preserving payload/order without playback or an audio
  backend.
- A verified headless Bevy Sprite API projection: `PresentationBevySpriteProjection` joins
  deterministic solid-color `Sprite` values to stable placeholder nodes with optional logical tile
  sizing while keeping Sprite/render plugins, textures, transforms, playback, and media deferred.
- A verified ECS Sprite-node attachment slice copies those typed values onto retained placeholder
  entities while preserving identity and default required components; render plugins, transform
  placement, texture loading, playback, and production media remain deferred.
- A verified headless Sprite-transform projection derives ordered map-space translations from checked
  pixel origins while keeping inventory unplaced and ECS transforms unchanged; fresh missing tile size
  starts unplaced while later removal preserves checked translations. Cameras, windows, rendering,
  playback, and production media remain deferred.
- A verified ECS Sprite-transform attachment boundary applies deterministic centered logical-pixel
  `(x + tile_width/2, y + tile_height/2, layer_depth)` values to retained map-node `Transform`
  components while leaving inventory unplaced and deferring anchor variants, cameras, visibility,
  rendering, playback, and production media.
- A verified ECS Sprite-depth boundary derives deterministic terrain/ground/actor z-layer values from
  typed render layer while preserving centered x/y placement and inventory default state.
- A verified headless ECS Camera2d attachment boundary adds only Bevy's typed camera marker/default
  orthographic components to the retained disposable camera projection entity while deferring
  windows, camera viewport policy, render plugins, visibility, playback, and production
  media.
- A verified headless ECS Window configuration boundary mirrors validated logical/physical dimensions
  and the exact integer scale onto a disposable `SceneWindow`, exposes a deterministic `f32` scale
  adapter on Bevy's `WindowResolution`, and defers OS/window plugins, render backends, camera policy,
  visibility, playback, and media.
- A verified headless ECS camera-transform boundary attaches checked centered logical-pixel
  `Transform` values to the retained disposable `SceneCamera` while deferring viewport policy,
  OS/window integration, render backends, visibility, playback, and media.
- A deterministic content-owned catalog of opaque item-definition identities with duplicate
  validation, while effects, capacity, and richer item gameplay remain deferred.
- A deterministic tester-only item transfer across core, protocol, and in-memory MCP boundaries;
  effects, capacity, and player-facing item commands remain deferred alongside the separate
  equipment contract below.
- A deterministic tester-only item drop with core-owned row-major ground-item stacks and complete
  protocol/MCP snapshot projection; pickup and item gameplay semantics remain deferred.
- A deterministic tester-only item pickup that removes from ordered ground stacks, appends to actor
  inventories, and projects typed ground-miss errors without player replay/history effects.
- A deterministic single-slot equipment contract with scheduled typed equip/unequip commands,
  ordered replacement events, versioned digest/replay state, protocol/MCP snapshot evidence, and a
  typed Bevy `SceneActor` projection; item effects, modifiers, capacity, and extra slots remain
  deferred.
- A deterministic single-item consumption preparation contract with scheduled `UseItem`, atomic
  ownership/equipment validation, typed `ItemConsumed` protocol/MCP evidence, standard action-time
  advancement, and stale Bevy inventory cleanup; item effects, stats, capacity, and richer gameplay
  remain deferred.
- A deterministic headless Bevy ground-item scene projection that preserves complete typed item
  data, stable item-identity entities, stack order, and stale cleanup without rendering policy.
- A deterministic headless Bevy inventory-item scene projection that preserves global item identity,
  owner/order updates, and stale cleanup without adding inventory gameplay or HUD policy.
- Optional ordered opaque item placements for authored starter floors, delegated to core's existing
  item-identity and inventory validation while preserving the item-free default scenario.
- Explicit starter-floor catalog binding that rejects duplicate catalog IDs and unknown placement
  definitions before core world construction without leaking catalog data into runtime state.
- A deterministic non-default starter-item content scenario that exercises the catalog-bound
  placement path while preserving the item-free default starter floor.
- Explicit headless Bevy `start_item_run` constructors that project the non-default scenario's
  typed inventory items while preserving the item-free default startup path.

### Changed

- Presentation art and audio binaries are now explicitly local-only under anchored root or
  crate-local media directories, with the concept-art reference and future screenshots retained as
  documented tracked exceptions.
- Replaced the root Bevy starter binary with package shells for Milestone 0.
- Limited Bevy to the presentation package and its minimal standard-library feature set.
