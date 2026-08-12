# Dreadstep Architecture

Last Reviewed: 2026-08-12
Status: Verified

## Overview

Dreadstep is organized as a functional domain kernel surrounded by explicit adapters. The
kernel decides game outcomes; adapters translate external input into semantic commands and
translate semantic events into presentation, files, telemetry, or transport responses.

The current implementation exposes a runnable desktop showcase alongside the headless and MCP
adapters. Milestone 1 exposes the first gameplay API in `dreadstep-core`: a typed rectangular map,
actors, movement, melee, and chase commands, semantic movement/blocking/combat/death events,
an integer ready-time scheduler, canonical run-outcome projection, and core-owned replay
traces/state digests. The
`dreadstep-headless` adapter now provides a fixed-scenario developer CLI that translates text
arguments into those core commands; it owns parsing and stdout only. `dreadstep-mcp` also provides
a minimal local stdio server for the bounded player tools `start_run`, `observe`, `legal_actions`,
`inspect`, `get_history`, `get_replay`, and typed `act`. The feature-gated `dreadstep-bevy`
desktop binary owns the optional OS window, human input, local art fallback, HUD, and diagnostic
journal around the same deterministic presentation runtime.

## Package Ownership

| Package | Owns | Must not own |
| --- | --- | --- |
| `dreadstep-core` | Domain state, commands, events, errors, deterministic rules | I/O, Bevy, MCP, authored-file formats |
| `dreadstep-protocol` | Versioned external representations and conversions | Domain decisions or transports |
| `dreadstep-content` | Validation of authored definitions into domain values | Hidden simulation rules |
| `dreadstep-headless` | CLI, files, processes, telemetry, batch execution | Authoritative game behavior |
| `dreadstep-mcp` | Bounded player and tester operations | Arbitrary host access or game truth |
| `dreadstep-bevy` | Headless projection plus optional desktop input, window/render setup, HUD, assets, and journal | Authoritative state or rules |

## Dependency Direction

```text
protocol ----> core <---- content
                  ^
                  |
       +----------+----------+
       |          |          |
    headless     MCP       Bevy
```

The adapter packages may depend on protocol and content as well as core. Core, protocol,
and content must never depend on Bevy or MCP runtime libraries. `dreadstep-bevy` keeps the
headless feature graph minimal; its opt-in `desktop` feature adds Bevy's winit, X11, 2D render,
UI/text, nearest-neighbor image, optional audio playback, and logging capabilities while continuing
to exclude Wayland and `default_platform`.

## Intended Data Flow

```text
external input -> adapter -> core command -> deterministic transition
                                           -> next state + semantic events
semantic events -> adapter -> output, presentation, telemetry, or protocol response
```

The desktop process boundary creates a create-new JSONL journal before opening the window. Its
timers, HUD text, asset handles, and shutdown status are disposable effects; only the runtime's
legal-command query and core execution determine simulation outcomes. The display-free `--smoke`
path calls those same boundary helpers without initializing winit or a renderer.

The visible showcase's tactical HUD is a disposable text projection over the authoritative runtime
snapshot plus optional `PresentationVisibility`: its compact health, turn, enemy-pressure, and
field-of-view summaries are formatted locally, while inventory, event, controls, and journal lines
remain existing presentation effects. Missing player data and absent optional visibility use
explicit safe fallbacks; the HUD cannot issue commands or alter core state.

The visible client may consume the existing `PresentationAnimationCues` buffer as a local visual
effect. Its fixed-duration actor pulse is driven by Bevy presentation time only, starts on a newly
observed non-empty cue batch, and uses the runtime replay digest to distinguish a later accepted
batch even when its cue values are identical. It leaves core action time, sprite identity, visibility,
assets, transforms, and diagnostic journal evidence untouched. Missing cue or pulse state is a no-op.

The visible client may also consume the existing `PresentationAudioCues` and validated
`PresentationAudioAssetManifest` through an optional desktop playback effect. It observes the replay
digest plus ordered cue values, requests each existing `assets/`-rooted local reference once per
distinct batch, and uses non-looping Bevy `AudioPlayer` entities. Root/crate-local references remain
valid headless metadata but are safe unsupported-root fallbacks at this desktop boundary. Missing
references or audio resources are safe recorded fallbacks; audio playback never changes core state,
timing, event payloads, or replay evidence.

