//! Typed message projections of core events.

use bevy::ecs::resource::Resource;
use dreadstep_core::{
  ActionTime, ActorId, AmmunitionResult, BlockReason, Damage, Event, HealingResult, HitPoints,
  ItemId, Position,
};

/// A typed message projection of one core semantic event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationMessage {
  /// An actor entered a new map position.
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
    /// The opened door position.
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
  /// An actor consumed an owned, unequipped item instance.
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
    /// The item instance moved into inventory.
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

impl PresentationMessage {
  pub(crate) fn from_event(event: Event) -> Self {
    match event {
      Event::Moved { actor, from, to } => Self::Moved { actor, from, to },
      Event::MovementBlocked {
        actor,
        from,
        to,
        reason,
      } => Self::MovementBlocked {
        actor,
        from,
        to,
        reason,
      },
      Event::Waited { actor, at } => Self::Waited { actor, at },
      Event::DoorOpened { actor, position } => Self::DoorOpened { actor, position },
      Event::NoiseCreated {
        actor,
        position,
        radius,
      } => Self::NoiseCreated {
        actor,
        position,
        radius,
      },
      Event::BreakableBroken { actor, position } => Self::BreakableBroken { actor, position },
      Event::TrapTriggered {
        actor,
        position,
        damage,
        remaining_hit_points,
      } => Self::TrapTriggered {
        actor,
        position,
        damage,
        remaining_hit_points,
      },
      Event::Attacked {
        attacker,
        target,
        damage,
        remaining_hit_points,
      } => Self::Attacked {
        attacker,
        target,
        damage,
        remaining_hit_points,
      },
      Event::Died { actor } => Self::Died { actor },
      Event::ItemEquipped { actor, item } => Self::ItemEquipped { actor, item },
      Event::ItemUnequipped { actor, item } => Self::ItemUnequipped { actor, item },
      Event::ItemConsumed {
        actor,
        item,
        healing,
        ammunition,
      } => Self::ItemConsumed {
        actor,
        item,
        healing,
        ammunition,
      },
      Event::ItemPickedUp { actor, item } => Self::ItemPickedUp { actor, item },
      Event::ItemDropped { actor, item } => Self::ItemDropped { actor, item },
      Event::Reloaded { actor, ammunition } => Self::Reloaded { actor, ammunition },
    }
  }
}

/// Returns the stable diagnostic kind name for one current core event.
///
/// This exhaustive adapter is intentionally kept at the presentation boundary so adding a new
/// core event requires the desktop journal and smoke coverage to be updated in the same change.
#[must_use]
pub const fn showcase_event_name(event: Event) -> &'static str {
  match event {
    Event::Moved { .. } => "moved",
    Event::MovementBlocked { .. } => "movement_blocked",
    Event::Waited { .. } => "waited",
    Event::DoorOpened { .. } => "door_opened",
    Event::NoiseCreated { .. } => "noise_created",
    Event::BreakableBroken { .. } => "breakable_broken",
    Event::TrapTriggered { .. } => "trap_triggered",
    Event::Attacked { .. } => "attacked",
    Event::Died { .. } => "died",
    Event::ItemEquipped { .. } => "item_equipped",
    Event::ItemUnequipped { .. } => "item_unequipped",
    Event::ItemConsumed { .. } => "item_consumed",
    Event::ItemPickedUp { .. } => "item_picked_up",
    Event::ItemDropped { .. } => "item_dropped",
    Event::Reloaded { .. } => "reloaded",
  }
}

/// A disposable ordered buffer of typed messages derived from the latest runtime output.
#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct PresentationMessages {
  pub(crate) messages: Vec<PresentationMessage>,
}

impl PresentationMessages {
  /// Creates an empty message projection.
  #[must_use]
  pub const fn new() -> Self {
    Self {
      messages: Vec::new(),
    }
  }

  /// Returns messages in the core event order of the latest runtime output.
  #[must_use]
  pub fn messages(&self) -> &[PresentationMessage] {
    &self.messages
  }
}
