# Runnable 2D showcase

The current player-facing slice is available as a Cargo-runnable Bevy desktop process:

```sh
cargo run -p dreadstep-bevy --features desktop --bin dreadstep -- --seed 7
```

The first launch creates `dreadstep-logs/` and a create-new JSONL journal. The directory is
ignored by Git. A checkout does not need local art: readable nearest-neighbor placeholder pixels
are used for every family until an optional image loads.

The HUD keeps the existing panel structure and now includes a fixed-width health bar, remaining-
enemy count, turn/position summary, and explicit field-of-view state. These are presentation-only
readings; inventory, messages, controls, journal output, and the smoke command/event matrix remain
unchanged.

New typed event-cue batches now add only a short pulse to visible living actor placeholders. The
runtime replay digest keeps distinct accepted events observable even when their cue values match.
The pulse does not move nodes, alter field-of-view masking, or change the smoke/journal path.

Typed audio-cue batches also route through the validated local audio manifest. Existing
`assets/`-rooted files are requested as short non-looping effects; root/crate-local metadata and
absent files are recorded as safe optional-media fallbacks and do not affect the simulation or
display-free smoke path.

## Controls

| Key | Action | Core command source |
| --- | --- | --- |
| Arrow keys / WASD | Move | `Command::Move` selected from `legal_commands` |
| Space / Enter | Wait | `Command::Wait` |
| F | Attack the lowest-ID adjacent target | `Command::Attack` |
| G | Ranged attack the lowest-ID clear-cardinal target at distance 2–3 (2 ticks; 3 shots) | `Command::RangedAttack` |
| Tab / Shift-Tab | Select the next/previous owned item | presentation-only selection |
| E | Equip selected item | `Command::Equip` |
| Q | Unequip | `Command::Unequip` |
| U | Consume selected item | `Command::UseItem` |
| P | Pick up the lowest-ID item at the player's position | `Command::Pickup` |
| X | Drop the selected unequipped item at the player's position | `Command::Drop` |
| R | Reload to the fixed three-shot capacity when ammo is below full | `Command::Reload` |
| Shift-R | Restart the same seed | new presentation runtime, same core scenario |
| Escape, close button, Ctrl-C | Shut down | process boundary only |

Only actor 1 acts from the keyboard. When an enemy is scheduled, the presentation driver waits
150 ms and chooses its legal `Chase` toward actor 1, falling back to legal `Wait`. The delay is
never simulation time. A presentation-only “showcase complete” status appears after every enemy
is dead; it does not add a canonical core victory rule.

## Optional local art

The six independent files below are looked up relative to the working directory:

```text
assets/dreadstep/terrain.png
assets/dreadstep/player.png
assets/dreadstep/enemy.png
assets/dreadstep/dead.png
assets/dreadstep/ground-item.png
assets/dreadstep/inventory-item.png
```

Each family is loaded independently with nearest-neighbor sampling. Missing or corrupt files are
journaled warnings and retain that family’s placeholder. Terrain placeholders are tinted
separately for floor and wall. Inventory render nodes stay unplaced and hidden; inventory is shown
in the HUD instead.

The selected CC0 prototype can be prepared reproducibly from the ignored archive:

```sh
scripts/prepare-local-assets.sh --check
scripts/prepare-local-assets.sh --install
```

The command validates the recorded SHA-256 and six source members before writing the files above.
It is optional and local-only; a clean checkout continues to use deterministic placeholders.

## Journal contract

Every line has:

```json
{"schema_version":1,"sequence":1,"elapsed_ms":0,"kind":"run_started","payload":{}}
```

`sequence` is monotonic and `elapsed_ms` is diagnostic monotonic elapsed time from the process
journal opening; it never enters simulation state. Action request/outcome records contain the complete map, actor,
inventory, ground-item, scheduler, state digest, and replay digest evidence. The journal records
run/restart, supported input requests, command requests before execution, accepted events,
unchanged rejected actions, asset outcomes, warnings, terminal victory/fault, shutdown, and
caught unexpected panic payloads. Every record is flushed before the process continues. Existing
files are never overwritten; a suffix is allocated on filename collision.

The journal is diagnostic evidence, not a protocol message or a promised replay-file format.
Failure to create the log directory or a mid-run write/flush fault is reported and returns exit 1.

## Coverage matrix

| Current player-facing surface | Visible/HUD | Journal | Display-free smoke |
| --- | --- | --- | --- |
| Move / wait / enemy chase | map, scheduler, messages | command + event + snapshots | yes |
| Attack / damage / death | actor colors, messages, terminal status | ordered `attacked`/`died` events | yes |
| Inventory / equip / unequip / consume / pickup / drop / reload | selected/equipped HUD rows, ammo, and ground stack | item/reload events and full actor snapshots | yes |
| Terrain and actor blocking | distinct wall/cover/floor/actor pixels | `movement_blocked` reason | yes |
| Presentation field of view | radius-3 floor reach plus readable wall edge | complete scene remains projected | no display required |
| Camera and 640×360 logical window | one primary window, centered camera | startup configuration | startup path |
| Optional art fallback | per-family placeholder | warning/outcome records | no display required |

## Smoke verification

Run without a display:

```sh
cargo run -p dreadstep-bevy --features desktop --bin dreadstep -- \
  --smoke --seed 7 --log-dir target/dreadstep-smoke-logs
```

The deterministic sequence first uses `RangedAttack` against the distance-two authored enemy, then
reloads the player's partial ammunition, drops authored item 102 into the player's current ground stack for smoke setup, picks it up with
`Pickup`, drops it again with `Drop`, moves east, drives enemy turns, equips item 101, unequips it, attacks enemy 2 until death,
consumes item 101, attempts north into terrain, then waits with scheduled enemy chase turns between
player actions. Exhaustive command/event mappings and the coverage lists make a new player-visible
core variant fail desktop-feature compilation or smoke coverage until it is documented and mapped.

## Manual checklist

- one non-resizable 640×360 logical (1280×720 physical at scale 2) primary window opens;
- floor, cover, wall, player, enemy, dead actor, and ground item placeholders are visibly distinct;
- radius-3 field of view follows the controlled actor, hides distant nodes without removing their
  typed mirrors, and keeps adjacent wall edges readable;
- movement, wait, combat, inventory selection, equip/unequip, consume, pickup, drop, reload, restart, and enemy delay
  work as documented;
- HUD shows HP/position, scheduler time/turn, inventory, controls, eight recent messages, status,
  and journal path;
- missing, valid, and intentionally corrupt optional images fall back per family;
- Escape, Ctrl-C, and the close button leave a final shutdown record.

## Deliberate exclusions

Tester-only spawn, teleport, HP, inventory transfer/drop, and scenario mutation remain in MCP/tests;
tester pickup remains a non-action test operation while player pickup is scheduled. This showcase does
not add enemy attacks, player-death loops, item effects, pickup/drop
commands, canonical victory/loss, production audio design/music, persistent exploration memory,
production media, installers, signing, save/load, or replay-file compatibility.
