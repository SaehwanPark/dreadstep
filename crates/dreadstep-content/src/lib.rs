//! Validated content definitions for Dreadstep.
//!
//! Authored data will enter through this boundary and become typed domain values. Content
//! may describe rules supported by `dreadstep-core`, but it must not introduce hidden
//! simulation behavior.

#![forbid(unsafe_code)]

use std::{collections::BTreeSet, error::Error, fmt};

use dreadstep_core::{
  Actor, ActorId, ActorKind, AmmunitionAmount, Damage, EnemyBehavior, GridMap, HealingAmount,
  HitPoints, Item, ItemAffix, ItemDefinitionId, ItemEffect, ItemId, ItemRarity, MapError,
  MeleeReach, Position, ThrowableEffect, Tile, WorldError, WorldState,
};

/// Errors raised while validating or building authored content and core-world inputs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentError {
  /// The authored rectangular map is invalid.
  Map(MapError),
  /// Authored world inputs, including actor or item placements, violate a core invariant.
  World(WorldError),
  /// The authored item catalog repeats one opaque definition identity.
  DuplicateItemDefinitionId(ItemDefinitionId),
  /// An authored item placement references a definition absent from its catalog.
  UnknownItemDefinitionId(ItemDefinitionId),
}

impl fmt::Display for ContentError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Map(error) => write!(formatter, "content map error: {error}"),
      Self::World(error) => write!(formatter, "content world error: {error}"),
      Self::DuplicateItemDefinitionId(definition) => write!(
        formatter,
        "content item definition id {} is duplicated",
        definition.value()
      ),
      Self::UnknownItemDefinitionId(definition) => write!(
        formatter,
        "content item definition id {} is not in the catalog",
        definition.value()
      ),
    }
  }
}

impl Error for ContentError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
      Self::Map(error) => Some(error),
      Self::World(error) => Some(error),
      Self::DuplicateItemDefinitionId(_) | Self::UnknownItemDefinitionId(_) => None,
    }
  }
}

impl From<MapError> for ContentError {
  fn from(error: MapError) -> Self {
    Self::Map(error)
  }
}

impl From<WorldError> for ContentError {
  fn from(error: WorldError) -> Self {
    Self::World(error)
  }
}

/// Typed authored input for one ordered catalog of opaque item-definition identities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemCatalogDefinition {
  definitions: Vec<ItemDefinitionId>,
}

impl ItemCatalogDefinition {
  /// Creates authored item-definition references; validation runs in [`Self::build`].
  #[must_use]
  pub const fn new(definitions: Vec<ItemDefinitionId>) -> Self {
    Self { definitions }
  }

  /// Converts authored references into an immutable, validated content catalog.
  ///
  /// Declaration order is preserved. Core remains the owner of item instances and ownership;
  /// this catalog only answers which opaque definition identities the content names.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError::DuplicateItemDefinitionId`] when one identity occurs more than once.
  pub fn build(&self) -> Result<ItemCatalog, ContentError> {
    let mut seen = BTreeSet::new();
    for definition in &self.definitions {
      if !seen.insert(*definition) {
        return Err(ContentError::DuplicateItemDefinitionId(*definition));
      }
    }
    Ok(ItemCatalog {
      definitions: self.definitions.clone(),
    })
  }
}

/// An immutable deterministic catalog of content-known opaque item-definition identities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemCatalog {
  definitions: Vec<ItemDefinitionId>,
}

impl ItemCatalog {
  /// Returns definitions in their authored declaration order.
  #[must_use]
  pub fn definitions(&self) -> &[ItemDefinitionId] {
    &self.definitions
  }

  /// Returns whether content declares the supplied opaque definition identity.
  #[must_use]
  pub fn contains(&self, definition: ItemDefinitionId) -> bool {
    self.definitions.contains(&definition)
  }
}

/// One opaque item instance authored into a starter-floor actor inventory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StarterItemPlacement {
  actor: ActorId,
  item: Item,
}

impl StarterItemPlacement {
  /// Creates an authored placement for one actor and complete opaque item instance.
  #[must_use]
  pub const fn new(actor: ActorId, item: Item) -> Self {
    Self { actor, item }
  }

  /// Returns the actor that receives this item during floor construction.
  #[must_use]
  pub const fn actor(self) -> ActorId {
    self.actor
  }

  /// Returns the complete opaque item instance to insert.
  #[must_use]
  pub const fn item(self) -> Item {
    self.item
  }
}

/// Typed authored input for one rectangular starter floor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StarterFloorDefinition {
  width: u32,
  height: u32,
  tiles: Vec<Tile>,
  actors: Vec<Actor>,
  item_catalog: ItemCatalogDefinition,
  items: Vec<StarterItemPlacement>,
}

