//! Versioned semantic events converted from core outcomes.

use dreadstep_core::{BlockReason as CoreBlockReason, Event as CoreEvent};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
  ActionTime, ActorId, AmmunitionResult, HealingResult, HitPoints, ItemId, Position, StatusKind,
};

/// A protocol damage value.
#[derive(
  Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, JsonSchema, Serialize,
)]
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
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockReason {
  /// The destination is outside the map or blocked terrain.
  Terrain,
  /// The destination is occupied by another living actor.
  Actor(ActorId),
}

/// A semantic event projected for agent responses.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
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
    /// The door position that changed to an open doorway.
    position: Position,
  },
  /// A kick opened a door and created a fixed-radius noise source.
  NoiseCreated {
    /// The actor whose action created the noise.
    actor: ActorId,
    /// The position where the noise originated.
    position: Position,
    /// The fixed radius carried as future propagation evidence.
    radius: u8,
  },
  /// An actor closed one adjacent open door.
  DoorClosed {
    /// The actor that closed the door.
    actor: ActorId,
    /// The door position that changed to closed terrain.
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
    /// The consumed trap position.
    position: Position,
    /// The fixed damage applied by the trap.
    damage: Damage,
    /// The actor's hit points after trap damage.
    remaining_hit_points: HitPoints,
  },
  /// An actor received Chilled from a trap or thrown item.
  StatusApplied {
    /// The actor receiving the status.
    actor: ActorId,
    /// The applied status kind.
    status: StatusKind,
    /// The number of affected actions remaining.
    remaining_actions: u8,
  },
  /// An actor's chilled status expired after its affected action.
  StatusExpired {
    /// The actor whose status expired.
    actor: ActorId,
    /// The expired status kind.
    status: StatusKind,
  },
  /// A Frostcaster successfully cast Chilled at a living target.
  ChillCast {
    /// The Frostcaster that completed the cast.
    caster: ActorId,
    /// The living actor that received Chilled.
    target: ActorId,
  },
  /// An attack reduced a target's hit points.
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
  /// A player consumed one throwable item against a living target.
  ItemThrown {
    /// The player that threw the item.
    actor: ActorId,
    /// The consumed item instance.
    item: ItemId,
    /// The actor that received the throwable effect.
    target: ActorId,
  },
  /// An actor reached zero hit points.
  Died {
    /// The actor that died.
    actor: ActorId,
  },
  /// An actor equipped an owned item.
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
  /// An actor consumed one owned, unequipped item instance.
  ItemConsumed {
    /// The actor whose inventory changed.
    actor: ActorId,
    /// The item instance removed from inventory.
    item: ItemId,
    /// Optional healing evidence produced by the item effect.
    healing: Option<HealingResult>,
    /// Optional ammunition evidence produced by the item effect.
    ammunition: Option<AmmunitionResult>,
  },
  /// An actor picked one item from its current ground stack.
  ItemPickedUp {
    /// The actor whose inventory changed.
    actor: ActorId,
    /// The item instance removed from the ground stack.
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

impl From<CoreBlockReason> for BlockReason {
  fn from(reason: CoreBlockReason) -> Self {
    match reason {
      CoreBlockReason::Terrain => Self::Terrain,
      CoreBlockReason::Actor(actor) => Self::Actor(ActorId::new(actor.value())),
    }
  }
}

impl From<CoreEvent> for Event {
  #[expect(
    clippy::too_many_lines,
    reason = "the protocol event projection is intentionally exhaustive"
  )]
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
      CoreEvent::DoorOpened { actor, position } => Self::DoorOpened {
        actor: ActorId::new(actor.value()),
        position: Position::new(position.x(), position.y()),
      },
      CoreEvent::NoiseCreated {
        actor,
        position,
        radius,
      } => Self::NoiseCreated {
        actor: ActorId::new(actor.value()),
        position: Position::new(position.x(), position.y()),
        radius,
      },
      CoreEvent::DoorClosed { actor, position } => Self::DoorClosed {
        actor: ActorId::new(actor.value()),
        position: Position::new(position.x(), position.y()),
      },
      CoreEvent::BreakableBroken { actor, position } => Self::BreakableBroken {
        actor: ActorId::new(actor.value()),
        position: Position::new(position.x(), position.y()),
      },
      CoreEvent::TrapTriggered {
        actor,
        position,
        damage,
        remaining_hit_points,
      } => Self::TrapTriggered {
        actor: ActorId::new(actor.value()),
        position: Position::new(position.x(), position.y()),
        damage: Damage::new(damage.value()),
        remaining_hit_points: HitPoints::new(remaining_hit_points.value()),
      },
      CoreEvent::StatusApplied {
        actor,
        status,
        remaining_actions,
      } => Self::StatusApplied {
        actor: ActorId::new(actor.value()),
        status: status.into(),
        remaining_actions,
      },
      CoreEvent::StatusExpired { actor, status } => Self::StatusExpired {
        actor: ActorId::new(actor.value()),
        status: status.into(),
      },
      CoreEvent::ChillCast { caster, target } => Self::ChillCast {
        caster: ActorId::new(caster.value()),
        target: ActorId::new(target.value()),
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
      CoreEvent::ItemThrown {
        actor,
        item,
        target,
      } => Self::ItemThrown {
        actor: ActorId::new(actor.value()),
        item: ItemId::new(item.value()),
        target: ActorId::new(target.value()),
      },
      CoreEvent::Died { actor } => Self::Died {
        actor: ActorId::new(actor.value()),
      },
      CoreEvent::ItemEquipped { actor, item } => Self::ItemEquipped {
        actor: ActorId::new(actor.value()),
        item: ItemId::new(item.value()),
      },
      CoreEvent::ItemUnequipped { actor, item } => Self::ItemUnequipped {
        actor: ActorId::new(actor.value()),
        item: ItemId::new(item.value()),
      },
      CoreEvent::ItemConsumed {
        actor,
        item,
        healing,
        ammunition,
      } => Self::ItemConsumed {
        actor: ActorId::new(actor.value()),
        item: ItemId::new(item.value()),
        healing: healing.map(HealingResult::from_core),
        ammunition: ammunition.map(AmmunitionResult::from_core),
      },
      CoreEvent::ItemPickedUp { actor, item } => Self::ItemPickedUp {
        actor: ActorId::new(actor.value()),
        item: ItemId::new(item.value()),
      },
      CoreEvent::ItemDropped { actor, item } => Self::ItemDropped {
        actor: ActorId::new(actor.value()),
        item: ItemId::new(item.value()),
      },
      CoreEvent::Reloaded { actor, ammunition } => Self::Reloaded {
        actor: ActorId::new(actor.value()),
        ammunition,
      },
    }
  }
}
