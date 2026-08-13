//! Item instances, authored effects, and ground stacks.
//!
//! Core stores opaque instances and optional effects. Catalog membership stays in content;
//! adapters only project these values.

use crate::{HitPoints, ItemDefinitionId, ItemId, Position};

/// A positive amount restored by a healing item effect.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HealingAmount(u16);

impl HealingAmount {
  /// The smallest valid healing amount.
  pub const ONE: Self = Self(1);

  /// The authored three-point healing amount used by the starter fixture.
  pub const THREE: Self = Self(3);

  /// Creates a positive healing amount.
  #[must_use]
  pub const fn new(value: u16) -> Option<Self> {
    if value == 0 { None } else { Some(Self(value)) }
  }

  /// Returns the numeric healing amount.
  #[must_use]
  pub const fn value(self) -> u16 {
    self.0
  }
}

/// A positive number of ranged shots restored by an ammunition item effect.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AmmunitionAmount(u16);

impl AmmunitionAmount {
  /// The smallest valid ammunition amount.
  pub const ONE: Self = Self(1);

  /// The authored two-round ammunition amount used by the starter fixture.
  pub const TWO: Self = Self(2);

  /// Creates a positive ammunition amount.
  #[must_use]
  pub const fn new(value: u16) -> Option<Self> {
    if value == 0 { None } else { Some(Self(value)) }
  }

  /// Returns the numeric ammunition amount.
  #[must_use]
  pub const fn value(self) -> u16 {
    self.0
  }
}

/// The gameplay effect authored for one item instance.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ItemEffect {
  /// The item has no gameplay effect when consumed.
  None,
  /// Restore the supplied amount of hit points, capped at the actor maximum.
  Heal {
    /// The positive amount to restore.
    amount: HealingAmount,
  },
  /// Restore ranged ammunition, capped at the actor's fixed capacity.
  RestoreAmmunition {
    /// The positive number of shots to restore.
    amount: AmmunitionAmount,
  },
}

/// The closed set of mechanical effects available from equipped items.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EquipmentEffect {
  /// Raise the actor's effective melee reach to at least the supplied value.
  MinimumMeleeReach {
    /// The minimum effective reach while this item is equipped.
    reach: crate::MeleeReach,
  },
}

/// The closed set of effects available from explicitly thrown items.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ThrowableEffect {
  /// Apply a refreshed Chilled status to the living target.
  Chill,
}

/// The observable result of applying an item effect.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HealingResult {
  amount: u16,
  remaining_hit_points: HitPoints,
}

impl HealingResult {
  /// Creates typed healing evidence for an accepted item use.
  #[must_use]
  pub const fn new(amount: u16, remaining_hit_points: HitPoints) -> Self {
    Self {
      amount,
      remaining_hit_points,
    }
  }

  /// Returns the actual amount restored after capacity clamping.
  #[must_use]
  pub const fn amount(self) -> u16 {
    self.amount
  }

  /// Returns the actor's hit points after healing.
  #[must_use]
  pub const fn remaining_hit_points(self) -> HitPoints {
    self.remaining_hit_points
  }
}

/// The observable result of applying an ammunition item effect.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AmmunitionResult {
  amount: u16,
  remaining_ammunition: u16,
}

impl AmmunitionResult {
  /// Creates typed ammunition evidence for an accepted item use.
  #[must_use]
  pub const fn new(amount: u16, remaining_ammunition: u16) -> Self {
    Self {
      amount,
      remaining_ammunition,
    }
  }

  /// Returns the actual number of rounds restored after capacity clamping.
  #[must_use]
  pub const fn amount(self) -> u16 {
    self.amount
  }

  /// Returns the actor's ammunition after restoration.
  #[must_use]
  pub const fn remaining_ammunition(self) -> u16 {
    self.remaining_ammunition
  }
}

/// One opaque item instance in world state, either in an actor inventory or on the ground.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Item {
  id: ItemId,
  definition: ItemDefinitionId,
  effect: ItemEffect,
  equipment_effect: Option<EquipmentEffect>,
  throwable_effect: Option<ThrowableEffect>,
}

impl Item {
  /// Creates an item instance with an explicit identity and content reference.
  #[must_use]
  pub const fn new(id: ItemId, definition: ItemDefinitionId) -> Self {
    Self {
      id,
      definition,
      effect: ItemEffect::None,
      equipment_effect: None,
      throwable_effect: None,
    }
  }

  /// Creates an item instance with an explicit authored gameplay effect.
  #[must_use]
  pub const fn with_effect(id: ItemId, definition: ItemDefinitionId, effect: ItemEffect) -> Self {
    Self {
      id,
      definition,
      effect,
      equipment_effect: None,
      throwable_effect: None,
    }
  }

  /// Creates a non-consumable item instance with one closed equipment effect.
  #[must_use]
  pub const fn with_equipment_effect(
    id: ItemId,
    definition: ItemDefinitionId,
    reach: crate::MeleeReach,
  ) -> Self {
    Self {
      id,
      definition,
      effect: ItemEffect::None,
      equipment_effect: Some(EquipmentEffect::MinimumMeleeReach { reach }),
      throwable_effect: None,
    }
  }

  /// Creates an item instance with one closed throwable effect.
  #[must_use]
  pub const fn with_throwable_effect(
    id: ItemId,
    definition: ItemDefinitionId,
    effect: ThrowableEffect,
  ) -> Self {
    Self {
      id,
      definition,
      effect: ItemEffect::None,
      equipment_effect: None,
      throwable_effect: Some(effect),
    }
  }

  /// Returns the globally unique instance identity.
  #[must_use]
  pub const fn id(self) -> ItemId {
    self.id
  }

  /// Returns the opaque content reference.
  #[must_use]
  pub const fn definition(self) -> ItemDefinitionId {
    self.definition
  }

  /// Returns the authored gameplay effect for this item instance.
  #[must_use]
  pub const fn effect(self) -> ItemEffect {
    self.effect
  }

  /// Returns the optional closed equipment effect.
  #[must_use]
  pub const fn equipment_effect(self) -> Option<EquipmentEffect> {
    self.equipment_effect
  }

  /// Returns the optional closed throwable effect.
  #[must_use]
  pub const fn throwable_effect(self) -> Option<ThrowableEffect> {
    self.throwable_effect
  }
}

/// One deterministic stack of opaque items at a map position.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundItemStack {
  position: Position,
  pub(crate) items: Vec<Item>,
}

impl GroundItemStack {
  pub(crate) fn new(position: Position, item: Item) -> Self {
    Self {
      position,
      items: vec![item],
    }
  }

  /// Returns the map position of this stack.
  #[must_use]
  pub const fn position(&self) -> Position {
    self.position
  }

  /// Returns items in deterministic insertion order.
  #[must_use]
  pub fn items(&self) -> &[Item] {
    &self.items
  }
}