The earlier headless presentation records below remain valid when the `desktop` feature is absent.
The runnable showcase is an opt-in process wrapper around those projections: its ECS scene, HUD,
asset handles, timers, and journal are effects and never a second simulation authority.

State, configuration, seeded randomness, and time inputs should be explicit. Prefer pure
transformations and returned outcomes; allow tightly scoped mutation when it is clearer or
materially more efficient in Rust.

## Current kernel slice

`dreadstep-core` owns the canonical `WorldState`. `GridMap` limits dimensions to the signed
`Position` coordinate domain and treats out-of-bounds and wall tiles as terrain blockers;
living actor occupancy is checked separately so events can distinguish terrain from another
actor. `WorldState::execute` accepts only the living actor at the minimum `ActionTime`, orders
ties by `ActorId`, applies fixed melee damage to adjacent targets or the bounded one-point
ranged attack on clear cardinal rays at Manhattan distance 2–3, resolves enemy chase steps with
horizontal-axis priority,
and advances the acting actor by the fixed action cost. Dead
actor records remain inspectable but are removed from scheduling and movement occupancy. The
stable state digest uses an explicit deterministic byte order and does not use a
process-randomized standard hasher. No wall-clock time or process-global randomness
participates in these transitions.

The headless CLI must not duplicate movement, combat, chase, scheduling, or digest logic. Its
fixed scenario is test data at the adapter boundary; every outcome still comes from
`dreadstep-core::WorldState::execute`.

The first Milestone 2 protocol slice is a read-only `WorldSnapshot` projection. It may expose
stable actor data and core digest evidence, but it must not decide legal actions or mutate the
world; the minimal MCP stdio observation server now packages this projection, while broader
transport and session effects remain later adapter work.

The protocol action envelope is likewise only a typed conversion boundary: it can translate
external request values into canonical core commands and back, but command validation and
execution remain owned by `dreadstep-core::WorldState`.

The first MCP player slice is an in-memory session over those protocol values. It owns session
seed/scenario setup and response shaping only; the minimal stdio server wraps its `start_run`,
`observe`, `legal_actions`, `inspect`, `get_history`, `get_replay`, and typed `act` operations without duplicating
core transition rules. Additional transports and tester operations remain future slices.

Legal-action discovery is a core query, not an MCP policy: `WorldState::legal_commands` decides
which typed commands are currently valid, and the session only maps those commands into protocol
requests without mutating state.

Session history is an adapter-owned view over core `ReplayTrace`: accepted commands are recorded
after successful execution, rejected requests are omitted, and only protocol requests plus the
core digest value cross the MCP boundary.

The `get_replay` projection packages that history, explicit seed, and core trace digest in a
protocol-owned `ReplayEvidence` value. It remains an in-memory read-only view. The desktop boundary
also writes a version-1 `*.replay.json` diagnostic artifact containing the explicit seed, accepted
core command order, replay digest, and canonical outcome beside each run journal. That artifact is
create-new and evidence-only; persistence, parsing, editing, playback, and transport registration
remain outside the current contract.

The versioned `WorldSnapshot` also projects core's deterministic `RunOutcome` (`in_progress`,
`defeat`, or `victory`). Core derives this value from retained actor records, with player defeat
precedence and no-enemy worlds remaining in progress; protocol and MCP only translate the result.

Protocol v13 projects the fixed four-item inventory capacity on each actor snapshot and carries the
optional healing result on `ItemConsumed`. Core enforces the limit for tester ownership/pickup/
transfer and scheduled player pickup; adapters only translate the typed rejection, legal-action
omission, and effect evidence, so stacking, upgrades, and richer item rules do not become hidden
adapter rules.

The player `inspect` operation is likewise a read-only lookup over the protocol world snapshot.
It returns one protocol `ActorSnapshot` or no value for an unknown identity, preserves dead actor
records for inspection, and adds no visibility policy or gameplay behavior.

The proposal's `get_history` name maps to the same adapter-owned protocol request projection as
`Session::history`; the alias does not create a second source of truth or alter core trace
recording.

