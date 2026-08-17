//! Protocol projections of item instances, effects, and ground stacks.

use dreadstep_core::{
  AmmunitionResult as CoreAmmunitionResult, EquipmentEffect as CoreEquipmentEffect,
  EquipmentSlot as CoreEquipmentSlot, GroundItemStack as CoreGroundItemStack,
  HealingResult as CoreHealingResult, Item as CoreItem, ItemAffix as CoreItemAffix,
  ItemRarity as CoreItemRarity, ThrowableEffect as CoreThrowableEffect,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Damage, HitPoints, ItemDefinitionId, ItemId, Position};

/// A protocol projection of one opaque item instance.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, JsonSchema, Serialize)]
pub struct ItemSnapshot {
  id: ItemId,
  definition: ItemDefinitionId,
  rarity: ItemRarity,
  equipment_effect: Option<EquipmentEffect>,
  affix: Option<ItemAffix>,
  equipment_slot: Option<EquipmentSlot>,
  throwable_effect: Option<ThrowableEffect>,
}

/// A stable presentation rarity for one item instance.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemRarity {
  /// A baseline item with no rarity modifier.
  Common,
  /// An uncommon item intended to stand out in inventory presentation.
  Magic,
  /// A high-value item intended to stand out in inventory presentation.
  Rare,
}

/// A protocol projection of the role implied by an equipment effect.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EquipmentSlot {
  /// A weapon-like effect that changes attack reach or damage.
  Weapon,
  /// An armor-like effect that reduces incoming damage.
  Armor,
}

/// A protocol projection of one closed additive equipment affix.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemAffix {
  /// Add melee damage while equipped.
  MeleeDamage {
    /// The authored damage bonus.
    amount: Damage,
  },
  /// Add ranged damage while equipped.
  RangedDamage {
    /// The authored damage bonus.
    amount: Damage,
  },
  /// Reduce incoming damage while equipped.
  DamageReduction {
    /// The authored damage reduction.
    amount: Damage,
  },
}

/// A protocol projection of the closed equipment effects supported by core.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EquipmentEffect {
  /// Raise effective melee reach to at least the supplied value while equipped.
  MinimumMeleeReach {
    /// The minimum effective reach while equipped.
    reach: crate::MeleeReach,
  },
  /// Add the supplied damage to each melee attack while equipped.
  MeleeDamage {
    /// The damage bonus applied to melee attacks.
    amount: Damage,
  },
  /// Add the supplied damage to each ranged attack while equipped.
  RangedDamage {
    /// The damage bonus applied to ranged attacks.
    amount: Damage,
  },
  /// Reduce scheduled incoming damage by the supplied amount while equipped.
  DamageReduction {
    /// The damage reduction applied to melee, ranged, and floor-trap damage.
    amount: Damage,
  },
}

/// A protocol projection of the closed throwable effects supported by core.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ThrowableEffect {
  /// Apply a refreshed Chilled status to the living target.
  Chill,
}

/// Protocol evidence for hit points restored by a healing item.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, JsonSchema, Serialize)]
pub struct HealingResult {
  amount: u16,
  remaining_hit_points: HitPoints,
}

impl HealingResult {
  pub(crate) fn from_core(result: CoreHealingResult) -> Self {
    Self {
      amount: result.amount(),
      remaining_hit_points: HitPoints::new(result.remaining_hit_points().value()),
    }
  }

