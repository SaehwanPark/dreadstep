# Runnable terminal showcase

The current player-facing slice is a NetHack-style terminal client. It needs no windowing
system, pixel art, or audio device:

```sh
cargo run -p dreadstep-tui -- --seed 7
```

An opt-in procedural floor can be launched with the same deterministic seed and an authored
depth:

```sh
cargo run -p dreadstep-tui -- --procedural --depth 1 --seed 7
```

When stdin is not a TTY, or when `--print-frames` is passed, the client prints a plain
(no ANSI) frame after each state change, separated by `----`. Agents can read those frames
from a terminal transcript and the JSONL `frame` records under `dreadstep-logs/` (or
`--log-dir`). Semantic play can still use MCP `observe` / `legal_actions` / `act`; glyphs
stay TUI policy and are not MCP tools.

`--no-delay` executes enemy turns immediately. `--smoke` is the display-free coverage gate.

After a procedural run reaches victory, `N` starts the next deterministic depth with the
same seed and records `floor_advanced`. `R` restarts the current seed/depth.

## Controls

Letter keys that collide with WASD movement keep the movement alias; the NetHack verb uses
the shifted letter or the earlier desktop alias.

| Key | Action | Core command source |
| --- | --- | --- |
| `h`/`j`/`k`/`l`, arrows, WASD | Move; bump a living enemy to attack, bump a closed door to open | `Command::Move`, or legal `Attack` / `Interact` |
| `y`/`u`/`b`/`n` | Refused; Dreadstep has no diagonals | none |
| `.` / Space / Enter | Wait | `Command::Wait` |
| `o` / `I` | Open the first legal adjacent closed door | `Command::Interact` |
| `c` | Close the first legal adjacent open door | `Command::Close` |
| Ctrl-d / `K` | Kick the first legal adjacent closed door | `Command::Kick` |
| `S` / `B` | Smash the first legal adjacent breakable | `Command::Break` |
| `F` / `A` | Attack the lowest-ID legal melee target | `Command::Attack` |
| `f` / `G` | Ranged attack the lowest-ID legal target | `Command::RangedAttack` |
| `t` | Throw the selected Frost Flask at the lowest-ID legal target | `Command::Throw` |
| `,` / `p` | Pick up the lowest-ID item at the player's position | `Command::Pickup` |
| `x` / `D` | Drop the selected unequipped item | `Command::Drop` |
| `e` / `E` | Equip selected item | `Command::Equip` |
| `T` / `Q` | Unequip | `Command::Unequip` |
| `q` / `U` | Use selected item | `Command::UseItem` |
| `r` | Reload | `Command::Reload` |
| Tab / Shift-Tab | Select next/previous owned item | presentation-only |
| `i` | Inventory overlay | presentation-only |
| `?` | Help overlay | presentation-only |
| `R` | Restart the same seed | new presentation runtime, same core scenario |
| `N` | Next procedural depth after victory | new presentation runtime |
| Escape, Ctrl-c | Shut down | process boundary only |

Only actor 1 acts from the keyboard. When an enemy is scheduled, the TTY client waits 150 ms
(skipped by `--no-delay` and `--smoke`) and chooses core's `preferred_enemy_command`. The
status line names the authored behavior next to that command. The delay is never simulation
time.

The authored item showcase places a closed door at `(2,1)`, immediately east of player actor
1. Start that showcase to exercise `o`/`c` without relying on display-free smoke.

## Agent monitoring

Every accepted command, rejection, overlay change, and startup appends a JSONL record:

```json
{"schema_version":1,"sequence":1,"elapsed_ms":0,"kind":"frame","payload":{}}
```

`kind: "frame"` payloads include the plain frame string, seed, digest, outcome, and next
actor. The journal also records `command_requested`, `action_accepted`, `action_rejected`,
terminal outcomes, and `shutdown`. A sibling `*.replay.json` export is written from the
accepted core trace, not reconstructed from the journal.

Do not add MCP tools that return TUI glyphs. Combine this log with existing MCP player tools
when an agent needs both a readable frame and typed legal actions.

## Coverage matrix

| Current player-facing surface | TUI frame | Journal | Display-free smoke |
| --- | --- | --- | --- |
| Move / wait / enemy attack/ranged/investigate/chase; named Kiter/Brute/Frostcaster/Blocker/Scavenger/Zombie intent | map, scheduler, behavior + intent, messages | command + event + frames | yes |
| Attack / damage / death | glyphs, messages, terminal status | ordered `attacked`/`died` events | yes |
| Inventory / equip / unequip / consume / throw / pickup / drop / reload | selected/equipped rows, rarity, and affix labels | item/reload/throw events | yes for authored fixture |
| Terrain, door/OpenDoor, trap, ChillTrap/Chilled, breakable, terrain-aware noise | distinct glyphs plus status duration | typed status and terrain events | yes |
| Presentation field of view | radius-3 cardinal reveal; unseen cells are spaces | complete world remains in core/journal state | no display required |
| Opt-in procedural floor and `N` advancement | seeded 13×9 floor, generated rarity/affix-tier label, and next-depth restart after victory | `run_started` depth and `floor_advanced` | no; smoke keeps item fixture |

## Smoke verification

Run without a TTY:

```sh
cargo run -p dreadstep-tui -- \
  --smoke --seed 7 --log-dir target/dreadstep-tui-smoke-logs
```

The deterministic sequence matches the former desktop smoke command/event matrix. Exhaustive
`SHOWCASE_COMMAND_KINDS` and `SHOWCASE_EVENT_KINDS` make a new player-visible core variant
fail TUI compilation or smoke coverage until it is documented and mapped.

## Manual checklist

- a colored NetHack-style map, message window, and status lines appear in the terminal;
- floor, cover, wall, door, OpenDoor, trap, chill trap, breakable, player, enemy behaviors,
  corpses, and ground items are distinct glyphs;
- radius-3 field of view follows the controlled actor and hides distant cells;
- movement, bump-to-open, bump-to-attack, wait, door close, smash, trap, combat, inventory,
  equip/unequip, consume, pickup, drop, reload, restart, procedural `N`, and enemy delay
  work as documented;
- `yubn` prints `You cannot move diagonally.` and does not call core;
- HUD shows HP/position, scheduler, inventory, intent, messages, and controls;
- Escape and Ctrl-c leave a final shutdown record and replay export.

## Deferred pixel 2D

The Bevy desktop client remains in the workspace but is **not** the default tester or CI
player-facing gate. Pixel-2D playtesting waits until a visual-enhancement stage after
core-facing milestones. The frozen command is:

```sh
cargo run -p dreadstep-bevy --features desktop --bin dreadstep -- --seed 7
```

The later visual-stage smoke wrapper is `scripts/verify-bevy-desktop.sh`. Do not treat
display-free Bevy smoke as a substitute for the terminal showcase during core work.

## Deliberate exclusions

Tester-only spawn, teleport, HP, inventory transfer/drop, and scenario mutation remain in
MCP/tests. This showcase does not add exploration memory, diagonal movement, player-death
loops, production art, music, save/load, or replay playback.