Tester savepoints are explicit in-memory `SessionSnapshot` values containing the session seed,
core world, and core replay trace. `restore` replaces that owned state, so branching and rollback
remain deterministic without exposing arbitrary mutation or adding storage effects.

The tester `inspect_world` name maps to the same protocol `WorldSnapshot` projection as player
`observe`; it is a read-only alias and does not create separate world storage or hidden rules.

Tester spawning crosses the boundary as a typed request to core `WorldState::spawn`. Core validates
identity, map, living occupancy, and hit points; MCP only converts protocol values and projects
typed world errors, preserving atomic failure and one source of game truth.

Tester hit-point mutation crosses the boundary as a typed request to core
`WorldState::set_hit_points`. Core owns dead-record retention, living occupancy, and scheduler-safe
reanimation at the current action time; MCP only converts the request and projects an unknown-actor
world error. Tester mutations remain outside accepted player history and replay evidence.

Tester scenario replacement crosses the boundary as a protocol-owned `Scenario` value. MCP maps
its tiles and actor specs into `GridMap` and `Actor` values, then delegates all map and world
validation to `WorldState::new` before replacing the session. Failed construction is atomic;
successful replacement preserves the seed and starts a fresh in-memory replay trace.

Opaque tester item ownership crosses the boundary as typed `ItemId` and `ItemDefinitionId` values.
Core owns global identity uniqueness, ordered actor inventories, digest inclusion, and snapshot
projection; MCP only converts the request to `WorldState::give_item`. The equipment
extension adds one optional core-owned `ItemId` reference per actor, scheduled equip/unequip
commands, ordered replacement events, and typed protocol/MCP projections; it does not create a
second item store or apply effects. The fixed four-item capacity is enforced by core across every
ownership ingress; richer item semantics remain deferred. The tester
transfer extension delegates source ownership, ordering, dead-record validity, and atomic errors
to core; MCP only projects the result and does not record player history or replay evidence.

The item-catalog foundation keeps definition membership on the content side: it validates ordered,
opaque `ItemDefinitionId` references and exposes read-only lookup without changing core world state.
The catalog is authoring support rather than a second item store; core remains authoritative for
item instances, ownership, effects, digests, and snapshots. The first authored healing effect is
carried on the core item instance and applied by `UseItem`; adapters only project its optional result.

The tester item-drop extension keeps ground-item records in core, keyed by stable map position with
deterministic stack order. Protocol projects those records as read-only snapshot values and MCP
delegates the mutation; the tester mutation remains outside player replay/history. An equipped item
is rejected before this tester mutation can invalidate the actor's optional equipment reference.

The tester item-pickup extension moves an item from the actor's current core-owned ground stack back
into that actor's ordered inventory. Protocol/MCP preserve this tester operation separately from the
scheduled player command: the player-facing `Pickup` command consumes one standard action, emits
`ItemPickedUp`, and records replay/history evidence, while the tester operation remains an atomic
non-action mutation. Both paths preserve stack and inventory order; richer effects remain deferred
while core enforces the shared fixed four-item capacity.

Validated tester teleport crosses the boundary as a typed actor identity and destination position.
Core owns bounds, terrain, living occupancy, and preservation of scheduler/inventory state; MCP only
converts the request and projects typed world errors. Dead actor records remain non-occupying, and
the mutation does not enter player history or replay evidence.

The minimal MCP stdio slice adds a process adapter around the existing session. The adapter owns
`rmcp` transport setup, tool schemas, and versioned JSON serialization for the bounded player tools
`start_run`, `observe`, `legal_actions`, `inspect`, `get_history`, `get_replay`, and typed `act`;
session and core
remain authoritative for seeded state and world truth. Stdout is reserved for MCP protocol traffic,
and tester mutations remain library-only.

The first Milestone 3 Bevy slice is a deterministic presentation bridge. `GridMap::tiles` gives
the adapter an immutable row-major terrain projection, while `dreadstep-bevy::PresentationState`
owns a core world and replay trace and exposes map/actor/time/digest snapshots. Its keyboard intent
mapping produces canonical core movement and wait commands for an explicit actor, and accepted
commands delegate to `WorldState::execute`; rejected commands are not recorded. The bridge is
headless-testable and enables only Bevy's keyboard feature, so windowing, rendering, assets, and
audio remain later presentation slices.

