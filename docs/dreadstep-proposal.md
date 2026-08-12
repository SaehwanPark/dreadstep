# Dreadstep
## Project Proposal and Development Roadmap

**Working genre:** Tactical systemic dungeon roguelike  
**Primary platform:** macOS  
**Target platforms:** macOS, Windows, Linux; Web considered later  
**Technology:** Rust + Bevy + MCP  
**Source model:** Open source  
**Proposed code license:** MIT  
**Development philosophy:** deterministic simulation first, AI-driven testing early, human-centered game-feel refinement later

> **Working pitch:**  
> *Dreadstep is a fast-playing, turn-based dungeon roguelike where every step matters: Diablo-like descent and item progression, compact tactical combat, and a deliberately constrained but deeply composable world of environmental interactions.*

Possible tagline:

> **Every step is a decision.**

---

# 1. Executive Summary

Dreadstep is a 2D pixel-art dungeon crawler combining four complementary ideas:

1. **Jupiter Hell Classic / DRL:** fast, legible turn-based tactical play.
2. **Diablo I:** oppressive dungeon descent, recognizable enemy archetypes, randomized equipment, and the excitement of discovering a build through loot.
3. **NetHack:** a smaller and more curated set of systemic interactions in which objects, terrain, status effects, enemies, and player actions obey composable rules.
4. **Modern agent-native development:** a deterministic headless simulation exposed through MCP from the beginning, enabling AI agents to play, explore, test, reproduce failures, and search for exploits before large-scale human playtesting.

The game should not attempt to reproduce the content volume of Diablo or the accumulated complexity of NetHack. Its distinguishing characteristic should instead be **density of meaningful decisions**.

A successful Dreadstep turn should frequently pose questions such as:

- Do I attack now or reposition?
- Should I retreat through the doorway?
- Is this consumable worth spending?
- Can I exploit the environment instead?
- Do I want to identify this item experimentally?
- Is that valuable object worth exposing myself to another enemy?
- Can two otherwise minor mechanics interact to solve this encounter?

The game should remain quick enough that these decisions do not become exhausting.

---

# 2. Project Thesis

The central design thesis is:

> **A traditional roguelike does not need enormous complexity to create emergent play. It needs a compact set of consistently interacting rules, tactical situations that expose those rules, and sufficiently low interaction friction that experimentation remains enjoyable.**

Dreadstep therefore optimizes for:

**low mechanical friction × high tactical consequence × moderate systemic depth.**

It deliberately avoids maximizing any one inspiration independently.

It is not:

- Diablo converted literally into turns;
- NetHack with modern graphics;
- Jupiter Hell Classic with medieval sprites;
- a giant procedurally generated RPG;
- an ARPG loot treadmill;
- an AI-controlled game disguised as a human game.

Instead, each influence has a specific role.

| Influence | Dreadstep should inherit |
|---|---|
| Jupiter Hell Classic / DRL | responsiveness, turn readability, tactical positioning |
| Diablo I | descent, atmosphere, loot identity, dangerous dungeon ecology |
| NetHack | interactions, object verbs, rule composability, experimentation |
| Modern roguelites | presentation clarity and accessibility, selectively |
| MCP/AI tooling | systematic exploration and automated gameplay testing |

---

# 3. Design Pillars

## Pillar 1 — Every Step Matters

Movement is gameplay.

Tiles should represent meaningful tactical choices involving:

- line of sight;
- distance;
- escape routes;
- doorways;
- hazards;
- enemy zones of influence;
- corpse locations;
- destructible terrain;
- item positions;
- environmental effects.

The game should reward positioning without requiring chess-like calculation every turn.

---

## Pillar 2 — Fast Thoughtful Turns

Turn-based must not imply slow.

Common actions should generally require one obvious input:

```text
move
attack
open
close
wait
pick up
use
throw
kick/interact
```

The player should be able to fly through trivial situations while naturally slowing down when the dungeon becomes dangerous.

Animations should communicate consequences without preventing the next decision unnecessarily.

---

## Pillar 3 — Fewer Things, Stronger Identities

Dreadstep should initially resist content inflation.

Prefer:

> 30 interesting items

over:

> 300 statistically interchangeable items.

Prefer:

> 12 behaviorally distinct monsters

over:

> 50 monsters differentiated mostly by HP.

Likewise, affixes should preferably alter decisions rather than only numbers.

Weak:

```text
+7% weapon damage
```

Stronger:

```text
attacks chill targets
```

because chill may subsequently interact with:

- action speed;
- water;
- fire;
- brittle enemies;
- another item;
- a spell;
- environmental hazards.

---

## Pillar 4 — Systems Should Compose

The NetHack influence should manifest as a controlled vocabulary of interacting properties.

For example:

```text
Fire
Cold
Liquid
Oil
Wood
Metal
Corpse
Noise
Breakable
Poison
Electricity
```

Individual rules then compose:

```text
fire + oil -> stronger/spreading fire

cold + water -> ice

fire + ice -> water

fire + wood -> burning/destruction

heavy object + pressure plate -> trigger

kick + closed door -> noise / damage / opening

fire + corpse -> destroyed corpse

necromancer + corpse -> possible resurrection
```

This creates emergent behavior without requiring thousands of hand-coded special cases.

---

## Pillar 5 — The Rules Must Be Legible

Emergence becomes frustrating if the player cannot reason about it.

The game therefore needs:

- strong visual state indicators;
- concise combat/event messages;
- discoverable item descriptions;
- status icons;
- consistent terminology;
- predictable rules;
- optional inspection;
- clear feedback after interactions.

The player should often be surprised by possibilities, but rarely confused about what just happened.

---

## Pillar 6 — Simulation First, Presentation Second

The authoritative game does not live inside Bevy rendering systems.

The game is a deterministic simulation receiving commands and producing state transitions and events.

Conceptually:

```text
WorldState
    +
PlayerCommand
    |
    v
Simulation
    |
    +------> WorldState'
    |
    +------> GameEvents
```

Bevy is one client.

MCP is another.

Automated tests are another.

This becomes a foundational project constraint rather than a later refactor.

---

# 4. Intended Player Experience

The desired emotional cycle is:

```text
confidence
    ↓
discovery
    ↓
uncertainty
    ↓
tactical problem
    ↓
experimentation
    ↓
consequence
    ↓
adaptation
    ↓
greater confidence
    ↓
deeper descent
```

