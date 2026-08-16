# Dreadstep Specification

This file tracks verified project state. The broader product vision and roadmap live in
[`docs/dreadstep-proposal.md`](docs/dreadstep-proposal.md). Slice-level history lives in
[`CHANGELOG.md`](CHANGELOG.md). Package ownership lives in [`ARCHITECTURE.md`](ARCHITECTURE.md).

## Past

- The Dreadstep proposal established a deterministic, simulation-first tactical roguelike
  built with Rust, Bevy, and an eventual MCP testing interface.
- The project adopted the MIT license and a Rust 2024 starter package.

Verified slices, newest last. Details remain in `CHANGELOG.md`.

| Completed | Slice |
| --- | --- |
| 2026-08-08 | Milestone 0: Project charter and development harness |
| 2026-08-08 | Milestone 1 slice: deterministic grid movement and scheduling |
| 2026-08-08 | Milestone 1 slice: basic melee combat and death |
| 2026-08-08 | Milestone 1 slice: deterministic enemy chase |
| 2026-08-08 | Milestone 1 slice: deterministic replay evidence |
| 2026-08-08 | Milestone 1 slice: deterministic headless CLI |
| 2026-08-09 | Milestone 2 slice: versioned agent observation |
| 2026-08-09 | Milestone 2 slice: versioned agent action requests |
| 2026-08-09 | Milestone 2 slice: in-memory MCP player session |
| 2026-08-09 | Milestone 2 slice: deterministic legal-action discovery |
| 2026-08-09 | Milestone 2 slice: session history and replay evidence |
| 2026-08-09 | Milestone 2 slice: typed session replay evidence |
| 2026-08-09 | Milestone 2 slice: player actor inspection |
| 2026-08-09 | Milestone 2 slice: named session history accessor |
| 2026-08-09 | Milestone 2 slice: in-memory tester snapshot and restore |
| 2026-08-09 | Milestone 2 slice: tester world inspection accessor |
| 2026-08-09 | Milestone 2 slice: validated tester actor spawning |
| 2026-08-09 | Milestone 2 slice: validated tester hit-point mutation |
| 2026-08-09 | Milestone 2 slice: typed tester scenario replacement |
| 2026-08-09 | Milestone 2 slice: opaque tester item ownership |
| 2026-08-09 | Milestone 2 slice: validated tester teleport |
| 2026-08-09 | Milestone 2 slice: minimal MCP stdio observation |
| 2026-08-08 | Milestone 2 slice: typed MCP player actions |
| 2026-08-08 | Milestone 2 slice: MCP legal-action discovery |
| 2026-08-08 | Milestone 2 slice: MCP actor inspection |
| 2026-08-08 | Milestone 2 slice: MCP accepted history |
| 2026-08-08 | Milestone 2 slice: MCP replay evidence |
| 2026-08-09 | Milestone 3 slice: deterministic Bevy presentation bridge |
| 2026-08-09 | Milestone 3 slice: shared authored starter floor |
| 2026-08-09 | Milestone 3 slice: headless Bevy scene synchronization |
| 2026-08-09 | Milestone 3 slice: headless Bevy application shell |
| 2026-08-09 | Milestone 3 slice: deterministic headless keyboard dispatch |
| 2026-08-09 | Milestone 3 slice: deterministic presentation feedback buffer |
| 2026-08-09 | Milestone 3 slice: typed headless presentation focus projection |
| 2026-08-09 | Milestone 3 slice: deterministic headless scene-focus marker |
| 2026-08-09 | Milestone 3 slice: deterministic headless ground-item scene projection |
| 2026-08-09 | Milestone 3 slice: deterministic headless inventory-item scene projection |
| 2026-08-09 | Milestone 3 slice: deterministic headless camera anchor |
| 2026-08-09 | Milestone 3 slice: deterministic headless viewport projection |
| 2026-08-09 | Milestone 3 slice: deterministic Bevy starter-item run projection |
| 2026-08-09 | Milestone 4 slice: deterministic authored starter-item scenario |
| 2026-08-09 | Milestone 4 slice: deterministic catalog-bound starter item placements |
| 2026-08-09 | Milestone 4 slice: deterministic authored starter-floor item placements |
| 2026-08-09 | Milestone 4 slice: deterministic content item-definition catalog |
| 2026-08-09 | Milestone 4 slice: deterministic tester item transfer |
| 2026-08-09 | Milestone 4 slice: deterministic tester item drop |
| 2026-08-09 | Milestone 4 slice: deterministic tester item pickup |
| 2026-08-09 | Milestone 3 slice: deterministic headless HUD status projection |
| 2026-08-09 | Milestone 3 slice: deterministic headless event-message evidence |
| 2026-08-09 | Milestone 3 slice: deterministic headless audio-cue evidence |
| 2026-08-09 | Milestone 3 slice: deterministic headless sprite-role metadata |
| 2026-08-09 | Milestone 3 slice: deterministic headless animation-cue evidence |
| 2026-08-09 | Milestone 3 slice: deterministic headless window request |
| 2026-08-09 | Milestone 3 slice: deterministic scene pixel placement |
| 2026-08-09 | Milestone 3 slice: presentation asset evaluation |
| 2026-08-09 | Milestone 3 slice: native tile-size evidence |
| 2026-08-09 | Milestone 3 slice: reversible headless-to-renderer spike |
| 2026-08-09 | Milestone 4 preparation slice: deterministic single-slot item equipment |
| 2026-08-09 | Milestone 4 preparation slice: deterministic single-item consumption |
| 2026-08-09 | Milestone 3 slice: typed sprite-key projection |
| 2026-08-09 | Milestone 3 slice: deterministic render-command plan |
| 2026-08-10 | Milestone 3 slice: deterministic placeholder render-node bootstrap |
| 2026-08-10 | Milestone 3 slice: validated local-only presentation asset manifest |
| 2026-08-09 | Milestone 3 slice: validated local-only audio cue asset manifest |
| 2026-08-09 | Milestone 3 slice: headless Bevy Sprite API bridge |
| 2026-08-09 | Milestone 3 slice: stable ECS Sprite-node attachment |
| 2026-08-09 | Milestone 3 slice: typed headless Sprite-transform projection |
| 2026-08-09 | Milestone 3 slice: ECS Sprite-transform attachment |
| 2026-08-09 | Milestone 3 slice: deterministic ECS Sprite depth |
| 2026-08-09 | Milestone 3 slice: centered ECS Sprite transforms |
| 2026-08-09 | Milestone 3 slice: headless ECS Camera2d attachment |
| 2026-08-09 | Milestone 3 slice: headless ECS Window configuration attachment |
| 2026-08-09 | Milestone 3 slice: headless ECS camera transform attachment |
| 2026-08-11 | Runnable 2D showcase and diagnostic journal |
| 2026-08-11 | Milestone 3 slice: deterministic presentation field of view |
| 2026-08-11 | Milestone 3 slice: deterministic desktop tactical HUD |
| 2026-08-11 | Milestone 3 slice: event-driven desktop animation pulse |
| 2026-08-12 | Milestone 3 slice: optional desktop audio-cue playback |
| 2026-08-12 | Milestone 3 slice: reproducible local CC0 art adoption preparation |
| 2026-08-12 | Milestone 4 slice: scheduled player-facing item pickup |
| 2026-08-12 | Milestone 4 slice: deterministic scheduled ranged attack |
| 2026-08-12 | Milestone 4 slice: deterministic ranged line of sight |
| 2026-08-12 | Milestone 5 preparation slice: deterministic kick-open doors with noise evidence |
| 2026-08-12 | Milestone 5 preparation slice: deterministic breakable terrain |
| 2026-08-12 | Milestone 5 preparation slice: deterministic one-shot floor trap |
| 2026-08-12 | Milestone 5 preparation slice: deterministic adjacent door interaction |
| 2026-08-12 | Milestone 6 preparation slice: deterministic ammunition consumable |
| 2026-08-12 | Milestone 6 preparation slice: deterministic healing consumable |
| 2026-08-12 | Milestone 4 slice: deterministic ranged cover terrain |
| 2026-08-12 | Milestone 4 slice: deterministic enemy intent presentation |
| 2026-08-12 | Milestone 4 slice: deterministic melee reach preparation |
| 2026-08-12 | Milestone 4 slice: deterministic ranged reload |
| 2026-08-12 | Milestone 4 slice: scheduled player-facing item drop |
| 2026-08-12 | Milestone 4 preparation slice: deterministic inventory capacity |
| 2026-08-12 | Milestone 4 slice: deterministic enemy melee intent |
| 2026-08-12 | Milestone 7 preparation slice: desktop player-defeat terminal |
| 2026-08-12 | Milestone 7 preparation slice: canonical run outcome projection |
| 2026-08-12 | Milestone 7 preparation slice: deterministic desktop replay export |
| 2026-08-12 | Milestone 4 slice: deterministic ranged ammunition |
| 2026-08-12 | Milestone 4 slice: deterministic ranged action cost |
| 2026-08-12 | Milestone 5 preparation slice: deterministic seeded floor generation |
| 2026-08-12 | Milestone 5 preparation slice: opt-in desktop procedural run |
| 2026-08-13 | Milestone 5 preparation slice: deterministic procedural floor advancement |
| 2026-08-13 | Milestone 5 preparation slice: readable procedural depth status |
| 2026-08-13 | Milestone 5 preparation slice: contextual terminal HUD guidance |
| 2026-08-13 | Milestone 4 slice: deterministic enemy ranged intent |
| 2026-08-13 | Milestone 5 preparation slice: deterministic kick-noise enemy investigation |
| 2026-08-13 | Milestone 6 preparation slice: deterministic equipment-derived melee reach |
| 2026-08-13 | Milestone 5 preparation slice: terrain-aware kick-noise propagation |
| 2026-08-13 | Milestone 4 slice: deterministic Chilled status and authored ChillTrap |
| 2026-08-13 | Milestone 4 slice: deterministic Frost Flask throw and Chilled application |
| 2026-08-13 | Milestone 5 preparation slice: deterministic reclosable doors |
| 2026-08-13 | Milestone 5 preparation slice: deterministic Brute break behavior |
| 2026-08-13 | Milestone 4 slice: deterministic Frostcaster Chilled casting |
| 2026-08-13 | Milestone 4 slice: behavior-named enemy intent HUD |
| 2026-08-13 | Milestone 5 preparation slice: deterministic stationary Blocker behavior |
| 2026-08-14 | NetHack-style terminal showcase as the default tester |
| 2026-08-15 | Milestone 4/5 slice: deterministic Scavenger enemy behavior |
| 2026-08-15 | Enhancing TUI presentation: colored text, symbols, and section spacing |
| 2026-08-16 | Milestone 4 slice: deterministic Zombie (slow pursuer) enemy behavior |
| 2026-08-16 | Milestone 6 slice: deterministic authored melee-damage equipment |
| 2026-08-16 | Milestone 6 slice: deterministic authored damage-reduction equipment |
| 2026-08-16 | Milestone 6 slice: deterministic authored trap-mitigating armor |
| 2026-08-16 | Milestone 6 slice: deterministic inventory item comparison UX |
| 2026-08-16 | Milestone 6 slice: deterministic authored ranged-damage equipment |
| 2026-08-16 | Milestone 6 slice: deterministic independent weapon and armor slots |
| 2026-08-16 | Milestone 6 slice: deterministic item-rarity presentation metadata |