The shared authored starter-floor slice adds `dreadstep-content::StarterFloorDefinition` and its
validated `starter_floor` constructor. Content owns the row-major map and initial actor records,
then delegates all dimension, terrain, identity, occupancy, and life validation to core.
`PresentationState::start_run` consumes that constructor and preserves the caller's seed; MCP and
future clients may choose their own adapter scenarios without making content or presentation
state authoritative over the rules kernel. `PresentationState::start_item_run` and the matching
`PresentationRuntime` constructor explicitly consume the separate catalog-bound item scenario;
the default startup remains item-free, and the normal plugin sync projects its inventory items as
disposable `SceneInventoryItem` mirrors.

The authored starter-item extension adds optional ordered `StarterItemPlacement` values and an
explicit `ItemCatalogDefinition` binding to that content input. Building a floor validates catalog
duplicates and placement definition membership before map/world construction, then delegates each
valid opaque item instance to `WorldState::give_item`. Core remains authoritative for actor
identity, global item identity, inventory order, and digest state; the catalog is never copied into
the world. The default starter floor remains item-free. A separate `starter_item_floor` content
helper provides one deterministic catalog-bound fixture for adapters and tests without changing
that default or introducing item gameplay semantics.

The headless scene-synchronization slice projects a complete `PresentationSnapshot` into disposable
`SceneTile` and `SceneActor` ECS components. The synchronizer keys entities by stable map position
and `ActorId`, preserves identity across updates, removes stale or duplicate keys deterministically,
and mirrors actor position, life, hit points, and scheduler readiness. Dead actor records remain
represented because core snapshots retain them. ECS data is a render mirror only; it cannot issue
commands or replace `WorldState` as game truth.

The headless application-shell slice adds `PresentationRuntime` as the sole Bevy resource owning a
`PresentationState`, and `PresentationPlugin` as an exclusive update system that clones a runtime
snapshot before calling `sync_scene`. This keeps the core-backed resource borrow separate from ECS
mutation while making startup and post-command projection automatic for a Bevy `App`. The plugin
still enables no window, rendering, audio, or desktop platform features; command submission remains
an explicit runtime API and never originates from scene components. Its keyboard-dispatch extension
uses an explicit `PresentationInput` actor and fixed key priority, consumes one frame's supported
just-pressed keys deterministically, delegates through core, and projects before the update ends.

The presentation-feedback slice extends that same runtime with an optional latest
`PresentationOutput`. Accepted direct or keyboard commands publish typed event/snapshot evidence;
consumers can inspect it without mutation or take it once. Rejected commands clear stale feedback
while preserving `WorldState`, replay digest, and disposable scene mirrors, so the buffer remains
adapter evidence rather than a second simulation store.

The focus-projection slice adds an optional `PresentationFocus` resource keyed by the explicit
`PresentationInput` actor. After dispatch and scene synchronization, it mirrors that actor's latest
core position or `None` when unknown. It is a camera-facing projection only: no viewport policy,
visibility rule, interpolation, or alternate actor record is introduced.

The scene-focus-marker slice adds a marker-only `SceneFocus` component to the existing keyed
`SceneActor` entity after that focus projection. It reuses stable actor identity without copying
position or gameplay state; unknown actors clear stale markers only when an authoritative runtime
snapshot exists, while missing resources leave the disposable scene unchanged.

The headless camera-anchor slice adds an optional `PresentationCamera` resource and one keyed
`SceneCamera` projection. The adapter mirrors only the selected actor's latest core position,
retains the existing camera entity across updates, and removes duplicate or unknown anchors
deterministically. It remains a center-point projection: windowing, transforms, viewport sizing,
clamping, interpolation, visibility, and fog policy belong to later presentation work.

The headless viewport slice adds an optional `PresentationViewport` request and one keyed
`SceneViewport` projection. The adapter clamps the requested tile rectangle to the current map,
centers it on the camera anchor with integer arithmetic, and mirrors only the effective origin and
dimensions. Oversized requests use the complete map; unknown or missing authority leaves the
disposable viewport unchanged or clears it deterministically without adding visibility policy.

