//! Versioned external representations for Dreadstep domain concepts.
//!
//! Protocol types translate between stable wire or replay formats and the semantic types
//! owned by `dreadstep-core`. Transport-specific behavior belongs in its adapter crate,
//! not here.

#![forbid(unsafe_code)]

use std::fmt;

use dreadstep_core::{
  Actor as CoreActor, ActorKind as CoreActorKind, BlockReason as CoreBlockReason,
  Command as CoreCommand, CommandError as CoreCommandError, Direction as CoreDirection,
  Event as CoreEvent, WorldError as CoreWorldError, WorldState,
};

/// Version of the in-memory agent observation projection.
pub const PROTOCOL_VERSION: u16 = 1;

/// A cardinal direction in a protocol action request.
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

/// A typed agent request that can be converted into one core command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandRequest {
  /// Move one tile in a cardinal direction.
  Move {
    /// The actor issuing the request.
    actor: ActorId,
    /// The requested direction.
    direction: Direction,
  },
  /// Spend one standard action without changing position.
  Wait {
    /// The actor issuing the request.
    actor: ActorId,
  },
  /// Attack an adjacent actor.
  Attack {
    /// The actor issuing the request.
    actor: ActorId,
    /// The actor being targeted.
    target: ActorId,
  },
  /// Chase a living actor by one deterministic step.
  Chase {
    /// The enemy issuing the request.
    actor: ActorId,
    /// The actor being pursued.
    target: ActorId,
  },
}

impl From<CommandRequest> for CoreCommand {
  fn from(request: CommandRequest) -> Self {
    match request {
      CommandRequest::Move { actor, direction } => Self::Move {
        actor: dreadstep_core::ActorId::new(actor.value()),
        direction: match direction {
          Direction::North => CoreDirection::North,
          Direction::South => CoreDirection::South,
          Direction::West => CoreDirection::West,
          Direction::East => CoreDirection::East,
        },
      },
      CommandRequest::Wait { actor } => Self::Wait {
        actor: dreadstep_core::ActorId::new(actor.value()),
      },
      CommandRequest::Attack { actor, target } => Self::Attack {
        actor: dreadstep_core::ActorId::new(actor.value()),
        target: dreadstep_core::ActorId::new(target.value()),
      },
      CommandRequest::Chase { actor, target } => Self::Chase {
        actor: dreadstep_core::ActorId::new(actor.value()),
        target: dreadstep_core::ActorId::new(target.value()),
      },
    }
  }
}

impl From<CoreCommand> for CommandRequest {
  fn from(command: CoreCommand) -> Self {
    match command {
      CoreCommand::Move { actor, direction } => Self::Move {
        actor: ActorId::new(actor.value()),
        direction: match direction {
          CoreDirection::North => Direction::North,
          CoreDirection::South => Direction::South,
          CoreDirection::West => Direction::West,
          CoreDirection::East => Direction::East,
        },
      },
      CoreCommand::Wait { actor } => Self::Wait {
        actor: ActorId::new(actor.value()),
      },
      CoreCommand::Attack { actor, target } => Self::Attack {
        actor: ActorId::new(actor.value()),
        target: ActorId::new(target.value()),
      },
      CoreCommand::Chase { actor, target } => Self::Chase {
        actor: ActorId::new(actor.value()),
        target: ActorId::new(target.value()),
      },
    }
  }
}

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

/// In-memory replay evidence exposed to an agent without claiming a serialized replay format.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayEvidence {
  seed: u64,
  commands: Vec<CommandRequest>,
  digest: StateDigest,
}

impl ReplayEvidence {
  /// Creates replay evidence from an explicit seed, accepted requests, and trace digest.
  #[must_use]
  pub const fn new(seed: u64, commands: Vec<CommandRequest>, digest: StateDigest) -> Self {
    Self {
      seed,
      commands,
      digest,
    }
  }