  /// Returns the actual amount restored after maximum-hit-point clamping.
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

/// Protocol evidence for ranged ammunition restored by an ammunition item.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, JsonSchema, Serialize)]
pub struct AmmunitionResult {
  amount: u16,
  remaining_ammunition: u16,
}

impl AmmunitionResult {
  pub(crate) fn from_core(result: CoreAmmunitionResult) -> Self {
    Self {
      amount: result.amount(),
      remaining_ammunition: result.remaining_ammunition(),
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

impl ItemSnapshot {
  /// Projects one complete core item into the versioned wire shape.
  #[must_use]
  pub fn from_item(item: CoreItem) -> Self {
    Self {
      id: ItemId::new(item.id().value()),
      definition: ItemDefinitionId::new(item.definition().value()),
      rarity: match item.rarity() {
        CoreItemRarity::Common => ItemRarity::Common,
        CoreItemRarity::Magic => ItemRarity::Magic,
        CoreItemRarity::Rare => ItemRarity::Rare,
      },
      equipment_effect: item.equipment_effect().map(|effect| match effect {
        CoreEquipmentEffect::MinimumMeleeReach { reach } => EquipmentEffect::MinimumMeleeReach {
          reach: crate::MeleeReach::new(reach.value()).unwrap_or(crate::MeleeReach::DEFAULT),
        },
        CoreEquipmentEffect::MeleeDamage { amount } => EquipmentEffect::MeleeDamage {
          amount: Damage::new(amount.value()),
        },
        CoreEquipmentEffect::RangedDamage { amount } => EquipmentEffect::RangedDamage {
          amount: Damage::new(amount.value()),
        },
        CoreEquipmentEffect::DamageReduction { amount } => EquipmentEffect::DamageReduction {
          amount: Damage::new(amount.value()),
        },
      }),
      affix: item.affix().map(|affix| match affix {
        CoreItemAffix::MeleeDamage { amount } => ItemAffix::MeleeDamage {
          amount: Damage::new(amount.value()),
        },
        CoreItemAffix::RangedDamage { amount } => ItemAffix::RangedDamage {
          amount: Damage::new(amount.value()),
        },
        CoreItemAffix::DamageReduction { amount } => ItemAffix::DamageReduction {
          amount: Damage::new(amount.value()),
        },
      }),
      equipment_slot: item.equipment_slot().map(|slot| match slot {
        CoreEquipmentSlot::Weapon => EquipmentSlot::Weapon,
        CoreEquipmentSlot::Armor => EquipmentSlot::Armor,
      }),
      throwable_effect: item.throwable_effect().map(|effect| match effect {
        CoreThrowableEffect::Chill => ThrowableEffect::Chill,
      }),
    }
  }

  /// Returns the stable item instance identity.
  #[must_use]
  pub const fn id(self) -> ItemId {
    self.id
  }

  /// Returns the opaque definition reference.
  #[must_use]
  pub const fn definition(self) -> ItemDefinitionId {
    self.definition
  }

  /// Returns the stable presentation rarity.
  #[must_use]
  pub const fn rarity(self) -> ItemRarity {
    self.rarity
  }

  /// Returns the optional closed equipment effect.
  #[must_use]
  pub const fn equipment_effect(self) -> Option<EquipmentEffect> {
    self.equipment_effect
  }

  /// Returns the optional closed additive affix.
  #[must_use]
  pub const fn affix(self) -> Option<ItemAffix> {
    self.affix
  }

  /// Returns the derived equipment role, when this item has a base equipment effect.
  #[must_use]
  pub const fn equipment_slot(self) -> Option<EquipmentSlot> {
    self.equipment_slot
  }

  /// Returns the optional closed effect when this item is thrown.
  #[must_use]
  pub const fn throwable_effect(self) -> Option<ThrowableEffect> {
    self.throwable_effect
  }
}

/// A read-only protocol projection of one ground-item stack.
#[derive(Clone, Debug, Eq, PartialEq, JsonSchema, Serialize)]
pub struct GroundItemSnapshot {
  position: Position,
  items: Vec<ItemSnapshot>,
}

impl GroundItemSnapshot {
  pub(crate) fn from_stack(stack: &CoreGroundItemStack) -> Self {
    Self {
      position: Position::new(stack.position().x(), stack.position().y()),
      items: stack
        .items()
        .iter()
        .copied()
        .map(ItemSnapshot::from_item)
        .collect(),
    }
  }

  /// Returns the map position of this stack.
  #[must_use]
  pub const fn position(&self) -> Position {
    self.position
  }

  /// Returns item snapshots in deterministic insertion order.
  #[must_use]
  pub fn items(&self) -> &[ItemSnapshot] {
    &self.items
  }
}