impl StarterFloorDefinition {
  /// Creates authored map and actor input; core validates it when [`Self::build`] is called.
  #[must_use]
  pub const fn new(width: u32, height: u32, tiles: Vec<Tile>, actors: Vec<Actor>) -> Self {
    Self {
      width,
      height,
      tiles,
      actors,
      item_catalog: ItemCatalogDefinition::new(Vec::new()),
      items: Vec::new(),
    }
  }

  /// Binds an authored item-definition catalog to this floor.
  ///
  /// The catalog is validated during [`Self::build`], remains content-owned, and is not copied
  /// into the resulting core world.
  #[must_use]
  pub fn with_item_catalog(mut self, catalog: ItemCatalogDefinition) -> Self {
    self.item_catalog = catalog;
    self
  }

  /// Adds ordered opaque item placements to this authored floor.
  ///
  /// Declaration order is preserved within each target actor's inventory. Every placement
  /// definition must be present in the catalog bound by [`Self::with_item_catalog`]; the default
  /// empty catalog therefore accepts no item placements. Core validates actor identities and
  /// global item identity when [`Self::build`] delegates valid placements.
  #[must_use]
  pub fn with_items(mut self, items: Vec<StarterItemPlacement>) -> Self {
    self.items = items;
    self
  }

  /// Converts this authored input into the validated core world.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError::Map`] or [`ContentError::World`] when authored dimensions, tiles,
  /// actor records, or item placements violate core validation rules. Returns
  /// [`ContentError::DuplicateItemDefinitionId`] for a duplicate catalog entry or
  /// [`ContentError::UnknownItemDefinitionId`] when a placement references a definition absent
  /// from the catalog.
  pub fn build(&self) -> Result<WorldState, ContentError> {
    let catalog = self.item_catalog.build()?;
    for placement in &self.items {
      let definition = placement.item().definition();
      if !catalog.contains(definition) {
        return Err(ContentError::UnknownItemDefinitionId(definition));
      }
    }
    let map = GridMap::from_tiles(self.width, self.height, self.tiles.clone())?;
    let mut world = WorldState::new(map, self.actors.clone())?;
    for placement in &self.items {
      world.give_item(placement.actor(), placement.item())?;
    }
    Ok(world)
  }
}

/// Returns the deterministic authored starter-floor definition.
#[must_use]
pub fn starter_floor_definition() -> StarterFloorDefinition {
  StarterFloorDefinition::new(
    7,
    5,
    vec![
      // y = 0
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      // y = 1
      Tile::Wall,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Wall,
      // y = 2
      Tile::Wall,
      Tile::Floor,
      Tile::Wall,
      Tile::Floor,
      Tile::Wall,
      Tile::Floor,
      Tile::Wall,
      // y = 3
      Tile::Wall,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Wall,
      // y = 4
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
    ],
    vec![
      Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(1, 1),
        HitPoints::new(10),
      ),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(5, 1),
        HitPoints::new(3),
      ),
      Actor::with_hit_points(
        ActorId::new(3),
        ActorKind::Enemy,
        Position::new(1, 3),
        HitPoints::new(3),
      ),
      Actor::with_hit_points(
        ActorId::new(4),
        ActorKind::Enemy,
        Position::new(5, 3),
        HitPoints::new(3),
      ),
    ],
  )
}

/// Builds the validated deterministic authored starter floor.
///
/// # Errors
///
/// Returns [`ContentError`] when the authored definition fails core map or world validation.
pub fn starter_floor() -> Result<WorldState, ContentError> {
  starter_floor_definition().build()
}

/// Builds the authored chilled-status showcase floor.
///
/// # Errors
///
/// Returns [`ContentError`] if the authored starter floor cannot be validated.
pub fn chill_trap_floor() -> Result<WorldState, ContentError> {
  let mut world = starter_floor()?;
  world.set_tile(Position::new(2, 1), Tile::ChillTrap);
  Ok(world)
}

/// Builds the authored reclosable-door showcase floor.
///
/// # Errors
///
/// Returns [`ContentError`] if the authored starter floor cannot be validated.
pub fn reclosable_door_floor() -> Result<WorldState, ContentError> {
  let mut world = starter_floor()?;
  world.set_tile(Position::new(2, 1), Tile::Door);
  Ok(world)
}