The dungeon should gradually convert knowledge into power.

A strong player improves not only because the character's numbers increase, but because the player learns:

- enemy tendencies;
- item interactions;
- terrain possibilities;
- resource economics;
- when to engage;
- when to escape.

This keeps the traditional roguelike principle that **player knowledge is itself progression**.

---

# 5. Target Audience

Primary audience:

- traditional roguelike players;
- Diablo I / early action-RPG enthusiasts;
- players interested in tactical turn-based games;
- players who appreciate systemic or emergent mechanics;
- players attracted to retro pixel aesthetics but wanting contemporary usability.

Secondary audiences:

- developers interested in Rust/Bevy;
- procedural-generation enthusiasts;
- AI-agent/game-environment researchers and hobbyists;
- open-source game developers;
- modders, eventually.

The game should remain approachable to players who have never memorized traditional roguelike keyboard conventions.

---

# 6. Core Gameplay Loop

The fundamental loop is:

```text
Enter dungeon
     ↓
Explore
     ↓
Detect threat/opportunity
     ↓
Fight / manipulate / evade
     ↓
Spend or preserve resources
     ↓
Acquire loot
     ↓
Adapt build
     ↓
Find descent
     ↓
Enter more dangerous floor
     ↓
...
     ↓
Death or final objective
```

On a smaller scale:

```text
observe
  ↓
choose action
  ↓
world advances
  ↓
read consequence
  ↓
update plan
```

The smallest loop must already be enjoyable.

---

# 7. Run Structure

For the initial full game, target a relatively concentrated run rather than a very long campaign.

A reasonable eventual structure is approximately:

```text
Surface / entrance

Depths I
  Floors 1–4
  introductory ecology

Depths II
  Floors 5–8
  stronger environmental interactions

Depths III
  Floors 9–12
  dangerous combined systems

Final descent / boss
```

Exact floor count should remain tunable.

Target full-run duration should probably be closer to **roughly one substantial play session** than to a 20-hour RPG campaign.

Dreadstep should support:

- permadeath as the standard mode;
- deterministic run seeds;
- save-and-quit;
- explicit daily/challenge seeds later;
- debugging/sandbox modes for development.

Persistent stat-grind metaprogression should not be a v1 requirement.

Content unlocks may eventually exist, but the central balance should remain viable without permanent numerical bonuses.

---

# 8. Turn and Time Model

Use discrete simulation time rather than strict:

```text
player moves
everyone moves
player moves
everyone moves
```

Each action receives an integer cost.

Conceptually:

$$
t_{\mathrm{next}}
=
t_{\mathrm{current}}
+
\frac{c(a)}{s}
$$

where:

- $c(a)$ = action cost;
- $s$ = actor speed.

In implementation, prefer integer or fixed-point calculations rather than floating-point simulation where practical.

Example conceptual costs:

```text
walk              100
wait              100
dagger attack      80
sword attack      100
heavy weapon      140
quick item use     80
complex spell     150
```

This creates natural mechanical meanings for:

- quick weapons;
- slow weapons;
- haste;
- chill;
- encumbrance;
- fast enemies;
- slow enemies;
- reload actions.

It also creates a useful bridge between roguelike turns and Diablo-style attack-speed differentiation.

---

# 9. Combat Model

Combat should emphasize spatial tactics over numerical optimization.

Core dimensions:

- adjacency;
- weapon reach;
- line of sight;
- range;
- accuracy;
- damage;
- armor/resistance;
- action cost;
- status effects;
- environment;
- enemy intent.

Avoid excessive stat dimensions initially.

A useful early combat model may involve:

```text
attack
  -> hit/miss resolution
  -> physical/magical damage
  -> mitigation
  -> status/effect application
  -> death/environmental consequences
```

Important combat properties should be observable through events so that both humans and test agents can reason about outcomes.

---

# 10. Enemy Design

Enemies should primarily differ through **behavior**, not merely statistics.

Initial behavioral vocabulary might include:

- aggressively closes distance;
- maintains range;
- retreats when isolated;
- protects another monster;
- investigates noise;
- avoids hazards;
- ignores hazards;
- resurrects corpses;
- creates hazards;
- teleports/repositions;
- blocks corridors;
- summons;
- flees at low health.

Example archetypes:

```text
Zombie
  slow
  durable
  direct pursuit

Skeleton Archer
  maintains range
  retreats from adjacency

Fallen-like Scavenger
  weak individually
  retreats when allies die
  may regroup

Brute
  slow heavy attacker
  destroys obstacles

Necromancer
  avoids direct combat
  uses corpses as resources

Stalker
  high mobility
  intermittent visibility
```

Interactions between enemy behaviors should generate encounter complexity naturally.

Enemy AI itself should remain deterministic and conventional. Shipping gameplay should not require an LLM.

---

# 11. Item Philosophy

Loot should produce adaptation rather than housekeeping.

Primary equipment categories could eventually include:

```text
weapon
off-hand
armor
head
boots
amulet
rings
```

but the earliest versions need fewer slots.

Base equipment should have strong mechanical identity.

Examples:

```text
Dagger
  low action cost
  modest damage

Great Axe
  high damage
  high action cost
  possible cleave

Spear
  extended reach

Short Bow
  ranged
  ammunition/reload tradeoff

Torch
  weak weapon
  light source
  fire interaction
```

Affixes should favor tactical differences.

Examples:

```text
of Frost
  applies chill

of Embers
  chance to ignite

Vampiric
  conditional life recovery

Echoing
  produces additional noise

Gravebound
  stronger near corpses

Quick
  reduced attack cost
```

Pure numerical affixes may exist, but they should not dominate the loot system.

---

# 12. Identification and Experimentation

A limited unidentified-item system could reinforce the NetHack influence.

However, identification must not become inventory bureaucracy.

Possible model:

```text
Unknown crimson potion
      ↓
drink / throw / identify
      ↓
effect discovered
      ↓
all matching potions become known
```

World-seed-dependent appearance mapping could make knowledge partly run-specific while preserving category learning.

This should be considered after the basic item economy already works.

---

# 13. Systemic Interaction Vocabulary

Rather than attempting arbitrary object interaction, define a bounded interaction language.

Candidate verbs:

