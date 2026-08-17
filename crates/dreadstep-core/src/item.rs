//! Item instances, authored effects, and ground stacks.
//!
//! Core stores opaque instances and optional effects. Catalog membership stays in content;
//! adapters only project these values.

use crate::{Damage, HitPoints, ItemDefinitionId, ItemId, Position};

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

/// The closed set of equipment roles used to explain an authored effect.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EquipmentSlot {
  /// A weapon-like effect that changes attack reach or damage.
  Weapon,
  /// An armor-like effect that reduces incoming damage.
  Armor,
}

/// The presentation rarity authored for an item instance.
///
/// Rarity is intentionally metadata in the current slice: it does not alter equipment effects,
/// consumable outcomes, inventory legality, or action timing.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ItemRarity {
  /// A baseline item with no rarity modifier.
  Common,
  /// An uncommon item intended to stand out in inventory presentation.
  Magic,
  /// A high-value item intended to stand out in inventory presentation.
  Rare,
}

impl ItemRarity {
  /// Returns the stable snake-case wire value used by adapters.
  #[must_use]
  pub const fn wire_name(self) -> &'static str {
    match self {
      Self::Common => "common",
      Self::Magic => "magic",
      Self::Rare => "rare",
    }
  }

  /// Returns the concise terminal prefix used for non-common items.
  #[must_use]
  pub const fn display_prefix(self) -> &'static str {
    match self {
      Self::Common => "",
      Self::Magic => "magic ",
      Self::Rare => "rare ",
    }
  }
}

/// The closed set of mechanical effects available from equipped items.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EquipmentEffect {
  /// Raise the actor's effective melee reach to at least the supplied value.
  MinimumMeleeReach {
    /// The minimum effective reach while this item is equipped.
    reach: crate::MeleeReach,
  },
  /// Add the supplied damage to each melee attack while equipped.
  MeleeDamage {
    /// The damage bonus applied to melee attacks.
    amount: crate::Damage,
  },
  /// Add the supplied damage to each ranged attack while equipped.
  RangedDamage {
    /// The damage bonus applied to ranged attacks.
    amount: crate::Damage,
  },
  /// Reduce scheduled incoming damage by the supplied amount while equipped.
  DamageReduction {
    /// The damage reduction applied to melee, ranged, and floor-trap damage.
    amount: crate::Damage,
  },
}

/// One closed, additive modifier authored on an equipment item.
///
/// Affixes are intentionally limited to existing combat statistics in this slice. They do not
/// introduce new timing, targeting, or inventory rules; an equipped affix simply adds to the
/// corresponding base equipment effect.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ItemAffix {
  /// Add melee damage while the item is equipped.
  MeleeDamage {
    /// The positive authored bonus.
    amount: crate::Damage,
  },
  /// Add ranged damage while the item is equipped.
  RangedDamage {
    /// The positive authored bonus.
    amount: crate::Damage,
  },
  /// Reduce incoming damage while the item is equipped.
  DamageReduction {
    /// The positive authored reduction.
    amount: crate::Damage,
  },
}

impl ItemAffix {
  /// Returns the stable wire name used by presentation adapters.
  #[must_use]
  pub const fn wire_name(self) -> &'static str {
    match self {
      Self::MeleeDamage { .. } => "melee_damage",
      Self::RangedDamage { .. } => "ranged_damage",
      Self::DamageReduction { .. } => "damage_reduction",
    }
  }

  /// Returns the authored numeric bonus.
  #[must_use]
  pub const fn amount(self) -> crate::Damage {
    match self {
      Self::MeleeDamage { amount }
      | Self::RangedDamage { amount }
      | Self::DamageReduction { amount } => amount,
    }
  }
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
  rarity: ItemRarity,
  effect: ItemEffect,
  equipment_effect: Option<EquipmentEffect>,
  affix: Option<ItemAffix>,
  throwable_effect: Option<ThrowableEffect>,
}

impl Item {
  /// Creates an item instance with an explicit identity and content reference.
  #[must_use]
  pub const fn new(id: ItemId, definition: ItemDefinitionId) -> Self {
    Self {
      id,
      definition,
      rarity: ItemRarity::Common,
      effect: ItemEffect::None,
      equipment_effect: None,
      affix: None,
      throwable_effect: None,
    }
  }

  /// Creates an item instance with an explicit authored gameplay effect.
  #[must_use]
  pub const fn with_effect(id: ItemId, definition: ItemDefinitionId, effect: ItemEffect) -> Self {
    Self {
      id,
      definition,
      rarity: ItemRarity::Common,
      effect,
      equipment_effect: None,
      affix: None,
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
      rarity: ItemRarity::Common,
      effect: ItemEffect::None,
      equipment_effect: Some(EquipmentEffect::MinimumMeleeReach { reach }),
      affix: None,
      throwable_effect: None,
    }
  }

