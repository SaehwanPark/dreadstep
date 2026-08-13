//! Versioned command requests converted to and from core commands.
//!
//! This module does not validate or execute actions. [`dreadstep_core::WorldState`] remains the
//! authority for legality and outcomes.

use dreadstep_core::{Command as CoreCommand, Direction as CoreDirection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{ActorId, ItemId, Position};

/// A cardinal direction in a protocol action request.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
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
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
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
  /// Open one adjacent closed door.
  Interact {
    /// The actor issuing the request.
    actor: ActorId,
    /// The adjacent closed door to open.
    position: Position,
  },
  /// Kick one adjacent closed door open and create a deterministic noise event.
  Kick {
    /// The actor issuing the kick.
    actor: ActorId,
    /// The adjacent closed door to kick open.
    position: Position,
  },
  /// Close one adjacent open door.
  Close {
    /// The actor issuing the close command.
    actor: ActorId,
    /// The adjacent open door to close.
    position: Position,
  },
  /// Break one adjacent breakable terrain cell into floor.
  Break {
    /// The actor issuing the break command.
    actor: ActorId,
    /// The adjacent breakable terrain to destroy.
    position: Position,
  },
  /// Attack an adjacent actor.
  Attack {
    /// The actor issuing the request.
    actor: ActorId,
    /// The actor being targeted.
    target: ActorId,
  },
  /// Make a bounded ranged attack against an actor two or three clear cardinal tiles away.
  RangedAttack {
    /// The actor issuing the request.
    actor: ActorId,
    /// The actor being targeted.
    target: ActorId,
  },
  /// Throw one owned, unequipped throwable item at a living cardinal ranged target.
  Throw {
    /// The player issuing the request.
    actor: ActorId,
    /// The owned throwable item instance to consume.
    item: ItemId,
    /// The living actor receiving the throwable effect.
    target: ActorId,
  },
  /// Move one authored kiter one tile away from an adjacent living target.
  Retreat {
    /// The kiter issuing the request.
    actor: ActorId,
    /// The adjacent living actor being escaped.
    target: ActorId,
  },
  /// Chase a living actor by one deterministic step.
  Chase {
    /// The enemy issuing the request.
    actor: ActorId,
    /// The actor being pursued.
    target: ActorId,
  },
  /// Move one step toward a one-use noise position heard by an enemy.
  Investigate {
    /// The enemy issuing the request.
    actor: ActorId,
    /// The exact noise position to approach.
    position: Position,
  },
  /// Equip one owned item, replacing any previous equipment.
  Equip {
    /// The actor issuing the request.
    actor: ActorId,
    /// The owned item instance to equip.
    item: ItemId,
  },
  /// Unequip the actor's current item reference.
  Unequip {
    /// The actor issuing the request.
    actor: ActorId,
  },
  /// Consume one owned, unequipped item instance.
  UseItem {
    /// The actor issuing the request.
    actor: ActorId,
    /// The owned item instance to consume.
    item: ItemId,
  },
  /// Pick one item from the actor's current ground stack.
  Pickup {
    /// The actor issuing the request.
    actor: ActorId,
    /// The ground item instance to pick up.
    item: ItemId,
  },
  /// Drop one owned unequipped item at the player's current position.
  Drop {
    /// The player issuing the request.
    actor: ActorId,
    /// The owned item instance to drop.
    item: ItemId,
  },
  /// Restore a player's ranged ammunition to the fixed capacity.
  Reload {
    /// The player issuing the request.
    actor: ActorId,
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
      CommandRequest::Interact { actor, position } => Self::Interact {
        actor: dreadstep_core::ActorId::new(actor.value()),
        position: dreadstep_core::Position::new(position.x(), position.y()),
      },
      CommandRequest::Kick { actor, position } => Self::Kick {
        actor: dreadstep_core::ActorId::new(actor.value()),
        position: dreadstep_core::Position::new(position.x(), position.y()),
      },
      CommandRequest::Close { actor, position } => Self::Close {
        actor: dreadstep_core::ActorId::new(actor.value()),
        position: dreadstep_core::Position::new(position.x(), position.y()),
      },
      CommandRequest::Break { actor, position } => Self::Break {
        actor: dreadstep_core::ActorId::new(actor.value()),
        position: dreadstep_core::Position::new(position.x(), position.y()),
      },
      CommandRequest::Attack { actor, target } => Self::Attack {
        actor: dreadstep_core::ActorId::new(actor.value()),
        target: dreadstep_core::ActorId::new(target.value()),
      },
      CommandRequest::RangedAttack { actor, target } => Self::RangedAttack {
        actor: dreadstep_core::ActorId::new(actor.value()),
        target: dreadstep_core::ActorId::new(target.value()),
      },
      CommandRequest::Throw {
        actor,
        item,
        target,
      } => Self::Throw {
        actor: dreadstep_core::ActorId::new(actor.value()),
        item: dreadstep_core::ItemId::new(item.value()),
        target: dreadstep_core::ActorId::new(target.value()),
      },
      CommandRequest::Retreat { actor, target } => Self::Retreat {
        actor: dreadstep_core::ActorId::new(actor.value()),
        target: dreadstep_core::ActorId::new(target.value()),
      },
      CommandRequest::Chase { actor, target } => Self::Chase {
        actor: dreadstep_core::ActorId::new(actor.value()),
        target: dreadstep_core::ActorId::new(target.value()),
      },
      CommandRequest::Investigate { actor, position } => Self::Investigate {
        actor: dreadstep_core::ActorId::new(actor.value()),
        position: dreadstep_core::Position::new(position.x(), position.y()),
      },
      CommandRequest::Equip { actor, item } => Self::Equip {
        actor: dreadstep_core::ActorId::new(actor.value()),
        item: dreadstep_core::ItemId::new(item.value()),
      },
      CommandRequest::Unequip { actor } => Self::Unequip {
        actor: dreadstep_core::ActorId::new(actor.value()),
      },
      CommandRequest::UseItem { actor, item } => Self::UseItem {
        actor: dreadstep_core::ActorId::new(actor.value()),
        item: dreadstep_core::ItemId::new(item.value()),
      },
      CommandRequest::Pickup { actor, item } => Self::Pickup {
        actor: dreadstep_core::ActorId::new(actor.value()),
        item: dreadstep_core::ItemId::new(item.value()),
      },
      CommandRequest::Drop { actor, item } => Self::Drop {
        actor: dreadstep_core::ActorId::new(actor.value()),
        item: dreadstep_core::ItemId::new(item.value()),
      },
      CommandRequest::Reload { actor } => Self::Reload {
        actor: dreadstep_core::ActorId::new(actor.value()),
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
      CoreCommand::Interact { actor, position } => Self::Interact {
        actor: ActorId::new(actor.value()),
        position: Position::new(position.x(), position.y()),
      },
      CoreCommand::Kick { actor, position } => Self::Kick {
        actor: ActorId::new(actor.value()),
        position: Position::new(position.x(), position.y()),
      },
      CoreCommand::Close { actor, position } => Self::Close {
        actor: ActorId::new(actor.value()),
        position: Position::new(position.x(), position.y()),
      },
      CoreCommand::Break { actor, position } => Self::Break {
        actor: ActorId::new(actor.value()),
        position: Position::new(position.x(), position.y()),
      },
      CoreCommand::Attack { actor, target } => Self::Attack {
        actor: ActorId::new(actor.value()),
        target: ActorId::new(target.value()),
      },
      CoreCommand::RangedAttack { actor, target } => Self::RangedAttack {
        actor: ActorId::new(actor.value()),
        target: ActorId::new(target.value()),
      },
      CoreCommand::Throw {
        actor,
        item,
        target,
      } => Self::Throw {
        actor: ActorId::new(actor.value()),
        item: ItemId::new(item.value()),
        target: ActorId::new(target.value()),
      },
      CoreCommand::Retreat { actor, target } => Self::Retreat {
        actor: ActorId::new(actor.value()),
        target: ActorId::new(target.value()),
      },
      CoreCommand::Chase { actor, target } => Self::Chase {
        actor: ActorId::new(actor.value()),
        target: ActorId::new(target.value()),
      },
      CoreCommand::Investigate { actor, position } => Self::Investigate {
        actor: ActorId::new(actor.value()),
        position: Position::new(position.x(), position.y()),
      },
      CoreCommand::Equip { actor, item } => Self::Equip {
        actor: ActorId::new(actor.value()),
        item: ItemId::new(item.value()),
      },
      CoreCommand::Unequip { actor } => Self::Unequip {
        actor: ActorId::new(actor.value()),
      },
      CoreCommand::UseItem { actor, item } => Self::UseItem {
        actor: ActorId::new(actor.value()),
        item: ItemId::new(item.value()),
      },
      CoreCommand::Pickup { actor, item } => Self::Pickup {
        actor: ActorId::new(actor.value()),
        item: ItemId::new(item.value()),
      },
      CoreCommand::Drop { actor, item } => Self::Drop {
        actor: ActorId::new(actor.value()),
        item: ItemId::new(item.value()),
      },
      CoreCommand::Reload { actor } => Self::Reload {
        actor: ActorId::new(actor.value()),
      },
    }
  }
}