```text
move
attack
open
close
kick
throw
use
equip
drop
inspect
wait
```

Potential later verbs:

```text
push
pull
break
ignite
pour
disarm
```

Every new verb should pass a simple test:

> Does it create several meaningful interactions?

A verb with one special-case use probably does not justify its complexity.

---

# 14. Dungeon Generation

Use **hybrid procedural generation**.

Pure random geometry tends to produce technically varied but aesthetically weak spaces.

Dreadstep should combine:

```text
procedural topology
+
room templates
+
hand-authored set pieces
+
encounter rules
+
environmental motifs
+
random population
```

Generation pipeline:

```text
seed
 ↓
macro layout
 ↓
rooms / corridors
 ↓
special structures
 ↓
terrain
 ↓
doors / hazards
 ↓
enemy placement
 ↓
items
 ↓
validation
 ↓
playable floor
```

Every generated level must be automatically validated for:

- reachability;
- valid stairs;
- accessible required objectives;
- legal entity placement;
- no invalid overlapping state.

Later MCP agents can test higher-order questions that graph validation cannot answer.

---

# 15. Visual Direction

![Aspirational Dreadstep concept art](../dreadstep-concept-art.png)

> **Concept-art note:** This tracked reference is an aspirational visual reference, not a
> screenshot of the current game, an implemented feature set, or a final asset direction. It is
> subject to change. Future screenshots may be tracked under `screenshots/`.

Presentation should reference the readability of JHC rather than imitate it directly.

The reference suggests the intended combination of top-down 2D pixel presentation, a dark
gothic atmosphere, strong silhouettes, tactical readability, and a clear information hierarchy.
The depicted characters, mechanics, statistics, HUD layout, map details, and content are
non-binding and must not be treated as requirements.

Recommended characteristics:

- top-down 2D tiles;
- pixel art;
- dark gothic palette;
- strong silhouettes;
- clear enemy identification;
- limited visual noise;
- brief readable animations;
- atmospheric lighting used cautiously;
- environmental states visibly represented.

Potential logical tile sizes:

```text
24 × 24
32 × 32
```

The exact choice should follow asset experiments rather than architecture.

Use a low logical rendering resolution and integer scaling where practical to preserve pixel consistency.

The player should be able to understand tactical state without reading combat logs continuously.

---

# 16. Audio Direction

Audio should serve information before spectacle.

Important cues include:

- door opening;
- enemy detection;
- ranged attack;
- hit;
- critical hit;
- blocked attack;
- status application;
- item pickup;
- hidden movement;
- stairs;
- environmental ignition;
- nearby danger.

Music can establish dungeon identity, but sound effects should reinforce the simulation.

---

# 17. Technical Baseline

As of August 8, 2026, Bevy 0.19 is the current released Bevy generation; it was released June 19, 2026. Bevy officially lists macOS, Windows, Linux, Web, iOS, and Android among its supported platforms.

Dreadstep should begin against:

```text
Rust stable
Rust 2024 edition
Bevy 0.19
Cargo workspace
```

Bevy itself is permissively dual-licensed under MIT or Apache-2.0, which is compatible with Dreadstep using MIT for its own code.

The MCP specification's current authoritative version is `2026-07-28`.

The official Rust MCP SDK is `rmcp`; its latest GitHub release as of August 8 is 3.1.2, released August 7, 2026.

Use current versions at project initialization, but isolate Bevy and MCP dependencies behind project-owned interfaces so upgrades remain deliberate rather than architectural.

---

# 18. Proposed Workspace Architecture

```text
dreadstep/
├── Cargo.toml
├── crates/
│
│   ├── dreadstep-core/
│   │   ├── world
│   │   ├── actors
│   │   ├── map
│   │   ├── combat
│   │   ├── actions
│   │   ├── scheduler
│   │   ├── effects
│   │   ├── inventory
│   │   ├── ai
│   │   ├── rng
│   │   └── generation
│   │
│   ├── dreadstep-protocol/
│   │   ├── commands
│   │   ├── observations
│   │   ├── events
│   │   ├── snapshots
│   │   └── replay
│   │
│   ├── dreadstep-content/
│   │   ├── monsters
│   │   ├── items
│   │   ├── affixes
│   │   ├── terrain
│   │   └── validation
│   │
│   ├── dreadstep-headless/
│   │   ├── runner
│   │   ├── telemetry
│   │   └── CLI
│   │
│   ├── dreadstep-mcp/
│   │   ├── player API
│   │   ├── tester API
│   │   └── resources
│   │
│   └── dreadstep-bevy/
│       ├── rendering
│       ├── animation
│       ├── input
│       ├── UI
│       └── audio
│
├── assets/
├── content/
├── scenarios/
├── replays/
├── tests/
└── docs/
```

Dependency direction should remain roughly:

```text
content ───────────┐
                   ▼
protocol ───────> core
                   ▲
                   │
          ┌────────┼────────┐
          │        │        │
       headless   MCP      Bevy
```

The key rule:

> `dreadstep-core` must not depend on Bevy or MCP.

---

# 19. Simulation Contract

The simulation boundary should resemble:

```rust
fn step(
  state: &mut WorldState,
  command: PlayerCommand,
) -> Result<Vec<GameEvent>, ActionError>;
```

Conceptually:

```text
PlayerCommand
      ↓
validation
      ↓
simulation mutation
      ↓
enemy/world scheduling
      ↓
GameEvent[]
      ↓
updated WorldState
```

Commands might include:

```text
Move(Direction)
Wait
Melee(EntityId)
Open(Position)
Close(Position)
Pickup(ItemId)
Drop(ItemId)
Equip(ItemId)
Use(ItemId, Target)
Throw(ItemId, Target)
Kick(Target)
```

Bevy should translate user input into these commands.

MCP should translate agent tool calls into the same commands.

Tests should construct the same commands directly.

---

# 20. Determinism

Determinism is a first-class feature.

The authoritative simulation should avoid depending upon:

- wall-clock time;
- frame rate;
- render state;
- uncontrolled thread scheduling;
- nondeterministic map iteration;
- operating-system randomness;
- floating-point behavior where integer alternatives are straightforward.

RNG should be centralized and explicitly seeded.

A replay should record at minimum:

```json
{
  "game_version": "0.x.y",
  "content_version": "...",
  "seed": 9817231,
  "commands": []
}
```