The verified presentation field-of-view slice adds an optional `PresentationVisibility` resource.
When a controlled actor and runtime snapshot are available, it performs a bounded cardinal floor
traversal from that actor and includes adjacent wall tiles as readable boundary evidence. The
resource is presentation-only: core snapshots, protocol/MCP observation, commands, replay evidence,
and the diagnostic journal remain complete. `SceneRenderNode` carries the derived visibility bit so
retained render-node entities can be hidden without despawning or mutating their typed scene
mirrors; removing the optional resource or losing its authority restores the fully visible
headless default. The desktop showcase opts into a radius-3 projection, while inventory nodes
remain hidden because they have no map placement.

The verified headless HUD-status slice adds an optional `PresentationHud` resource keyed by the
controlled actor. It mirrors only typed actor kind, position, hit points, and scheduler readiness;
unknown actors clear those optional values and missing resources preserve existing status. This is
future-HUD data, not text, layout, widget, rendering, audio, or gameplay policy.

The verified event-message slice adds an optional `PresentationMessages` resource that mirrors the
latest runtime output as ordered typed `PresentationMessage` values. It clears stale evidence when
runtime output disappears, preserves state when authority is absent, and performs no formatting,
localization, widget, audio, or gameplay work.

The verified audio-placeholder slice adds an optional `PresentationAudioCues` resource that maps the
same latest events to ordered typed cue values. It is a headless contract for a future player only:
it loads no assets, enables no audio backend, performs no playback, and preserves the adapter's
authority and stale-output rules.

The verified sprite-role slice adds a `SceneSpriteRole` component alongside each existing keyed scene
mirror. It classifies terrain, living player/enemy actors, retained dead records, and item mirrors;
the existing typed scene components remain authoritative projections, while textures, transforms,
asset selection, and rendering stay outside this headless boundary.

The verified animation-cue slice adds an optional `PresentationAnimationCues` resource that mirrors
latest runtime events as ordered typed movement and combat signals. It is evidence for a future
renderer only: no timers, interpolation, animation state machine, assets, transforms, or rendering
work occurs at this boundary.

The verified window-request slice adds an optional `PresentationWindow` resource that validates logical
dimensions, integer pixel scale, and checked physical dimensions. It is configuration for a future
desktop client only; no OS window, platform event loop, desktop feature, or rendering state is
created here.

The verified scene-pixel-placement slice adds an optional `PresentationTileSize` resource and a
`ScenePixelPosition` component. The adapter converts valid core map coordinates to checked logical
pixel origins on the already keyed terrain, actor, and ground-item mirrors. Tile-size selection
remains a caller/asset-experiment decision; the component is disposable placement metadata, not a
Bevy transform or a new source of simulation truth. Missing configuration preserves the existing
scene, and inventory items remain unplaced because they have no map coordinate. Textures, assets,
window/render plugins, audio, timers, interpolation, visibility, persistence, and gameplay remain
outside this boundary.

The verified presentation asset evaluation and native tile-size evidence are tracked outside the
Rust boundary in `docs/presentation/asset-evaluation.md` and `docs/presentation/tile-samples.md`.
The selected local CC0 art fallback is prepared by the repository-side
`scripts/prepare-local-assets.sh` adapter. It validates the recorded archive hash and source
members, writes only to ignored local-media paths, and leaves `PresentationAssetManifest` plus
the deterministic placeholder path authoritative when the archive or files are absent. This
preparation step has no core, protocol, replay, journal, or public schema effect.
They record local-only generated and CC0 candidates, exact nearest-neighbor 24×24/32×32 samples,
and a provisional 32×32 working scale; dungeon audio sourcing remains open after a UI-only fallback
evaluation. The verified reversible renderer boundary consumes this metadata without loading
production assets or enabling render plugins. `PresentationRenderProjection` is a read-only ordered
resource over the existing keyed mirrors: it carries complete typed values and per-kind roles,
derives map-backed pixel positions from each mirror's own typed position when tile-size configuration
is present, keeps inventory items unplaced, and preserves retained metadata when configuration is
absent. It does not become another source of simulation truth. Actual windowing, rendering, and
production texture loading remain deferred to later presentation slices; optional audio playback is
implemented separately at the desktop effect boundary.

