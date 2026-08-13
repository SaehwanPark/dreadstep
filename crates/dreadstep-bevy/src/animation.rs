//! Animation cue projections derived from core events.

use bevy::ecs::resource::Resource;
use dreadstep_core::{
  ActionTime, ActorId, BlockReason, Damage, Event, HitPoints, ItemId, Position,
};

/// A typed animation signal derived from one core semantic event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationAnimationCue {
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
  },
  /// An actor picked one item from its current ground stack.
  ItemPickedUp {
    /// The actor whose inventory changed.
    actor: ActorId,
    /// The item instance moved into inventory.
    item: ItemId,
  },
}

impl PresentationAnimationCue {
  pub(crate) fn from_event(event: Event) -> Option<Self> {
    match event {
      Event::Moved { actor, from, to } => Some(Self::Moved { actor, from, to }),
      Event::MovementBlocked {
        actor,
        from,
        to,
        reason,
      } => Some(Self::MovementBlocked {
        actor,
        from,
        to,
        reason,
      }),
      Event::Waited { actor, at } => Some(Self::Waited { actor, at }),
      Event::Attacked {
        attacker,
        target,
        damage,
        remaining_hit_points,
      } => Some(Self::Attacked {
        attacker,
        target,
        damage,
        remaining_hit_points,
      }),
      Event::Died { actor } => Some(Self::Died { actor }),
      Event::ItemEquipped { actor, item } => Some(Self::ItemEquipped { actor, item }),
      Event::ItemUnequipped { actor, item } => Some(Self::ItemUnequipped { actor, item }),
      Event::ItemConsumed { actor, item, .. } => Some(Self::ItemConsumed { actor, item }),
      Event::ItemPickedUp { actor, item } => Some(Self::ItemPickedUp { actor, item }),
      Event::ItemThrown { .. }
      | Event::DoorOpened { .. }
      | Event::DoorClosed { .. }
      | Event::NoiseCreated { .. }
      | Event::ItemDropped { .. }
      | Event::Reloaded { .. }
      | Event::BreakableBroken { .. }
      | Event::TrapTriggered { .. }
      | Event::StatusApplied { .. }
      | Event::StatusExpired { .. } => None,
    }
  }
}

/// A disposable ordered buffer of typed animation signals from the latest runtime output.
#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct PresentationAnimationCues {
  pub(crate) cues: Vec<PresentationAnimationCue>,
}

impl PresentationAnimationCues {
  /// Creates an empty animation-cue projection.
  #[must_use]
  pub const fn new() -> Self {
    Self { cues: Vec::new() }
  }

  /// Returns cues in the core event order of the latest runtime output.
  #[must_use]
  pub fn cues(&self) -> &[PresentationAnimationCue] {
    &self.cues
  }
}
