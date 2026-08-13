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

Workspace version `0.0.0`, protocol **v20**. Core owns deterministic combat, inventory, and
environmental rules; protocol/MCP/headless/Bevy translate those values. The opt-in desktop
showcase journals each run and can start an authored item fixture or a seeded procedural floor.

- Verified: core rules and terrain-aware kick-noise investigation, MCP/headless adapters, Bevy projections, desktop `--smoke`,
  and optional `--procedural` runs. Details: [`SPEC.md`](SPEC.md) Present.
- How to play the showcase: [`docs/demo.md`](docs/demo.md).
- Ownership and invariants: [`ARCHITECTURE.md`](ARCHITECTURE.md).
- Still deferred: production art, richer AI and item systems, core-owned floor history,
  persistence, and playback-compatible saves.

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

To launch the opt-in seeded procedural floor in the visible client, pass `--procedural` and an
authored depth (the default is depth 1):

```sh
cargo run -p dreadstep-bevy --features desktop --bin dreadstep -- \
  --procedural --depth 1 --seed 7
```

The default and `--smoke` paths retain the authored item fixture so inventory and command coverage
remain stable. In an opt-in procedural visible run, press `N` after victory to start the next
deterministic depth with the same seed; `Shift+R` restarts the current depth.

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