The verified sprite-key slice derives a closed `SceneSpriteKey` from each complete
`SceneRenderEntry` and exposes it through `PresentationSpriteProjection`. Terrain retains its typed
tile, actors retain player/enemy/dead roles, and item keys retain opaque definition identity; the
projection preserves retained entities and placement metadata without loading assets or becoming
another source of simulation truth. Actual texture loading, render plugins, transforms, and media
remain deferred.

The verified render-command-plan slice derives `PresentationRenderCommandPlan` from the verified
sprite projection. Each `SceneRenderCommand` retains its complete typed entry, ECS identity, sprite
key, optional placement, deterministic layer, and source order. This is a read-only draw plan for a
future renderer; it does not load assets, create transforms or windows, enable render plugins, or
become simulation authority.

The verified placeholder render-node bootstrap reconciles `PresentationRenderNodeProjection` from
that command plan. `SceneRenderNode` entities are disposable renderer-facing metadata: they retain
source/layer identity across role refreshes, typed placeholder families, ordering, and optional
placement before the separate ECS Sprite attachment. The attachment adds raw Sprite components with
default required state; no render plugins, windows, transform placement, asset loading, or audio are
introduced.

The verified headless Sprite API bridge enables only Bevy's `bevy_sprite` API feature. It adds
`PresentationBevySpriteProjection`, whose ordered `SceneBevySpriteEntry` values join each stable
placeholder node to a deterministic solid-color `Sprite` with the caller-selected tile size when
available. The Sprite keeps Bevy's default image handle unset. The verified ECS attachment copies
those typed values onto retained node entities, where Bevy's required Transform/Visibility components
remain defaults; no Sprite/render plugin, texture loading, transform placement, window, playback, or
production media policy is introduced. Missing runtime, node source, projection destination, or node
entities preserve existing components safely, and the wrapped core runtime remains authoritative.

The verified Sprite-transform projection adds `PresentationBevySpriteTransformProjection`, an ordered
read-only join from each retained node to a map-space translation derived from checked
`ScenePixelPosition` metadata. Inventory entries remain unplaced; a fresh missing tile-size request
starts unplaced while later removal preserves previously checked translations. This boundary does not
attach or mutate ECS Transform/Visibility/Sprite components; camera, window, renderer, and production
media remain deferred.

The verified ECS Sprite-transform attachment consumes that projection and writes only deterministic
centered logical-pixel `Transform` translations `(pixel_x + tile_width/2, pixel_y + tile_height/2,
layer_depth)` onto retained map-backed node entities. Inventory nodes remain unplaced/default; anchor
variants, cameras, visibility, render plugins, windows, and production media remain separate deferred
boundaries.

The verified ECS Sprite-depth boundary extends those transforms with deterministic layer depth:
terrain `0.0`, ground items `1.0`, actors `2.0`, and inventory default/unplaced `0.0`. It preserves
the existing centered x/y values, source/order/identity semantics, and authority guards without adding
anchor variants, cameras, visibility, render plugins, windows, or production media.

The verified headless ECS Camera2d attachment adds Bevy's typed `Camera2d` marker and required default
camera components to the retained disposable `SceneCamera` projection entity. `PresentationCamera`
and runtime remain the source of camera center/origin truth; window creation, camera
viewport/visibility policy, render plugins, and production media remain deferred.

The verified headless ECS Window configuration attachment enables only Bevy's `bevy_window` API
feature and mirrors the exact validated integer `PresentationWindow` request onto a disposable
`SceneWindow` entity with a `Window` component. Bevy's `WindowResolution` receives a deterministic
`f32` scale adapter; `PresentationWindow` remains authoritative and this boundary does not create
OS windows or enable WindowPlugin/winit/default-platform, render backends, camera policy, visibility,
or production media.

The verified headless ECS camera-transform attachment derives centered logical-pixel `Transform`
values for the retained disposable `SceneCamera` from checked map origins and caller-selected tile
half-extents. `PresentationCamera` and runtime remain authoritative; viewport policy, OS/window
integration, render backends, visibility, and production media remain deferred.

