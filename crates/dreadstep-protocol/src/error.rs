//! Versioned protocol errors converted from core construction and command failures.

use std::fmt;

use crate::{ActorId, ItemId, Position};
use dreadstep_core::{
  CommandError as CoreCommandError, MapError as CoreMapError, WorldError as CoreWorldError,
};

pub use crate::command_error::CommandError;

/// A protocol map-construction error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MapError {
  /// The map width is zero.
  ZeroWidth,
  /// The map height is zero.
  ZeroHeight,
  /// The map dimensions exceed the available tile buffer.
  TooLarge {
    /// The requested width.
    width: u32,
    /// The requested height.
    height: u32,
  },
  /// A dimension exceeds the signed coordinate domain.
  CoordinateRange {
    /// The requested width.
    width: u32,
    /// The requested height.
    height: u32,
  },
  /// The supplied tile count does not match the dimensions.
  TileCountMismatch {
    /// The expected tile count.
    expected: usize,
    /// The supplied tile count.
    actual: usize,
  },
}

impl From<CoreMapError> for MapError {
  fn from(error: CoreMapError) -> Self {
    match error {
      CoreMapError::ZeroWidth => Self::ZeroWidth,
      CoreMapError::ZeroHeight => Self::ZeroHeight,
      CoreMapError::TooLarge { width, height } => Self::TooLarge { width, height },
      CoreMapError::CoordinateRange { width, height } => Self::CoordinateRange { width, height },
      CoreMapError::TileCountMismatch { expected, actual } => {
        Self::TileCountMismatch { expected, actual }
      }
    }
  }
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

/// A protocol-owned scenario construction error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScenarioError {
  /// Core rejected the map description.
  Map(MapError),
  /// Core rejected one of the initial actors.
  World(WorldError),
}

impl From<CoreMapError> for ScenarioError {
  fn from(error: CoreMapError) -> Self {
    Self::Map(error.into())
  }
}

impl From<CoreWorldError> for ScenarioError {
  fn from(error: CoreWorldError) -> Self {
    Self::World(error.into())
  }
}

impl fmt::Display for ScenarioError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Map(error) => write!(formatter, "scenario map rejected: {error}"),
      Self::World(error) => write!(formatter, "scenario world rejected: {error}"),
    }
  }
}

impl std::error::Error for ScenarioError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Self::Map(error) => Some(error),
      Self::World(error) => Some(error),
    }
  }
}

