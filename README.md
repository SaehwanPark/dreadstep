# Dreadstep

> Every step is a decision.

Dreadstep is a fast, gothic tactical roguelike about deliberate movement, dangerous dungeon
descent, distinctive loot, and a compact vocabulary of systems that combine in surprising
but understandable ways.

The game is also being built as a deterministic simulation first. Rust tests, headless
tools, MCP agents, and the future Bevy client will issue the same semantic commands and
observe the same events. The engineering exists to make the game better; players should
not need to care about the testing architecture to enjoy it.

## Current Status

Dreadstep is beginning Milestone 2: the agent interface. The active slice adds a pure in-memory
MCP player session over versioned snapshots and typed action requests; the deterministic
headless CLI remains available, but there is still no MCP transport runtime, graphical client,
interactive input, or authored scenario system.

The long-term design and roadmap are in
[`docs/dreadstep-proposal.md`](docs/dreadstep-proposal.md). Verified current and planned
work lives in [`SPEC.md`](SPEC.md).

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