The verified local-only asset-manifest slice adds `PresentationAssetManifest` and
`PresentationRenderAssetProjection` as another read-only boundary. Validated relative references
join the ordered placeholder nodes while preserving node identity and metadata; the projection does
not inspect the filesystem, create asset handles, or load pixel/audio binaries. Missing authority,
source, manifest, or destination resources preserve the prior projection, and the repository media
policy keeps binaries ignored while tracked provenance remains visible.

The verified local-only audio cue manifest adds `PresentationAudioAssetManifest` and
`PresentationAudioAssetProjection` over the typed `PresentationAudioCues` resource. It binds all
eight cue families to validated root/crate-local `audio/` references while preserving event payload
and order. This metadata boundary performs no filesystem reads, handle creation, backend setup, or
playback; audio remains outside simulation authority.

The ground-item scene-projection slice extends the same disposable snapshot boundary with
core-owned ground stacks. `PresentationSnapshot` preserves row-major stack and item order, while
`SceneGroundItem` carries only the typed item identity, opaque definition reference, position, and
stack index; `sync_scene` keys those entities by globally unique `ItemId` and removes stale picked-up
items. Bevy does not own item data, effects, or pickup/drop rules, and this projection adds no
rendering or camera policy.

The inventory-item scene-projection slice mirrors core-owned actor inventories through the
same boundary. `SceneInventoryItem` carries only global item identity, owner actor, opaque definition
reference, and insertion order; `sync_scene` updates retained item entities after core-authoritative
owner/order changes and removes stale records. Bevy does not own inventory or item gameplay rules,
and no HUD or rendering policy is introduced.

The verified deterministic single-slot equipment slice extends the same core authority with an optional
`Actor::equipped_item` identity that must point into that actor's ordered inventory. `Equip` and
`Unequip` are scheduled player commands; replacement emits `ItemUnequipped` before
`ItemEquipped`, accepted commands advance time and replay evidence, and rejected commands preserve
world, snapshot, and digest state. Protocol/MCP expose the typed field and events, while Bevy
`SceneActor` mirrors the field without owning item storage. Effects, modifiers, and additional slots
remain outside this boundary; core-owned capacity enforcement is projected through protocol.

The verified single-item consumption slice adds scheduled `UseItem` for one owned, unequipped item.
Core removes only that inventory instance, applies an authored healing effect capped at the actor's
maximum hit points, advances standard action time, and emits `ItemConsumed` with optional typed
healing evidence; protocol and MCP preserve the typed action/evidence, while Bevy removes the stale
inventory mirror and retains the actor and remaining item entities. Richer effects, identification,
and rendering policy remain outside this boundary.

The verified scheduled item-pickup slice adds `Pickup` for the controlled actor's current ground
stack. Core discovers item identities in stable stack order, moves only the requested identity into
the ordered inventory, advances the standard action, emits `ItemPickedUp`, and records replay
evidence. Protocol/MCP convert the request, event, and typed ground-miss error; the desktop binds
`P` to the lowest-ID available ground item and covers the transition in its journal and display-free
smoke path. The verified player-drop follow-up moves one owned unequipped item back to the current
ground stack with the same standard timing; richer item effects, enemy pickup, and new media policy
remain outside this boundary.

The verified Milestone 4 ranged-combat slice adds `RangedAttack` as a second player combat command.
Core discovers stable target IDs at Manhattan distance 2–3, reuses the existing typed `Attacked`
and `Died` evidence, and records the command in replay. Protocol/MCP and the headless CLI only
translate the new typed request; the desktop binds `G` to the lowest-ID legal target and reuses the
existing attack animation/audio cue families. A follow-up slice now adds a cardinal line-of-sight
predicate over interior cells, where the terrain predicate decides whether `Cover` or `Wall` blocks;
blocked and diagonal requests return typed rejection without mutating state or replay evidence.
Weapon effects and enemy ranged behavior remain future rules.

The verified ranged-cost slice gives only `RangedAttack` a two-tick scheduler advance; all other
player and enemy commands retain the standard one-tick cost. The cost is selected in core execution
and guarded during legal discovery so adapters cannot advertise an overflowing ranged action.

