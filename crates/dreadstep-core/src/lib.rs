//! Deterministic game rules and domain state for Dreadstep.
//!
//! This crate owns semantic commands, events, and domain errors. It stays independent of
//! presentation, transport, storage, wall-clock time, and operating-system services so
//! the same rules can serve tests, headless tools, agents, and human-facing clients.

#![forbid(unsafe_code)]

use std::{collections::BTreeMap, fmt};

/// A stable identity for an actor in a world.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ActorId(u32);

impl ActorId {
  /// Creates an actor identity from its stable numeric value.
  #[must_use]
  pub const fn new(value: u32) -> Self {
    Self(value)
  }

  /// Returns the stable numeric value of this identity.
  #[must_use]
  pub const fn value(self) -> u32 {
    self.0
  }
}

/// A globally unique identity for one opaque item instance.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ItemId(u32);

impl ItemId {
  /// Creates an item identity from its stable numeric value.
  #[must_use]
  pub const fn new(value: u32) -> Self {
    Self(value)
  }

  /// Returns the stable numeric value of this identity.
  #[must_use]
  pub const fn value(self) -> u32 {
    self.0
  }
}

/// An opaque content reference for an item instance.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ItemDefinitionId(u32);

impl ItemDefinitionId {
  /// Creates an item-definition reference from its stable numeric value.
  #[must_use]
  pub const fn new(value: u32) -> Self {
    Self(value)
  }

  /// Returns the stable numeric value of this definition reference.
  #[must_use]
  pub const fn value(self) -> u32 {
    self.0
  }
}

/// One opaque item instance in world state, either in an actor inventory or on the ground.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Item {
  id: ItemId,
  definition: ItemDefinitionId,
}

impl Item {
  /// Creates an item instance with an explicit identity and content reference.
  #[must_use]
  pub const fn new(id: ItemId, definition: ItemDefinitionId) -> Self {
    Self { id, definition }
  }

  /// Returns the globally unique instance identity.
  #[must_use]
  pub const fn id(self) -> ItemId {
    self.id
  }

  /// Returns the opaque content reference.
  #[must_use]
  pub const fn definition(self) -> ItemDefinitionId {
    self.definition
  }
}

/// One deterministic stack of opaque items at a map position.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundItemStack {
  position: Position,
  items: Vec<Item>,
}

impl GroundItemStack {
  fn new(position: Position, item: Item) -> Self {
    Self {
      position,
      items: vec![item],
    }
  }

  /// Returns the map position of this stack.
  #[must_use]
  pub const fn position(&self) -> Position {
    self.position
  }

  /// Returns items in deterministic insertion order.
  #[must_use]
  pub fn items(&self) -> &[Item] {
    &self.items
  }
}

/// A tile coordinate in the simulation's row-major grid.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Position {
  x: i32,
  y: i32,
}

impl Position {
  /// Creates a coordinate from its horizontal and vertical components.
  #[must_use]
  pub const fn new(x: i32, y: i32) -> Self {
    Self { x, y }
  }

  /// Returns the horizontal component.
  #[must_use]
  pub const fn x(self) -> i32 {
    self.x
  }

  /// Returns the vertical component.
  #[must_use]
  pub const fn y(self) -> i32 {
    self.y
  }

  /// Returns the adjacent coordinate in the supplied cardinal direction.
  #[must_use]
  pub const fn translated(self, direction: Direction) -> Self {
    let (dx, dy) = direction.delta();
    Self {
      x: self.x.saturating_add(dx),
      y: self.y.saturating_add(dy),
    }
  }
}

/// The four movement directions supported by the first rules-kernel slice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Direction {
  /// One row toward decreasing vertical coordinates.
  North,
  /// One row toward increasing vertical coordinates.
  South,
  /// One column toward decreasing horizontal coordinates.
  West,
  /// One column toward increasing horizontal coordinates.
  East,
}

impl Direction {
  const fn delta(self) -> (i32, i32) {
    match self {
      Self::North => (0, -1),
      Self::South => (0, 1),
      Self::West => (-1, 0),
      Self::East => (1, 0),
    }
  }
}

/// The terrain occupying one grid cell.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Tile {
  /// A cell that actors may enter when it is not occupied.
  Floor,
  /// A walkable cell that blocks ranged line of sight.
  Cover,
  /// A cell that blocks movement.
  Wall,
}

impl Tile {
  /// Returns whether this tile permits an actor to enter it.
  #[must_use]
  pub const fn is_walkable(self) -> bool {
    matches!(self, Self::Floor | Self::Cover)
  }

  /// Returns whether this tile blocks a ranged line of sight.
  #[must_use]
  pub const fn blocks_ranged_line_of_sight(self) -> bool {
    matches!(self, Self::Cover | Self::Wall)
  }
}

/// Errors produced while constructing a rectangular grid map.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MapError {
  /// The width is zero, so no columns exist.
  ZeroWidth,
  /// The height is zero, so no rows exist.
  ZeroHeight,
  /// The dimensions cannot be represented as an in-memory tile buffer.
  TooLarge {
    /// The requested width.
    width: u32,
    /// The requested height.
    height: u32,
  },
  /// A dimension would contain coordinates outside the signed position domain.
  CoordinateRange {
    /// The requested width.
    width: u32,
    /// The requested height.
    height: u32,
  },
  /// The tile buffer does not match the dimensions.
  TileCountMismatch {
    /// The number of tiles implied by the dimensions.
    expected: usize,
    /// The number of tiles supplied by the caller.
    actual: usize,
  },
}

impl fmt::Display for MapError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::ZeroWidth => formatter.write_str("map width must be greater than zero"),
      Self::ZeroHeight => formatter.write_str("map height must be greater than zero"),
      Self::TooLarge { width, height } => {
        write!(formatter, "map dimensions {width}x{height} are too large")
      }
      Self::CoordinateRange { width, height } => write!(
        formatter,
        "map dimensions {width}x{height} exceed the signed position range"
      ),
      Self::TileCountMismatch { expected, actual } => {
        write!(
          formatter,
          "map needs {expected} tiles but received {actual}"
        )
      }
    }
  }
}

impl std::error::Error for MapError {}

/// A finite, row-major rectangular map of terrain tiles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GridMap {
  width: u32,
  height: u32,
  tiles: Vec<Tile>,
}

impl GridMap {
  /// Creates a map filled with one terrain tile.
  ///
  /// # Errors
  ///
  /// Returns [`MapError::ZeroWidth`], [`MapError::ZeroHeight`], [`MapError::CoordinateRange`],
  /// or [`MapError::TooLarge`] when the requested dimensions cannot describe a valid buffer.
  pub fn filled(width: u32, height: u32, tile: Tile) -> Result<Self, MapError> {
    let tile_count = tile_count(width, height)?;
    Ok(Self {
      width,
      height,
      tiles: vec![tile; tile_count],
    })
  }

  /// Creates a map from row-major tiles, validating the dimensions and buffer length.
  ///
  /// # Errors
  ///
  /// Returns a [`MapError`] when a dimension is zero, the dimensions are too large, or the
  /// supplied tile count does not match the dimensions.
  pub fn from_tiles(width: u32, height: u32, tiles: Vec<Tile>) -> Result<Self, MapError> {
    let expected = tile_count(width, height)?;
    if tiles.len() != expected {
      return Err(MapError::TileCountMismatch {
        expected,
        actual: tiles.len(),
      });
    }
    Ok(Self {
      width,
      height,
      tiles,
    })
  }

  /// Returns the number of columns in the map.
  #[must_use]
  pub const fn width(&self) -> u32 {
    self.width
  }

  /// Returns the number of rows in the map.
  #[must_use]
  pub const fn height(&self) -> u32 {
    self.height
  }

  /// Returns the validated terrain tiles in row-major order.
  ///
  /// The returned slice is immutable, so presentation and content adapters can inspect map
  /// terrain without bypassing the map's construction invariants or mutating core state.
  #[must_use]
  pub fn tiles(&self) -> &[Tile] {
    &self.tiles
  }

  /// Returns whether a position is inside the map bounds.
  #[must_use]
  pub fn in_bounds(&self, position: Position) -> bool {
    position.x >= 0
      && position.y >= 0
      && u32::try_from(position.x).is_ok_and(|x| x < self.width)
      && u32::try_from(position.y).is_ok_and(|y| y < self.height)
  }

  /// Returns the terrain at a position, or `None` outside the map.
  #[must_use]
  pub fn tile_at(&self, position: Position) -> Option<Tile> {
    self
      .in_bounds(position)
      .then(|| self.tiles[self.index(position)])
  }

  /// Returns whether an actor may enter a position based on terrain alone.
  #[must_use]
  pub fn is_walkable(&self, position: Position) -> bool {
    self.tile_at(position).is_some_and(Tile::is_walkable)
  }

  fn index(&self, position: Position) -> usize {
    let x = usize::try_from(position.x).expect("in-bounds positions have non-negative x");
    let y = usize::try_from(position.y).expect("in-bounds positions have non-negative y");
    let width = usize::try_from(self.width).expect("map width must fit usize");
    y * width + x
  }
}

fn tile_count(width: u32, height: u32) -> Result<usize, MapError> {
  if width == 0 {
    return Err(MapError::ZeroWidth);
  }
  if height == 0 {
    return Err(MapError::ZeroHeight);
  }
  if width > i32::MAX as u32 || height > i32::MAX as u32 {
    return Err(MapError::CoordinateRange { width, height });
  }
  usize::try_from(u64::from(width) * u64::from(height))
    .map_err(|_| MapError::TooLarge { width, height })
}

/// The kind of actor represented in the world.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ActorKind {
  /// The player-controlled actor.
  Player,
  /// An actor controlled by the simulation or a future AI adapter.
  Enemy,
}

/// The deterministic terminal state derived from retained actor records.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RunOutcome {
  /// The run has not reached a terminal condition.
  InProgress,
  /// The player is dead.
  Defeat,
  /// At least one enemy exists and every enemy is dead.
  Victory,
}

/// An integer timestamp used by the deterministic action scheduler.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ActionTime(u64);

impl ActionTime {
  /// Creates an action timestamp from its numeric value.
  #[must_use]
  pub const fn new(value: u64) -> Self {
    Self(value)
  }

  /// Returns the numeric timestamp.
  #[must_use]
  pub const fn value(self) -> u64 {
    self.0
  }

  fn checked_add(self, cost: ActionCost) -> Option<Self> {
    self.0.checked_add(cost.0).map(Self)
  }
}

/// A non-negative integer action duration.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ActionCost(u64);

impl ActionCost {
  /// The fixed cost used by movement, waiting, melee, chase, and item actions.
  pub const STANDARD: Self = Self(1);

  /// The fixed cost used by the bounded ranged attack.
  pub const RANGED: Self = Self(2);

  /// Creates an action cost from its numeric value.
  #[must_use]
  pub const fn new(value: u64) -> Self {
    Self(value)
  }

  /// Returns the numeric cost.
  #[must_use]
  pub const fn value(self) -> u64 {
    self.0
  }
}

/// The current integer hit points of an actor.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HitPoints(u16);

impl HitPoints {
  /// Creates hit points from a numeric value. Zero represents a dead actor.
  #[must_use]
  pub const fn new(value: u16) -> Self {
    Self(value)
  }

  /// Returns the numeric hit-point value.
  #[must_use]
  pub const fn value(self) -> u16 {
    self.0
  }

  /// Returns whether these hit points represent a living actor.
  #[must_use]
  pub const fn is_alive(self) -> bool {
    self.0 > 0
  }

  fn reduced_by(self, damage: Damage) -> Self {
    Self(self.0.saturating_sub(damage.0))
  }
}

/// A typed amount of damage applied by an attack.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Damage(u16);

impl Damage {
  /// The fixed damage dealt by the basic melee command.
  pub const MELEE: Self = Self(1);

  /// The fixed damage dealt by the bounded ranged command.
  pub const RANGED: Self = Self(1);

  /// Creates damage from a numeric value.
  #[must_use]
  pub const fn new(value: u16) -> Self {
    Self(value)
  }

  /// Returns the numeric damage value.
  #[must_use]
  pub const fn value(self) -> u16 {
    self.0
  }
}

/// A non-zero Manhattan distance at which an actor may perform melee attacks.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MeleeReach(u8);

impl MeleeReach {
  /// The default adjacent melee reach.
  pub const DEFAULT: Self = Self(1);