/// Returns a deterministic seeded corridor-floor definition.
///
/// This procedural-content boundary varies authored terrain, enemy durability, two ordered
/// generated starter-loot equipment choices, one consumable inventory choice, and two ground
/// equipment choices with bounded seed/depth-derived affix tiers. The returned definition still
/// delegates all map, catalog, and actor/item validation to core when
/// [`StarterFloorDefinition::build`] is called. `depth` is one-based in authored callers, but zero
/// remains a valid deterministic fixture value.
#[must_use]
pub fn procedural_floor_definition(seed: u64, depth: u32) -> StarterFloorDefinition {
  const WIDTH: u32 = 13;
  const HEIGHT: u32 = 9;

  let mut tiles = vec![Tile::Floor; (WIDTH * HEIGHT) as usize];
  for y in 0..HEIGHT {
    for x in 0..WIDTH {
      if x == 0 || x + 1 == WIDTH || y == 0 || y + 1 == HEIGHT {
        let index = (y * WIDTH + x) as usize;
        tiles[index] = Tile::Wall;
      }
    }
  }

  for partition_x in [3_u32, 6, 9] {
    let gap_y = procedural_partition_gap(seed, depth, partition_x);
    for y in 1..(HEIGHT - 1) {
      if y != gap_y {
        let index = (y * WIDTH + partition_x) as usize;
        tiles[index] = Tile::Wall;
      }
    }
  }

  let enemy_hit_points = HitPoints::new(3 + depth.min(5) as u16);
  let mut definition = StarterFloorDefinition::new(
    WIDTH,
    HEIGHT,
    tiles,
    vec![
      Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(1, 1),
        HitPoints::new(10),
      ),
      Actor::with_melee_reach_and_behavior(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(11, 1),
        enemy_hit_points,
        MeleeReach::DEFAULT,
        procedural_enemy_behavior(seed, depth, 0),
      ),
      Actor::with_melee_reach_and_behavior(
        ActorId::new(3),
        ActorKind::Enemy,
        Position::new(11, 7),
        enemy_hit_points,
        MeleeReach::DEFAULT,
        procedural_enemy_behavior(seed, depth, 1),
      ),
      Actor::with_melee_reach_and_behavior(
        ActorId::new(4),
        ActorKind::Enemy,
        Position::new(1, 7),
        enemy_hit_points,
        MeleeReach::DEFAULT,
        procedural_enemy_behavior(seed, depth, 2),
      ),
    ],
  )
  .with_item_catalog(starter_item_catalog_definition());
  definition = definition.with_items(vec![
    StarterItemPlacement::new(ActorId::new(1), procedural_loot(seed, depth, 0)),
    StarterItemPlacement::new(ActorId::new(1), procedural_loot(seed, depth, 1)),
    StarterItemPlacement::new(ActorId::new(1), procedural_consumable(seed, depth, 3)),
  ]);
  definition
}

fn procedural_loot(seed: u64, depth: u32, variant: u64) -> Item {
  let mixed = procedural_loot_mix(seed, depth, variant);
  let item_id = procedural_item_id(mixed, variant);
  let rarity = procedural_rarity(mixed, depth);
  let affix_amount = procedural_affix_amount(mixed, depth);
  let role_seed = procedural_loot_mix(seed, depth, 0);
  let role = (role_seed / 6 + variant) % 4;
  let (item, affix) = match role {
    0 => (
      Item::with_equipment_damage(item_id, ItemDefinitionId::new(1), Damage::new(1)),
      ItemAffix::MeleeDamage {
        amount: affix_amount,
      },
    ),
    1 => (
      Item::with_equipment_effect(item_id, ItemDefinitionId::new(4), MeleeReach::TWO),
      ItemAffix::MeleeDamage {
        amount: affix_amount,
      },
    ),
    2 => (
      Item::with_damage_reduction(item_id, ItemDefinitionId::new(6), Damage::new(1)),
      ItemAffix::DamageReduction {
        amount: affix_amount,
      },
    ),
    _ => (
      Item::with_ranged_damage(item_id, ItemDefinitionId::new(7), Damage::new(1)),
      ItemAffix::RangedDamage {
        amount: affix_amount,
      },
    ),
  };
  item.with_affix(affix).with_rarity(rarity)
}

fn procedural_consumable(seed: u64, depth: u32, variant: u64) -> Item {
  let mixed = procedural_loot_mix(seed, depth, variant);
  let item_id = procedural_item_id(mixed, variant);
  let rarity = procedural_rarity(mixed, depth);
  let amount = procedural_consumable_amount(mixed, depth);
  let effect = if mixed.is_multiple_of(2) {
    ItemEffect::Heal {
      amount: HealingAmount::new(amount).expect("procedural healing amount should be positive"),
    }
  } else {
    ItemEffect::RestoreAmmunition {
      amount: AmmunitionAmount::new(amount)
        .expect("procedural ammunition amount should be positive"),
    }
  };
  let definition = if mixed.is_multiple_of(2) {
    ItemDefinitionId::new(2)
  } else {
    ItemDefinitionId::new(3)
  };
  Item::with_effect(item_id, definition, effect).with_rarity(rarity)
}