The verified ranged-ammunition slice gives each actor a finite default of three ranged shots. Core
decrements ammunition only after an accepted ranged attack, omits empty actions from legal discovery,
and returns typed no-ammunition rejection without mutating scheduler or replay evidence. The verified
reload slice adds a scheduled player-only `Reload` command that restores the same fixed capacity,
uses the standard action cost, and emits typed event evidence; full-ammo rejection remains atomic.
Protocol v9 carries the reload command/event/error additions. The verified player-drop slice is the
version-10 follow-up: it moves one owned unequipped item into the current ground stack with standard
action timing and typed replay evidence; ammo pickups, item-derived ammunition, and weapon capacities
remain future rules.

The verified cover slice adds a walkable `Cover` tile that blocks interior ranged rays while
retaining the existing typed no-line-of-sight rejection. Presentation FOV continues to traverse
walkable cover and treats only walls as visible boundaries. Protocol v7 carries the tester terrain
variant; cover damage modifiers, directionality, destruction, and environmental mutation remain
future rules.

The verified enemy-intent presentation slice reads core's current legal command projection for the
scheduled living enemy and exposes the selected exact command as a disposable Bevy resource. The
follow-up enemy-melee slice makes adjacent `Attack` legal before fallback `Chase`, and the Bevy
intent/desktop drivers share that attack-before-chase policy while retaining core-owned damage,
events, scheduling, and replay truth. Ranged enemy AI and new status behavior remain future rules.

The verified player-defeat preparation keeps death semantics in core while the desktop boundary marks
an accepted `Died { actor: PLAYER }` event as a terminal presentation status and records
`terminal_defeat`. The follow-up canonical outcome projection derives defeat/victory once in core,
projects it through protocol v11 and Bevy snapshots, and lets the desktop consume that value.
Normal input stops after defeat, while Escape and same-seed restart remain available; persistence,
respawn, and replay playback remain future behavior.

The verified melee-reach preparation slice adds a typed actor reach value with a one-tile default and
an explicit extended-reach constructor for authored/test scenarios. Core uses the same Manhattan
reach predicate for `Attack` legal discovery and execution; protocol snapshots and Bevy scene
mirrors carry the value, while weapon classes, equipment-derived effects, and new presentation
controls remain future rules.

The typed MCP player-action slice extends that same process boundary with JSON command requests and
structured `SessionOutput` event/snapshot evidence. MCP maps invalid command results to protocol
errors only; core still owns scheduling, target validation, semantic events, and replay recording.
Tester mutations remain outside the process wire contract.

The legal-action MCP slice exposes `Session::legal_actions` as a no-argument, read-only tool. Core
selects the scheduled actor and deterministic command order; the MCP adapter only serializes the
typed protocol request array. A legal-action call cannot mutate world, history, or replay state.

The actor-inspection MCP slice exposes `Session::inspect` through a typed actor-ID parameter. The
session performs the existing snapshot lookup and returns an `ActorSnapshot` or `None`; MCP only
serializes that visible projection and does not invent hidden-information or visibility rules.

The accepted-history MCP slice exposes `Session::get_history` as a no-argument, read-only array of
protocol requests. The session remains the adapter-owned view over core replay recording; MCP does
not expose `ReplayTrace` internals or add a second history source.

The replay-evidence MCP slice exposes `Session::get_replay` as a no-argument, read-only structured
`ReplayEvidence` value. Protocol owns its JSON/JSON Schema projection; the MCP adapter does not add
persistence or playback semantics and core remains authoritative for the digest.

## Constraints

- Core owns canonical semantic commands, events, and domain errors.
- Protocol owns versioning and external representation, not domain semantics.
- Rendering, ECS scheduling, wall-clock time, host randomness, and transport state cannot
  determine authoritative game outcomes.
- `unsafe` code is forbidden at the workspace level. A future exception requires an ADR,
  evidence, and explicit review before changing that policy.
- Public concepts should use typed domain representations rather than strings, boolean
  modes, or unvalidated map-shaped data.
- See `docs/adr/0001-functional-core-and-adapters.md` for the decision rationale.