## Present

Workspace version is `0.0.0`. Protocol version is **35**. Simulation truth stays in
`dreadstep-core`; adapters translate only. The default player-facing showcase is the
NetHack-style terminal client in `dreadstep-tui`; controls, frame goldens, and smoke
coverage are documented in [`docs/demo.md`](docs/demo.md). Pixel 2D Bevy playtesting is
deferred until a later visual-enhancement stage.

### Core

- Typed map, actors, scheduling, melee and ranged combat, chase/investigation, inventory, equipment,
  pickup/drop, consumables, equipment effects, doors, reclosable OpenDoor terrain, traps,
  ChillTrap/Chilled, breakables, terrain-aware kick noise, cover, reach, reload,
  ammunition, canonical `RunOutcome`, replay traces, state digests, and authored Pursuer, Kiter,
  Brute, Frostcaster, Blocker, Scavenger, and Zombie enemy behaviors. Authored equipment may add
  closed melee-damage, ranged-damage, and incoming-damage-reduction effects resolved in core
  attack/trap evidence
  and projected through every adapter.
- `legal_commands` and `execute` remain the only semantic mutation path.

### Protocol, MCP, and headless

- Versioned snapshots, commands, events, errors, replay evidence, and scenario replacement.
- MCP stdio tools: `start_run`, `observe`, `legal_actions`, `inspect`, `get_history`,
  `get_replay`, and typed `act`.