Replays should be considered version-scoped rather than promising permanent compatibility across all game releases.

This allows:

```text
seed + version + commands
            ↓
     reproducible run
```

which becomes enormously valuable for both conventional and AI-discovered bugs.

---

# 21. Event Architecture

Core gameplay should emit semantic events:

```text
ActorMoved
AttackStarted
AttackHit
AttackMissed
DamageApplied
ActorKilled
DoorOpened
ItemPickedUp
ItemUsed
StatusApplied
TerrainChanged
NoiseCreated
EntitySpotted
EntityLost
FloorEntered
RunEnded
```

Bevy consumes them for:

- animation;
- sound;
- particles;
- UI;
- messages.

Telemetry consumes them for measurement.

Replay/debug tools consume them for inspection.

This prevents presentation code from becoming the hidden source of gameplay truth.

---

# 22. MCP as a Foundational Interface

MCP should exist very early.

Two distinct capability surfaces should be provided.

## Player Surface

The AI sees only legitimate player information.

Candidate tools:

```text
run.start
run.observe
run.legal_actions
run.act
run.inspect_item
run.inspect_entity
run.character
run.history
run.finish
```

It must not receive:

```text
hidden enemies
unrevealed tiles
unidentified item truth
future RNG
hidden traps
enemy internal state
```

The observation layer therefore becomes an explicit information-security boundary within the simulation.

---

## Tester Surface

Development agents can additionally receive controlled debugging capabilities:

```text
test.create_scenario
test.snapshot
test.restore
test.spawn_entity
test.give_item
test.set_status
test.teleport
test.reveal_map
test.inspect_world
test.inspect_rng
```

These should be unavailable from production gameplay builds.

---

# 23. MCP Transport

Begin with local `stdio` MCP operation.

Reasons:

- simple local development;
- no networking requirement;
- small attack surface;
- easy integration with coding agents;
- deterministic one-process test workflows.

Remote HTTP operation can be added only if there is a concrete use case.

MCP must never become a generic shell or filesystem escape hatch.

Expose semantic game operations, not arbitrary code execution.

---

# 24. AI Playtesting Model

AI testing should operate at several levels.

### Level A — Scenario tests

Examples:

> Kill one skeleton.

> Reach the stairs.

> Escape from two melee enemies with less than 30% HP.

> Use a potion to create an environmental effect.

These measure basic playability.

### Level B — Behavioral agents

Agent personas:

```text
optimizer
conservative
aggressive
greedy
explorer
interaction hunter
novice-like
adversarial exploiter
```

These create policy diversity.

### Level C — Semantic QA

Give the tester expected rules:

> Burning oil should ignite flammable wooden objects.

Then have the agent construct situations testing that claim.

### Level D — Exploratory QA

Example:

> Find any sequence of legal actions that generates inconsistent entity ownership, duplicates an item, avoids an intended cost, or creates impossible terrain state.

This is particularly well suited to LLM agents.

---

# 25. What AI Testing Must Not Replace

LLMs should not replace deterministic testing where expectations can be encoded directly.

Normal tests should cover invariants such as:

```text
HP <= max HP

dead actors cannot act

an item cannot occupy two inventories

actors cannot occupy illegal solid tiles

stairs remain reachable

same run inputs reproduce the same state

inventory capacity rules hold

action costs are nonnegative

entity IDs remain unique
```

Suggested testing hierarchy:

```text
unit tests
    ↓
property tests
    ↓
scenario tests
    ↓
replay regressions
    ↓
headless simulations
    ↓
MCP exploratory agents
    ↓
human testing
```

Higher layers answer richer questions but should not duplicate cheaper guarantees.

---

# 26. AI Bug-to-Regression Pipeline

One particularly important workflow should be designed explicitly:

```text
AI tester finds anomaly
        ↓
run automatically retains:
  seed
  version
  state snapshot
  command trace
  observations
        ↓
developer classifies:
  bug
  intended behavior
  desirable emergence
        ↓
bug:
  minimize replay if practical
        ↓
convert into regression fixture
        ↓
fix
        ↓
CI verifies permanently
```

This can become one of Dreadstep's strongest development practices.

---

# 27. Simulation Telemetry

Headless and MCP runs should record structured telemetry.

Examples:

```text
Run
├── result
├── turns
├── deepest floor
├── death cause
│
├── combat
│   ├── damage dealt
│   ├── damage received
│   ├── attacks
│   ├── kills
│   └── escapes
│
├── resources
│   ├── items discovered
│   ├── consumables used
│   ├── items discarded
│   └── equipment changes
│
├── exploration
│   ├── map coverage
│   ├── rooms visited
│   └── secrets discovered
│
└── interaction coverage
    ├── doors manipulated
    ├── items thrown
    ├── environmental kills
    ├── traps triggered
    └── status combinations
```

These metrics diagnose the game.

They should **not** be treated as substitutes for human behavior.

An AI agent ignoring a weapon does not prove that humans will dislike it.

---

# 28. Human Testing Philosophy

Human testing should enter when subjective quality becomes meaningful.

Early development:

```text
correctness -> machines
state-space exploration -> agents
```

Later:

```text
clarity -> humans
feel -> humans
pacing -> humans
atmosphere -> humans
frustration -> humans
surprise -> humans
fun -> humans
```

The question changes over time.

Early:

> Does combat work?

Later:

> Does combat feel dangerous and satisfying?

Early:

> Can the player identify enemy intent?

Later:

> Is that information visually intuitive?

Early:

> Is this item balanced?

Later:

> Is finding this item exciting?

---

# 29. Content Architecture

Content should become increasingly data-driven.

Definitions should use stable IDs:

```text
monster.skeleton_archer
item.iron_sabre
affix.frost
terrain.oil
status.chilled
```

Avoid using display strings as identity.

Data should be Serde-backed and validated during development.

Whether the authored format ultimately becomes RON, TOML, JSON5, or another structured representation matters less than maintaining:

```text
data definition
    ↓
validation
    ↓
typed runtime object
```

Invalid content should fail loudly during development rather than cause mysterious runtime behavior.

---

# 30. Open-Source Strategy

Recommended licensing structure:

```text
Source code
  MIT

Original game data
  MIT or CC0

Original artwork
  CC BY 4.0 or CC0

Original audio
  CC BY 4.0 or CC0

Third-party assets
  explicit per-asset licenses
```

