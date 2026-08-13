//! Semantic commands accepted by the rules kernel.
//!
//! Validation and execution remain in [`crate::WorldState`]. This enum is the typed vocabulary
//! shared by every adapter.

use crate::{ActorId, Direction, ItemId, Position};

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
  /// Open one adjacent closed door.
  Interact {
    /// The actor issuing the interaction.
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
  /// Apply the existing Chilled status to a living actor within a clear cardinal ranged ray.
  CastChill {
    /// The Frostcaster issuing the cast.
    actor: ActorId,
    /// The living actor being chilled.
    target: ActorId,
  },
  /// Throw one owned authored item at a living cardinal ranged target.
  Throw {
    /// The player issuing the throw.
    actor: ActorId,
    /// The owned unequipped throwable item.
    item: ItemId,
    /// The living actor being targeted.
    target: ActorId,
  },
  /// Move an authored kiter one tile away from an adjacent living target.
  Retreat {
    /// The kiter issuing the retreat.
    actor: ActorId,
    /// The living actor whose adjacency triggered the retreat.
    target: ActorId,
  },
  /// Move an enemy one deterministic step toward a living target.
  Chase {
    /// The enemy issuing the chase command.
    actor: ActorId,
    /// The living actor being pursued.
    target: ActorId,
  },
  /// Move one step toward a one-use noise position heard by an enemy.
  Investigate {
    /// The enemy issuing the investigation.
    actor: ActorId,
    /// The exact noise position to approach.
    position: Position,
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
  pub(crate) const fn actor(self) -> ActorId {
    match self {
      Self::Move { actor, .. }
      | Self::Wait { actor }
      | Self::Interact { actor, .. }
      | Self::Kick { actor, .. }
      | Self::Close { actor, .. }
      | Self::Break { actor, .. }
      | Self::Attack { actor, .. }
      | Self::RangedAttack { actor, .. }
      | Self::CastChill { actor, .. }
      | Self::Throw { actor, .. }
      | Self::Retreat { actor, .. }
      | Self::Chase { actor, .. }
      | Self::Investigate { actor, .. }
      | Self::Equip { actor, .. }
      | Self::Unequip { actor }
      | Self::UseItem { actor, .. }
      | Self::Pickup { actor, .. }
      | Self::Drop { actor, .. }
      | Self::Reload { actor } => actor,
    }
  }
}