fn procedural_ground_loot(seed: u64, depth: u32, variant: u64) -> Item {
  let mixed = procedural_loot_mix(seed, depth, variant);
  if depth < 2 || (mixed >> 32).is_multiple_of(3) {
    procedural_consumable(seed, depth, variant)
  } else {
    procedural_loot(seed, depth, variant)
  }
}

fn procedural_consumable_amount(mixed: u64, depth: u32) -> u16 {
  if depth >= 3 {
    2
  } else {
    1 + u16::try_from((mixed / 48) % 2).expect("procedural consumable potency fits")
  }
}

fn procedural_affix_amount(mixed: u64, depth: u32) -> Damage {
  let tier = if depth >= 3 {
    1
  } else {
    u16::try_from((mixed / 24) % 2).expect("affix tier fits")
  };
  Damage::new(1 + tier)
}

fn procedural_item_id(mixed: u64, variant: u64) -> ItemId {
  let low_bits = mixed & u64::from(u32::MAX);
  let variant_bits = u32::try_from(variant & 0x3).expect("procedural loot variant fits") << 30;
  ItemId::new(
    0x8000_0000
      | variant_bits
      | (u32::try_from(low_bits).expect("masked procedural item identity fits") & 0x3fff_ffff),
  )
}

fn procedural_rarity(mixed: u64, depth: u32) -> ItemRarity {
  let rarity = match mixed % 6 {
    0 => ItemRarity::Rare,
    1 | 2 => ItemRarity::Magic,
    _ => ItemRarity::Common,
  };
  if depth >= 3 && rarity == ItemRarity::Common {
    ItemRarity::Magic
  } else {
    rarity
  }
}

fn procedural_loot_mix(seed: u64, depth: u32, variant: u64) -> u64 {
  seed
    .wrapping_add(u64::from(depth).wrapping_mul(0x9E37_79B9_7F4A_7C15))
    .wrapping_add(variant.wrapping_mul(0xA24B_AED4_963E_E407))
    .wrapping_add(0xD1B5_4A32_D192_ED03)
    .rotate_left(17)
    .wrapping_mul(0xBF58_476D_1CE4_E5B9)
}

fn procedural_enemy_behavior(seed: u64, depth: u32, slot: u64) -> EnemyBehavior {
  if depth < 2 {
    return match slot {
      0 | 1 => EnemyBehavior::Pursuer,
      _ => EnemyBehavior::Kiter,
    };
  }

  let mixed = procedural_loot_mix(seed, depth, 8 + slot);
  match (mixed >> 32) % 4 {
    0 => EnemyBehavior::Pursuer,
    1 => EnemyBehavior::Kiter,
    2 => EnemyBehavior::Scavenger,
    _ => EnemyBehavior::Zombie,
  }
}

fn procedural_partition_gap(seed: u64, depth: u32, partition_x: u32) -> u32 {
  let mixed = seed
    .wrapping_add(u64::from(depth).wrapping_mul(0x9E37_79B9_7F4A_7C15))
    .wrapping_add(u64::from(partition_x).wrapping_mul(0xBF58_476D_1CE4_E5B9));
  (mixed % 7) as u32 + 1
}

/// Builds a validated deterministic seeded corridor floor with generated ground items placed at
/// the first two enemies' authored positions for the existing pickup/drop rules to consume later.
///
/// # Errors
///
/// Returns [`ContentError`] if generated content violates a core map or world invariant.
pub fn procedural_floor(seed: u64, depth: u32) -> Result<WorldState, ContentError> {
  let mut world = procedural_floor_definition(seed, depth).build()?;
  let ground_item = procedural_loot(seed, depth, 2);
  world.give_item(ActorId::new(2), ground_item)?;
  world.drop_item(ActorId::new(2), ground_item.id())?;
  let second_ground_item = procedural_loot(seed, depth, 4);
  world.give_item(ActorId::new(3), second_ground_item)?;
  world.drop_item(ActorId::new(3), second_ground_item.id())?;
  let third_ground_item = procedural_ground_loot(seed, depth, 5);
  world.give_item(ActorId::new(4), third_ground_item)?;
  world.drop_item(ActorId::new(4), third_ground_item.id())?;
  Ok(world)
}

