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

Dreadstep is continuing the transition from Milestone 3's human presentation boundary through
Milestone 4 tactical combat and Milestone 5 environmental state.

The scheduled ranged attack, clear-cardinal line-of-sight, distinct ranged action-cost, finite
ammunition, walkable cover, deterministic scheduled-enemy intent presentation, typed melee reach,
player-facing reload, item pickup/drop, healing and ammunition consumables, deterministic adjacent
door interaction, one-shot floor traps, deterministic breakable terrain, and canonical run-outcome projection are verified across
the deterministic core, protocol/MCP/headless adapters, and desktop boundary. Reload restores the fixed
three-shot capacity with `R` when ammo is partial; `I` opens a legal adjacent closed door; `X` drops the selected unequipped item and
`Shift+R` restarts the same seed. Adjacent scheduled enemies use the existing fixed melee attack
before falling back to chase; core and protocol snapshots now expose deterministic `in_progress`,
`defeat`, or `victory` outcomes, and the desktop showcase consumes that projection while keeping
restart available. Ranged enemy AI, ammo pickups, cover damage modifiers, weapon effects beyond the
first healing and ammunition consumables, persistence, save/load, and replay playback remain deliberately deferred. Each desktop smoke or
visible run now also writes a versioned `*.replay.json` evidence artifact beside its JSONL journal;
the artifact is not a playback-compatible save file.

- Verified foundations
  - Deterministic core rules, replay evidence, the headless CLI, protocol/MCP observation and
    actions, tester operations, and stdio tools.
  - Opaque item ownership, catalog binding, authored starter placements, deterministic starter-item
    content, tester transfer/drop/pickup, and Bevy item-run startup; default `start_run` remains
    item-free.
  - Scheduled single-slot equipment and single-item consumption contracts with digest/replay/
    snapshot evidence, protocol/MCP projections, atomic tester/player guards, typed Bevy scene and
    cue projections, a capped three-point authored healing effect, a capped two-round ammunition
    effect, and optional event evidence.
  - Scheduled player-facing item pickup and drop with deterministic ground-stack ordering, typed
    events/errors, protocol/MCP replay evidence, and desktop `P`/`X` smoke coverage; damage/status
    effects, modifiers, and richer item semantics remain deferred.
  - Deterministic adjacent closed-door interaction with typed `Door`, `Interact`, and `DoorOpened`
    values, atomic rejection, protocol v17/MCP/headless/Bevy mappings, desktop `I` control, and
    display-free smoke/journal coverage; lock/key systems, closing doors, and procedural floors
    remain deferred.
  - Deterministic one-shot floor traps with walkable/non-blocking `Trap` terrain, shared `Move`/
    `Chase` trigger semantics, ordered `Moved`/`TrapTriggered`/`Died` evidence, atomic state
    mutation, protocol v17/MCP/Bevy mappings, and smoke/journal coverage; discovery, disarming,
    rearming, archetypes, and procedural placement remain deferred.
  - Deterministic adjacent breakable terrain with blocking `Breakable` tiles, scheduled `Break`,
    typed `BreakableBroken` evidence, atomic rejection, protocol v17/MCP/headless/Bevy mappings,
    desktop `B` control, and smoke/journal coverage; damage/tool stats, durability, procedural
    placement, and noise propagation remain deferred.
  - Deterministic desktop replay-evidence export with seed, accepted command order, replay digest,
    and canonical outcome; persistence, editing, and playback remain deferred.
  - Fixed four-item inventory capacity with atomic full-inventory rejection across core, protocol
    v17 snapshots, MCP, headless command coverage, and Bevy legal-action projections; richer effects and
    upgrades remain deferred.
- Verified headless Bevy boundary
  - Shared authored floors, runtime/app projection, deterministic keyboard dispatch, feedback,
    focus, scene focus, camera, viewport, tile/actor/ground/inventory mirrors, typed HUD status,
    ordered messages, audio cues, animation cues, and sprite-role metadata.
  - Caller-selected checked pixel placement, native 24×24/32×32 evidence with a provisional 32×32
    working size, and complete ordered render/sprite-key/command/node projections.
  - Optional deterministic presentation field of view: radius-3 cardinal walkable-terrain
    traversal with readable wall boundaries, retained-but-hidden out-of-view render nodes, and
    complete headless snapshots/MCP visibility unchanged.
  - Local-only pixel-art and audio manifests preserve typed placeholder/cue metadata; the desktop
    feature can optionally request existing local audio through Bevy playback while missing media
    remains a safe fallback and binaries remain ignored.
- Verified Sprite API boundary
  - `PresentationBevySpriteProjection` derives deterministic solid-color `Sprite` values with
    optional 32×32 sizing from stable placeholder nodes while preserving inventory-unplaced and
    authority-guard semantics; no Sprite/render plugin or production image is loaded.
- Verified ECS Sprite-node attachment
  - The current slice attaches those typed `Sprite` values to retained render-node ECS entities,
    preserving node identity and default required components without enabling a render plugin,
    transform placement, texture loading, or production media.
- Verified typed Sprite-transform projection
  - The headless boundary derives ordered map-space translations from checked pixel origins while
    keeping inventory unplaced and ECS transforms unchanged; fresh missing tile size starts
    unplaced while later removal preserves checked translations. Camera, visibility, window,
    renderer, and production media remain deferred.
- Verified ECS Sprite-transform attachment
  - The presentation boundary attaches deterministic centered logical-pixel `Transform` values to
    retained map-node entities from checked origins plus caller-selected tile half-extents; inventory
    remains unplaced and layer depth is preserved.
