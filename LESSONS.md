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
  `std` until a presentation milestone needs a reviewed feature set. A representative
  Bevy 0.19 package with this configuration passed Clippy on the same environment.
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