/// A protocol-owned world validation error returned by tester mutation operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldError {
  /// A tester mutation addresses no actor in the world.
  UnknownActor(ActorId),
  /// An item identity is already owned by an actor in the world.
  DuplicateItemId(ItemId),
  /// A tester mutation would exceed the actor's fixed inventory capacity.
  InventoryFull(ActorId),
  /// A tester transfer source does not own the requested item identity.
  ItemNotOwned {
    /// The actor whose inventory was searched.
    actor: ActorId,
    /// The item identity that was not found.
    item: ItemId,
  },
  /// A tester mutation attempted to move an equipped item.
  ItemEquipped {
    /// The actor whose equipment references the item.
    actor: ActorId,
    /// The equipped item identity.
    item: ItemId,
  },
  /// A tester pickup source has no matching item in its current ground stack.
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
      CoreWorldError::DuplicateItemId(item) => Self::DuplicateItemId(ItemId::new(item.value())),
      CoreWorldError::InventoryFull(actor) => Self::InventoryFull(ActorId::new(actor.value())),
      CoreWorldError::ItemNotOwned { actor, item } => Self::ItemNotOwned {
        actor: ActorId::new(actor.value()),
        item: ItemId::new(item.value()),
      },
      CoreWorldError::ItemEquipped { actor, item } => Self::ItemEquipped {
        actor: ActorId::new(actor.value()),
        item: ItemId::new(item.value()),
      },
      CoreWorldError::ItemNotOnGround { actor, item } => Self::ItemNotOnGround {
        actor: ActorId::new(actor.value()),
        item: ItemId::new(item.value()),
      },
      CoreWorldError::TeleportOutOfBounds { actor, position } => Self::TeleportOutOfBounds {
        actor: ActorId::new(actor.value()),
        position: Position::new(position.x(), position.y()),
      },
      CoreWorldError::TeleportOnBlockedTile { actor, position } => Self::TeleportOnBlockedTile {
        actor: ActorId::new(actor.value()),
        position: Position::new(position.x(), position.y()),
      },
      CoreWorldError::TeleportOccupied {
        actor,
        blocker,
        position,
      } => Self::TeleportOccupied {
        actor: ActorId::new(actor.value()),
        blocker: ActorId::new(blocker.value()),
        position: Position::new(position.x(), position.y()),
      },
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
      Self::DuplicateItemId(item) => {
        write!(formatter, "item id {} is duplicated", item.value())
      }
      Self::InventoryFull(actor) => {
        write!(formatter, "actor {} inventory is full", actor.value())
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

impl From<CoreCommandError> for CommandError {
  #[expect(
    clippy::too_many_lines,
    reason = "the protocol boundary keeps every typed core rejection exhaustive"
  )]
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
      CoreCommandError::PickupRequiresPlayer(actor) => {
        Self::PickupRequiresPlayer(ActorId::new(actor.value()))
      }
      CoreCommandError::DropRequiresPlayer(actor) => {
        Self::DropRequiresPlayer(ActorId::new(actor.value()))
      }
      CoreCommandError::ReloadRequiresPlayer(actor) => {
        Self::ReloadRequiresPlayer(ActorId::new(actor.value()))
      }
      CoreCommandError::InteractTargetInvalid { actor, position } => Self::InteractTargetInvalid {
        actor: ActorId::new(actor.value()),
        position: Position::new(position.x(), position.y()),
      },
      CoreCommandError::KickTargetInvalid { actor, position } => Self::KickTargetInvalid {
        actor: ActorId::new(actor.value()),
        position: Position::new(position.x(), position.y()),
      },
      CoreCommandError::BreakTargetInvalid { actor, position } => Self::BreakTargetInvalid {
        actor: ActorId::new(actor.value()),
        position: Position::new(position.x(), position.y()),
      },
      CoreCommandError::CannotChaseSelf(actor) => {
        Self::CannotChaseSelf(ActorId::new(actor.value()))
      }
      CoreCommandError::RetreatRequiresKiter(actor) => {
        Self::RetreatRequiresKiter(ActorId::new(actor.value()))
      }
      CoreCommandError::CannotRetreatSelf(actor) => {
        Self::CannotRetreatSelf(ActorId::new(actor.value()))
      }
      CoreCommandError::RetreatTargetNotAdjacent { actor, target } => {
        Self::RetreatTargetNotAdjacent {
          actor: ActorId::new(actor.value()),
          target: ActorId::new(target.value()),
        }
      }
      CoreCommandError::RetreatNoEscape(actor) => {
        Self::RetreatNoEscape(ActorId::new(actor.value()))
      }
      CoreCommandError::InvestigateRequiresEnemy(actor) => {
        Self::InvestigateRequiresEnemy(ActorId::new(actor.value()))
      }
      CoreCommandError::NoNoiseToInvestigate(actor) => {
        Self::NoNoiseToInvestigate(ActorId::new(actor.value()))
      }
      CoreCommandError::InvestigateTargetInvalid { actor, position } => {
        Self::InvestigateTargetInvalid {
          actor: ActorId::new(actor.value()),
          position: Position::new(position.x(), position.y()),
        }
      }
      CoreCommandError::AttackOutOfRange { attacker, target } => Self::AttackOutOfRange {
        attacker: ActorId::new(attacker.value()),
        target: ActorId::new(target.value()),
      },
      CoreCommandError::RangedAttackOutOfRange { attacker, target } => {
        Self::RangedAttackOutOfRange {
          attacker: ActorId::new(attacker.value()),
          target: ActorId::new(target.value()),
        }
      }
      CoreCommandError::RangedAttackNoLineOfSight { attacker, target } => {
        Self::RangedAttackNoLineOfSight {
          attacker: ActorId::new(attacker.value()),
          target: ActorId::new(target.value()),
        }
      }
      CoreCommandError::RangedAttackNoAmmunition(actor) => {
        Self::RangedAttackNoAmmunition(ActorId::new(actor.value()))
      }
      CoreCommandError::ReloadNotNeeded(actor) => {
        Self::ReloadNotNeeded(ActorId::new(actor.value()))
      }
      CoreCommandError::ThrowRequiresPlayer(actor) => {
        Self::ThrowRequiresPlayer(ActorId::new(actor.value()))
      }
      CoreCommandError::CannotThrowSelf(actor) => {
        Self::CannotThrowSelf(ActorId::new(actor.value()))
      }
      CoreCommandError::ThrowOutOfRange { attacker, target } => Self::ThrowOutOfRange {
        attacker: ActorId::new(attacker.value()),
        target: ActorId::new(target.value()),
      },
      CoreCommandError::ThrowNoLineOfSight { attacker, target } => Self::ThrowNoLineOfSight {
        attacker: ActorId::new(attacker.value()),
        target: ActorId::new(target.value()),
      },
      CoreCommandError::ItemNotOwned { actor, item } => Self::ItemNotOwned {
        actor: ActorId::new(actor.value()),
        item: ItemId::new(item.value()),
      },
      CoreCommandError::InventoryFull(actor) => Self::InventoryFull(ActorId::new(actor.value())),
      CoreCommandError::ItemAlreadyEquipped { actor, item } => Self::ItemAlreadyEquipped {
        actor: ActorId::new(actor.value()),
        item: ItemId::new(item.value()),
      },
      CoreCommandError::NothingEquipped(actor) => {
        Self::NothingEquipped(ActorId::new(actor.value()))
      }
      CoreCommandError::ItemEquipped { actor, item } => Self::ItemEquipped {
        actor: ActorId::new(actor.value()),
        item: ItemId::new(item.value()),
      },
      CoreCommandError::ItemNotConsumable { actor, item } => Self::ItemNotConsumable {
        actor: ActorId::new(actor.value()),
        item: ItemId::new(item.value()),
      },
      CoreCommandError::ItemNotThrowable { actor, item } => Self::ItemNotThrowable {
        actor: ActorId::new(actor.value()),
        item: ItemId::new(item.value()),
      },
      CoreCommandError::ItemNotOnGround { actor, item } => Self::ItemNotOnGround {
        actor: ActorId::new(actor.value()),
        item: ItemId::new(item.value()),
      },
    }
  }
}

