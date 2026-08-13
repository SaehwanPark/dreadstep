# Runnable 2D showcase

The current player-facing slice is available as a Cargo-runnable Bevy desktop process:

```sh
cargo run -p dreadstep-bevy --features desktop --bin dreadstep -- --seed 7
```

An opt-in procedural floor can be launched visibly with the same deterministic seed and an authored
depth:

```sh
cargo run -p dreadstep-bevy --features desktop --bin dreadstep -- \
  --procedural --depth 1 --seed 7
```

The run journal names this scenario `procedural_floor` and records its depth. `Shift+R` restarts
the same procedural seed/depth. The HUD shows the active scenario and depth. After a procedural run reaches victory, `N` starts the next depth
with the same seed and records `floor_advanced`; the transition resets the disposable presentation
state and replay trace. The display-free `--smoke` path intentionally remains on the authored item
fixture and its exhaustive command/event matrix.

At a terminal outcome, the HUD names the available recovery action: procedural victory identifies the
next depth and `N` when another depth is available, while authored victory and defeat identify
`Shift+R` restart.
The controls panel likewise shows the `N` progression action only for procedural sessions.

The first launch creates `dreadstep-logs/` and a create-new JSONL journal. The directory is
ignored by Git. A checkout does not need local art: readable nearest-neighbor placeholder pixels
are used for every family until an optional image loads.

The HUD keeps the existing panel structure and now includes a fixed-width health bar, remaining-
enemy count, turn/position summary, and explicit field-of-view state. These are presentation-only
readings; an accepted player death also changes the disposable status to defeat and writes a
`terminal_defeat` journal record. Inventory, messages, controls, and the smoke command/event matrix
remain unchanged.

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
| I | Open the first legal adjacent closed door | `Command::Interact` |
| K | Kick the first legal adjacent closed door | `Command::Kick` |
| C | Close the first legal adjacent open door | `Command::Close` |
| B | Break the first legal adjacent breakable tile | `Command::Break` |
| F | Attack the lowest-ID adjacent target | `Command::Attack` |
| G | Ranged attack the lowest-ID clear-cardinal target at distance 2–3 (2 ticks; 3 shots) | `Command::RangedAttack` |
| Tab / Shift-Tab | Select the next/previous owned item | presentation-only selection |
| E | Equip selected item | `Command::Equip` |
| Q | Unequip | `Command::Unequip` |
| U | Consume selected item | `Command::UseItem` |
| T | Throw the selected Frost Flask at the lowest-ID legal target | `Command::Throw` |
| P | Pick up the lowest-ID item at the player's position | `Command::Pickup` |
| X | Drop the selected unequipped item at the player's position | `Command::Drop` |
| R | Reload to the fixed three-shot capacity when ammo is below full | `Command::Reload` |
| Shift-R | Restart the same seed | new presentation runtime, same core scenario |
| Escape, close button, Ctrl-C | Shut down | process boundary only |

Only actor 1 acts from the keyboard. When an enemy is scheduled, the presentation driver waits
150 ms and chooses its core-ranked legal intent. The HUD names the authored behavior next to the
command: an authored Kiter retreats from an adjacent actor
when an escape tile exists, then enemies use adjacent `Attack`, a Frostcaster uses `CastChill` on a
clear-cardinal target at distance 2–3, other enemies use clear-cardinal `RangedAttack`,
one-use `Investigate` toward a nearby kick-noise position, a Brute `Break` when a Breakable blocks
its next horizontal-first chase step, `Chase`, and finally `Wait`. The delay is never simulation
time. A
presentation-only “showcase complete” status appears after every enemy is dead and consumes core's
canonical `RunOutcome` projection.

The authored item showcase places a closed door at `(2,1)`, immediately east of player actor 1,
a Frostcaster enemy as actor 3, and a Brute enemy with a Breakable obstacle at `(4,3)` so both
archetype intents are visible during enemy turns. The display-free smoke specifically asserts
actor 3's `CastChill`/`ChillCast` evidence and actor 4's enemy-driver `Break` with the matching
`BreakableBroken` event; later player actions do not substitute for those enemy-driver checks.
Start the item showcase to exercise `I` (open) and `C` (close) without relying on the display-free
smoke fixture; the item-free core starter floor remains the stable adapter test fixture.

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

Actor snapshots include the fixed four-item inventory capacity. A full player's pickup action is
omitted from legal actions and rejected atomically if requested directly. The authored item-run
fixture's item `101` is a three-point healing consumable, item `102` is a two-round ammunition
consumable, item `103` is a reach weapon, and item `104` is the Frost Flask. The journal records
capped recovery in `item_consumed` and ordered `item_thrown`/`status_applied` evidence. Other item
effects and capacity upgrades remain outside this showcase slice.

