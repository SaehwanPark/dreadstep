# Dreadstep Architecture

Last Reviewed: 2026-08-09
Status: Verified

## Overview

Dreadstep is organized as a functional domain kernel surrounded by explicit adapters. The
kernel decides game outcomes; adapters translate external input into semantic commands and
translate semantic events into presentation, files, telemetry, or transport responses.

Milestone 1 now exposes the first gameplay API in `dreadstep-core`: a typed rectangular map,
actors, movement, melee, and chase commands, semantic movement/blocking/combat/death events,
an integer ready-time scheduler, and core-owned replay traces/state digests. The
`dreadstep-headless` adapter now provides a fixed-scenario developer CLI that translates text
arguments into those core commands; it owns parsing and stdout only. `dreadstep-mcp` also provides
a minimal local stdio server for the bounded player tools `start_run`, `observe`, `legal_actions`,
`inspect`, `get_history`, `get_replay`, and typed `act`; no graphical client exists yet.

## Package Ownership

| Package | Owns | Must not own |
| --- | --- | --- |
| `dreadstep-core` | Domain state, commands, events, errors, deterministic rules | I/O, Bevy, MCP, authored-file formats |
| `dreadstep-protocol` | Versioned external representations and conversions | Domain decisions or transports |
| `dreadstep-content` | Validation of authored definitions into domain values | Hidden simulation rules |
| `dreadstep-headless` | CLI, files, processes, telemetry, batch execution | Authoritative game behavior |
| `dreadstep-mcp` | Bounded player and tester operations | Arbitrary host access or game truth |
| `dreadstep-bevy` | Input and presentation | Authoritative state or rules |

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
and content must never depend on Bevy or MCP runtime libraries. Bevy currently enables only its
`std` and `keyboard` features, so headless Linux checks do not require desktop system libraries.

## Intended Data Flow

```text
external input -> adapter -> core command -> deterministic transition
                                           -> next state + semantic events
semantic events -> adapter -> output, presentation, telemetry, or protocol response
```

State, configuration, seeded randomness, and time inputs should be explicit. Prefer pure
transformations and returned outcomes; allow tightly scoped mutation when it is clearer or
materially more efficient in Rust.

## Current kernel slice

`dreadstep-core` owns the canonical `WorldState`. `GridMap` limits dimensions to the signed
`Position` coordinate domain and treats out-of-bounds and wall tiles as terrain blockers;
living actor occupancy is checked separately so events can distinguish terrain from another
actor. `WorldState::execute` accepts only the living actor at the minimum `ActionTime`, orders
ties by `ActorId`, applies fixed melee damage to adjacent targets, resolves enemy chase steps
with horizontal-axis priority, and advances the acting actor by the fixed action cost. Dead
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
protocol-owned `ReplayEvidence` value. It remains an in-memory read-only view; persistence,
serialization, playback, and transport registration stay outside this slice.

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
second item store or apply effects. Capacity and richer item semantics remain deferred. The tester
transfer extension delegates source ownership, ordering, dead-record validity, and atomic errors
to core; MCP only projects the result and does not record player history or replay evidence.

The item-catalog foundation keeps definition membership on the content side: it validates ordered,
opaque `ItemDefinitionId` references and exposes read-only lookup without changing core world state.
The catalog is authoring support rather than a second item store; core remains authoritative for
item instances, ownership, digests, and snapshots, and gameplay semantics remain deferred.

The tester item-drop extension keeps ground-item records in core, keyed by stable map position with
deterministic stack order. Protocol projects those records as read-only snapshot values and MCP
delegates the mutation; neither boundary adds a player-facing pickup command, effects, capacity, or
replay/history entries. An equipped item is rejected before this tester mutation can invalidate the
actor's optional equipment reference.

The tester item-pickup extension moves an item from the actor's current core-owned ground stack back
into that actor's ordered inventory. Protocol/MCP only convert the typed ground-miss error and
project the existing version-3 snapshot; pickup remains outside player commands, replay/history, and
gameplay effects.

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
They record local-only generated and CC0 candidates, exact nearest-neighbor 24×24/32×32 samples,
and a provisional 32×32 working scale; dungeon audio sourcing remains open after a UI-only fallback
evaluation. The verified reversible renderer boundary consumes this metadata without loading
production assets or enabling render plugins. `PresentationRenderProjection` is a read-only ordered
resource over the existing keyed mirrors: it carries complete typed values and per-kind roles,
derives map-backed pixel positions from each mirror's own typed position when tile-size configuration
is present, keeps inventory items unplaced, and preserves retained metadata when configuration is
absent. It does not become another source of simulation truth. Actual windowing, rendering, asset
loading, and playback remain deferred to later presentation slices.

The verified sprite-key slice derives a closed `SceneSpriteKey` from each complete
`SceneRenderEntry` and exposes it through `PresentationSpriteProjection`. Terrain retains its typed
tile, actors retain player/enemy/dead roles, and item keys retain opaque definition identity; the
projection preserves retained entities and placement metadata without loading assets or becoming
another source of simulation truth. Actual texture loading, render plugins, transforms, and media
remain deferred.

The active render-command-plan slice derives `PresentationRenderCommandPlan` from the verified
sprite projection. Each `SceneRenderCommand` retains its complete typed entry, ECS identity, sprite
key, optional placement, deterministic layer, and source order. This is a read-only draw plan for a
future renderer; it does not load assets, create transforms or windows, enable render plugins, or
become simulation authority.

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
`SceneActor` mirrors the field without owning item storage. Effects, modifiers, capacity, and
additional slots remain outside this boundary.

The verified single-item consumption preparation slice adds scheduled `UseItem` for one owned,
unequipped item. Core removes only that inventory instance, advances the standard action time, and
emits `ItemConsumed`; protocol and MCP preserve the typed action/evidence, while Bevy removes the
stale inventory mirror and retains the actor and remaining item entities. No effect, stat,
capacity, identification, or rendering policy is inferred here.

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
