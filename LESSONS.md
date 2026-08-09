# Lessons Learned

Read this file before implementation and again before final review. Record only verified,
recurring traps that are not already obvious from code, tests, or canonical documentation.
Update an existing lesson instead of adding a duplicate.

## Keep desktop engine features at the presentation boundary

- Context: The initial root package depended on Bevy 0.19 with all default features.
- Symptom: `cargo clippy` on a headless Linux/WSL2 environment failed while building
  `wayland-sys` because the `wayland-client` system package was unavailable.
- Cause: Bevy's default platform features pulled windowing, Wayland, X11, input, and audio
  dependencies into every repository check before any project code was analyzed.
- Resolution: Move Bevy into `dreadstep-bevy`, disable default features, and enable only
  `std` until a presentation milestone needs a reviewed feature set. Later headless Sprite API
  work enabled only Bevy's `bevy_sprite` feature; its image/mesh/camera support remains usable
  without adding render plugins or desktop backends. A representative Bevy 0.19 package with this
  configuration passed Clippy on the same environment.
- Prevention: Keep engine dependencies out of core, protocol, and content; inspect enabled
  features before adding presentation capabilities; verify the headless Linux workflow.

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

- Context: The single-item consumption slice added `UseItem` to core legal-action discovery while
  preserving deterministic inventory order.
- Symptom: Focused new tests passed, but Linux CI failed in an older equipment ordering test whose
  exact expected vector still ended at `Equip`.
- Cause: Adding a public command changes every deterministic discovery projection, not only the
  new command's own tests; the stale assertion was outside the edited slice.
- Resolution: Add `UseItem` immediately after each eligible `Equip`, update the prior exact-order
  expectation, and rerun the full workspace verification.
- Prevention: Whenever a command variant changes legal discovery, search all workspace tests and
  adapters for exact command vectors, update those expectations deliberately, and require the full
  cross-platform verification before handoff.
