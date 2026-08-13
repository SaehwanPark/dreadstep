//! Semantic events emitted by accepted commands.
//!
//! Events are outcomes, not requests. Adapters format them; they must not decide game results.

use crate::{
  ActionTime, ActorId, AmmunitionResult, Damage, HealingResult, HitPoints, ItemId, Position,
  StatusKind,
};

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
  /// An actor opened a closed door at an adjacent position.
  DoorOpened {
    /// The actor that opened the door.
    actor: ActorId,
    /// The door position that changed to open.
    position: Position,
  },
  /// A kick opened a door and created a fixed-radius noise source.
  NoiseCreated {
    /// The actor whose action created the noise.
    actor: ActorId,
    /// The position where the noise originated.
    position: Position,
    /// The fixed radius used by the terrain-aware propagation query.
    radius: u8,
  },
  /// An actor closed one adjacent open door.
  DoorClosed {
    /// The actor that closed the door.
    actor: ActorId,
    /// The door position that changed to closed.
    position: Position,
  },
  /// An actor broke one adjacent breakable terrain cell into floor.
  BreakableBroken {
    /// The actor that broke the terrain.
    actor: ActorId,
    /// The terrain position that changed to floor.
    position: Position,
  },
  /// An actor entered a one-shot floor trap and took fixed damage.
  TrapTriggered {
    /// The actor that entered the trap.
    actor: ActorId,
    /// The trap position that was consumed.
    position: Position,
    /// The fixed damage applied by the trap.
    damage: Damage,
    /// The actor's hit points after trap damage.
    remaining_hit_points: HitPoints,
  },
  /// An actor entered a one-shot chill trap or otherwise received chilled.
  StatusApplied {
    /// The actor whose status changed.
    actor: ActorId,
    /// The applied status kind.
    status: StatusKind,
    /// The refreshed number of affected actions.
    remaining_actions: u8,
  },
  /// An actor's final affected action consumed a status.
  StatusExpired {
    /// The actor whose status expired.
    actor: ActorId,
    /// The expired status kind.
    status: StatusKind,
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
  /// A Frostcaster applied Chilled to a living ranged target.
  ChillCast {
    /// The Frostcaster that cast the status effect.
    caster: ActorId,
    /// The actor that received the status effect.
    target: ActorId,
  },
  /// A player consumed one throwable item against a living target.
  ItemThrown {
    /// The actor that threw the item.
    actor: ActorId,
    /// The consumed item identity.
    item: ItemId,
    /// The actor that received the throwable effect.
    target: ActorId,
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
  /// An actor consumed an owned item instance, optionally applying its authored effect.
  ItemConsumed {
    /// The actor whose inventory changed.
    actor: ActorId,
    /// The consumed item identity.
    item: ItemId,
    /// The deterministic healing result, when this item restores hit points.
    healing: Option<HealingResult>,
    /// The deterministic ammunition result, when this item restores ranged shots.
    ammunition: Option<AmmunitionResult>,
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
