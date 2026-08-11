# Runnable 2D showcase

The current player-facing slice is available as a Cargo-runnable Bevy desktop process:

```sh
cargo run -p dreadstep-bevy --features desktop --bin dreadstep -- --seed 7
```

The first launch creates `dreadstep-logs/` and a create-new JSONL journal. The directory is
ignored by Git. A checkout does not need local art: readable nearest-neighbor placeholder pixels
are used for every family until an optional image loads.

## Controls

| Key | Action | Core command source |
| --- | --- | --- |
| Arrow keys / WASD | Move | `Command::Move` selected from `legal_commands` |
| Space / Enter | Wait | `Command::Wait` |
| F | Attack the lowest-ID adjacent target | `Command::Attack` |
| Tab / Shift-Tab | Select the next/previous owned item | presentation-only selection |
| E | Equip selected item | `Command::Equip` |
| Q | Unequip | `Command::Unequip` |
| U | Consume selected item | `Command::UseItem` |
| R | Restart the same seed | new presentation runtime, same core scenario |
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
| Inventory / equip / unequip / consume | selected/equipped HUD rows | item events and full inventory snapshots | yes |
| Terrain and actor blocking | distinct wall/floor/actor pixels | `movement_blocked` reason | yes |
| Camera and 640×360 logical window | one primary window, centered camera | startup configuration | startup path |
| Optional art fallback | per-family placeholder | warning/outcome records | no display required |

## Smoke verification

Run without a display:

```sh
cargo run -p dreadstep-bevy --features desktop --bin dreadstep -- \
  --smoke --seed 7 --log-dir target/dreadstep-smoke-logs
```

The deterministic sequence moves east, drives enemy turns, equips item 101, unequips it, attacks
enemy 2 until death, consumes item 101, attempts north into terrain, then waits with scheduled
enemy chase turns between player actions. Exhaustive command/event mappings and the coverage lists
make a new player-visible core variant fail desktop-feature compilation or smoke coverage until it
is documented and mapped.

## Manual checklist

- one non-resizable 640×360 logical (1280×720 physical at scale 2) primary window opens;
- floor, wall, player, enemy, dead actor, and ground item placeholders are visibly distinct;
- movement, wait, combat, inventory selection, equip/unequip, consume, restart, and enemy delay
  work as documented;
- HUD shows HP/position, scheduler time/turn, inventory, controls, eight recent messages, status,
  and journal path;
- missing, valid, and intentionally corrupt optional images fall back per family;
- Escape, Ctrl-C, and the close button leave a final shutdown record.

## Deliberate exclusions

Tester-only spawn, teleport, HP, inventory transfer/drop/pickup, and scenario mutation remain in
MCP/tests. This showcase does not add enemy attacks, player-death loops, item effects, pickup/drop
commands, canonical victory/loss, audio, animation playback, fog of war, production media,
installers, signing, save/load, or replay-file compatibility.