  /// Returns the explicit run seed.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.seed
  }

  /// Returns accepted protocol requests in execution order.
  #[must_use]
  pub fn commands(&self) -> &[CommandRequest] {
    &self.commands
  }

  /// Returns the deterministic core trace digest.
  #[must_use]
  pub const fn digest(&self) -> StateDigest {
    self.digest
  }
}

/// A protocol-owned world validation error returned by tester mutation operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldError {
  /// A tester mutation addresses no actor in the world.
  UnknownActor(ActorId),
  /// The requested actor identity already exists.
  DuplicateActorId(ActorId),
  /// The requested actor position is outside the map.
  ActorOutOfBounds {
    /// The actor with the invalid position.
    actor: ActorId,
    /// The invalid position.
    position: Position,
  },
  /// The requested actor position is blocked terrain.
  ActorOnBlockedTile {
    /// The actor with the invalid position.
    actor: ActorId,
    /// The blocked position.
    position: Position,
  },
  /// The requested actor overlaps an existing living actor.
  OverlappingActors {
    /// The living actor already at the position.
    first: ActorId,
    /// The actor being inserted.
    second: ActorId,
    /// The shared position.
    position: Position,
  },
  /// The requested actor has zero hit points.
  ActorDeadAtStart {
    /// The actor with invalid hit points.
    actor: ActorId,
  },
}

impl From<CoreWorldError> for WorldError {
  fn from(error: CoreWorldError) -> Self {
    match error {
      CoreWorldError::UnknownActor(actor) => Self::UnknownActor(ActorId::new(actor.value())),
      CoreWorldError::DuplicateActorId(actor) => {
        Self::DuplicateActorId(ActorId::new(actor.value()))
      }
      CoreWorldError::ActorOutOfBounds { actor, position } => Self::ActorOutOfBounds {
        actor: ActorId::new(actor.value()),
        position: Position::new(position.x(), position.y()),
      },
      CoreWorldError::ActorOnBlockedTile { actor, position } => Self::ActorOnBlockedTile {
        actor: ActorId::new(actor.value()),
        position: Position::new(position.x(), position.y()),
      },
      CoreWorldError::OverlappingActors {
        first,
        second,
        position,
      } => Self::OverlappingActors {
        first: ActorId::new(first.value()),
        second: ActorId::new(second.value()),
        position: Position::new(position.x(), position.y()),
      },
      CoreWorldError::ActorDeadAtStart { actor } => Self::ActorDeadAtStart {
        actor: ActorId::new(actor.value()),
      },
    }
  }
}

impl fmt::Display for WorldError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnknownActor(actor) => write!(formatter, "unknown actor {}", actor.value()),
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

/// Protocol damage evidence emitted by an accepted attack.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Damage(u16);

impl Damage {
  /// Creates protocol damage evidence.
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

/// A protocol movement-blocking reason.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BlockReason {
  /// The destination is outside the map or blocked terrain.
  Terrain,
  /// The destination is occupied by another living actor.
  Actor(ActorId),
}

/// A semantic event projected for agent responses.
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
  /// A melee attack reduced a target's hit points.
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
  /// An actor reached zero hit points.
  Died {
    /// The actor that died.
    actor: ActorId,
  },
}

impl From<CoreBlockReason> for BlockReason {
  fn from(reason: CoreBlockReason) -> Self {
    match reason {
      CoreBlockReason::Terrain => Self::Terrain,
      CoreBlockReason::Actor(actor) => Self::Actor(ActorId::new(actor.value())),
    }
  }
}