  /// Creates a melee reach, rejecting zero because it cannot target another tile.
  #[must_use]
  pub const fn new(value: u8) -> Option<Self> {
    if value == 0 { None } else { Some(Self(value)) }
  }

  /// Returns the numeric Manhattan reach.
  #[must_use]
  pub const fn value(self) -> u8 {
    self.0
  }
}

impl Default for MeleeReach {
  fn default() -> Self {
    Self::DEFAULT
  }
}

/// A stable, non-cryptographic digest used for deterministic regression evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StateDigest(u64);

impl StateDigest {
  /// Returns the numeric digest value.
  #[must_use]
  pub const fn value(self) -> u64 {
    self.0
  }
}

const FNV_OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;

struct StableHasher {
  state: u64,
}

impl StableHasher {
  const fn new() -> Self {
    Self {
      state: FNV_OFFSET_BASIS,
    }
  }

  fn write_bytes(&mut self, bytes: &[u8]) {
    for byte in bytes {
      self.state ^= u64::from(*byte);
      self.state = self.state.wrapping_mul(FNV_PRIME);
    }
  }

  fn write_u8(&mut self, value: u8) {
    self.write_bytes(&[value]);
  }

  fn write_u16(&mut self, value: u16) {
    self.write_bytes(&value.to_le_bytes());
  }

  fn write_u32(&mut self, value: u32) {
    self.write_bytes(&value.to_le_bytes());
  }

  fn write_i32(&mut self, value: i32) {
    self.write_bytes(&value.to_le_bytes());
  }

  fn write_u64(&mut self, value: u64) {
    self.write_bytes(&value.to_le_bytes());
  }

  const fn finish(self) -> StateDigest {
    StateDigest(self.state)
  }
}

/// An actor with a stable identity, kind, position, hit points, melee reach, ranged ammunition,
/// inventory, optional equipment, and next ready time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Actor {
  id: ActorId,
  kind: ActorKind,
  position: Position,
  hit_points: HitPoints,
  melee_reach: MeleeReach,
  inventory: Vec<Item>,
  equipped: Option<ItemId>,
  ranged_ammo: u16,
  ready_at: ActionTime,
}

impl Actor {
  /// The fixed capacity restored by the deterministic reload command.
  pub const RANGED_AMMO_CAPACITY: u16 = 3;

  /// The default number of ranged shots available to a newly created actor.
  pub const DEFAULT_RANGED_AMMO: u16 = Self::RANGED_AMMO_CAPACITY;

  /// Creates an actor that is ready at the beginning of the world timeline.
  #[must_use]
  pub const fn new(id: ActorId, kind: ActorKind, position: Position) -> Self {
    Self::with_hit_points(id, kind, position, HitPoints::new(10))
  }

  /// Creates an actor with explicit hit points that is ready at the beginning of the timeline.
  #[must_use]
  pub const fn with_hit_points(
    id: ActorId,
    kind: ActorKind,
    position: Position,
    hit_points: HitPoints,
  ) -> Self {
    Self::with_ranged_ammo(id, kind, position, hit_points, Self::DEFAULT_RANGED_AMMO)
  }

  /// Creates an actor with explicit hit points and ranged ammunition.
  #[must_use]
  pub const fn with_ranged_ammo(
    id: ActorId,
    kind: ActorKind,
    position: Position,
    hit_points: HitPoints,
    ranged_ammo: u16,
  ) -> Self {
    Self::with_ranged_ammo_and_melee_reach(
      id,
      kind,
      position,
      hit_points,
      ranged_ammo,
      MeleeReach::DEFAULT,
    )
  }

  /// Creates an actor with explicit hit points, melee reach, and ranged ammunition.
  #[must_use]
  pub const fn with_ranged_ammo_and_melee_reach(
    id: ActorId,
    kind: ActorKind,
    position: Position,
    hit_points: HitPoints,
    ranged_ammo: u16,
    melee_reach: MeleeReach,
  ) -> Self {
    Self {
      id,
      kind,
      position,
      hit_points,
      melee_reach,
      inventory: Vec::new(),
      equipped: None,
      ranged_ammo,
      ready_at: ActionTime::new(0),
    }
  }

  /// Creates an actor with explicit hit points and melee reach using default ranged ammunition.
  #[must_use]
  pub const fn with_melee_reach(
    id: ActorId,
    kind: ActorKind,
    position: Position,
    hit_points: HitPoints,
    melee_reach: MeleeReach,
  ) -> Self {
    Self::with_ranged_ammo_and_melee_reach(
      id,
      kind,
      position,
      hit_points,
      Self::DEFAULT_RANGED_AMMO,
      melee_reach,
    )
  }

  /// Returns this actor's stable identity.
  #[must_use]
  pub const fn id(&self) -> ActorId {
    self.id
  }

  /// Returns this actor's kind.
  #[must_use]
  pub const fn kind(&self) -> ActorKind {
    self.kind
  }

  /// Returns this actor's current position.
  #[must_use]
  pub const fn position(&self) -> Position {
    self.position
  }

  /// Returns this actor's current hit points.
  #[must_use]
  pub const fn hit_points(&self) -> HitPoints {
    self.hit_points
  }

  /// Returns this actor's non-zero Manhattan melee reach.
  #[must_use]
  pub const fn melee_reach(&self) -> MeleeReach {
    self.melee_reach
  }

  /// Returns this actor's items in deterministic insertion order.
  #[must_use]
  pub fn inventory(&self) -> &[Item] {
    &self.inventory
  }

  /// Returns the optional equipped item identity, which always points into this inventory.
  #[must_use]
  pub const fn equipped_item(&self) -> Option<ItemId> {
    self.equipped
  }

  /// Returns the number of ranged shots remaining for this actor.
  #[must_use]
  pub const fn ranged_ammo(&self) -> u16 {
    self.ranged_ammo
  }

  /// Returns whether this actor can be scheduled, targeted, or moved around.
  #[must_use]
  pub const fn is_alive(&self) -> bool {
    self.hit_points.is_alive()
  }

  /// Returns the timestamp when this actor can next act.
  #[must_use]
  pub const fn ready_at(&self) -> ActionTime {
    self.ready_at
  }
}

/// A command interpreted by the deterministic rules kernel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
  /// Move one tile in a cardinal direction.
  Move {
    /// The actor issuing the command.
    actor: ActorId,
    /// The direction of movement.
    direction: Direction,
  },
  /// Spend one standard action without changing position.
  Wait {
    /// The actor issuing the command.
    actor: ActorId,
  },
  /// Make a fixed basic melee attack against an actor within melee reach.
  Attack {
    /// The actor issuing the attack.
    actor: ActorId,
    /// The actor being targeted within melee reach.
    target: ActorId,
  },
  /// Make a fixed ranged attack against a target two or three tiles away.
  RangedAttack {
    /// The actor issuing the attack.
    actor: ActorId,
    /// The living actor being targeted.
    target: ActorId,
  },
  /// Move an enemy one deterministic step toward a living target.
  Chase {
    /// The enemy issuing the chase command.
    actor: ActorId,
    /// The living actor being pursued.
    target: ActorId,
  },
  /// Equip one item already owned by the actor, replacing any previous equipment.
  Equip {
    /// The actor issuing the command.
    actor: ActorId,
    /// The owned item instance to equip.
    item: ItemId,
  },
  /// Unequip the actor's current item reference.
  Unequip {
    /// The actor issuing the command.
    actor: ActorId,
  },
  /// Consume one owned item instance without defining its future effect.
  UseItem {
    /// The actor issuing the command.
    actor: ActorId,
    /// The owned item instance to consume.
    item: ItemId,
  },
  /// Pick one item from the actor's current ground stack.
  Pickup {
    /// The actor issuing the command.
    actor: ActorId,
    /// The ground item instance to pick up.
    item: ItemId,
  },
  /// Drop one owned unequipped item at the player's current position.
  Drop {
    /// The player issuing the command.
    actor: ActorId,
    /// The owned item instance to drop.
    item: ItemId,
  },
  /// Restore a player's ranged ammunition to its fixed capacity.
  Reload {
    /// The player issuing the reload.
    actor: ActorId,
  },
}

impl Command {
  const fn actor(self) -> ActorId {
    match self {
      Self::Move { actor, .. }
      | Self::Wait { actor }
      | Self::Attack { actor, .. }
      | Self::RangedAttack { actor, .. }
      | Self::Chase { actor, .. }
      | Self::Equip { actor, .. }
      | Self::Unequip { actor }
      | Self::UseItem { actor, .. }
      | Self::Pickup { actor, .. }
      | Self::Drop { actor, .. }
      | Self::Reload { actor } => actor,
    }
  }
}

/// An ordered, seeded command trace for deterministic replay evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayTrace {
  seed: u64,
  commands: Vec<Command>,
}

impl ReplayTrace {
  /// Creates an empty trace with an explicit run seed.
  #[must_use]
  pub const fn new(seed: u64) -> Self {
    Self {
      seed,
      commands: Vec::new(),
    }
  }

  /// Returns the seed recorded with this trace.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.seed
  }

  /// Appends one semantic command in execution order.
  pub fn record(&mut self, command: Command) {
    self.commands.push(command);
  }

  /// Returns the commands in their recorded order.
  #[must_use]
  pub fn commands(&self) -> &[Command] {
    &self.commands
  }

  /// Returns a deterministic trace identity based on seed and command order.
  ///
  /// This is regression evidence, not a cryptographic integrity check or serialized replay
  /// format. The explicit FNV-1a byte order remains stable across process invocations.
  #[must_use]
  pub fn digest(&self) -> StateDigest {
    let mut hasher = StableHasher::new();
    hasher.write_bytes(b"DREADSTEP-REPLAY-V2");
    hasher.write_u64(self.seed);
    hasher.write_u64(u64::try_from(self.commands.len()).unwrap_or(u64::MAX));
    for command in &self.commands {
      hash_command(&mut hasher, *command);
    }
    hasher.finish()
  }
}

fn hash_command(hasher: &mut StableHasher, command: Command) {
  match command {
    Command::Move { actor, direction } => {
      hasher.write_u8(1);
      hasher.write_u32(actor.value());
      hasher.write_u8(direction_code(direction));
    }
    Command::Wait { actor } => {
      hasher.write_u8(2);
      hasher.write_u32(actor.value());
    }
    Command::Attack { actor, target } => {
      hasher.write_u8(3);
      hasher.write_u32(actor.value());
      hasher.write_u32(target.value());
    }
    Command::RangedAttack { actor, target } => {
      hasher.write_u8(9);
      hasher.write_u32(actor.value());
      hasher.write_u32(target.value());
    }
    Command::Chase { actor, target } => {
      hasher.write_u8(4);
      hasher.write_u32(actor.value());
      hasher.write_u32(target.value());
    }
    Command::Equip { actor, item } => {
      hasher.write_u8(5);
      hasher.write_u32(actor.value());
      hasher.write_u32(item.value());
    }
    Command::Unequip { actor } => {
      hasher.write_u8(6);
      hasher.write_u32(actor.value());
    }
    Command::UseItem { actor, item } => {
      hasher.write_u8(7);
      hasher.write_u32(actor.value());
      hasher.write_u32(item.value());
    }
    Command::Pickup { actor, item } => {
      hasher.write_u8(8);
      hasher.write_u32(actor.value());
      hasher.write_u32(item.value());
    }
    Command::Reload { actor } => {
      hasher.write_u8(10);
      hasher.write_u32(actor.value());
    }
    Command::Drop { actor, item } => {
      hasher.write_u8(11);
      hasher.write_u32(actor.value());
      hasher.write_u32(item.value());
    }
  }
}

const fn direction_code(direction: Direction) -> u8 {
  match direction {
    Direction::North => 1,
    Direction::South => 2,
    Direction::West => 3,
    Direction::East => 4,
  }
}

/// The reason a movement command could not enter its destination.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockReason {
  /// The destination is outside the map or is a blocking terrain tile.
  Terrain,
  /// The destination is occupied by another actor.
  Actor(ActorId),
}

