# Dreadstep

![Concept Art](./dreadstep-concept-art.png)

_Concept art only—not a screenshot of the current game. It illustrates an aspirational direction and is subject to change._

> Every step is a decision.

Dreadstep is a fast, gothic tactical roguelike about deliberate movement, dangerous dungeon
descent, distinctive loot, and a compact vocabulary of systems that combine in surprising
but understandable ways.

The game is also being built as a deterministic simulation first. Rust tests, headless
tools, MCP agents, and the future Bevy client will issue the same semantic commands and
observe the same events. The engineering exists to make the game better; players should
not need to care about the testing architecture to enjoy it.

## Current Status

Dreadstep is continuing Milestone 3: the human presentation boundary.

- Verified simulation and agent foundations: deterministic core rules, replay evidence, headless
  CLI, protocol/MCP observation and actions, tester operations, and stdio tools.
- Verified headless Bevy bridge: shared authored floors, runtime/app projection, keyboard dispatch,
  feedback evidence, focus, scene focus, camera, viewport, tile/actor, ground-item, and
  inventory-item projections, typed `PresentationHud` actor status, and ordered typed
  `PresentationMessages` for every current core event, plus typed `PresentationAudioCues`
  placeholders preserving event order without loading assets or enabling playback, plus typed
  `SceneSpriteRole` metadata classifying terrain, living actors, dead records, and item mirrors
  without textures, assets, or rendering plugins.
- Verified animation boundary: typed `PresentationAnimationCues` preserves movement and combat
  event order without timers, interpolation, textures, or rendering plugins.
- Verified item authoring boundaries: opaque item ownership, catalog binding, authored starter
  placements, deterministic starter-item content, tester transfer/drop/pickup, and Bevy item-run
  startup. The default `start_run` remains item-free.
- Verified deterministic single-slot equipment: scheduled typed equip/unequip commands, ordered
  replacement events, digest/replay/snapshot evidence, tester guards against moving equipped items,
  protocol/MCP projections, and a typed Bevy `SceneActor` field. Item effects, modifiers, capacity,
  and additional slots remain deferred.
- Verified deterministic single-item consumption: scheduled `UseItem` removes one owned,
  unequipped instance, advances one standard action, and emits typed core/protocol/MCP/Bevy
  evidence with atomic ownership/equipment guards; effects, stat changes, capacity, and richer
  item semantics remain outside this preparation slice.
- Verified window boundary: validated typed logical dimensions and integer pixel scale for a future
  desktop client without creating an OS window or enabling desktop features.
- Verified scene placement boundary: caller-selected logical tile extents project checked pixel
  origins onto terrain, actor, and ground-item mirrors while inventory items remain unplaced; the
  native sample evidence supports a 32×32 working size and no rendering is enabled.
- Verified asset evaluation: local candidates, provenance, and the local-only media policy are
  recorded in [`asset-evaluation.md`](docs/presentation/asset-evaluation.md). The Kenney CC0 pack
  is a reusable fallback and UI audio is only a fallback; dungeon combat/movement/item audio remains
  open.
- Verified tile-size evidence: exact 24×24/32×32 nearest-neighbor samples and the provisional 32×32
  working-size decision are recorded in [`tile-samples.md`](docs/presentation/tile-samples.md).
- Verified render-boundary projection: ordered typed `PresentationRenderProjection` entries preserve
  complete terrain, actor, ground-item, and inventory mirrors, deterministic retained entities,
  per-kind 32×32 placement, and inventory-unplaced semantics without render plugins, textures,
  asset loading, or playback.
- Verified typed sprite-key projection: `SceneSpriteKey` and `PresentationSpriteProjection` map
  complete render entries to stable terrain/actor/item selectors while retaining entities, roles, and
  placement metadata; texture loading, render plugins, transforms, and media remain deferred.
- Verified deterministic render-command plan: `PresentationRenderCommandPlan` derives typed layer,
  source-order, and optional pixel-placement metadata from sprite entries for a future renderer;
  texture loading, render plugins, transforms, windows, and media remain deferred.
- Verified placeholder render-node bootstrap: `PresentationRenderNodeProjection` reconciles stable
  ECS nodes from typed commands with deterministic placeholder families; actual Sprite components,
  render plugins, windows, texture loading, animation, audio, and media remain deferred.