  /// Creates a non-consumable item instance with a melee-damage equipment effect.
  #[must_use]
  pub const fn with_equipment_damage(
    id: ItemId,
    definition: ItemDefinitionId,
    amount: crate::Damage,
  ) -> Self {
    Self {
      id,
      definition,
      rarity: ItemRarity::Common,
      effect: ItemEffect::None,
      equipment_effect: Some(EquipmentEffect::MeleeDamage { amount }),
      affix: None,
      throwable_effect: None,
    }
  }

  /// Creates a non-consumable item instance with a ranged-damage equipment effect.
  #[must_use]
  pub const fn with_ranged_damage(
    id: ItemId,
    definition: ItemDefinitionId,
    amount: crate::Damage,
  ) -> Self {
    Self {
      id,
      definition,
      rarity: ItemRarity::Common,
      effect: ItemEffect::None,
      equipment_effect: Some(EquipmentEffect::RangedDamage { amount }),
      affix: None,
      throwable_effect: None,
    }
  }

  /// Creates a non-consumable item instance with a closed attack-damage reduction effect.
  #[must_use]
  pub const fn with_damage_reduction(
    id: ItemId,
    definition: ItemDefinitionId,
    amount: crate::Damage,
  ) -> Self {
    Self {
      id,
      definition,
      rarity: ItemRarity::Common,
      effect: ItemEffect::None,
      equipment_effect: Some(EquipmentEffect::DamageReduction { amount }),
      affix: None,
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
      rarity: ItemRarity::Common,
      effect: ItemEffect::None,
      equipment_effect: None,
      affix: None,
      throwable_effect: Some(effect),
    }
  }

  /// Returns this item with an explicit presentation rarity.
  #[must_use]
  pub const fn with_rarity(mut self, rarity: ItemRarity) -> Self {
    self.rarity = rarity;
    self
  }

  /// Returns this item with one explicit closed equipment affix.
  #[must_use]
  pub const fn with_affix(mut self, affix: ItemAffix) -> Self {
    self.affix = Some(affix);
    self
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

  /// Returns the authored presentation rarity for this item instance.
  #[must_use]
  pub const fn rarity(self) -> ItemRarity {
    self.rarity
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

  /// Returns the optional closed equipment affix.
  #[must_use]
  pub const fn affix(self) -> Option<ItemAffix> {
    self.affix
  }

  /// Returns the derived equipment role for this item's closed effect.
  #[must_use]
  pub const fn equipment_slot(self) -> Option<EquipmentSlot> {
    match self.equipment_effect {
      Some(
        EquipmentEffect::MinimumMeleeReach { .. }
        | EquipmentEffect::MeleeDamage { .. }
        | EquipmentEffect::RangedDamage { .. },
      ) => Some(EquipmentSlot::Weapon),
      Some(EquipmentEffect::DamageReduction { .. }) => Some(EquipmentSlot::Armor),
      None => None,
    }
  }

  /// Returns this item's melee-damage contribution, including its base effect and affix.
  #[must_use]
  pub const fn melee_damage_bonus(self) -> Damage {
    let base = match self.equipment_effect {
      Some(EquipmentEffect::MeleeDamage { amount }) => amount.value(),
      _ => 0,
    };
    let affix = match self.affix {
      Some(ItemAffix::MeleeDamage { amount }) => amount.value(),
      _ => 0,
    };
    Damage::new(base.saturating_add(affix))
  }

  /// Returns this item's ranged-damage contribution, including its base effect and affix.
  #[must_use]
  pub const fn ranged_damage_bonus(self) -> Damage {
    let base = match self.equipment_effect {
      Some(EquipmentEffect::RangedDamage { amount }) => amount.value(),
      _ => 0,
    };
    let affix = match self.affix {
      Some(ItemAffix::RangedDamage { amount }) => amount.value(),
      _ => 0,
    };
    Damage::new(base.saturating_add(affix))
  }

  /// Returns this item's incoming-damage reduction, including its base effect and affix.
  #[must_use]
  pub const fn damage_reduction(self) -> Damage {
    let base = match self.equipment_effect {
      Some(EquipmentEffect::DamageReduction { amount }) => amount.value(),
      _ => 0,
    };
    let affix = match self.affix {
      Some(ItemAffix::DamageReduction { amount }) => amount.value(),
      _ => 0,
    };
    Damage::new(base.saturating_add(affix))
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