/// A semantic outcome emitted by a world transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Event {
  /// An actor entered an unoccupied walkable tile.
  Moved {
    /// The actor that moved.
    actor: ActorId,
    /// The position before movement.
    from: Position,
    /// The position after movement.
    to: Position,
  },
  /// An actor attempted movement but remained in place.
  MovementBlocked {
    /// The actor that attempted movement.
    actor: ActorId,
    /// The position before the attempt.
    from: Position,
    /// The requested destination.
    to: Position,
    /// Why the destination could not be entered.
    reason: BlockReason,
  },
  /// An actor spent a standard action without moving.
  Waited {
    /// The actor that waited.
    actor: ActorId,
    /// The action time at which the wait began.
    at: ActionTime,
  },
  /// An attack reduced a living target's hit points.
  Attacked {
    /// The actor that attacked.
    attacker: ActorId,
    /// The actor that was hit.
    target: ActorId,
    /// The fixed damage applied.
    damage: Damage,
    /// The target's hit points after damage.
    remaining_hit_points: HitPoints,
  },
  /// An actor reached zero hit points and became dead.
  Died {
    /// The actor that died.
    actor: ActorId,
  },
  /// An actor equipped an owned item, optionally after another item was unequipped.
  ItemEquipped {
    /// The actor whose equipment changed.
    actor: ActorId,
    /// The item now equipped.
    item: ItemId,
  },
  /// An actor removed its equipped item reference.
  ItemUnequipped {
    /// The actor whose equipment changed.
    actor: ActorId,
    /// The item that was unequipped.
    item: ItemId,
  },
  /// An actor consumed an owned item instance; effect semantics remain outside this slice.
  ItemConsumed {
    /// The actor whose inventory changed.
    actor: ActorId,
    /// The consumed item identity.
    item: ItemId,
  },
  /// An actor picked one item from its current ground stack.
  ItemPickedUp {
    /// The actor whose inventory changed.
    actor: ActorId,
    /// The picked-up item identity.
    item: ItemId,
  },
  /// A player dropped one owned unequipped item at the current position.
  ItemDropped {
    /// The player whose inventory changed.
    actor: ActorId,
    /// The item instance moved to the ground.
    item: ItemId,
  },
  /// A player restored ranged ammunition to the fixed capacity.
  Reloaded {
    /// The player whose ammunition was restored.
    actor: ActorId,
    /// The restored ammunition count.
    ammunition: u16,
  },
}

/// Errors produced while constructing or explicitly mutating a world state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldError {
  /// A tester mutation addresses no actor in the world.
  UnknownActor(ActorId),
  /// An item identity is already owned by an actor in the world.
  DuplicateItemId(ItemId),
  /// An actor does not own the item requested by a tester transfer.
  ItemNotOwned {
    /// The actor whose inventory was searched.
    actor: ActorId,
    /// The item identity that was not found.
    item: ItemId,
  },
  /// An equipped item cannot be moved by a tester inventory mutation.
  ItemEquipped {
    /// The actor whose equipment references the item.
    actor: ActorId,
    /// The equipped item identity.
    item: ItemId,
  },
  /// An actor has no matching item in the ground stack at its current position.
  ItemNotOnGround {
    /// The actor whose current ground stack was searched.
    actor: ActorId,
    /// The item identity that was not found.
    item: ItemId,
  },
  /// A tester teleport destination is outside the map.
  TeleportOutOfBounds {
    /// The actor being teleported.
    actor: ActorId,
    /// The invalid destination.
    position: Position,
  },
  /// A tester teleport destination is blocking terrain.
  TeleportOnBlockedTile {
    /// The actor being teleported.
    actor: ActorId,
    /// The blocked destination.
    position: Position,
  },
  /// A living tester teleport destination is occupied by another living actor.
  TeleportOccupied {
    /// The actor being teleported.
    actor: ActorId,
    /// The living actor already at the destination.
    blocker: ActorId,
    /// The occupied destination.
    position: Position,
  },
  /// Two actors use the same stable identity.
  DuplicateActorId(ActorId),
  /// An actor starts outside the map.
  ActorOutOfBounds {
    /// The actor outside the map.
    actor: ActorId,
    /// The invalid starting position.
    position: Position,
  },
  /// An actor starts on a blocking terrain tile.
  ActorOnBlockedTile {
    /// The actor on blocked terrain.
    actor: ActorId,
    /// The invalid starting position.
    position: Position,
  },
  /// Two distinct actors start on one position.
  OverlappingActors {
    /// The actor inserted first.
    first: ActorId,
    /// The actor that overlaps the first actor.
    second: ActorId,
    /// The shared position.
    position: Position,
  },
  /// An actor starts with zero hit points.
  ActorDeadAtStart {
    /// The actor that starts dead.
    actor: ActorId,
  },
}

impl fmt::Display for WorldError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnknownActor(actor) => write!(formatter, "unknown actor {}", actor.value()),
      Self::DuplicateItemId(item) => {
        write!(formatter, "item id {} is duplicated", item.value())
      }
      Self::ItemNotOwned { actor, item } => write!(
        formatter,
        "actor {} does not own item {}",
        actor.value(),
        item.value()
      ),
      Self::ItemEquipped { actor, item } => write!(
        formatter,
        "actor {} cannot move equipped item {}",
        actor.value(),
        item.value()
      ),
      Self::ItemNotOnGround { actor, item } => write!(
        formatter,
        "actor {} has no item {} on the ground at its position",
        actor.value(),
        item.value()
      ),
      Self::TeleportOutOfBounds { actor, position } => write!(
        formatter,
        "actor {} cannot teleport out of bounds to ({}, {})",
        actor.value(),
        position.x(),
        position.y()
      ),
      Self::TeleportOnBlockedTile { actor, position } => write!(
        formatter,
        "actor {} cannot teleport onto blocked tile at ({}, {})",
        actor.value(),
        position.x(),
        position.y()
      ),
      Self::TeleportOccupied {
        actor,
        blocker,
        position,
      } => write!(
        formatter,
        "actor {} cannot teleport onto actor {} at ({}, {})",
        actor.value(),
        blocker.value(),
        position.x(),
        position.y()
      ),
      Self::DuplicateActorId(actor) => {
        write!(formatter, "actor id {} is duplicated", actor.value())
      }
      Self::ActorOutOfBounds { actor, position } => write!(
        formatter,
        "actor {} starts out of bounds at ({}, {})",
        actor.value(),
        position.x(),
        position.y()
      ),
      Self::ActorOnBlockedTile { actor, position } => write!(
        formatter,
        "actor {} starts on blocked tile at ({}, {})",
        actor.value(),
        position.x(),
        position.y()
      ),
      Self::OverlappingActors {
        first,
        second,
        position,
      } => write!(
        formatter,
        "actors {} and {} overlap at ({}, {})",
        first.value(),
        second.value(),
        position.x(),
        position.y()
      ),
      Self::ActorDeadAtStart { actor } => {
        write!(
          formatter,
          "actor {} starts with zero hit points",
          actor.value()
        )
      }
    }
  }
}

impl std::error::Error for WorldError {}

/// Errors produced when a command cannot be applied to the current world state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandError {
  /// The command addresses no actor in the world.
  UnknownActor(ActorId),
  /// The command addresses an actor other than the deterministic next actor.
  ActorNotScheduled {
    /// The actor addressed by the command.
    requested: ActorId,
    /// The actor selected by ready time and identity.
    scheduled: ActorId,
  },
  /// The actor's next ready time would overflow the integer timeline.
  ScheduleOverflow(ActorId),
  /// The command actor is dead and cannot act.
  ActorDead(ActorId),
  /// The attack target is not present in the world.
  UnknownTarget(ActorId),
  /// The attack target is already dead.
  TargetDead(ActorId),
  /// An actor cannot target itself with an attack.
  CannotAttackSelf(ActorId),
  /// A chase command must be issued by an enemy actor.
  ChaseRequiresEnemy(ActorId),
  /// A pickup command must be issued by a player actor.
  PickupRequiresPlayer(ActorId),
  /// A drop command must be issued by a player actor.
  DropRequiresPlayer(ActorId),
  /// A reload command must be issued by a player actor.
  ReloadRequiresPlayer(ActorId),
  /// An enemy cannot chase itself.
  CannotChaseSelf(ActorId),
  /// The attack target is outside the attacker's melee reach.
  AttackOutOfRange {
    /// The actor issuing the attack.
    attacker: ActorId,
    /// The actor outside melee range.
    target: ActorId,
  },
  /// The ranged target is not two or three tiles from the attacker.
  RangedAttackOutOfRange {
    /// The actor issuing the ranged attack.
    attacker: ActorId,
    /// The actor outside the bounded ranged interval.
    target: ActorId,
  },
  /// A ranged target is not visible along a clear cardinal ray.
  RangedAttackNoLineOfSight {
    /// The actor issuing the ranged attack.
    attacker: ActorId,
    /// The actor hidden by a diagonal path or blocking terrain.
    target: ActorId,
  },
  /// The actor has no ranged ammunition remaining.
  RangedAttackNoAmmunition(ActorId),
  /// The actor already has the full ranged ammunition capacity.
  ReloadNotNeeded(ActorId),
  /// The actor does not own the requested item.
  ItemNotOwned {
    /// The actor whose inventory was searched.
    actor: ActorId,
    /// The item identity that was not found.
    item: ItemId,
  },
  /// The requested item is already equipped.
  ItemAlreadyEquipped {
    /// The actor whose equipment was queried.
    actor: ActorId,
    /// The already equipped item identity.
    item: ItemId,
  },
  /// The actor has no equipped item to remove.
  NothingEquipped(ActorId),
  /// The requested item is equipped and cannot be moved or consumed.
  ItemEquipped {
    /// The actor whose equipment references the item.
    actor: ActorId,
    /// The equipped item identity.
    item: ItemId,
  },
  /// The requested item is not in the actor's current ground stack.
  ItemNotOnGround {
    /// The actor whose current ground stack was searched.
    actor: ActorId,
    /// The missing ground item identity.
    item: ItemId,
  },
}

impl fmt::Display for CommandError {
  #[expect(
    clippy::too_many_lines,
    reason = "the command boundary keeps each typed rejection message exhaustive"
  )]
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnknownActor(actor) => write!(formatter, "unknown actor {}", actor.value()),
      Self::ActorNotScheduled {
        requested,
        scheduled,
      } => write!(
        formatter,
        "actor {} is not scheduled; actor {} must act next",
        requested.value(),
        scheduled.value()
      ),
      Self::ScheduleOverflow(actor) => {
        write!(
          formatter,
          "actor {} cannot advance its ready time",
          actor.value()
        )
      }
      Self::ActorDead(actor) => write!(formatter, "actor {} is dead", actor.value()),
      Self::UnknownTarget(target) => write!(formatter, "unknown attack target {}", target.value()),
      Self::TargetDead(target) => write!(formatter, "attack target {} is dead", target.value()),
      Self::CannotAttackSelf(actor) => {
        write!(formatter, "actor {} cannot attack itself", actor.value())
      }
      Self::ChaseRequiresEnemy(actor) => {
        write!(
          formatter,
          "actor {} cannot issue an enemy chase",
          actor.value()
        )
      }
      Self::PickupRequiresPlayer(actor) => {
        write!(
          formatter,
          "actor {} cannot issue a player pickup",
          actor.value()
        )
      }
      Self::DropRequiresPlayer(actor) => {
        write!(
          formatter,
          "actor {} cannot issue a player drop",
          actor.value()
        )
      }
      Self::ReloadRequiresPlayer(actor) => {
        write!(
          formatter,
          "actor {} cannot reload because only players may reload",
          actor.value()
        )
      }
      Self::CannotChaseSelf(actor) => {
        write!(formatter, "actor {} cannot chase itself", actor.value())
      }
      Self::AttackOutOfRange { attacker, target } => write!(
        formatter,
        "actor {} cannot attack non-adjacent target {}",
        attacker.value(),
        target.value()
      ),
      Self::RangedAttackOutOfRange { attacker, target } => write!(
        formatter,
        "actor {} cannot ranged attack target {} outside distance 2..=3",
        attacker.value(),
        target.value()
      ),
      Self::RangedAttackNoLineOfSight { attacker, target } => write!(
        formatter,
        "actor {} cannot ranged attack target {} without a clear cardinal line of sight",
        attacker.value(),
        target.value()
      ),
      Self::RangedAttackNoAmmunition(actor) => write!(
        formatter,
        "actor {} cannot ranged attack without ammunition",
        actor.value()
      ),
      Self::ReloadNotNeeded(actor) => write!(
        formatter,
        "actor {} cannot reload with full ammunition",
        actor.value()
      ),
      Self::ItemNotOwned { actor, item } => write!(
        formatter,
        "actor {} does not own item {}",
        actor.value(),
        item.value()
      ),
      Self::ItemAlreadyEquipped { actor, item } => write!(
        formatter,
        "actor {} already equips item {}",
        actor.value(),
        item.value()
      ),
      Self::NothingEquipped(actor) => {
        write!(formatter, "actor {} has no equipped item", actor.value())
      }
      Self::ItemEquipped { actor, item } => write!(
        formatter,
        "actor {} cannot move or consume equipped item {}",
        actor.value(),
        item.value()
      ),
      Self::ItemNotOnGround { actor, item } => write!(
        formatter,
        "actor {} does not have item {} on the ground",
        actor.value(),
        item.value()
      ),
    }
  }
}