- Headless CLI parses text into the same core commands.

### Content

- Authored starter and starter-item floors, item catalog, and a seeded corridor-floor
  generator with reachability checks. The starter-item showcase includes a closed door beside the
  player for the documented open/close controls; the item-free starter remains unchanged.

### Bevy and desktop

- Headless projections (scene, HUD, FOV, intent, audio/animation cues, sprites) plus an
  opt-in desktop showcase with JSONL journal, replay-export evidence, `--smoke`, and
  optional `--procedural` floor selection. This pixel client remains in the workspace but is
  not the default tester gate; `scripts/verify-bevy-desktop.sh` is the later visual-stage
  check.

### Terminal client

- `dreadstep-tui` is the default human and agent-playable showcase: a colored NetHack-style
  map, message window, and status lines over the same core commands and events. Distinct blank-line
  separators structure the visual hierarchy between messages, dungeon map/overlays, status, and
  intent/controls sections. Semantic color styling accents header, message events, health bars,
  ammo/status/outcome, inventory items, and enemy intent. Display-free `--smoke`, JSONL `frame`
  records, replay export, `--print-frames`, and README screenshot goldens are adapter effects.
  Pixel 2D Bevy playtesting is deferred.

## Active

The current slice is complete: authored equipment now includes a ranged-damage bonus resolved at the
core ranged-attack boundary, with protocol version 35 and synchronized replay/state-digest and
adapter projections. Actors may hold one active weapon and one active armor item; effects aggregate
deterministically and the existing single-item accessor remains a compatibility projection. Item
snapshots and client labels project typed roles and common/magic/rare presentation rarity, while the
terminal inventory overlay shows a deterministic comparison line. Rarity is metadata only here;
generated loot, affixes, identification, and richer inventory actions remain deferred.