Maintain:

```text
LICENSE
LICENSES/
CREDITS.md
```

Do not allow asset licensing to become implicit.

Before significant public branding or commercial distribution, perform a separate name/trademark/storefront clearance for **Dreadstep**.

---

# 31. Contribution Model

Initially optimize for architectural consistency rather than maximum contributor accessibility.

Later, make adding content relatively easy:

```text
add monster definition
add sprite
add sound
run validator
run scenario tests
submit PR
```

A contributor should not need to understand Bevy ECS scheduling merely to add an enemy.

Useful eventual contributor documentation:

```text
ARCHITECTURE.md
CONTRIBUTING.md
CONTENT_GUIDE.md
MCP.md
REPLAY_FORMAT.md
GAME_RULES.md
```

---

# 32. Platform Strategy

Primary development:

> **macOS first.**

Primary development should not mean macOS-only assumptions.

Core simulation code should remain platform-neutral.

Target CI model:

```text
PR
├── formatting
├── lint
├── unit tests
├── property/scenario tests
└── representative native builds

release
├── macOS
├── Windows
└── Linux
```

Prefer native CI builds for release targets rather than relying exclusively on cross-compilation from macOS.

Bevy officially targets all three desktop platforms and additional platforms as well.

Web should be considered opportunistically after desktop architecture stabilizes, not made an early constraint.

---

# 33. Dependency Philosophy

Keep dependencies intentional.

Likely foundational categories:

```text
Bevy             presentation
rmcp             MCP integration
serde            serialization
tracing          diagnostics
thiserror        domain errors
clap             headless developer CLI
deterministic RNG
property testing
```

Avoid accumulating Bevy plugins before a concrete feature requires them.

Pin major development baselines and upgrade intentionally.

Engine upgrades are milestones, not chores.

---

# 34. ROADMAP

The roadmap intentionally develops **simulation → agent interface → playable game → systemic depth → content → human refinement**.

---

# Milestone 0 — Project Charter

### Version

`v0.0.0`

### Goal

Freeze the core architectural and gameplay constraints before feature implementation.

### Deliverables

- project README;
- MIT license;
- architecture decision record;
- gameplay principles;
- initial Rust workspace;
- CI;
- formatting/lint policies;
- dependency boundaries.

### Required written decisions

```text
turn scheduler
command/event architecture
determinism policy
RNG policy
entity identity
coordinate system
content ID policy
replay versioning
MCP trust boundaries
```

### Exit criterion

A new contributor can explain:

> what belongs in core, protocol, MCP, and Bevy

without ambiguity.

---

# Milestone 1 — The Rules Kernel

### Version

`v0.0.1`

### Goal

Create Dreadstep as a deterministic headless simulation.

### Scope

Implement:

- grid map;
- actors;
- position;
- blocking;
- HP;
- movement;
- basic melee attack;
- death;
- integer action scheduler;
- RNG seed;
- basic enemy chase;
- commands;
- semantic events.

Add a developer CLI:

```text
dreadstep-headless
```

It should be possible to run:

```text
seed
map
player
enemy
commands
```

without initializing Bevy.

### Tests

- deterministic movement;
- blocking;
- combat;
- death;
- actor scheduling;
- replay.

### Exit criterion

The same supported version, seed, scenario, and command sequence always reproduce the same final-state digest.

No graphics are required.

---

# Milestone 2 — Agent Laboratory

### Version

`v0.0.2`

### Goal

Make the simulation natively controllable by AI agents.

### Implement MCP player tools

```text
start_run
observe
legal_actions
act
inspect
get_history
get_replay
```

### Implement initial tester tools

```text
create_scenario
spawn
give_item
set_hp
snapshot
restore
inspect_world
```

### First automated agent scenarios

1. navigate around obstacle;
2. defeat one enemy;
3. reach stairs;
4. survive a small encounter;
5. detect deliberately introduced invalid behavior.

### Exit criterion

An external MCP-capable agent can autonomously play a complete tiny scenario without viewing pixels.

This is the first major architectural proof.

---

# Milestone 3 — First Visible Dreadstep

### Version

`v0.1.0`

### Goal

Turn the existing simulation into a human-playable Bevy application.

### Features

- Bevy window;
- pixel-art map;
- camera;
- player sprite;
- enemy sprites;
- movement animations;
- attack feedback;
- simple HUD;
- event/combat messages;
- keyboard input;
- basic audio placeholders;
- fog of war / field of view.

Human and MCP commands must pass through the same simulation API.

### Asset sourcing workflow

Before committing to a production presentation, evaluate both original/generated assets and
free, legally reusable assets for each pixel-art and audio family:

1. Define the required asset families and technical constraints.
2. Prototype representative original/generated candidates and free reusable candidates.
3. Evaluate visual or audio fit, tactical clarity, editability, animation or cue coverage,
   integration effort, and license obligations.
4. Select original, reused, or mixed sourcing per asset family and record the decision.
5. Normalize selected assets for tile scale, palette, UI readability, audio levels, formats,
   and naming.
6. Preserve source, creator, license, attribution, and modification records when each asset
   enters the repository.

OpenGameArt.org and similar catalogs are discovery sources, not blanket licenses. Inspect each
asset's terms individually; prefer CC0 or CC BY 4.0, and require explicit project approval for
other terms before integration.

Current exploratory gate (2026-08-09):

- [`docs/presentation/asset-evaluation.md`](presentation/asset-evaluation.md) records local-only
  generated candidates, a Kenney Tiny Dungeon CC0 fallback, and Kenney UI Audio CC0 evidence.
- Exact nearest-neighbor samples from the official 16×16 CC0 source support a provisional 32×32
  working renderer tile size; the retained generated sheets remain unconstrained visual direction
  only, and this is not production asset approval.
- This is not a production asset selection: dungeon combat, movement, pickup, detection, and
  environmental audio still require a targeted source or an explicit original-audio decision.
- The reversible renderer-boundary spike is now verified: a typed ordered projection over existing
  scene mirrors preserves complete keyed values, per-kind checked placement, and inventory-unplaced
  semantics without enabling render plugins or loading production media. Actual windowing,
  rendering, asset loading, and playback remain future slices.
