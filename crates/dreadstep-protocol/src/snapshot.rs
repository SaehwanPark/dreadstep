//! Read-only world and actor snapshots for observation.

use dreadstep_core::{
  Actor as CoreActor, ActorKind as CoreActorKind, RunOutcome as CoreRunOutcome, WorldState,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
  ActionTime, ActorId, ActorKind, GroundItemSnapshot, HitPoints, ItemId, ItemSnapshot, MeleeReach,
  PROTOCOL_VERSION, Position, StateDigest, StatusSnapshot,
};

/// Protocol life state for an actor record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LifeState {
  /// The actor can participate in scheduling and actions.
  Alive,
  /// The actor remains inspectable but cannot act or occupy a tile.
  Dead,
}

/// The canonical terminal outcome projected from core actor records.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunOutcome {
  /// The run has not reached a terminal condition.
  InProgress,
  /// The player is dead.
  Defeat,
  /// At least one enemy exists and every enemy is dead.
  Victory,
}

impl From<CoreRunOutcome> for RunOutcome {
  fn from(outcome: CoreRunOutcome) -> Self {
    match outcome {
      CoreRunOutcome::InProgress => Self::InProgress,
      CoreRunOutcome::Defeat => Self::Defeat,
      CoreRunOutcome::Victory => Self::Victory,
    }
  }
}

/// A read-only actor projection for agent observation.
#[derive(Clone, Debug, Eq, PartialEq, JsonSchema, Serialize)]
pub struct ActorSnapshot {
  id: ActorId,
  kind: ActorKind,
  position: Position,
  hit_points: HitPoints,
  life: LifeState,
  ready_at: ActionTime,
  melee_reach: MeleeReach,
  ranged_ammo: u16,
  /// The fixed maximum number of inventory entries this actor may carry.
  inventory_capacity: u16,
  inventory: Vec<ItemSnapshot>,
  equipped_item: Option<ItemId>,
  heard_noise: Option<Position>,
  status: Option<StatusSnapshot>,
}

impl ActorSnapshot {
  fn from_actor(actor: &CoreActor) -> Self {
    Self {
      id: ActorId::new(actor.id().value()),
      kind: match actor.kind() {
        CoreActorKind::Player => ActorKind::Player,
        CoreActorKind::Enemy => ActorKind::Enemy,
      },
      position: Position::new(actor.position().x(), actor.position().y()),
      hit_points: HitPoints::new(actor.hit_points().value()),
      life: if actor.is_alive() {
        LifeState::Alive
      } else {
        LifeState::Dead
      },
      ready_at: ActionTime::new(actor.ready_at().value()),
      // Core constructors guarantee a non-zero reach; the fallback keeps this read-only
      // projection panic-free if that invariant ever changes at the domain boundary.
      melee_reach: MeleeReach::new(actor.melee_reach().value()).unwrap_or_default(),
      ranged_ammo: actor.ranged_ammo(),
      inventory_capacity: u16::try_from(dreadstep_core::Actor::INVENTORY_CAPACITY)
        .expect("fixed inventory capacity fits protocol integer"),
      inventory: actor
        .inventory()
        .iter()
        .copied()
        .map(ItemSnapshot::from_item)
        .collect(),
      equipped_item: actor.equipped_item().map(|item| ItemId::new(item.value())),
      heard_noise: actor
        .heard_noise()
        .map(|position| Position::new(position.x(), position.y())),
      status: actor.status().map(StatusSnapshot::from_core),
    }
  }

  /// Returns the stable actor identity.
  #[must_use]
  pub const fn id(&self) -> ActorId {
    self.id
  }

  /// Returns the actor kind.
  #[must_use]
  pub const fn kind(&self) -> ActorKind {
    self.kind
  }

  /// Returns the actor position.
  #[must_use]
  pub const fn position(&self) -> Position {
    self.position
  }

  /// Returns the actor's current hit points.
  #[must_use]
  pub const fn hit_points(&self) -> HitPoints {
    self.hit_points
  }