impl From<CoreEvent> for Event {
  fn from(event: CoreEvent) -> Self {
    match event {
      CoreEvent::Moved { actor, from, to } => Self::Moved {
        actor: ActorId::new(actor.value()),
        from: Position::new(from.x(), from.y()),
        to: Position::new(to.x(), to.y()),
      },
      CoreEvent::MovementBlocked {
        actor,
        from,
        to,
        reason,
      } => Self::MovementBlocked {
        actor: ActorId::new(actor.value()),
        from: Position::new(from.x(), from.y()),
        to: Position::new(to.x(), to.y()),
        reason: reason.into(),
      },
      CoreEvent::Waited { actor, at } => Self::Waited {
        actor: ActorId::new(actor.value()),
        at: ActionTime::new(at.value()),
      },
      CoreEvent::Attacked {
        attacker,
        target,
        damage,
        remaining_hit_points,
      } => Self::Attacked {
        attacker: ActorId::new(attacker.value()),
        target: ActorId::new(target.value()),
        damage: Damage::new(damage.value()),
        remaining_hit_points: HitPoints::new(remaining_hit_points.value()),
      },
      CoreEvent::Died { actor } => Self::Died {
        actor: ActorId::new(actor.value()),
      },
    }
  }
}

/// A structured command rejection projected for an adapter boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandError {
  /// The command addresses no actor in the world.
  UnknownActor(ActorId),
  /// A different actor is scheduled to act first.
  ActorNotScheduled {
    /// The actor addressed by the request.
    requested: ActorId,
    /// The actor selected by the scheduler.
    scheduled: ActorId,
  },
  /// The actor's ready time cannot advance.
  ScheduleOverflow(ActorId),
  /// The command actor is dead.
  ActorDead(ActorId),
  /// The attack target is not present.
  UnknownTarget(ActorId),
  /// The attack target is dead.
  TargetDead(ActorId),
  /// An actor cannot attack itself.
  CannotAttackSelf(ActorId),
  /// A chase request must come from an enemy.
  ChaseRequiresEnemy(ActorId),
  /// An enemy cannot chase itself.
  CannotChaseSelf(ActorId),
  /// The attack target is not adjacent.
  AttackOutOfRange {
    /// The actor issuing the attack.
    attacker: ActorId,
    /// The actor outside melee range.
    target: ActorId,
  },
}

impl From<CoreCommandError> for CommandError {
  fn from(error: CoreCommandError) -> Self {
    match error {
      CoreCommandError::UnknownActor(actor) => Self::UnknownActor(ActorId::new(actor.value())),
      CoreCommandError::ActorNotScheduled {
        requested,
        scheduled,
      } => Self::ActorNotScheduled {
        requested: ActorId::new(requested.value()),
        scheduled: ActorId::new(scheduled.value()),
      },
      CoreCommandError::ScheduleOverflow(actor) => {
        Self::ScheduleOverflow(ActorId::new(actor.value()))
      }
      CoreCommandError::ActorDead(actor) => Self::ActorDead(ActorId::new(actor.value())),
      CoreCommandError::UnknownTarget(target) => Self::UnknownTarget(ActorId::new(target.value())),
      CoreCommandError::TargetDead(target) => Self::TargetDead(ActorId::new(target.value())),
      CoreCommandError::CannotAttackSelf(actor) => {
        Self::CannotAttackSelf(ActorId::new(actor.value()))
      }
      CoreCommandError::ChaseRequiresEnemy(actor) => {
        Self::ChaseRequiresEnemy(ActorId::new(actor.value()))
      }
      CoreCommandError::CannotChaseSelf(actor) => {
        Self::CannotChaseSelf(ActorId::new(actor.value()))
      }
      CoreCommandError::AttackOutOfRange { attacker, target } => Self::AttackOutOfRange {
        attacker: ActorId::new(attacker.value()),
        target: ActorId::new(target.value()),
      },
    }
  }
}

impl fmt::Display for CommandError {
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
      Self::UnknownTarget(target) => {
        write!(formatter, "unknown attack target {}", target.value())
      }
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
      Self::CannotChaseSelf(actor) => {
        write!(formatter, "actor {} cannot chase itself", actor.value())
      }
      Self::AttackOutOfRange { attacker, target } => write!(
        formatter,
        "actor {} cannot attack non-adjacent target {}",
        attacker.value(),
        target.value()
      ),
    }
  }
}

impl std::error::Error for CommandError {}

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
