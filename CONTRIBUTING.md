# Contributing to Dreadstep

Thank you for helping build Dreadstep. Contributors may arrive from Rust, game design,
testing, art, open-source, or agent-tooling backgrounds. Ask when a project term is unclear;
shared understanding matters more than familiarity with a particular vocabulary.

## Start Here

Read these sources in order:

1. `README.md` for the project and current status.
2. `SPEC.md` for verified, active, and planned work.
3. `ARCHITECTURE.md` and applicable ADRs for ownership and dependency constraints.
4. `LESSONS.md` for recurring traps already encountered.
5. The tests and code nearest to the proposed change.

The proposal in `docs/dreadstep-proposal.md` provides long-term product context. It does
not override current operational state in the specification and architecture documents.

## Development Setup

Install Rust through `rustup`. The committed toolchain file selects Rust 1.97.1 with
Rustfmt and Clippy automatically.

- Apple Silicon macOS: install Xcode command-line tools with `xcode-select --install`.
- Linux or WSL2: the Milestone 0 headless workflow needs no Bevy desktop system packages.
- Windows: use a Rustup installation with the MSVC host toolchain and Windows build tools.

Verify the repository:

```sh
scripts/verify.sh
```

If a check fails, read the first project-owned error before changing code. Do not install
desktop packages merely to hide an adapter-boundary failure.

## Spec-First TDD

For behavior changes:

1. State observable intent, verification, and exclusions in `SPEC.md`.
2. Add a focused test and confirm it fails for the intended missing behavior.
3. Implement the smallest correct typed change.
4. Refactor while tests remain green.
5. Run verification and review both sides of every changed boundary.
6. Reconcile docs and revisit `LESSONS.md` before handoff.

Do not create artificial tests for empty scaffolding. Tests should specify behavior or a
real structural invariant.

## Presentation Assets

Pixel-art and audio binaries are local-only project inputs. Put them in `assets/`, `art/`, or
`audio/` at the repository root or in a crate-local directory; `.gitignore` excludes everything
under those media paths so contributors can keep and use any format without publishing it to
GitHub. Synchronize the binaries manually through the project’s external asset service. Keep source,
creator, license, attribution, and modification records outside ignored media paths in tracked
documentation, and never commit credentials. The tracked concept-art reference and future
screenshots under root `screenshots/` are explicit exceptions because they are outside the ignored
media paths.

## Code and Documentation Style

- Use spaces only. Indentation and tab display width are both 2 throughout human-authored
  files. Generated and vendored files are exempt and must not be hand-edited.
- Prefer immutable values, explicit state, pure domain transformations, and effects at
  adapter boundaries. Local mutation is welcome when it is clearer or measurably cheaper.
- Use structs, newtypes, enums, `Option`, and typed `Result` errors to express the domain.
- Choose names that describe game meaning. Keep functions and modules cohesive, and add
  abstractions only for a present, stable concept.
- Use `//!` for crate or module contracts and `///` for public item contracts. Comment why
  a rule or constraint exists; do not narrate syntax, preserve dead code, or record history
  that belongs in an ADR or version control.
- Define uncommon game, Rust, or agent terms on first use. Helpful documentation should be
  accurate for success and failure paths without assuming a contributor's background.

## Reviews and Handoffs

Cross-package changes, public contracts, determinism, randomness, replay, serialization,
MCP trust boundaries, dependencies, unsafe-code requests, and toolchain changes require the
Dreadstep review workflow. See `docs/harness/dreadstep/team-spec.md`.

A handoff reports files changed, checks run, deviations, unresolved concerns, and whether
`LESSONS.md` was consulted or updated. Commits, pushes, releases, and external messages are
separate actions and require explicit authorization.
