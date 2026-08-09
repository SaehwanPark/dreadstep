//! Versioned external representations for Dreadstep domain concepts.
//!
//! Protocol types translate between stable wire or replay formats and the semantic types
//! owned by `dreadstep-core`. Transport-specific behavior belongs in its adapter crate,
//! not here.

#![forbid(unsafe_code)]

use dreadstep_core::{Actor as CoreActor, ActorKind as CoreActorKind, WorldState};

/// Version of the in-memory agent observation projection.
pub const PROTOCOL_VERSION: u16 = 1;

/// A stable actor identity in the protocol projection.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ActorId(u32);

impl ActorId {
  /// Creates a protocol actor identity from its numeric value.
  #[must_use]
  pub const fn new(value: u32) -> Self {
    Self(value)
  }

  /// Returns the numeric identity.
  #[must_use]
  pub const fn value(self) -> u32 {
    self.0
  }
}

/// An actor kind represented by the protocol projection.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ActorKind {
  /// The player-controlled actor.
  Player,
  /// An actor controlled by simulation or an adapter.
  Enemy,
}

/// A protocol position value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Position {
  x: i32,
  y: i32,
}

impl Position {
  /// Creates a protocol position from its coordinates.
  #[must_use]
  pub const fn new(x: i32, y: i32) -> Self {
    Self { x, y }
  }

  /// Returns the horizontal coordinate.
  #[must_use]
  pub const fn x(self) -> i32 {
    self.x
  }

  /// Returns the vertical coordinate.
  #[must_use]
  pub const fn y(self) -> i32 {
    self.y
  }
}

/// Protocol hit-point evidence for an actor.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HitPoints(u16);

impl HitPoints {
  /// Creates protocol hit-point evidence.
  #[must_use]
  pub const fn new(value: u16) -> Self {
    Self(value)
  }

  /// Returns the numeric hit-point value.
  #[must_use]
  pub const fn value(self) -> u16 {
    self.0
  }
}

/// Protocol life state for an actor record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LifeState {
  /// The actor can participate in scheduling and actions.
  Alive,
  /// The actor remains inspectable but cannot act or occupy a tile.
  Dead,
}

/// A protocol action timestamp.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ActionTime(u64);

impl ActionTime {
  /// Creates protocol action-time evidence.
  #[must_use]
  pub const fn new(value: u64) -> Self {
    Self(value)
  }

  /// Returns the numeric action time.
  #[must_use]
  pub const fn value(self) -> u64 {
    self.0
  }
}

/// A protocol view of the core's non-cryptographic state digest.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StateDigest(u64);

impl StateDigest {
  /// Creates protocol digest evidence.
  #[must_use]
  pub const fn new(value: u64) -> Self {
    Self(value)
  }

  /// Returns the numeric digest value.
  #[must_use]
  pub const fn value(self) -> u64 {
    self.0
  }
}

/// A read-only actor projection for agent observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActorSnapshot {
  id: ActorId,
  kind: ActorKind,
  position: Position,
  hit_points: HitPoints,
  life: LifeState,
  ready_at: ActionTime,
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
}

/// A versioned, read-only projection of semantic world state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldSnapshot {
  protocol_version: u16,
  current_time: ActionTime,
  next_actor: Option<ActorId>,
  digest: StateDigest,
  actors: Vec<ActorSnapshot>,
}

impl WorldSnapshot {
  /// Projects core-owned state without mutating it or applying game rules.
  #[must_use]
  pub fn from_world(world: &WorldState) -> Self {
    Self {
      protocol_version: PROTOCOL_VERSION,
      current_time: ActionTime::new(world.current_time().value()),
      next_actor: world.next_actor().map(|actor| ActorId::new(actor.value())),
      digest: StateDigest::new(world.digest().value()),
      actors: world.actors().map(ActorSnapshot::from_actor).collect(),
    }
  }

  /// Returns the protocol projection version.
  #[must_use]
  pub const fn protocol_version(&self) -> u16 {
    self.protocol_version
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
}