- The verified sprite-key presentation boundary derives typed sprite selectors from those complete
  render entries while keeping actual texture loading, render plugins, transforms, and media
  deferred to a later renderer slice.
- The verified render-command presentation boundary derives deterministic typed layer, source-order,
  and optional placement metadata while keeping actual rendering, texture loading, transforms,
  windows, and media deferred to a later renderer slice.
- The verified placeholder ECS render-node boundary reconciles stable nodes from those commands,
  preserving deterministic identity while keeping actual Sprite components, render plugins, windows,
  texture loading, animation, audio, and media deferred to a later renderer slice.
- The verified local-only asset-manifest boundary validates one anchored reference for each typed
  placeholder family and joins those references to stable node metadata without filesystem reads,
  asset handles, or committed pixel-art/audio binaries. Provenance remains in tracked documents;
  production loading and rendering remain a later slice.
- The verified local-only audio cue manifest binds all eight typed cue families to validated root,
  `assets/audio/`, or crate-local `audio/` references and preserves ordered payloads without
  filesystem reads at the headless boundary. The verified desktop adapter requests existing local
  `assets/`-rooted references as non-looping playback effects and records safe missing/unsupported-
  root fallbacks; production audio selection, mastering, and music remain later work.
- The verified desktop tactical HUD polish keeps the existing panel structure while formatting a
  fixed-width health bar, turn/position, remaining-enemy pressure, and optional field-of-view
  summary from authoritative runtime/projection data; production media, localization, and playback
  remain deferred.
- The verified animation polish slice is scoped to a fixed-duration pulse on visible living actor
  placeholders when a new typed animation-cue batch arrives; the runtime replay digest preserves
  distinct accepted events even when cue values match. Movement interpolation, sprite sheets, audio
  playback, and production media remain deferred.
- The verified optional audio-placeholder slice observes replay-digest cue batches and routes each
  existing local reference through non-looping Bevy playback without changing simulation timing or
  smoke evidence; production sound design and media selection remain deferred.
- The verified headless Bevy Sprite API bridge enables only the `bevy_sprite` API feature and joins
  deterministic solid-color Sprite values to stable placeholder nodes with optional 32×32 sizing;
  Sprite/render plugins, texture loading, transforms, windows, playback, and production media remain
  deferred.
- The verified ECS Sprite attachment slice attaches those typed Sprite values to retained placeholder
  node entities,
  keeping Bevy's required components at defaults and preserving stable identity; render plugins,
  transform placement, texture loading, playback, and production media remain deferred.
- The verified Sprite-transform boundary derives ordered map-space translations from checked pixel
  origins without attaching ECS transforms; inventory remains unplaced, fresh missing tile size
  starts unplaced while later removal preserves checked translations, and cameras, windows,
  rendering, playback, and production media remain deferred.
- The verified ECS Sprite-transform attachment boundary applies centered logical-pixel
  `(x + tile_width/2, y + tile_height/2, layer_depth)` values to retained map-node transforms while
  leaving inventory unplaced; anchor variants, cameras, visibility, rendering, playback, and
  production media remain deferred.
- The verified ECS Sprite-depth boundary derives deterministic terrain/ground/actor z-layer values
  from typed render layer while preserving centered x/y placement and inventory default state.
- A verified headless ECS Camera2d attachment boundary adds only Bevy's typed camera marker/default
  orthographic components to the retained disposable camera projection entity; runtime/
  `PresentationCamera` remain authoritative, while window creation, camera viewport policy,
  render plugins, visibility, playback, and production media remain deferred.
- A verified headless ECS Window configuration boundary mirrors the exact validated integer
  logical/physical dimensions and scale onto a disposable `SceneWindow`, exposes a deterministic
  `f32` scale adapter on Bevy's `WindowResolution`, and defers OS/window plugins,
  winit/default-platform integration, render backends, camera policy, visibility, playback, and
  production media.
- A verified headless ECS camera-transform boundary attaches checked centered logical-pixel
  `Transform` values to the retained disposable `SceneCamera` from caller-selected tile extents,
  while deferring viewport policy, OS/window integration, render backends, visibility, playback, and
  production media.
- The first bounded fog-of-war preparation is now verified as a presentation-only field-of-view
  projection: radius-limited cardinal floor traversal with adjacent wall boundaries, retained
  hidden render nodes, and a radius-3 desktop configuration. Core/agent snapshots remain complete;
  persistent exploration memory and richer visibility policy remain future work.

### Content

Approximately:

```text
1 player archetype
3–5 enemy types
1 dungeon theme
5–8 item types
3 small floors
```

### Exit criterion

A human can start the application and finish or die during a short 10–15 minute run.

The same run remains playable headlessly.

---

# Milestone 4 — Tactical Combat

### Version

`v0.2.0`

### Goal

Establish Dreadstep's tactical language.

### Add

- varied action costs;
- ranged attacks;
- weapon reach;
- multiple enemy behavior families;
- retreat behavior;
- line-of-sight behavior;
- basic enemy intent visualization;
- status effects;
- consumables;
- improved death handling;
- combat inspection.

### Representative enemies

```text
slow pursuer
fast melee attacker
ranged kiter
pack creature
support/caster
```

### Exit criterion

Encounter outcomes meaningfully depend on positioning and action selection rather than mostly equipment statistics.

AI agents demonstrate more than one viable encounter strategy.

---

# Milestone 5 — The Living Dungeon

### Version

`v0.3.0`

### Goal

Establish the NetHack-inspired systemic identity.

### Introduce a deliberately small interaction set

For example:

```text
fire
cold
water
oil
wood
corpses
noise
doors
traps
breakables
```

### Add verbs

```text
throw
kick
context interact
```

### Add environmental consequences

Examples:

```text
oil ignition
water freezing
ice melting
burning doors
environmental damage
corpse destruction
noise attraction
pressure-trigger interactions
```

### MCP objective

Create an **interaction hunter** agent suite designed specifically to combine systems unexpectedly.

### Exit criterion

Several interesting tactical solutions arise from interactions that were not individually scripted as encounter solutions.

---

# Milestone 6 — Loot and Build Formation

### Version

`v0.4.0`

### Goal

Establish the Diablo-derived equipment loop.

### Repository status note