  /// Returns the actor's explicit life state.
  #[must_use]
  pub const fn life(&self) -> LifeState {
    self.life
  }

  /// Returns whether the actor is living.
  #[must_use]
  pub const fn is_alive(&self) -> bool {
    matches!(self.life, LifeState::Alive)
  }

  /// Returns the actor's next ready time.
  #[must_use]
  pub const fn ready_at(&self) -> ActionTime {
    self.ready_at
  }

  /// Returns the actor's non-zero Manhattan melee reach.
  #[must_use]
  pub const fn melee_reach(&self) -> MeleeReach {
    self.melee_reach
  }

  /// Returns the actor's remaining ranged ammunition.
  #[must_use]
  pub const fn ranged_ammo(&self) -> u16 {
    self.ranged_ammo
  }

  /// Returns the actor's fixed maximum inventory size.
  #[must_use]
  pub const fn inventory_capacity(&self) -> u16 {
    self.inventory_capacity
  }

  /// Returns owned item snapshots in deterministic insertion order.
  #[must_use]
  pub fn inventory(&self) -> &[ItemSnapshot] {
    &self.inventory
  }

  /// Returns the optional equipped item identity, which points into [`Self::inventory`].
  #[must_use]
  pub const fn equipped_item(&self) -> Option<ItemId> {
    self.equipped_item
  }

  /// Returns the one-use noise position currently heard by this actor, if any.
  #[must_use]
  pub const fn heard_noise(&self) -> Option<Position> {
    self.heard_noise
  }

  /// Returns the actor's active status, if any.
  #[must_use]
  pub const fn status(&self) -> Option<StatusSnapshot> {
    self.status
  }
}

/// A versioned, read-only projection of semantic world state.
#[derive(Clone, Debug, Eq, PartialEq, JsonSchema, Serialize)]
pub struct WorldSnapshot {
  protocol_version: u16,
  outcome: RunOutcome,
  current_time: ActionTime,
  next_actor: Option<ActorId>,
  digest: StateDigest,
  actors: Vec<ActorSnapshot>,
  ground_items: Vec<GroundItemSnapshot>,
}

impl WorldSnapshot {
  /// Projects core-owned state without mutating it or applying game rules.
  #[must_use]
  pub fn from_world(world: &WorldState) -> Self {
    Self {
      protocol_version: PROTOCOL_VERSION,
      outcome: world.outcome().into(),
      current_time: ActionTime::new(world.current_time().value()),
      next_actor: world.next_actor().map(|actor| ActorId::new(actor.value())),
      digest: StateDigest::new(world.digest().value()),
      actors: world.actors().map(ActorSnapshot::from_actor).collect(),
      ground_items: world
        .ground_items()
        .iter()
        .map(GroundItemSnapshot::from_stack)
        .collect(),
    }
  }

  /// Returns the protocol projection version.
  #[must_use]
  pub const fn protocol_version(&self) -> u16 {
    self.protocol_version
  }

  /// Returns the canonical run outcome projected from core.
  #[must_use]
  pub const fn outcome(&self) -> RunOutcome {
    self.outcome
  }

  /// Returns the world's minimum ready time.
  #[must_use]
  pub const fn current_time(&self) -> ActionTime {
    self.current_time
  }

  /// Returns the actor selected by the core scheduler.
  #[must_use]
  pub const fn next_actor(&self) -> Option<ActorId> {
    self.next_actor
  }

  /// Returns the core-owned stable state digest.
  #[must_use]
  pub const fn digest(&self) -> StateDigest {
    self.digest
  }

  /// Returns actor records in stable identity order.
  #[must_use]
  pub fn actors(&self) -> &[ActorSnapshot] {
    &self.actors
  }

  /// Returns ground-item stacks in deterministic row-major position order.
  #[must_use]
  pub fn ground_items(&self) -> &[GroundItemSnapshot] {
    &self.ground_items
  }
}