impl fmt::Display for CommandError {
  #[expect(
    clippy::too_many_lines,
    reason = "the protocol boundary keeps each typed rejection message exhaustive"
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
      Self::InteractTargetInvalid { actor, position } => write!(
        formatter,
        "actor {} cannot interact with ({}, {}): target is not an adjacent closed door",
        actor.value(),
        position.x(),
        position.y()
      ),
      Self::KickTargetInvalid { actor, position } => write!(
        formatter,
        "actor {} cannot kick ({}, {}): target is not an adjacent closed door",
        actor.value(),
        position.x(),
        position.y()
      ),
      Self::BreakTargetInvalid { actor, position } => write!(
        formatter,
        "actor {} cannot break ({}, {}): target is not an adjacent breakable tile",
        actor.value(),
        position.x(),
        position.y()
      ),
      Self::CannotChaseSelf(actor) => {
        write!(formatter, "actor {} cannot chase itself", actor.value())
      }
      Self::RetreatRequiresKiter(actor) => write!(
        formatter,
        "actor {} cannot retreat because only kiters may retreat",
        actor.value()
      ),
      Self::CannotRetreatSelf(actor) => {
        write!(
          formatter,
          "actor {} cannot retreat from itself",
          actor.value()
        )
      }
      Self::RetreatTargetNotAdjacent { actor, target } => write!(
        formatter,
        "kiter {} cannot retreat from non-adjacent target {}",
        actor.value(),
        target.value()
      ),
      Self::RetreatNoEscape(actor) => {
        write!(
          formatter,
          "kiter {} has no valid retreat tile",
          actor.value()
        )
      }
      Self::InvestigateRequiresEnemy(actor) => write!(
        formatter,
        "actor {} cannot investigate noise because only enemies may investigate",
        actor.value()
      ),
      Self::NoNoiseToInvestigate(actor) => {
        write!(
          formatter,
          "actor {} has no pending noise to investigate",
          actor.value()
        )
      }
      Self::InvestigateTargetInvalid { actor, position } => write!(
        formatter,
        "actor {} cannot investigate noise at ({}, {}): target is stale",
        actor.value(),
        position.x(),
        position.y()
      ),
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
      Self::ThrowRequiresPlayer(actor) => write!(
        formatter,
        "actor {} cannot throw because only players may throw",
        actor.value()
      ),
      Self::CannotThrowSelf(actor) => {
        write!(formatter, "actor {} cannot throw at itself", actor.value())
      }
      Self::ThrowOutOfRange { attacker, target } => write!(
        formatter,
        "actor {} cannot throw at target {} outside distance 2..=3",
        attacker.value(),
        target.value()
      ),
      Self::ThrowNoLineOfSight { attacker, target } => write!(
        formatter,
        "actor {} cannot throw at target {} without a clear cardinal line of sight",
        attacker.value(),
        target.value()
      ),
      Self::ItemNotOwned { actor, item } => write!(
        formatter,
        "actor {} does not own item {}",
        actor.value(),
        item.value()
      ),
      Self::InventoryFull(actor) => {
        write!(formatter, "actor {} inventory is full", actor.value())
      }
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
      Self::ItemNotConsumable { actor, item } => write!(
        formatter,
        "actor {} cannot consume non-consumable item {}",
        actor.value(),
        item.value()
      ),
      Self::ItemNotThrowable { actor, item } => write!(
        formatter,
        "actor {} cannot throw non-throwable item {}",
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