The current implementation roadmap has verified a deterministic preparation boundary: one scheduled
actor may equip or unequip one owned opaque item reference, with ordered replacement events and
replay/snapshot projections. The full equipment loop below remains future work; effects, weapon and
armor rules, consumables, affixes, rarity, generation, and inventory UX are not implied by that
preparation slice.

The next bounded preparation slice is now verified: one scheduled actor may consume one owned,
unequipped opaque item instance, removing it with a typed event, standard action-time advancement,
and replay/snapshot evidence. This does not select or implement item effects, stat changes,
capacity, identification, or inventory UX; those remain part of the future equipment loop.

### Implement

- equipment;
- weapon classes;
- armor;
- consumables;
- affixes;
- rarity presentation;
- item comparison;
- procedural item generation;
- inventory UX.

### Target philosophy

Items should change player decisions.

Representative builds should emerge naturally from loot rather than from extensive skill-tree planning.

### Initial content target

Approximately:

```text
20–30 base item definitions
15–25 meaningful affixes
several consumable families
```

These are ceilings to explore, not quotas that must be filled.

### Exit criterion

Two runs with different loot commonly encourage visibly different tactics.

---

# Milestone 7 — Vertical Slice

### Version

`v0.5.0`

### Goal

Produce the first version representing the intended finished experience.

This is the most important pre-alpha milestone.

### Include

- complete opening;
- procedural floors;
- multiple floor themes;
- meaningful item progression;
- systemic interactions;
- mature enemy AI;
- environmental storytelling;
- music;
- polished combat sounds;
- coherent pixel-art direction;
- boss;
- death;
- victory;
- save-and-quit;
- replay export.

Placeholders introduced in Milestone 3 must be replaced or deliberately refined into a coherent
production-quality pixel-art and audio direction before this milestone exits.

### Intended run

Approximately:

```text
30–60 minutes
```

depending on final pacing.

### Human involvement

Begin structured internal human testing here.

Humans are now primarily evaluating:

```text
fun
clarity
pacing
feel
visual hierarchy
audio feedback
frustration
```

### Exit criterion

If development stopped adding systems and only polished/expanded content, the result would already be recognizably **Dreadstep**.

---

# Milestone 8 — Agent QA and Balance Laboratory

### Version

`v0.6.0`

### Goal

Scale the MCP foundation into a systematic testing environment.

MCP is not newly introduced here—it matures here.

### Build

- batch headless runner;
- standardized scenarios;
- tester personas;
- structured run telemetry;
- failure archives;
- replay minimization workflow;
- exploit-hunting prompts;
- interaction-coverage reporting.

### Agent families

```text
optimal-ish
cautious
aggressive
greedy
explorer
interaction hunter
adversarial
```

### Questions to investigate

- Are some enemies trivialized by doorways?
- Can resource loops become infinite?
- Are some affixes never valuable?
- Does a generated room class produce anomalous mortality?
- Can items be duplicated?
- Can status combinations become impossible?
- Can the player bypass required progression?
- Can AI agents discover degenerate strategies?

### Exit criterion

AI-found reproducible defects routinely become automated regression cases.

---

# Milestone 9 — Content Alpha

### Version

`v0.7.0`

### Goal

Expand the vertical slice into a substantially complete game.

Target direction:

```text
~3 dungeon depth bands
~10–12 floors
~20–30 behaviorally distinct enemies
~3 player archetypes
meaningful item families
multiple bosses/minibosses
secrets
rare encounters
environmental set pieces
```

Exact counts remain subordinate to quality.

### Introduce

- full run structure;
- advanced enemies;
- rare items;
- more dungeon templates;
- optional rooms/objectives;
- better item identification if retained;
- difficulty tuning;
- first accessibility options.

### External human testing

Begin limited external alpha.

### Exit criterion

Most major systems and content categories intended for v1.0 exist.

---

# Milestone 10 — Human-Centered Alpha

### Version

`v0.8.0`

### Goal

Shift development emphasis from feature construction toward human experience.

### Primary questions

- Are turns fast enough?
- Is danger legible?
- Can players predict enemy behavior?
- Are interactions discoverable?
- Does inventory interrupt flow?
- Are deaths understandable?
- Is loot exciting?
- Does descent maintain tension?
- Are animations too slow?
- Are audio cues useful?
- Is the UI readable at different resolutions?
- Do players understand what their equipment actually changes?

### Process

Combine:

```text
playtest observation
+
player interviews
+
run telemetry
+
AI diagnostics
```

Do not automatically convert every player request into a feature.

Search for repeated underlying problems.

### Exit criterion

Remaining high-priority issues are primarily tuning, content, usability, performance, accessibility, or defects—not missing foundational systems.

---

# Milestone 11 — Beta / Release Candidate

### Version

`v0.9.0`

### Goal

Prepare Dreadstep as a distributable game rather than merely a development project.

### Focus

- performance;
- save robustness;
- content balance;
- graphics consistency;
- audio consistency;
- input configuration;
- controller evaluation;
- accessibility;
- release packaging;
- macOS distribution;
- Windows build;
- Linux build;
- crash reporting strategy;
- documentation;
- attribution/licensing audit.

### Freeze

Avoid introducing major new mechanics after this milestone except where playtesting reveals a fundamental flaw.

### Exit criterion

Release builds can be produced reproducibly for all supported desktop targets and known critical issues are resolved.

---

# Milestone 12 — Dreadstep 1.0

### Version

`v1.0.0`

### Definition of done

Dreadstep provides:

- a complete dungeon descent;
- meaningful procedural variation;
- fast turn-based tactical combat;
- recognizable enemy behaviors;
- Diablo-inspired item progression;
- systemic environmental interactions;
- strong pixel-art readability;
- atmospheric sound;
- permadeath;
- seeded runs;
- save-and-quit;
- deterministic replay;
- macOS support;
- Windows support;
- Linux support;
- mature headless simulation;
- maintained MCP test interface;
- open-source code;
- documented contribution process.

Most importantly:

> Dreadstep has a recognizable gameplay identity independent of the games that inspired it.

---

# 35. Continuous Workstreams

Some concerns should not wait for dedicated milestones.

## Testing

Begins at Milestone 1 and grows continuously.

## MCP

Begins at Milestone 2 and remains supported continuously.

## Documentation

Architectural decisions should be recorded when made.

## Performance

Measure throughout, optimize when justified.