/// Returns the deterministic authored starter-item scenario definition.
///
/// This scenario is separate from [`starter_floor_definition`], which intentionally remains
/// item-free. It binds the shared starter catalog and uses interleaved placements to provide a
/// stable content fixture for adapters and tests, including authored consumables and closed
/// equipment effects.
#[must_use]
pub fn starter_item_floor_definition() -> StarterFloorDefinition {
  let mut definition = starter_floor_definition();
  definition.tiles[(3 * definition.width + 4) as usize] = Tile::Breakable;
  definition.actors[3] = Actor::with_melee_reach_and_behavior(
    ActorId::new(4),
    ActorKind::Enemy,
    Position::new(5, 3),
    HitPoints::new(3),
    MeleeReach::DEFAULT,
    EnemyBehavior::Brute,
  );
  definition
    .with_item_catalog(starter_item_catalog_definition())
    .with_items(vec![
      StarterItemPlacement::new(
        ActorId::new(1),
        Item::with_effect(
          ItemId::new(101),
          ItemDefinitionId::new(2),
          ItemEffect::Heal {
            amount: HealingAmount::THREE,
          },
        ),
      ),
      StarterItemPlacement::new(
        ActorId::new(1),
        Item::with_equipment_effect(ItemId::new(103), ItemDefinitionId::new(4), MeleeReach::TWO),
      ),
      StarterItemPlacement::new(
        ActorId::new(1),
        Item::with_throwable_effect(
          ItemId::new(104),
          ItemDefinitionId::new(5),
          ThrowableEffect::Chill,
        ),
      ),
      StarterItemPlacement::new(
        ActorId::new(2),
        Item::with_equipment_damage(ItemId::new(100), ItemDefinitionId::new(1), Damage::new(1)),
      ),
      StarterItemPlacement::new(
        ActorId::new(2),
        Item::with_damage_reduction(ItemId::new(105), ItemDefinitionId::new(6), Damage::new(1)),
      ),
      StarterItemPlacement::new(
        ActorId::new(2),
        Item::with_ranged_damage(ItemId::new(106), ItemDefinitionId::new(7), Damage::new(1)),
      ),
      StarterItemPlacement::new(
        ActorId::new(1),
        Item::with_effect(
          ItemId::new(102),
          ItemDefinitionId::new(3),
          ItemEffect::RestoreAmmunition {
            amount: AmmunitionAmount::TWO,
          },
        ),
      ),
    ])
}

/// Builds the validated deterministic authored starter-item scenario.
///
/// # Errors
///
/// Returns [`ContentError`] when the authored scenario fails catalog, map, world, or item
/// placement validation.
pub fn starter_item_floor() -> Result<WorldState, ContentError> {
  starter_item_floor_definition().build()
}

/// Builds the desktop-authored item showcase with a reachable closed door beside the player.
///
/// The underlying item fixture remains stable for adapter tests; this presentation fixture adds
/// the authored door, Frostcaster, and Blocker identities exercised by the desktop showcase.
///
/// # Errors
///
/// Returns [`ContentError`] when the authored item floor cannot be validated.
pub fn starter_item_showcase_floor() -> Result<WorldState, ContentError> {
  let mut definition = starter_item_floor_definition();
  definition.tiles[9] = Tile::Door;
  definition.actors[2] = Actor::with_melee_reach_and_behavior(
    ActorId::new(3),
    ActorKind::Enemy,
    Position::new(1, 3),
    HitPoints::new(3),
    MeleeReach::DEFAULT,
    EnemyBehavior::Frostcaster,
  );
  definition.actors[3] = Actor::with_melee_reach_and_behavior(
    ActorId::new(4),
    ActorKind::Enemy,
    Position::new(3, 3),
    HitPoints::new(3),
    MeleeReach::DEFAULT,
    EnemyBehavior::Blocker,
  );
  definition.build()
}

/// Returns the deterministic authored starter item-definition references.
#[must_use]
pub fn starter_item_catalog_definition() -> ItemCatalogDefinition {
  ItemCatalogDefinition::new(vec![
    ItemDefinitionId::new(1),
    ItemDefinitionId::new(2),
    ItemDefinitionId::new(3),
    ItemDefinitionId::new(4),
    ItemDefinitionId::new(5),
    ItemDefinitionId::new(6),
    ItemDefinitionId::new(7),
  ])
}

/// Builds the validated deterministic starter item-definition catalog.
///
/// # Errors
///
/// Returns [`ContentError::DuplicateItemDefinitionId`] if the authored starter references are
/// accidentally repeated.
pub fn starter_item_catalog() -> Result<ItemCatalog, ContentError> {
  starter_item_catalog_definition().build()
}