impl std::error::Error for CommandError {}

/// The observable result of one accepted command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionResult {
  events: Vec<Event>,
  next_actor: Option<ActorId>,
  current_time: ActionTime,
}

impl ActionResult {
  /// Returns semantic events emitted by this command.
  #[must_use]
  pub fn events(&self) -> &[Event] {
    &self.events
  }

  /// Returns the actor selected to act after this command.
  #[must_use]
  pub const fn next_actor(&self) -> Option<ActorId> {
    self.next_actor
  }

  /// Returns the world's minimum ready time after this command.
  #[must_use]
  pub const fn current_time(&self) -> ActionTime {
    self.current_time
  }
}

/// The authoritative deterministic state for the current grid slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldState {
  map: GridMap,
  actors: BTreeMap<ActorId, Actor>,
  ground_items: Vec<GroundItemStack>,
  current_time: ActionTime,
}

impl WorldState {
  /// Validates and creates a world from a map and its initial actors.
  ///
  /// # Errors
  ///
  /// Returns a [`WorldError`] when an actor identity is duplicated, an actor is outside the
  /// map, an actor starts on blocking terrain, an actor starts dead, or two actors overlap.
  pub fn new(map: GridMap, actors: Vec<Actor>) -> Result<Self, WorldError> {
    let mut indexed_actors = BTreeMap::new();
    for actor in actors {
      let actor_id = actor.id();
      let position = actor.position();
      if indexed_actors.contains_key(&actor_id) {
        return Err(WorldError::DuplicateActorId(actor_id));
      }
      if !actor.is_alive() {
        return Err(WorldError::ActorDeadAtStart { actor: actor_id });
      }
      if !map.in_bounds(position) {
        return Err(WorldError::ActorOutOfBounds {
          actor: actor_id,
          position,
        });
      }
      if !map.is_walkable(position) {
        return Err(WorldError::ActorOnBlockedTile {
          actor: actor_id,
          position,
        });
      }
      if let Some(first) = indexed_actors
        .values()
        .find(|existing: &&Actor| existing.position() == position)
      {
        return Err(WorldError::OverlappingActors {
          first: first.id(),
          second: actor_id,
          position,
        });
      }
      indexed_actors.insert(actor_id, actor);
    }
    let current_time = indexed_actors
      .values()
      .map(Actor::ready_at)
      .min()
      .unwrap_or(ActionTime::new(0));
    Ok(Self {
      map,
      actors: indexed_actors,
      ground_items: Vec::new(),
      current_time,
    })
  }

  /// Validates and inserts one living actor for an explicit tester operation.
  ///
  /// Dead actor records do not occupy tiles, so a new living actor may use a position retained by
  /// a dead record. The inserted actor becomes ready at the world's current action time, so a
  /// tester mutation cannot rewind the deterministic timeline.
  ///
  /// # Errors
  ///
  /// Returns a [`WorldError`] when the identity, hit points, position, terrain, or living
  /// occupancy is invalid. A rejected actor is not inserted.
  pub fn spawn(&mut self, actor: Actor) -> Result<(), WorldError> {
    let actor_id = actor.id();
    let position = actor.position();
    if self.actors.contains_key(&actor_id) {
      return Err(WorldError::DuplicateActorId(actor_id));
    }
    if !actor.is_alive() {
      return Err(WorldError::ActorDeadAtStart { actor: actor_id });
    }
    if !self.map.in_bounds(position) {
      return Err(WorldError::ActorOutOfBounds {
        actor: actor_id,
        position,
      });
    }
    if !self.map.is_walkable(position) {
      return Err(WorldError::ActorOnBlockedTile {
        actor: actor_id,
        position,
      });
    }
    if let Some(first) = self
      .actors
      .values()
      .find(|existing| existing.is_alive() && existing.position() == position)
    {
      return Err(WorldError::OverlappingActors {
        first: first.id(),
        second: actor_id,
        position,
      });
    }

    let mut actor = actor;
    actor.ready_at = self.current_time;
    self.actors.insert(actor_id, actor);
    Ok(())
  }

  /// Gives one opaque item instance to an existing actor for an explicit tester operation.
  ///
  /// Item ownership is recorded in insertion order. The instance identity is global across all
  /// actor inventories; item effects and capacity rules are intentionally outside this slice.
  /// Explicit tester transfers are handled separately by [`Self::transfer_item`]. Dead actor
  /// records remain valid ownership targets because the mutation does not alter scheduling or
  /// occupancy.
  ///
  /// # Errors
  ///
  /// Returns [`WorldError::UnknownActor`] when the target identity is absent or
  /// [`WorldError::DuplicateItemId`] when any actor inventory or ground stack already owns the
  /// item identity. A rejected item is not inserted.
  pub fn give_item(&mut self, actor_id: ActorId, item: Item) -> Result<(), WorldError> {
    if !self.actors.contains_key(&actor_id) {
      return Err(WorldError::UnknownActor(actor_id));
    }
    if self.actors.values().any(|actor| {
      actor
        .inventory()
        .iter()
        .any(|owned| owned.id() == item.id())
    }) || self
      .ground_items
      .iter()
      .any(|stack| stack.items().iter().any(|owned| owned.id() == item.id()))
    {
      return Err(WorldError::DuplicateItemId(item.id()));
    }
    self
      .actors
      .get_mut(&actor_id)
      .ok_or(WorldError::UnknownActor(actor_id))?
      .inventory
      .push(item);
    Ok(())
  }

  /// Drops one opaque item at an actor's current position for an explicit tester operation.
  ///
  /// The item is removed from the actor's ordered inventory and appended unchanged to the
  /// position's ground stack. Dead actor records remain valid sources because their retained
  /// positions remain part of the inspectable world state.
  ///
  /// # Errors
  ///
  /// Returns [`WorldError::UnknownActor`] when the actor identity is absent,
  /// [`WorldError::ItemNotOwned`] when the actor does not own the requested item, or
  /// [`WorldError::ItemEquipped`] when moving the requested item would invalidate the equipment
  /// reference. Rejected drops leave the world unchanged.
  pub fn drop_item(&mut self, actor_id: ActorId, item_id: ItemId) -> Result<(), WorldError> {
    if !self.actors.contains_key(&actor_id) {
      return Err(WorldError::UnknownActor(actor_id));
    }
    let Some(item_index) = self
      .actors
      .get(&actor_id)
      .and_then(|actor| actor.inventory.iter().position(|item| item.id() == item_id))
    else {
      return Err(WorldError::ItemNotOwned {
        actor: actor_id,
        item: item_id,
      });
    };
    if self.actors.get(&actor_id).and_then(Actor::equipped_item) == Some(item_id) {
      return Err(WorldError::ItemEquipped {
        actor: actor_id,
        item: item_id,
      });
    }
    let position = self
      .actors
      .get(&actor_id)
      .map(Actor::position)
      .ok_or(WorldError::UnknownActor(actor_id))?;
    let mut actor = self
      .actors
      .remove(&actor_id)
      .ok_or(WorldError::UnknownActor(actor_id))?;
    let item = actor.inventory.remove(item_index);
    self.actors.insert(actor_id, actor);

    let position_key = (position.y(), position.x());
    match self
      .ground_items
      .binary_search_by_key(&position_key, |stack| {
        (stack.position().y(), stack.position().x())
      }) {
      Ok(index) => self.ground_items[index].items.push(item),
      Err(index) => self
        .ground_items
        .insert(index, GroundItemStack::new(position, item)),
    }
    Ok(())
  }

  /// Picks one opaque item from an actor's current ground stack for an explicit tester operation.
  ///
  /// The item is removed while preserving the remaining stack order, and appended unchanged to
  /// the actor's ordered inventory. Empty ground stacks are removed. Dead actor records remain
  /// valid sources because their retained positions remain part of the inspectable world state.
  ///
  /// # Errors
  ///
  /// Returns [`WorldError::UnknownActor`] when the actor identity is absent or
  /// [`WorldError::ItemNotOnGround`] when the actor's current stack does not contain the item.
  /// Rejected pickups leave the world unchanged.
  pub fn pickup_item(&mut self, actor_id: ActorId, item_id: ItemId) -> Result<(), WorldError> {
    if !self.actors.contains_key(&actor_id) {
      return Err(WorldError::UnknownActor(actor_id));
    }
    let position = self
      .actors
      .get(&actor_id)
      .map(Actor::position)
      .ok_or(WorldError::UnknownActor(actor_id))?;
    let position_key = (position.y(), position.x());
    let Ok(stack_index) = self
      .ground_items
      .binary_search_by_key(&position_key, |stack| {
        (stack.position().y(), stack.position().x())
      })
    else {
      return Err(WorldError::ItemNotOnGround {
        actor: actor_id,
        item: item_id,
      });
    };
    let Some(item_index) = self.ground_items[stack_index]
      .items()
      .iter()
      .position(|item| item.id() == item_id)
    else {
      return Err(WorldError::ItemNotOnGround {
        actor: actor_id,
        item: item_id,
      });
    };

    let mut actor = self
      .actors
      .remove(&actor_id)
      .ok_or(WorldError::UnknownActor(actor_id))?;
    let item = self.ground_items[stack_index].items.remove(item_index);
    if self.ground_items[stack_index].items.is_empty() {
      self.ground_items.remove(stack_index);
    }
    actor.inventory.push(item);
    self.actors.insert(actor_id, actor);
    Ok(())
  }

  /// Transfers one opaque item between existing actor records for an explicit tester operation.
  ///
  /// Cross-actor transfer removes the item from the source while preserving the relative order of
  /// remaining items, then appends the unchanged item to the target. Same-actor transfer is an
  /// idempotent no-op after ownership validation. Dead actor records remain valid endpoints because
  /// this mutation does not affect scheduling or occupancy.
  ///
  /// # Errors
  ///
  /// Returns [`WorldError::UnknownActor`] when either actor identity is absent,
  /// [`WorldError::ItemNotOwned`] when the source does not own the requested item, or
  /// [`WorldError::ItemEquipped`] when moving the requested item would invalidate the equipment
  /// reference. Rejected transfers leave the world unchanged.
  pub fn transfer_item(
    &mut self,
    source_actor: ActorId,
    target_actor: ActorId,
    item_id: ItemId,
  ) -> Result<(), WorldError> {
    if !self.actors.contains_key(&source_actor) {
      return Err(WorldError::UnknownActor(source_actor));
    }
    if !self.actors.contains_key(&target_actor) {
      return Err(WorldError::UnknownActor(target_actor));
    }
    let Some(item_index) = self
      .actors
      .get(&source_actor)
      .and_then(|actor| actor.inventory.iter().position(|item| item.id() == item_id))
    else {
      return Err(WorldError::ItemNotOwned {
        actor: source_actor,
        item: item_id,
      });
    };
    if self
      .actors
      .get(&source_actor)
      .and_then(Actor::equipped_item)
      == Some(item_id)
    {
      return Err(WorldError::ItemEquipped {
        actor: source_actor,
        item: item_id,
      });
    }
    if source_actor == target_actor {
      return Ok(());
    }
    let Some(mut source) = self.actors.remove(&source_actor) else {
      return Err(WorldError::UnknownActor(source_actor));
    };
    let Some(mut target) = self.actors.remove(&target_actor) else {
      self.actors.insert(source_actor, source);
      return Err(WorldError::UnknownActor(target_actor));
    };
    let item = source.inventory.remove(item_index);
    target.inventory.push(item);
    self.actors.insert(source_actor, source);
    self.actors.insert(target_actor, target);
    Ok(())
  }