## Accessibility

Start with basic UX discipline early, then formalize during alpha.

## Licensing

Track every imported or generated asset from the moment it enters the repository. Record its
source, creator, license, attribution, and any modifications so licensing never becomes implicit.

## Cross-platform health

Compile on non-macOS targets regularly rather than discovering portability problems immediately before release.

---

# 36. Scope Guardrails

Dreadstep should explicitly exclude certain attractive distractions from the initial 1.0 scope.

### Not required for 1.0

```text
multiplayer
network services
live-service systems
procedural narrative generation
LLMs inside enemy AI
mobile release
level editor
Steam Workshop
full mod SDK
large overworld
town-management simulation
crafting economy
hundreds of spells
hundreds of monsters
thousands of items
cinematic story campaign
```

Any one of these could consume a substantial fraction of the project's development effort.

---

# 37. Major Risks

## Risk: NetHack-style scope explosion

**Failure mode:** every object gains special interactions with every other object.

**Mitigation:** maintain a formal vocabulary of properties and verbs. Prefer composition over special cases.

---

## Risk: Loot inflation

**Failure mode:** procedural loot produces many meaningless comparisons.

**Mitigation:** cap item complexity and measure whether affixes alter decisions.

---

## Risk: Turn-based combat becomes tedious

**Failure mode:** tactical depth increases the number of uninteresting actions.

**Mitigation:** trivial situations must play quickly; animation and UI should never unnecessarily delay decisions.

---

## Risk: Bevy becomes the architecture

**Failure mode:** simulation logic leaks into ECS/render systems and prevents deterministic testing.

**Mitigation:** keep `dreadstep-core` Bevy-free.

---

## Risk: MCP becomes debug spaghetti

**Failure mode:** tools directly mutate arbitrary internal structures.

**Mitigation:** MCP primarily consumes project-owned protocol APIs; privileged tester operations should remain explicit.

---

## Risk: AI testing produces false confidence

**Failure mode:** good AI metrics are interpreted as proof of fun or human balance.

**Mitigation:** agents test mechanics, robustness, exploration, and approximate strategic behavior. Humans judge experience.

---

## Risk: Content growth outpaces refinement

**Failure mode:** dozens of mediocre enemies appear before five excellent enemies exist.

**Mitigation:** every content family begins small and expands only after demonstrating useful gameplay roles.

---

## Risk: Engine churn

**Failure mode:** frequent Bevy upgrades consume project effort.

**Mitigation:** pin stable project baselines and upgrade intentionally at milestone boundaries.

---

# 38. Project Health Metrics

Development progress should not be measured mainly by lines of code or number of items.

Better indicators include:

### Engineering

```text
deterministic replay success
test coverage of core invariants
AI-discovered regression capture
crash frequency
scenario pass rate
build stability
```

### Gameplay

```text
meaningful tactical alternatives
enemy behavioral differentiation
item/build differentiation
interaction coverage
resource pressure
death explainability
```

### Human experience

```text
decision clarity
input friction
perceived pacing
visual readability
sense of danger
loot excitement
desire to attempt another run
```

---

# 39. The First Development Slice

The first genuinely important implementation target should be extremely small:

```text
###########
#.........#
#..s......#
#.........#
#....@....#
#.........#
#......>..#
###########
```

Where:

```text
@ = player
s = skeleton
> = stairs
```

The player can:

```text
move
wait
attack
die
reach stairs
```

The enemy can:

```text
see
approach
attack
die
```

And the same scenario can be controlled through:

```text
Rust tests
CLI
MCP
```

Only after this works deterministically should the Bevy rendering client become important.

Then add:

```text
door
ranged enemy
potion
fire
throwing
```

one conceptual dimension at a time.

This miniature environment is Dreadstep's equivalent of a laboratory bench.

---

# 40. Architectural Success Criterion

The architecture is working if the following workflow is mundane:

```text
Developer adds:
  frost potion
        ↓
unit tests verify:
  chill rules
        ↓
scenario tests verify:
  potion application
        ↓
MCP interaction agent discovers:
  freeze water
        ↓
Bevy automatically represents:
  thrown potion
  impact
  frozen tile
        ↓
human evaluates:
  whether this is understandable and satisfying
```

That workflow captures the entire project philosophy.

---

# 41. Product Success Criterion

Dreadstep succeeds if a player can recount a run as a sequence of decisions rather than stat checks.

For example:

> I opened a room with two archers and a necromancer. I couldn't safely cross it, so I retreated behind the door, threw oil into the corridor, ignited it, and forced the melee enemies through the fire. But that destroyed the corpses I had planned to exploit with my Gravebound sword, so the next fight became harder.

That story contains:

- positioning;
- enemy behavior;
- resources;
- terrain;
- items;
- systemic interaction;
- consequence.

It is exactly the kind of gameplay Dreadstep should optimize for.

---

# 42. Recommended Immediate Sequence

The first implementation sequence should therefore be:

```text
Project charter
      ↓
Rust workspace
      ↓
domain types
      ↓
deterministic simulation
      ↓
command/event protocol
      ↓
seeded replay
      ↓
headless CLI
      ↓
MCP player interface
      ↓
MCP test interface
      ↓
tiny AI-playable scenario
      ↓
Bevy presentation
      ↓
first human-playable descent
```

The unusual ordering is intentional.

Most indie games build the visible game first and try to automate it later.

Dreadstep should instead build a **game model that happens to have a graphical client**.

That decision is what makes deterministic replay, AI testing, headless simulation, conventional automated testing, alternative clients, and eventually sophisticated modding much easier to support without continuously fighting the engine architecture.

---

# 43. Long-Term Identity

If the project develops successfully, it can ultimately be described along two complementary axes.

## As a game

> **Dreadstep is a fast, gothic tactical roguelike combining deliberate turn-based combat, meaningful randomized equipment, and a compact world of deeply interacting dungeon systems.**

## As an open-source technical project

> **Dreadstep is an agent-native deterministic game simulation whose human client is built with Bevy and whose testing environment is exposed through MCP, allowing conventional tests, autonomous agents, and human players to interact with the same underlying rules engine.**

The second description should remain subordinate to the first from the player's perspective.

Players should care about MCP only if they are curious about how Dreadstep was built.

The engineering exists to make the game better—not to turn the game into a technology demonstration.
