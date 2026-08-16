//! Actor records, scheduling scalars, and combat resources.
//!
//! Ready time, inventory, and ammunition live on the actor record. World transitions mutate
//! these fields; adapters must not invent a second copy.

use crate::{ActorId, EnemyBehavior, HitPoints, Item, ItemId, Position, Status, StatusKind};

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

  pub(crate) fn checked_add(self, cost: ActionCost) -> Option<Self> {
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

  /// The fixed cost used by slow actors such as Zombies.
  pub const SLOW: Self = Self(2);

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
/// A typed amount of damage applied by an attack.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Damage(pub(crate) u16);

impl Damage {
  /// The fixed damage dealt by the basic melee command.
  pub const MELEE: Self = Self(1);

  /// The fixed damage dealt by the bounded ranged command.
  pub const RANGED: Self = Self(1);

  /// The fixed damage dealt when an actor enters a floor trap.
  pub const TRAP: Self = Self(1);

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

  /// Returns this damage amount after saturating reduction by another amount.
  #[must_use]
  pub const fn saturating_sub(self, reduction: Self) -> Self {
    Self(self.0.saturating_sub(reduction.0))
  }
}

/// A non-zero Manhattan distance at which an actor may perform melee attacks.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MeleeReach(u8);

impl MeleeReach {
  /// The default adjacent melee reach.
  pub const DEFAULT: Self = Self(1);

  /// The authored reach used by the starter weapon.
  pub const TWO: Self = Self(2);

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
/// An actor with a stable identity, kind, position, hit points, melee reach, ranged ammunition,
/// inventory, optional equipment, and next ready time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Actor {
  id: ActorId,
  kind: ActorKind,
  enemy_behavior: EnemyBehavior,
  pub(crate) position: Position,
  pub(crate) hit_points: HitPoints,
  max_hit_points: HitPoints,
  melee_reach: MeleeReach,
  pub(crate) inventory: Vec<Item>,
  pub(crate) equipped: Option<ItemId>,
  pub(crate) ranged_ammo: u16,
  pub(crate) ready_at: ActionTime,
  pub(crate) heard_noise: Option<Position>,
  pub(crate) status: Option<Status>,
}

impl Actor {
  /// The fixed number of opaque item instances an actor may carry.
  pub const INVENTORY_CAPACITY: usize = 4;

  /// The fixed capacity restored by the deterministic reload command.
  pub const RANGED_AMMO_CAPACITY: u16 = 3;

  /// The default number of ranged shots available to a newly created actor.
  pub const DEFAULT_RANGED_AMMO: u16 = Self::RANGED_AMMO_CAPACITY;

  /// Creates an actor that is ready at the beginning of the world timeline.
  #[must_use]
  pub const fn new(id: ActorId, kind: ActorKind, position: Position) -> Self {
    Self::with_hit_points(id, kind, position, HitPoints::new(10))
  }

  /// Creates a default-hit-point enemy with an authored behavior policy.
  #[must_use]
  pub const fn with_enemy_behavior(
    id: ActorId,
    position: Position,
    enemy_behavior: EnemyBehavior,
  ) -> Self {
    let mut actor = Self::new(id, ActorKind::Enemy, position);
    actor.enemy_behavior = enemy_behavior;
    actor
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
      enemy_behavior: EnemyBehavior::Pursuer,
      position,
      hit_points,
      max_hit_points: hit_points,
      melee_reach,
      inventory: Vec::new(),
      equipped: None,
      ranged_ammo,
      ready_at: ActionTime::new(0),
      heard_noise: None,
      status: None,
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

  /// Creates an actor with explicit hit points, melee reach, and authored enemy behavior.
  #[must_use]
  pub const fn with_melee_reach_and_behavior(
    id: ActorId,
    kind: ActorKind,
    position: Position,
    hit_points: HitPoints,
    melee_reach: MeleeReach,
    enemy_behavior: EnemyBehavior,
  ) -> Self {
    let mut actor = Self::with_melee_reach(id, kind, position, hit_points, melee_reach);
    actor.enemy_behavior = enemy_behavior;
    actor
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

  /// Returns the closed behavior policy authored for this actor.
  #[must_use]
  pub const fn enemy_behavior(&self) -> EnemyBehavior {
    self.enemy_behavior
  }

  pub(crate) fn set_enemy_behavior(&mut self, behavior: EnemyBehavior) -> EnemyBehavior {
    let previous = self.enemy_behavior;
    self.enemy_behavior = behavior;
    previous
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

  /// Returns this actor's authored maximum hit points.
  #[must_use]
  pub const fn max_hit_points(&self) -> HitPoints {
    self.max_hit_points
  }

  /// Returns this actor's non-zero Manhattan melee reach.
  #[must_use]
  pub fn melee_reach(&self) -> MeleeReach {
    self
      .equipped
      .and_then(|equipped| {
        self
          .inventory
          .iter()
          .find(|item| item.id() == equipped)
          .and_then(|item| {
            item.equipment_effect().and_then(|effect| match effect {
              crate::EquipmentEffect::MinimumMeleeReach { reach } => Some(reach),
              crate::EquipmentEffect::MeleeDamage { .. }
              | crate::EquipmentEffect::DamageReduction { .. } => None,
            })
          })
      })
      .map_or(self.melee_reach, |minimum| self.melee_reach.max(minimum))
  }

  /// Returns this actor's authored reach before equipment effects.
  #[must_use]
  pub const fn base_melee_reach(&self) -> MeleeReach {
    self.melee_reach
  }

  /// Returns the fixed melee damage plus any equipped authored damage bonus.
  #[must_use]
  pub fn melee_damage(&self) -> Damage {
    let bonus = self
      .equipped
      .and_then(|equipped| {
        self
          .inventory
          .iter()
          .find(|item| item.id() == equipped)
          .and_then(|item| match item.equipment_effect() {
            Some(crate::EquipmentEffect::MeleeDamage { amount }) => Some(amount.value()),
            Some(
              crate::EquipmentEffect::MinimumMeleeReach { .. }
              | crate::EquipmentEffect::DamageReduction { .. },
            )
            | None => None,
          })
      })
      .unwrap_or(0);
    Damage::new(Damage::MELEE.value().saturating_add(bonus))
  }

  /// Returns the equipped incoming-damage reduction, or zero when no armor effect is active.
  #[must_use]
  pub fn damage_reduction(&self) -> Damage {
    self
      .equipped
      .and_then(|equipped| {
        self
          .inventory
          .iter()
          .find(|item| item.id() == equipped)
          .and_then(|item| match item.equipment_effect() {
            Some(crate::EquipmentEffect::DamageReduction { amount }) => Some(amount),
            Some(
              crate::EquipmentEffect::MinimumMeleeReach { .. }
              | crate::EquipmentEffect::MeleeDamage { .. },
            )
            | None => None,
          })
      })
      .unwrap_or(Damage::new(0))
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

  /// Returns the one-use noise position currently heard by this actor, if any.
  #[must_use]
  pub const fn heard_noise(&self) -> Option<Position> {
    self.heard_noise
  }

  /// Returns the actor's currently active status, if any.
  #[must_use]
  pub const fn status(&self) -> Option<Status> {
    self.status
  }

  pub(crate) fn apply_chilled(&mut self) -> Status {
    let status = Status::chilled();
    self.status = Some(status);
    status
  }

  pub(crate) fn consume_status_action(&mut self) -> Option<StatusKind> {
    let status = self.status?;
    let remaining = status.after_action();
    self.status = remaining;
    remaining.is_none().then_some(status.kind())
  }
}