  /// Teleports one existing actor for an explicit tester operation.
  ///
  /// Teleport preserves the actor's identity, life, hit points, inventory, and ready time, and
  /// does not alter the world's current action time. Living actors occupy destinations; dead
  /// records do not, so a dead actor may be positioned on a living actor's tile until it is
  /// revived. The destination must remain a walkable map position.
  ///
  /// # Errors
  ///
  /// Returns [`WorldError::UnknownActor`] when no actor has the requested identity,
  /// [`WorldError::TeleportOutOfBounds`] or [`WorldError::TeleportOnBlockedTile`] when the
  /// destination is invalid, or [`WorldError::TeleportOccupied`] when a living actor would
  /// overlap another living actor. Rejected teleports leave the world unchanged.
  pub fn teleport(&mut self, actor_id: ActorId, position: Position) -> Result<(), WorldError> {
    let Some(existing) = self.actors.get(&actor_id) else {
      return Err(WorldError::UnknownActor(actor_id));
    };
    if !self.map.in_bounds(position) {
      return Err(WorldError::TeleportOutOfBounds {
        actor: actor_id,
        position,
      });
    }
    if !self.map.is_walkable(position) {
      return Err(WorldError::TeleportOnBlockedTile {
        actor: actor_id,
        position,
      });
    }
    if existing.is_alive()
      && let Some(blocker) = self
        .actors
        .values()
        .find(|actor| actor.id() != actor_id && actor.is_alive() && actor.position() == position)
    {
      return Err(WorldError::TeleportOccupied {
        actor: actor_id,
        blocker: blocker.id(),
        position,
      });
    }
    self
      .actors
      .get_mut(&actor_id)
      .ok_or(WorldError::UnknownActor(actor_id))?
      .position = position;
    Ok(())
  }

  /// Sets one existing actor's hit points for an explicit tester operation.
  ///
  /// Setting zero leaves the dead actor record inspectable while existing scheduling and
  /// occupancy queries exclude it. Reviving a dead actor anchors its readiness at the current
  /// action time so the mutation cannot rewind the deterministic timeline. Removing a living
  /// actor may advance the current time to the next surviving actor's readiness, but never moves
  /// it backward; other actor fields remain unchanged.
  ///
  /// # Errors
  ///
  /// Returns [`WorldError::UnknownActor`] when no actor has the requested identity or
  /// [`WorldError::OverlappingActors`] when reviving would overlap a living actor.
  pub fn set_hit_points(
    &mut self,
    actor_id: ActorId,
    hit_points: HitPoints,
  ) -> Result<(), WorldError> {
    let current_time = self.current_time;
    let Some(existing) = self.actors.get(&actor_id) else {
      return Err(WorldError::UnknownActor(actor_id));
    };
    let was_alive = existing.is_alive();
    if !was_alive && hit_points.is_alive() {
      let position = existing.position();
      if let Some(first) = self
        .actors
        .values()
        .find(|actor| actor.is_alive() && actor.position() == position)
      {
        return Err(WorldError::OverlappingActors {
          first: first.id(),
          second: actor_id,
          position,
        });
      }
    }
    let actor = self
      .actors
      .get_mut(&actor_id)
      .ok_or(WorldError::UnknownActor(actor_id))?;
    actor.hit_points = hit_points;
    if !was_alive && hit_points.is_alive() {
      actor.ready_at = current_time;
    } else if was_alive
      && !hit_points.is_alive()
      && let Some(next_actor) = self.next_actor()
      && let Some(next_ready_at) = self.actors.get(&next_actor).map(Actor::ready_at)
    {
      self.current_time = next_ready_at;
    }
    Ok(())
  }

  /// Returns the immutable map owned by this world.
  #[must_use]
  pub const fn map(&self) -> &GridMap {
    &self.map
  }

  /// Returns an actor by stable identity.
  #[must_use]
  pub fn actor(&self, actor: ActorId) -> Option<&Actor> {
    self.actors.get(&actor)
  }