- Verified local-only asset manifest: `PresentationAssetManifest` validates one anchored reference
  per placeholder family and `PresentationRenderAssetProjection` joins those references to stable
  node metadata without loading files or creating asset handles; pixel-art/audio binaries remain
  ignored local files with provenance kept in tracked documents.
- Active local-only audio cue manifest: `PresentationAudioAssetManifest` binds all eight typed cue
  families to validated `audio/` references and `PresentationAudioAssetProjection` preserves cue
  payload/order without loading files, creating handles, enabling playback, or adding an audio
  backend.
- Still deferred: windowing, rendering assets, sprites, animation, HUD widgets, event/combat
  message presentation, audio assets/playback, fog of war, multiple floors, and richer gameplay
  item semantics such as effects, modifiers, capacity, and additional slots.

The long-term design and roadmap are in
[`docs/dreadstep-proposal.md`](docs/dreadstep-proposal.md). Verified current and planned
work lives in [`SPEC.md`](SPEC.md).

## Local Presentation Assets

Pixel-art and audio binaries are intentionally excluded from GitHub. Keep local copies under
the root or crate-local `assets/`, `art/`, or `audio/` directories; the repository ignore rules
keep everything in those media directories available in a working tree without allowing accidental
commits. Store or sync the files manually through the project’s external asset service. Keep
source, creator, license, attribution, and modification records outside those directories in
tracked documentation such as
[`CREDITS.md`](CREDITS.md) and the proposal; do not commit service credentials.
The tracked concept-art reference and future README screenshots under root `screenshots/` are
explicit exceptions because they are outside the local-media directories.

## Design Principles

- Every movement choice should matter without making routine turns laborious.
- Prefer a small number of strongly differentiated items, enemies, and interactions.
- Compose explicit rules instead of accumulating special cases.
- Make outcomes legible so players can learn from surprise rather than confusion.
- Keep the simulation authoritative and presentation replaceable.
- Use agents for deterministic exploration and regression discovery, not as a substitute
  for human judgment about clarity, pacing, atmosphere, and fun.

## Architecture

```text
protocol ----> core <---- content
                  ^
                  |
       +----------+----------+
       |          |          |
    headless     MCP       Bevy
```

`dreadstep-core` owns semantic game truth. The other packages translate content or external
effects around that kernel. See [`ARCHITECTURE.md`](ARCHITECTURE.md) for ownership,
dependency direction, and invariants.

## Getting Started

Install [Rustup](https://rustup.rs/), then clone the repository. The committed toolchain
file installs the pinned Rust compiler, Rustfmt, and Clippy.

On Apple Silicon macOS, first install Xcode command-line tools:

```sh
xcode-select --install
```

On Linux and WSL2, Milestone 0 verification is headless and does not require Wayland, X11,
or audio development packages. Windows contributors should use the MSVC Rust toolchain and
Windows build tools.

Run the complete local verification suite:

```sh
scripts/verify.sh
```

Run the developer scenario directly after building the headless package:

```sh
cargo run -p dreadstep-headless -- --seed 7 --commands 'move:1:east,wait:2'
```

The first build downloads and checks Bevy's minimal dependency set and can take longer than
later runs.

## Repository Guide

- `crates/`: domain, content, protocol, and adapter packages.
- `SPEC.md`: completed, active, and planned capabilities with verification criteria.
- `ARCHITECTURE.md`: current package ownership, data flow, and constraints.
- `LESSONS.md`: verified recurring traps and their prevention.
- `CONTRIBUTING.md`: setup, spec-first TDD, style, review, and handoff guidance.
- `.agents/skills/`: reusable Dreadstep development and review workflows.

Terms used in the project:

- **Functional core:** deterministic domain transformations separated from external effects.
- **Adapter:** code that translates external input or output around the domain kernel.
- **MCP:** Model Context Protocol, planned here as a bounded interface for player and tester
  agents.
- **Semantic event:** a game-meaningful outcome such as movement or damage, independent of
  animation or transport formatting.

## Contributing

Contributors from different technical and creative backgrounds are welcome. Begin with
[`CONTRIBUTING.md`](CONTRIBUTING.md), which explains the development workflow without
assuming prior knowledge of Rust, Bevy, roguelikes, or agent tooling.

## License

Dreadstep source code is available under the [MIT License](LICENSE).