The journal is diagnostic evidence, not a protocol message or a replay playback format. At clean
smoke or visible-run completion, the desktop boundary also writes a versioned sibling
`*.replay.json` artifact containing the seed, accepted command order, replay digest, and canonical
outcome. It is create-new evidence only; save/load and playback remain deferred. Failure to create
the log directory or a mid-run write/flush fault is reported and returns exit 1.

## Coverage matrix

| Current player-facing surface | Visible/HUD | Journal | Display-free smoke |
| --- | --- | --- | --- |
| Move / wait / enemy attack/ranged/investigate/chase; named Kiter/Brute/Frostcaster intent | map, scheduler, behavior + intent, messages | command + event + snapshots | yes |
| Attack / damage / death | actor colors, messages, terminal status | ordered `attacked`/`died` events | yes |
| Inventory / equip / unequip / consume / throw / pickup / drop / reload | selected/equipped HUD rows, reach-weapon/Frost Flask effects, healing/ammo results, and ground stack | item/reload/throw events, equipment effect, optional healing/ammo evidence, and full actor snapshots | yes |
| Terrain, door/OpenDoor, trap, ChillTrap/Chilled, breakable, terrain-aware noise, and actor blocking | distinct wall/cover/floor/door/open-door/trap/chill-trap/breakable/actor pixels plus status duration | typed status application/expiry and terrain event evidence | yes |
| Presentation field of view | radius-3 floor reach plus readable wall edge | complete scene remains projected | no display required |
| Opt-in procedural floor and `N` advancement | seeded 13×9 floor and next-depth restart after victory | `run_started` depth and `floor_advanced` evidence | no; smoke keeps item fixture |
| Camera and 640×360 logical window | one primary window, centered camera | startup configuration | startup path |
| Optional art fallback | per-family placeholder | warning/outcome records | no display required |

## Smoke verification

Run without a display:

```sh
cargo run -p dreadstep-bevy --features desktop --bin dreadstep -- \
  --smoke --seed 7 --log-dir target/dreadstep-smoke-logs
```

The deterministic sequence first crosses a one-shot ChillTrap (recording the two-action Chilled
status and expiry), uses `RangedAttack` against the distance-two authored enemy, then teleports that
enemy to a clear distance-three throw fixture and throws Frost Flask item 104 (recording
`ItemThrown` and an applied Chilled status), then adds a breakable-terrain
smoke fixture and breaks it with `Break`, then adds a closed-door fixture and kicks it with `Kick`
(including terrain-aware noise evidence and a nearby enemy `Investigate` turn), re-adds a door and opens it with
`Interact`, closes the resulting OpenDoor with `Close`, then reloads the player's
partial ammunition,
drops authored item 102 into the player's current ground stack for smoke setup, picks it up with
`Pickup`, drops it again with `Drop`, moves east, drives enemy turns, equips item 103, attacks enemy 2 from two tiles until death, then unequips it,
consumes item 101, attempts north into terrain, then waits with scheduled enemy chase turns between
player actions. During the earlier enemy drive, a deterministic Kiter fixture retreats from
adjacency; the smoke journal records its accepted `Retreat` command and `Moved` evidence. That
enemy is then re-authored as a Frostcaster at distance two so the driver records `CastChill`,
`ChillCast`, and the ordered `StatusApplied` result.
Exhaustive command/event mappings and the coverage lists make a new player-visible core variant
fail desktop-feature compilation or smoke coverage until it is documented and mapped.

## Manual checklist

- one non-resizable 640×360 logical (1280×720 physical at scale 2) primary window opens;
- floor, cover, wall, door, OpenDoor, trap, breakable, player, enemy, dead actor, and ground item placeholders are visibly distinct;
- radius-3 field of view follows the controlled actor, hides distant nodes without removing their
  typed mirrors, and keeps adjacent wall edges readable;
- movement, wait, adjacent door interaction, door closing, breakable terrain, trap triggering, combat, inventory selection, equip/unequip, consume, pickup, drop, reload, restart, procedural `N` floor advancement after victory, and enemy delay
  work as documented;
- a clear distance-2 Frostcaster turn shows `CastChill` intent and chilled-status feedback;
- HUD shows HP/position, scheduler time/turn, inventory, controls, eight recent messages, status,
  and journal path;
- missing, valid, and intentionally corrupt optional images fall back per family;
- Escape, Ctrl-C, and the close button leave a final shutdown record.
- Ctrl-C finalization also writes the sibling replay-evidence artifact before process exit.

## Deliberate exclusions

Tester-only spawn, teleport, HP, inventory transfer/drop, and scenario mutation remain in MCP/tests;
tester pickup remains a non-action test operation while player pickup/drop are scheduled commands.
This showcase does not add player-death loops, respawn, item effects, victory rewards, production
audio design/music, persistent exploration memory, production media, installers, signing, save/load,
or replay playback/compatibility.