  /// Returns all actor records in stable [`ActorId`] order.
  ///
  /// Dead actors remain in this read-only projection so adapters can report their final state;
  /// scheduling and occupancy continue to consider living actors only.
  #[must_use = "iterate over the actor records"]
  pub fn actors(&self) -> impl Iterator<Item = &Actor> + '_ {
    self.actors.values()
  }

  /// Returns the deterministic terminal outcome derived from retained actor records.
  ///
  /// Player defeat takes precedence so a world containing no living player can never be reported
  /// as a victory. A world without an enemy remains in progress until authored content provides a
  /// concrete opponent to defeat.
  #[must_use]
  pub fn outcome(&self) -> RunOutcome {
    if self
      .actors
      .values()
      .any(|actor| actor.kind() == ActorKind::Player && !actor.is_alive())
    {
      return RunOutcome::Defeat;
    }
    let has_enemy = self
      .actors
      .values()
      .any(|actor| actor.kind() == ActorKind::Enemy);
    if has_enemy
      && self
        .actors
        .values()
        .filter(|actor| actor.kind() == ActorKind::Enemy)
        .all(|actor| !actor.is_alive())
    {
      RunOutcome::Victory
    } else {
      RunOutcome::InProgress
    }
  }

  /// Returns ground-item stacks in deterministic row-major position order.
  #[must_use = "inspect the ground-item stacks"]
  pub fn ground_items(&self) -> &[GroundItemStack] {
    &self.ground_items
  }

  /// Returns the world's minimum ready time.
  #[must_use]
  pub const fn current_time(&self) -> ActionTime {
    self.current_time
  }

  /// Returns a stable digest of all semantic world state.
  ///
  /// The digest includes map dimensions and terrain, current action time, and every actor's
  /// identity, kind, life, position, hit points, ranged ammunition, ready time, ordered inventory
  /// item identities and definition references, optional equipped item identity, and ordered
  /// ground-item stacks. It is deterministic regression evidence, not a cryptographic integrity
  /// check or serialized state format.
  #[must_use]
  pub fn digest(&self) -> StateDigest {
    let mut hasher = StableHasher::new();
    hasher.write_bytes(b"DREADSTEP-STATE-V2");
    hasher.write_u32(self.map.width());
    hasher.write_u32(self.map.height());
    for tile in &self.map.tiles {
      hasher.write_u8(match tile {
        Tile::Floor => 1,
        Tile::Cover => 2,
        Tile::Wall => 3,
      });
    }
    hasher.write_u64(self.current_time.value());
    hasher.write_u64(u64::try_from(self.actors.len()).unwrap_or(u64::MAX));
    for actor in self.actors.values() {
      hasher.write_u32(actor.id().value());
      hasher.write_u8(match actor.kind() {
        ActorKind::Player => 1,
        ActorKind::Enemy => 2,
      });
      hasher.write_i32(actor.position().x());
      hasher.write_i32(actor.position().y());
      hasher.write_u16(actor.hit_points().value());
      hasher.write_u8(actor.melee_reach().value());
      hasher.write_u16(actor.ranged_ammo());
      hasher.write_u64(actor.ready_at().value());
      hasher.write_u64(u64::try_from(actor.inventory().len()).unwrap_or(u64::MAX));
      for item in actor.inventory() {
        hasher.write_u32(item.id().value());
        hasher.write_u32(item.definition().value());
      }
      match actor.equipped_item() {
        Some(item) => {
          hasher.write_u8(1);
          hasher.write_u32(item.value());
        }
        None => hasher.write_u8(0),
      }
    }
    if !self.ground_items.is_empty() {
      hasher.write_u64(u64::try_from(self.ground_items.len()).unwrap_or(u64::MAX));
      for stack in &self.ground_items {
        hasher.write_i32(stack.position().x());
        hasher.write_i32(stack.position().y());
        hasher.write_u64(u64::try_from(stack.items().len()).unwrap_or(u64::MAX));
        for item in stack.items() {
          hasher.write_u32(item.id().value());
          hasher.write_u32(item.definition().value());
        }
      }
    }
    hasher.finish()
  }

  /// Returns the actor selected by ready time, then stable identity.
  #[must_use]
  pub fn next_actor(&self) -> Option<ActorId> {
    self
      .actors
      .values()
      .filter(|actor| actor.is_alive())
      .min_by_key(|actor| (actor.ready_at(), actor.id()))
      .map(Actor::id)
  }

  /// Returns commands currently available to the scheduled living actor.
  ///
  /// Cardinal movement and waiting are always listed because blocked movement still produces an
  /// accepted semantic action. Each owned item that is not already equipped contributes an Equip
  /// action followed by a `UseItem` action; the optional unequip action follows inventory order.
  /// Player attacks include targets within the actor's melee reach and clear cardinal rays two or
  /// three tiles away for the bounded ranged command; enemies attack adjacent living targets and
  /// otherwise chase every distinct living target.
  /// Results follow the fixed direction, inventory, and then stable actor identity order.
  #[must_use]
  #[expect(
    clippy::too_many_lines,
    reason = "the legal command projection keeps deterministic action ordering explicit"
  )]
  pub fn legal_commands(&self) -> Vec<Command> {
    let Some(actor_id) = self.next_actor() else {
      return Vec::new();
    };
    let Some(actor) = self.actors.get(&actor_id) else {
      return Vec::new();
    };
    if actor.ready_at().checked_add(ActionCost::STANDARD).is_none() {
      return Vec::new();
    }

    let mut commands = vec![
      Command::Move {
        actor: actor_id,
        direction: Direction::North,
      },
      Command::Move {
        actor: actor_id,
        direction: Direction::South,
      },
      Command::Move {
        actor: actor_id,
        direction: Direction::West,
      },
      Command::Move {
        actor: actor_id,
        direction: Direction::East,
      },
      Command::Wait { actor: actor_id },
    ];
    if actor.kind() == ActorKind::Player && actor.ranged_ammo() < Actor::RANGED_AMMO_CAPACITY {
      commands.push(Command::Reload { actor: actor_id });
    }
    if actor.kind() == ActorKind::Player
      && let Some(stack) = self
        .ground_items
        .iter()
        .find(|stack| stack.position() == actor.position())
    {
      for item in stack.items() {
        commands.push(Command::Pickup {
          actor: actor_id,
          item: item.id(),
        });
      }
    }
    if actor.kind() == ActorKind::Player {
      for item in actor.inventory() {
        if actor.equipped_item() != Some(item.id()) {
          commands.push(Command::Drop {
            actor: actor_id,
            item: item.id(),
          });
        }
      }
    }
    for item in actor.inventory() {
      if actor.equipped_item() != Some(item.id()) {
        commands.push(Command::Equip {
          actor: actor_id,
          item: item.id(),
        });
        commands.push(Command::UseItem {
          actor: actor_id,
          item: item.id(),
        });
      }
    }
    if actor.equipped_item().is_some() {
      commands.push(Command::Unequip { actor: actor_id });
    }
    let living_targets = self
      .actors
      .values()
      .filter(|target| target.is_alive() && target.id() != actor_id)
      .collect::<Vec<_>>();
    if actor.kind() == ActorKind::Enemy {
      for target in living_targets.iter().copied().filter(|target| {
        Self::is_melee_distance(actor.position(), target.position(), actor.melee_reach())
      }) {
        commands.push(Command::Attack {
          actor: actor_id,
          target: target.id(),
        });
      }
      for target in living_targets.iter().copied().filter(|target| {
        !Self::is_melee_distance(actor.position(), target.position(), actor.melee_reach())
      }) {
        commands.push(Command::Chase {
          actor: actor_id,
          target: target.id(),
        });
      }
    } else {
      for target in living_targets {
        if Self::is_melee_distance(actor.position(), target.position(), actor.melee_reach()) {
          commands.push(Command::Attack {
            actor: actor_id,
            target: target.id(),
          });
        } else if Self::is_ranged_distance(actor.position(), target.position())
          && self.has_ranged_line_of_sight(actor.position(), target.position())
          && actor.ranged_ammo() > 0
          && actor.ready_at().checked_add(ActionCost::RANGED).is_some()
        {
          commands.push(Command::RangedAttack {
            actor: actor_id,
            target: target.id(),
          });
        }
      }
    }
    commands
  }

  /// Applies one command from the deterministically scheduled actor.
  ///
  /// # Errors
  ///
  /// Returns [`CommandError::UnknownActor`] for an unknown identity,
  /// [`CommandError::ActorDead`] for a dead command actor,
  /// [`CommandError::ActorNotScheduled`] when a different actor must act first, an equipment or
  /// target error for invalid requests, or [`CommandError::ScheduleOverflow`] if the integer
  /// timeline cannot advance.
  pub fn execute(&mut self, command: Command) -> Result<ActionResult, CommandError> {
    let actor_id = command.actor();
    let actor = self
      .actors
      .get(&actor_id)
      .ok_or(CommandError::UnknownActor(actor_id))?;
    if !actor.is_alive() {
      return Err(CommandError::ActorDead(actor_id));
    }
    let ready_at = actor.ready_at();
    if let Some(scheduled) = self.next_actor()
      && scheduled != actor_id
    {
      return Err(CommandError::ActorNotScheduled {
        requested: actor_id,
        scheduled,
      });
    }
    if matches!(command, Command::RangedAttack { .. }) && actor.ranged_ammo() == 0 {
      return Err(CommandError::RangedAttackNoAmmunition(actor_id));
    }
    if matches!(command, Command::Reload { .. }) {
      if actor.kind() != ActorKind::Player {
        return Err(CommandError::ReloadRequiresPlayer(actor_id));
      }
      if actor.ranged_ammo() >= Actor::RANGED_AMMO_CAPACITY {
        return Err(CommandError::ReloadNotNeeded(actor_id));
      }
    }
    let action_cost = match command {
      Command::RangedAttack { .. } => ActionCost::RANGED,
      _ => ActionCost::STANDARD,
    };
    let next_ready_at = ready_at
      .checked_add(action_cost)
      .ok_or(CommandError::ScheduleOverflow(actor_id))?;
    let events = match command {
      Command::Move { direction, .. } => vec![self.move_actor(actor_id, direction)?],
      Command::Wait { .. } => vec![Event::Waited {
        actor: actor_id,
        at: self.current_time,
      }],
      Command::Attack { target, .. } => self.attack(actor_id, target)?,
      Command::RangedAttack { target, .. } => self.ranged_attack(actor_id, target)?,
      Command::Chase { target, .. } => {
        let direction = self.chase_direction(actor_id, target)?;
        vec![self.move_actor(actor_id, direction)?]
      }
      Command::Equip { item, .. } => self.equip_item(actor_id, item)?,
      Command::Unequip { .. } => vec![self.unequip_item(actor_id)?],
      Command::UseItem { item, .. } => vec![self.use_item(actor_id, item)?],
      Command::Pickup { item, .. } => vec![self.pickup_item_command(actor_id, item)?],
      Command::Drop { item, .. } => vec![self.drop_item_command(actor_id, item)?],
      Command::Reload { .. } => {
        self
          .actors
          .get_mut(&actor_id)
          .ok_or(CommandError::UnknownActor(actor_id))?
          .ranged_ammo = Actor::RANGED_AMMO_CAPACITY;
        vec![Event::Reloaded {
          actor: actor_id,
          ammunition: Actor::RANGED_AMMO_CAPACITY,
        }]
      }
    };
    if matches!(command, Command::RangedAttack { .. }) {
      self
        .actors
        .get_mut(&actor_id)
        .ok_or(CommandError::UnknownActor(actor_id))?
        .ranged_ammo -= 1;
    }
    self
      .actors
      .get_mut(&actor_id)
      .ok_or(CommandError::UnknownActor(actor_id))?
      .ready_at = next_ready_at;
    self.current_time = self
      .next_actor()
      .and_then(|next| self.actors.get(&next).map(Actor::ready_at))
      .unwrap_or(next_ready_at);
    Ok(ActionResult {
      events,
      next_actor: self.next_actor(),
      current_time: self.current_time,
    })
  }

  fn actor_at(&self, position: Position) -> Option<ActorId> {
    self
      .actors
      .values()
      .find(|actor| actor.is_alive() && actor.position() == position)
      .map(Actor::id)
  }

  fn equip_item(&mut self, actor_id: ActorId, item_id: ItemId) -> Result<Vec<Event>, CommandError> {
    let actor = self
      .actors
      .get(&actor_id)
      .ok_or(CommandError::UnknownActor(actor_id))?;
    if !actor.inventory().iter().any(|item| item.id() == item_id) {
      return Err(CommandError::ItemNotOwned {
        actor: actor_id,
        item: item_id,
      });
    }
    if actor.equipped_item() == Some(item_id) {
      return Err(CommandError::ItemAlreadyEquipped {
        actor: actor_id,
        item: item_id,
      });
    }
    let previous = actor.equipped_item();
    self
      .actors
      .get_mut(&actor_id)
      .ok_or(CommandError::UnknownActor(actor_id))?
      .equipped = Some(item_id);
    let mut events = Vec::with_capacity(2);
    if let Some(previous) = previous {
      events.push(Event::ItemUnequipped {
        actor: actor_id,
        item: previous,
      });
    }
    events.push(Event::ItemEquipped {
      actor: actor_id,
      item: item_id,
    });
    Ok(events)
  }

  fn unequip_item(&mut self, actor_id: ActorId) -> Result<Event, CommandError> {
    let item = self
      .actors
      .get(&actor_id)
      .ok_or(CommandError::UnknownActor(actor_id))?
      .equipped_item()
      .ok_or(CommandError::NothingEquipped(actor_id))?;
    self
      .actors
      .get_mut(&actor_id)
      .ok_or(CommandError::UnknownActor(actor_id))?
      .equipped = None;
    Ok(Event::ItemUnequipped {
      actor: actor_id,
      item,
    })
  }

  fn use_item(&mut self, actor_id: ActorId, item_id: ItemId) -> Result<Event, CommandError> {
    let item_index = self
      .actors
      .get(&actor_id)
      .ok_or(CommandError::UnknownActor(actor_id))?
      .inventory()
      .iter()
      .position(|item| item.id() == item_id)
      .ok_or(CommandError::ItemNotOwned {
        actor: actor_id,
        item: item_id,
      })?;
    if self.actors.get(&actor_id).and_then(Actor::equipped_item) == Some(item_id) {
      return Err(CommandError::ItemEquipped {
        actor: actor_id,
        item: item_id,
      });
    }
    self
      .actors
      .get_mut(&actor_id)
      .ok_or(CommandError::UnknownActor(actor_id))?
      .inventory
      .remove(item_index);
    Ok(Event::ItemConsumed {
      actor: actor_id,
      item: item_id,
    })
  }

  fn pickup_item_command(
    &mut self,
    actor_id: ActorId,
    item_id: ItemId,
  ) -> Result<Event, CommandError> {
    if self
      .actors
      .get(&actor_id)
      .is_some_and(|actor| actor.kind() != ActorKind::Player)
    {
      return Err(CommandError::PickupRequiresPlayer(actor_id));
    }
    self
      .pickup_item(actor_id, item_id)
      .map_err(|error| match error {
        WorldError::UnknownActor(actor) => CommandError::UnknownActor(actor),
        WorldError::ItemNotOnGround { actor, item } => {
          CommandError::ItemNotOnGround { actor, item }
        }
        _ => CommandError::ItemNotOnGround {
          actor: actor_id,
          item: item_id,
        },
      })?;
    Ok(Event::ItemPickedUp {
      actor: actor_id,
      item: item_id,
    })
  }

  fn drop_item_command(
    &mut self,
    actor_id: ActorId,
    item_id: ItemId,
  ) -> Result<Event, CommandError> {
    if self
      .actors
      .get(&actor_id)
      .is_some_and(|actor| actor.kind() != ActorKind::Player)
    {
      return Err(CommandError::DropRequiresPlayer(actor_id));
    }
    self
      .drop_item(actor_id, item_id)
      .map_err(|error| match error {
        WorldError::UnknownActor(actor) => CommandError::UnknownActor(actor),
        WorldError::ItemNotOwned { actor, item } => CommandError::ItemNotOwned { actor, item },
        WorldError::ItemEquipped { actor, item } => CommandError::ItemEquipped { actor, item },
        _ => CommandError::ItemNotOwned {
          actor: actor_id,
          item: item_id,
        },
      })?;
    Ok(Event::ItemDropped {
      actor: actor_id,
      item: item_id,
    })
  }

  fn move_actor(&mut self, actor_id: ActorId, direction: Direction) -> Result<Event, CommandError> {
    let from = self
      .actors
      .get(&actor_id)
      .map(Actor::position)
      .ok_or(CommandError::UnknownActor(actor_id))?;
    let to = from.translated(direction);
    if !self.map.is_walkable(to) {
      Ok(Event::MovementBlocked {
        actor: actor_id,
        from,
        to,
        reason: BlockReason::Terrain,
      })
    } else if let Some(blocker) = self.actor_at(to) {
      Ok(Event::MovementBlocked {
        actor: actor_id,
        from,
        to,
        reason: BlockReason::Actor(blocker),
      })
    } else {
      self
        .actors
        .get_mut(&actor_id)
        .ok_or(CommandError::UnknownActor(actor_id))?
        .position = to;
      Ok(Event::Moved {
        actor: actor_id,
        from,
        to,
      })
    }
  }

  fn chase_direction(&self, actor: ActorId, target: ActorId) -> Result<Direction, CommandError> {
    let chaser = self
      .actors
      .get(&actor)
      .ok_or(CommandError::UnknownActor(actor))?;
    if chaser.kind() != ActorKind::Enemy {
      return Err(CommandError::ChaseRequiresEnemy(actor));
    }
    if actor == target {
      return Err(CommandError::CannotChaseSelf(actor));
    }
    let target_actor = self
      .actors
      .get(&target)
      .ok_or(CommandError::UnknownTarget(target))?;
    if !target_actor.is_alive() {
      return Err(CommandError::TargetDead(target));
    }
    let from = chaser.position();
    let to = target_actor.position();
    if from.x() < to.x() {
      Ok(Direction::East)
    } else if from.x() > to.x() {
      Ok(Direction::West)
    } else if from.y() < to.y() {
      Ok(Direction::South)
    } else {
      Ok(Direction::North)
    }
  }

  fn attack(&mut self, attacker: ActorId, target: ActorId) -> Result<Vec<Event>, CommandError> {
    let reach = self
      .actors
      .get(&attacker)
      .map(Actor::melee_reach)
      .ok_or(CommandError::UnknownActor(attacker))?;
    self.attack_with_distance(
      attacker,
      target,
      |first, second| Self::is_melee_distance(first, second, reach),
      Damage::MELEE,
      false,
    )
  }

  fn ranged_attack(
    &mut self,
    attacker: ActorId,
    target: ActorId,
  ) -> Result<Vec<Event>, CommandError> {
    self.attack_with_distance(
      attacker,
      target,
      Self::is_ranged_distance,
      Damage::RANGED,
      true,
    )
  }

  fn attack_with_distance(
    &mut self,
    attacker: ActorId,
    target: ActorId,
    in_range: impl FnOnce(Position, Position) -> bool,
    damage: Damage,
    ranged: bool,
  ) -> Result<Vec<Event>, CommandError> {
    if attacker == target {
      return Err(CommandError::CannotAttackSelf(attacker));
    }
    let attacker_position = self
      .actors
      .get(&attacker)
      .map(Actor::position)
      .ok_or(CommandError::UnknownActor(attacker))?;
    let target_actor = self
      .actors
      .get(&target)
      .ok_or(CommandError::UnknownTarget(target))?;
    if !target_actor.is_alive() {
      return Err(CommandError::TargetDead(target));
    }
    let target_position = target_actor.position();
    if !in_range(attacker_position, target_position) {
      return Err(if ranged {
        CommandError::RangedAttackOutOfRange { attacker, target }
      } else {
        CommandError::AttackOutOfRange { attacker, target }
      });
    }
    if ranged && !self.has_ranged_line_of_sight(attacker_position, target_position) {
      return Err(CommandError::RangedAttackNoLineOfSight { attacker, target });
    }
    let remaining_hit_points = target_actor.hit_points().reduced_by(damage);
    self
      .actors
      .get_mut(&target)
      .ok_or(CommandError::UnknownTarget(target))?
      .hit_points = remaining_hit_points;
    let mut events = vec![Event::Attacked {
      attacker,
      target,
      damage,
      remaining_hit_points,
    }];
    if !remaining_hit_points.is_alive() {
      events.push(Event::Died { actor: target });
    }
    Ok(events)
  }

  fn is_ranged_distance(first: Position, second: Position) -> bool {
    let distance = first
      .x()
      .abs_diff(second.x())
      .saturating_add(first.y().abs_diff(second.y()));
    (2..=3).contains(&distance)
  }

  fn is_melee_distance(first: Position, second: Position, reach: MeleeReach) -> bool {
    let distance = first
      .x()
      .abs_diff(second.x())
      .saturating_add(first.y().abs_diff(second.y()));
    distance <= u32::from(reach.value())
  }

  fn has_ranged_line_of_sight(&self, first: Position, second: Position) -> bool {
    if first.x() == second.x() {
      let step = if first.y() < second.y() { 1 } else { -1 };
      let mut y = first.y() + step;
      while y != second.y() {
        if self
          .map
          .tile_at(Position::new(first.x(), y))
          .is_none_or(Tile::blocks_ranged_line_of_sight)
        {
          return false;
        }
        y += step;
      }
      true
    } else if first.y() == second.y() {
      let step = if first.x() < second.x() { 1 } else { -1 };
      let mut x = first.x() + step;
      while x != second.x() {
        if self
          .map
          .tile_at(Position::new(x, first.y()))
          .is_none_or(Tile::blocks_ranged_line_of_sight)
        {
          return false;
        }
        x += step;
      }
      true
    } else {
      false
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn floor_map(width: u32, height: u32) -> GridMap {
    GridMap::filled(width, height, Tile::Floor).expect("test map should be valid")
  }

  #[test]
  fn moves_the_scheduled_actor_and_reports_the_next_actor() {
    let map = floor_map(3, 1);
    let mut world = WorldState::new(
      map,
      vec![
        Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
        Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 0)),
      ],
    )
    .expect("test world should be valid");

    let result = world
      .execute(Command::Move {
        actor: ActorId::new(1),
        direction: Direction::East,
      })
      .expect("actor one is scheduled first");

    assert_eq!(
      world.actor(ActorId::new(1)).unwrap().position(),
      Position::new(1, 0)
    );
    assert_eq!(
      world.actor(ActorId::new(1)).unwrap().ready_at(),
      ActionTime::new(1)
    );
    assert_eq!(result.next_actor(), Some(ActorId::new(2)));
    assert_eq!(
      result.events(),
      &[Event::Moved {
        actor: ActorId::new(1),
        from: Position::new(0, 0),
        to: Position::new(1, 0),
      }]
    );
  }

  #[test]
  fn ranged_cost_overflow_is_filtered_and_rejected_atomically() {
    let mut world = WorldState::new(
      floor_map(7, 1),
      vec![
        Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
        Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 0)),
      ],
    )
    .expect("test world should be valid");
    let near_max = ActionTime::new(u64::MAX - 1);
    world.current_time = near_max;
    world
      .actors
      .get_mut(&ActorId::new(1))
      .expect("attacker exists")
      .ready_at = near_max;
    world
      .actors
      .get_mut(&ActorId::new(2))
      .expect("target exists")
      .ready_at = ActionTime::new(u64::MAX);

    assert!(world.legal_commands().contains(&Command::Wait {
      actor: ActorId::new(1),
    }));
    assert!(!world.legal_commands().contains(&Command::RangedAttack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }));

    let result = world
      .execute(Command::Wait {
        actor: ActorId::new(1),
      })
      .expect("standard cost should reach the maximum timeline value");
    assert_eq!(
      world
        .actor(ActorId::new(1))
        .expect("attacker exists")
        .ready_at(),
      ActionTime::new(u64::MAX)
    );
    assert_eq!(result.current_time(), ActionTime::new(u64::MAX));

    let before = world.clone();
    assert_eq!(
      world.execute(Command::RangedAttack {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      }),
      Err(CommandError::ScheduleOverflow(ActorId::new(1)))
    );
    assert_eq!(world, before);
  }

  #[test]
  fn reports_terrain_blocking_and_still_consumes_the_action() {
    let map = GridMap::from_tiles(2, 1, vec![Tile::Floor, Tile::Wall]).unwrap();
    let mut world = WorldState::new(
      map,
      vec![Actor::new(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
      )],
    )
    .unwrap();

    let result = world
      .execute(Command::Move {
        actor: ActorId::new(1),
        direction: Direction::East,
      })
      .unwrap();

    assert_eq!(
      world.actor(ActorId::new(1)).unwrap().position(),
      Position::new(0, 0)
    );
    assert_eq!(
      world.actor(ActorId::new(1)).unwrap().ready_at(),
      ActionTime::new(1)
    );
    assert_eq!(
      result.events(),
      &[Event::MovementBlocked {
        actor: ActorId::new(1),
        from: Position::new(0, 0),
        to: Position::new(1, 0),
        reason: BlockReason::Terrain,
      }]
    );
  }

  #[test]
  fn reports_actor_blocking_without_moving_either_actor() {
    let map = floor_map(2, 1);
    let mut world = WorldState::new(
      map,
      vec![
        Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
        Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(1, 0)),
      ],
    )
    .unwrap();

    let result = world
      .execute(Command::Move {
        actor: ActorId::new(1),
        direction: Direction::East,
      })
      .unwrap();

    assert_eq!(
      result.events(),
      &[Event::MovementBlocked {
        actor: ActorId::new(1),
        from: Position::new(0, 0),
        to: Position::new(1, 0),
        reason: BlockReason::Actor(ActorId::new(2)),
      }]
    );
    assert_eq!(
      world.actor(ActorId::new(2)).unwrap().position(),
      Position::new(1, 0)
    );
  }

  #[test]
  fn treats_out_of_bounds_movement_as_terrain_blocking() {
    let mut world = WorldState::new(
      floor_map(1, 1),
      vec![Actor::new(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
      )],
    )
    .unwrap();

    let result = world
      .execute(Command::Move {
        actor: ActorId::new(1),
        direction: Direction::West,
      })
      .unwrap();

    assert_eq!(
      result.events(),
      &[Event::MovementBlocked {
        actor: ActorId::new(1),
        from: Position::new(0, 0),
        to: Position::new(-1, 0),
        reason: BlockReason::Terrain,
      }]
    );
  }

  #[test]
  fn adjacent_melee_attack_reduces_hit_points_and_consumes_an_action() {
    let mut world = WorldState::new(
      floor_map(2, 1),
      vec![
        Actor::with_hit_points(
          ActorId::new(1),
          ActorKind::Player,
          Position::new(0, 0),
          HitPoints::new(10),
        ),
        Actor::with_hit_points(
          ActorId::new(2),
          ActorKind::Enemy,
          Position::new(1, 0),
          HitPoints::new(3),
        ),
      ],
    )
    .unwrap();

    let result = world
      .execute(Command::Attack {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      })
      .unwrap();

    assert_eq!(
      result.events(),
      &[Event::Attacked {
        attacker: ActorId::new(1),
        target: ActorId::new(2),
        damage: Damage::MELEE,
        remaining_hit_points: HitPoints::new(2),
      }]
    );
    assert_eq!(
      world.actor(ActorId::new(2)).unwrap().hit_points(),
      HitPoints::new(2)
    );
    assert_eq!(
      world.actor(ActorId::new(1)).unwrap().ready_at(),
      ActionTime::new(1)
    );
    assert_eq!(result.next_actor(), Some(ActorId::new(2)));
  }

  #[test]
  fn killing_an_actor_emits_death_and_removes_it_from_scheduling_and_occupancy() {
    let mut world = WorldState::new(
      floor_map(2, 1),
      vec![
        Actor::with_hit_points(
          ActorId::new(1),
          ActorKind::Player,
          Position::new(0, 0),
          HitPoints::new(10),
        ),
        Actor::with_hit_points(
          ActorId::new(2),
          ActorKind::Enemy,
          Position::new(1, 0),
          HitPoints::new(1),
        ),
      ],
    )
    .unwrap();

    let attack = world
      .execute(Command::Attack {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      })
      .unwrap();

    assert_eq!(
      attack.events(),
      &[
        Event::Attacked {
          attacker: ActorId::new(1),
          target: ActorId::new(2),
          damage: Damage::MELEE,
          remaining_hit_points: HitPoints::new(0),
        },
        Event::Died {
          actor: ActorId::new(2),
        },
      ]
    );
    assert!(!world.actor(ActorId::new(2)).unwrap().is_alive());
    assert_eq!(attack.next_actor(), Some(ActorId::new(1)));

    let move_result = world
      .execute(Command::Move {
        actor: ActorId::new(1),
        direction: Direction::East,
      })
      .unwrap();
    assert_eq!(
      move_result.events(),
      &[Event::Moved {
        actor: ActorId::new(1),
        from: Position::new(0, 0),
        to: Position::new(1, 0),
      }]
    );
  }

  #[test]
  fn rejects_unknown_dead_self_and_out_of_range_attack_targets() {
    let mut world = WorldState::new(
      floor_map(3, 1),
      vec![
        Actor::with_hit_points(
          ActorId::new(1),
          ActorKind::Player,
          Position::new(0, 0),
          HitPoints::new(10),
        ),
        Actor::with_hit_points(
          ActorId::new(2),
          ActorKind::Enemy,
          Position::new(2, 0),
          HitPoints::new(1),
        ),
      ],
    )
    .unwrap();

    assert_eq!(
      world.execute(Command::Attack {
        actor: ActorId::new(1),
        target: ActorId::new(99),
      }),
      Err(CommandError::UnknownTarget(ActorId::new(99)))
    );
    assert_eq!(
      world.execute(Command::Attack {
        actor: ActorId::new(1),
        target: ActorId::new(1),
      }),
      Err(CommandError::CannotAttackSelf(ActorId::new(1)))
    );
    assert_eq!(
      world.execute(Command::Attack {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      }),
      Err(CommandError::AttackOutOfRange {
        attacker: ActorId::new(1),
        target: ActorId::new(2),
      })
    );

    let mut dead_target_world = WorldState::new(
      floor_map(2, 1),
      vec![
        Actor::with_hit_points(
          ActorId::new(1),
          ActorKind::Player,
          Position::new(0, 0),
          HitPoints::new(10),
        ),
        Actor::with_hit_points(
          ActorId::new(2),
          ActorKind::Enemy,
          Position::new(1, 0),
          HitPoints::new(1),
        ),
      ],
    )
    .unwrap();
    dead_target_world
      .execute(Command::Attack {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      })
      .unwrap();
    assert_eq!(
      dead_target_world.execute(Command::Attack {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      }),
      Err(CommandError::TargetDead(ActorId::new(2)))
    );
  }

  #[test]
  fn rejects_an_actor_that_starts_with_zero_hit_points() {
    assert_eq!(
      WorldState::new(
        floor_map(1, 1),
        vec![Actor::with_hit_points(
          ActorId::new(1),
          ActorKind::Player,
          Position::new(0, 0),
          HitPoints::new(0),
        )],
      ),
      Err(WorldError::ActorDeadAtStart {
        actor: ActorId::new(1),
      })
    );
  }

  #[test]
  fn enemy_chase_uses_horizontal_priority_for_diagonal_targets() {
    let mut world = WorldState::new(
      floor_map(4, 4),
      vec![
        Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(0, 0)),
        Actor::new(ActorId::new(2), ActorKind::Player, Position::new(2, 2)),
      ],
    )
    .unwrap();

    let result = world
      .execute(Command::Chase {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      })
      .unwrap();

    assert_eq!(
      result.events(),
      &[Event::Moved {
        actor: ActorId::new(1),
        from: Position::new(0, 0),
        to: Position::new(1, 0),
      }]
    );
    assert_eq!(
      world.actor(ActorId::new(1)).unwrap().ready_at(),
      ActionTime::new(1)
    );
  }

  #[test]
  fn enemy_chase_uses_vertical_direction_when_columns_align() {
    let mut world = WorldState::new(
      floor_map(1, 3),
      vec![
        Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(0, 0)),
        Actor::new(ActorId::new(2), ActorKind::Player, Position::new(0, 2)),
      ],
    )
    .unwrap();

    let result = world
      .execute(Command::Chase {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      })
      .unwrap();

    assert_eq!(
      result.events(),
      &[Event::Moved {
        actor: ActorId::new(1),
        from: Position::new(0, 0),
        to: Position::new(0, 1),
      }]
    );
  }

  #[test]
  fn enemy_chase_reuses_terrain_and_actor_blocking() {
    let map = GridMap::from_tiles(3, 1, vec![Tile::Floor, Tile::Wall, Tile::Floor]).unwrap();
    let mut terrain_world = WorldState::new(
      map,
      vec![
        Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(0, 0)),
        Actor::new(ActorId::new(2), ActorKind::Player, Position::new(2, 0)),
      ],
    )
    .unwrap();
    let terrain_result = terrain_world
      .execute(Command::Chase {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      })
      .unwrap();
    assert_eq!(
      terrain_result.events(),
      &[Event::MovementBlocked {
        actor: ActorId::new(1),
        from: Position::new(0, 0),
        to: Position::new(1, 0),
        reason: BlockReason::Terrain,
      }]
    );
    assert_eq!(
      terrain_world.actor(ActorId::new(1)).unwrap().ready_at(),
      ActionTime::new(1)
    );

    let mut actor_world = WorldState::new(
      floor_map(2, 1),
      vec![
        Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(0, 0)),
        Actor::new(ActorId::new(2), ActorKind::Player, Position::new(1, 0)),
      ],
    )
    .unwrap();
    let actor_result = actor_world
      .execute(Command::Chase {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      })
      .unwrap();
    assert_eq!(
      actor_result.events(),
      &[Event::MovementBlocked {
        actor: ActorId::new(1),
        from: Position::new(0, 0),
        to: Position::new(1, 0),
        reason: BlockReason::Actor(ActorId::new(2)),
      }]
    );
  }

  #[test]
  fn rejects_player_self_unknown_and_dead_chase_targets() {
    let mut player_world = WorldState::new(
      floor_map(2, 1),
      vec![
        Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
        Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(1, 0)),
      ],
    )
    .unwrap();
    assert_eq!(
      player_world.execute(Command::Chase {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      }),
      Err(CommandError::ChaseRequiresEnemy(ActorId::new(1)))
    );

    let mut self_world = WorldState::new(
      floor_map(1, 1),
      vec![Actor::new(
        ActorId::new(1),
        ActorKind::Enemy,
        Position::new(0, 0),
      )],
    )
    .unwrap();
    assert_eq!(
      self_world.execute(Command::Chase {
        actor: ActorId::new(1),
        target: ActorId::new(1),
      }),
      Err(CommandError::CannotChaseSelf(ActorId::new(1)))
    );

    assert_eq!(
      self_world.execute(Command::Chase {
        actor: ActorId::new(1),
        target: ActorId::new(99),
      }),
      Err(CommandError::UnknownTarget(ActorId::new(99)))
    );

    let mut dead_world = WorldState::new(
      floor_map(2, 1),
      vec![
        Actor::with_hit_points(
          ActorId::new(1),
          ActorKind::Enemy,
          Position::new(0, 0),
          HitPoints::new(10),
        ),
        Actor::with_hit_points(
          ActorId::new(2),
          ActorKind::Player,
          Position::new(1, 0),
          HitPoints::new(1),
        ),
      ],
    )
    .unwrap();
    dead_world
      .execute(Command::Attack {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      })
      .unwrap();
    assert_eq!(
      dead_world.execute(Command::Chase {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      }),
      Err(CommandError::TargetDead(ActorId::new(2)))
    );
  }

  #[test]
  fn enemy_chase_can_enter_a_dead_non_target_tile() {
    let mut world = WorldState::new(
      floor_map(3, 1),
      vec![
        Actor::with_hit_points(
          ActorId::new(1),
          ActorKind::Enemy,
          Position::new(0, 0),
          HitPoints::new(10),
        ),
        Actor::with_hit_points(
          ActorId::new(2),
          ActorKind::Player,
          Position::new(1, 0),
          HitPoints::new(1),
        ),
        Actor::new(ActorId::new(3), ActorKind::Player, Position::new(2, 0)),
      ],
    )
    .unwrap();

    world
      .execute(Command::Attack {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      })
      .unwrap();
    world
      .execute(Command::Wait {
        actor: ActorId::new(3),
      })
      .unwrap();
    let result = world
      .execute(Command::Chase {
        actor: ActorId::new(1),
        target: ActorId::new(3),
      })
      .unwrap();

    assert_eq!(
      result.events(),
      &[Event::Moved {
        actor: ActorId::new(1),
        from: Position::new(0, 0),
        to: Position::new(1, 0),
      }]
    );
    assert_eq!(
      world.actor(ActorId::new(1)).unwrap().ready_at(),
      ActionTime::new(2)
    );
  }

  #[test]
  fn replay_trace_digest_is_sensitive_to_seed_and_command_order() {
    let move_command = Command::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    };
    let wait_command = Command::Wait {
      actor: ActorId::new(1),
    };
    let mut first = ReplayTrace::new(7);
    first.record(move_command);
    first.record(wait_command);

    let mut reordered = ReplayTrace::new(7);
    reordered.record(wait_command);
    reordered.record(move_command);

    let mut reseeded = ReplayTrace::new(8);
    reseeded.record(move_command);
    reseeded.record(wait_command);

    let mut identical = ReplayTrace::new(7);
    identical.record(move_command);
    identical.record(wait_command);

    assert_eq!(first.seed(), 7);
    assert_eq!(first.commands(), &[move_command, wait_command]);
    assert_eq!(first.digest(), identical.digest());
    assert_ne!(first.digest(), reordered.digest());
    assert_ne!(first.digest(), reseeded.digest());
  }

  #[test]
  fn equivalent_worlds_have_equal_digests_after_identical_combat_transitions() {
    let actors = vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(1, 0),
        HitPoints::new(1),
      ),
    ];
    let mut first = WorldState::new(floor_map(2, 1), actors.clone()).unwrap();
    let mut second = WorldState::new(floor_map(2, 1), actors).unwrap();
    let initial_digest = first.digest();
    let commands = [
      Command::Attack {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      },
      Command::Move {
        actor: ActorId::new(1),
        direction: Direction::East,
      },
    ];
    for command in commands {
      first.execute(command).unwrap();
      second.execute(command).unwrap();
    }

    assert_ne!(initial_digest, first.digest());
    assert_eq!(first.digest(), second.digest());
  }

  #[test]
  fn state_digest_changes_when_map_semantics_differ() {
    let actors = vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )];
    let floor_world =
      WorldState::new(GridMap::filled(2, 1, Tile::Floor).unwrap(), actors.clone()).unwrap();
    let wall_world = WorldState::new(
      GridMap::from_tiles(2, 1, vec![Tile::Floor, Tile::Wall]).unwrap(),
      actors,
    )
    .unwrap();

    assert_ne!(floor_world.digest(), wall_world.digest());
  }

  #[test]
  fn scheduler_orders_equal_ready_times_by_actor_id() {
    let mut world = WorldState::new(
      floor_map(3, 1),
      vec![
        Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 0)),
        Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      ],
    )
    .unwrap();

    assert_eq!(world.next_actor(), Some(ActorId::new(1)));
    let first = world
      .execute(Command::Wait {
        actor: ActorId::new(1),
      })
      .unwrap();
    assert_eq!(first.next_actor(), Some(ActorId::new(2)));
    let second = world
      .execute(Command::Wait {
        actor: ActorId::new(2),
      })
      .unwrap();
    assert_eq!(second.next_actor(), Some(ActorId::new(1)));
  }

  #[test]
  fn rejects_a_command_for_an_unscheduled_actor() {
    let mut world = WorldState::new(
      floor_map(2, 1),
      vec![
        Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
        Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(1, 0)),
      ],
    )
    .unwrap();

    assert_eq!(
      world.execute(Command::Wait {
        actor: ActorId::new(2)
      }),
      Err(CommandError::ActorNotScheduled {
        requested: ActorId::new(2),
        scheduled: ActorId::new(1),
      })
    );
  }

  #[test]
  fn rejects_invalid_world_occupancy_and_map_data() {
    assert_eq!(GridMap::from_tiles(0, 1, vec![]), Err(MapError::ZeroWidth));
    assert_eq!(
      GridMap::from_tiles(2, 1, vec![Tile::Floor]),
      Err(MapError::TileCountMismatch {
        expected: 2,
        actual: 1,
      })
    );
    assert_eq!(
      GridMap::from_tiles(i32::MAX as u32 + 1, 1, vec![]),
      Err(MapError::CoordinateRange {
        width: i32::MAX as u32 + 1,
        height: 1,
      })
    );

    let map = GridMap::filled(2, 1, Tile::Floor).unwrap();
    assert_eq!(
      WorldState::new(
        map.clone(),
        vec![
          Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
          Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(0, 0)),
        ],
      ),
      Err(WorldError::OverlappingActors {
        first: ActorId::new(1),
        second: ActorId::new(2),
        position: Position::new(0, 0),
      })
    );

    assert_eq!(
      WorldState::new(
        map.clone(),
        vec![
          Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
          Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(1, 0)),
        ],
      ),
      Err(WorldError::DuplicateActorId(ActorId::new(1)))
    );

    assert_eq!(
      WorldState::new(
        map,
        vec![Actor::new(
          ActorId::new(1),
          ActorKind::Player,
          Position::new(2, 0),
        )],
      ),
      Err(WorldError::ActorOutOfBounds {
        actor: ActorId::new(1),
        position: Position::new(2, 0),
      })
    );

    let wall_map = GridMap::from_tiles(1, 1, vec![Tile::Wall]).unwrap();
    assert_eq!(
      WorldState::new(
        wall_map,
        vec![Actor::new(
          ActorId::new(1),
          ActorKind::Player,
          Position::new(0, 0),
        )],
      ),
      Err(WorldError::ActorOnBlockedTile {
        actor: ActorId::new(1),
        position: Position::new(0, 0),
      })
    );
  }

  #[test]
  fn scheduled_pickup_preserves_ground_order_and_advances_action() {
    let mut world = WorldState::new(
      floor_map(1, 1),
      vec![Actor::new(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
      )],
    )
    .expect("test world should be valid");
    world
      .give_item(
        ActorId::new(1),
        Item::new(ItemId::new(11), ItemDefinitionId::new(101)),
      )
      .expect("first item should be added");
    world
      .give_item(
        ActorId::new(1),
        Item::new(ItemId::new(12), ItemDefinitionId::new(102)),
      )
      .expect("second item should be added");
    world
      .drop_item(ActorId::new(1), ItemId::new(11))
      .expect("first item should drop");
    world
      .drop_item(ActorId::new(1), ItemId::new(12))
      .expect("second item should drop");

    assert_eq!(
      world.legal_commands(),
      vec![
        Command::Move {
          actor: ActorId::new(1),
          direction: Direction::North,
        },
        Command::Move {
          actor: ActorId::new(1),
          direction: Direction::South,
        },
        Command::Move {
          actor: ActorId::new(1),
          direction: Direction::West,
        },
        Command::Move {
          actor: ActorId::new(1),
          direction: Direction::East,
        },
        Command::Wait {
          actor: ActorId::new(1),
        },
        Command::Pickup {
          actor: ActorId::new(1),
          item: ItemId::new(11),
        },
        Command::Pickup {
          actor: ActorId::new(1),
          item: ItemId::new(12),
        },
      ]
    );
    let before = world.digest();
    let result = world
      .execute(Command::Pickup {
        actor: ActorId::new(1),
        item: ItemId::new(11),
      })
      .expect("ground pickup should be accepted");
    assert_eq!(
      result.events(),
      &[Event::ItemPickedUp {
        actor: ActorId::new(1),
        item: ItemId::new(11),
      }]
    );
    assert_eq!(
      world.actor(ActorId::new(1)).unwrap().ready_at(),
      ActionTime::new(1)
    );
    assert_eq!(
      world.actor(ActorId::new(1)).unwrap().inventory()[0].id(),
      ItemId::new(11)
    );
    assert_eq!(world.ground_items()[0].items()[0].id(), ItemId::new(12));
    assert_ne!(world.digest(), before);
  }

  #[test]
  fn enemy_pickup_is_not_legal_and_rejected_atomically() {
    let mut world = WorldState::new(
      floor_map(2, 1),
      vec![
        Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(0, 0)),
        Actor::new(ActorId::new(2), ActorKind::Player, Position::new(1, 0)),
      ],
    )
    .expect("test world should be valid");
    let item = Item::new(ItemId::new(11), ItemDefinitionId::new(101));
    world
      .give_item(ActorId::new(1), item)
      .expect("item should be accepted");
    world
      .drop_item(ActorId::new(1), item.id())
      .expect("item should drop");
    assert!(!world.legal_commands().iter().any(|command| {
      matches!(command, Command::Pickup { actor, item: candidate } if *actor == ActorId::new(1) && *candidate == item.id())
    }));
    let before = world.clone();
    assert_eq!(
      world.execute(Command::Pickup {
        actor: ActorId::new(1),
        item: item.id(),
      }),
      Err(CommandError::PickupRequiresPlayer(ActorId::new(1)))
    );
    assert_eq!(world, before);
  }

  #[test]
  fn rejected_pickup_preserves_world_and_replay_evidence() {
    let mut world = WorldState::new(
      floor_map(1, 1),
      vec![Actor::new(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
      )],
    )
    .expect("test world should be valid");
    let before = world.clone();
    assert_eq!(
      world.execute(Command::Pickup {
        actor: ActorId::new(1),
        item: ItemId::new(99),
      }),
      Err(CommandError::ItemNotOnGround {
        actor: ActorId::new(1),
        item: ItemId::new(99),
      })
    );
    assert_eq!(world, before);
  }
}