## Future

### Remaining roadmap milestones

Each remaining proposal milestone needs its own bounded acceptance slice before it can
move into `Past`. Core-facing work updates the terminal client plus MCP/headless/protocol
as needed. Pixel-art Bevy polish waits until core is mature enough for a visual-enhancement
stage. Richer combat, living-dungeon progression, loot, playback-compatible saves, and
release quality remain future work.

- Milestone 3 — First visible Dreadstep is satisfied by the verified terminal client.
  Production-art adoption, pixel 2D window polish, and Bevy visual playtesting are deferred
  to the visual-enhancement stage after core-facing milestones 4–6.
- Milestone 4 — Tactical Combat: richer player verbs and systemic combat interactions.
- Milestone 5 — The Living Dungeon: enemy archetypes, richer environmental state, and core-owned
  floor progression.
- Milestone 6 — Loot and Build Formation: curated item progression, identification, and build
  choices.
- Milestone 7 — Vertical Slice: opening-to-victory run, mature presentation, music, polished
  combat feedback, boss, death, victory, save/quit, and playback-compatible replay export.
- Milestone 8 — Agent QA and Balance Laboratory: scenario agents, behavioral agents, and balance
  experiments.
- Milestone 9 — Content Alpha: broader content, authored scenarios, and coherent production
  direction.
- Milestone 10 — Human-Centered Alpha: structured human playtesting for fun, clarity, pacing,
  feel, hierarchy, and audio feedback.
- Milestone 11 — Beta / Release Candidate: stability, accessibility, performance, and release
  hardening.
- Milestone 12 — Dreadstep 1.0: final content, presentation, documentation, and release quality.