- Verified deterministic ECS Sprite depth
  - The presentation boundary derives terrain/ground/actor z-layer values from the existing typed
    render layer while preserving centered x/y placement and inventory default state.
- Verified headless ECS Camera2d attachment
  - The presentation boundary attaches Bevy's typed `Camera2d` marker and required default
    orthographic camera components to the retained disposable camera projection entity while
    retaining runtime/PresentationCamera authority and leaving window creation, camera viewport
    policy, render plugins, visibility, production assets/audio, and media deferred.
- Verified headless ECS Window configuration attachment
  - The presentation boundary mirrors validated logical/physical dimensions plus the exact integer
    scale onto a disposable `SceneWindow`, and exposes a deterministic `f32` scale adapter on
    Bevy's `WindowResolution`; OS/window plugins, render backends, camera policy, visibility,
    production assets/audio, and media remain deferred.
- Verified headless ECS camera transform attachment
  - The presentation boundary attaches checked centered logical-pixel `Transform` values to the
    retained disposable `SceneCamera` when a tile size is present; viewport policy, OS/window
    integration, render backends, visibility, production assets/audio, and media remain deferred.
- Verified runnable desktop showcase
  - `cargo run -p dreadstep-bevy --features desktop --bin dreadstep --` opens one non-resizable
    640×360 logical (2× physical) Bevy window with nearest-neighbor placeholders, camera, HUD,
    player controls, deterministic enemy chase turns, combat/death, inventory actions, restart,
    and a presentation-only completion status.
  - `--smoke` runs the same action-selection, enemy-driver, and JSONL journal path without a
    display; malformed CLI, startup, asset, input, journal, and caught-panic failures are
    recoverable at the process boundary.
- Verified tactical HUD polish
  - The existing panel now renders a fixed-width health bar, turn/position, remaining-enemy
    pressure, and radius-3 field-of-view state without changing simulation or smoke behavior.
- Verified animation polish
  - New typed animation-cue batches trigger a short pulse on visible living actor placeholders, with
    the runtime replay digest preserving retriggers for distinct accepted events with identical cue
    values; movement interpolation, production sheets, production audio, and simulation timing remain
    deferred.
- Verified local CC0 art adoption preparation
  - The recorded Kenney Tiny Dungeon archive can be checked and installed into the ignored
    `assets/dreadstep/` showcase paths with `scripts/prepare-local-assets.sh`; clean checkouts retain
    deterministic placeholders, and final palette/art direction remain open.
- Verified audio placeholder playback
  - Distinct typed audio-cue batches now route through the validated eight-family manifest and request
    existing `assets/`-rooted files as non-looping Bevy effects; root/crate-local metadata, absent
    audio, audio resources, and audio devices remain safe fallbacks. Production sound design, music,
    and mastering remain deferred.
- Still deferred
  - production texture/media adoption, anchor policy beyond centering, animation playback,
    production audio assets/mastering/music, multiple floors, and richer gameplay item semantics such as
    effects, modifiers, and additional slots.

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

To prepare the selected CC0 local art fallback from the recorded archive:

```sh
scripts/prepare-local-assets.sh --check
scripts/prepare-local-assets.sh --install
```

The script validates the archive hash and source members before writing six ignored PNGs. It is
optional; the desktop client keeps its readable per-family placeholders when local media is absent.

### Moving a local desktop setup to another Mac

Git intentionally does not carry presentation binaries. To reproduce this environment on a new
macOS/Apple Silicon checkout, transfer the following ignored files (or transfer the source archive
and regenerate the derived files):

| Purpose | Transfer | Notes |
| --- | --- | --- |
| Selected local art source | `art/kenney-tiny-dungeon.zip` | Preferred; verify its recorded SHA-256 with `scripts/prepare-local-assets.sh --check`, then run `--install` to create the six files below. |
| Desktop art fallback | `assets/dreadstep/terrain.png`, `player.png`, `enemy.png`, `dead.png`, `ground-item.png`, `inventory-item.png` | Transfer these six generated files only when the archive is unavailable; keep the same relative paths. |
| Optional desktop audio | `assets/audio/dreadstep/moved.ogg`, `movement-blocked.ogg`, `waited.ogg`, `attacked.ogg`, `died.ogg`, `item-equipped.ogg`, `item-unequipped.ogg`, `item-consumed.ogg` | These are the only audio paths the Bevy desktop loader requests. Missing files are safe fallbacks, but transferring them preserves local playback. |

The evaluation-only files under `art/` and `audio/` (such as generated previews, `audio/generated-cue-click.wav`,
and `audio/kenney-ui-audio.zip`) are not required to run the game. Do not transfer `target/` or
`_workspace/`; those are build and agent-work files and are recreated locally. Crate-local media may
be retained for experiments, but the current desktop playback contract is rooted at `assets/`.

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

On Linux and WSL2, core/headless checks remain display-free; the full showcase gate uses the
reviewed X11/XWayland Bevy feature path and requires `pkg-config` plus ALSA development headers
(for example, `sudo apt-get install pkg-config libasound2-dev`). Windows contributors should use
the MSVC Rust toolchain and Windows build tools.

Run the complete local verification suite:

```sh
scripts/verify.sh
```

Run the developer scenario directly after building the headless package:

```sh
cargo run -p dreadstep-headless -- --seed 7 --commands 'move:1:east,wait:2'
```

Run the human-testable 2D showcase (optional local images are documented in
[`docs/demo.md`](docs/demo.md)):

```sh
cargo run -p dreadstep-bevy --features desktop --bin dreadstep -- --seed 7
```

Use `--smoke` for the display-free deterministic sequence and inspect the flushed JSONL file in
`dreadstep-logs/` or the supplied `--log-dir`.

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
