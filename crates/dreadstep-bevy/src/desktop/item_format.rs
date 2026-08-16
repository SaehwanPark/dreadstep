//! JSON and HUD projections for item instances.

use dreadstep_core::Item;
use serde_json::{Value, json};

pub(crate) fn item_value(item: Item) -> Value {
  json!({
    "id": item.id().value(),
    "definition": item.definition().value(),
    "rarity": item.rarity().wire_name(),
    "equipment_slot": item.equipment_slot().map(equipment_slot_name),
    "equipment_effect": item.equipment_effect().map(|effect| match effect {
      dreadstep_core::EquipmentEffect::MinimumMeleeReach { reach } => {
        json!({ "minimum_melee_reach": reach.value() })
      }
      dreadstep_core::EquipmentEffect::MeleeDamage { amount } => {
        json!({ "melee_damage_bonus": amount.value() })
      }
      dreadstep_core::EquipmentEffect::RangedDamage { amount } => json!({
        "ranged_damage_bonus": amount.value()
      }),
      dreadstep_core::EquipmentEffect::DamageReduction { amount } => {
        json!({ "damage_reduction": amount.value() })
      }
    }),
    "throwable_effect": item.throwable_effect().map(|effect| match effect {
      dreadstep_core::ThrowableEffect::Chill => "chill",
    }),
  })
}

fn equipment_slot_name(slot: dreadstep_core::EquipmentSlot) -> &'static str {
  match slot {
    dreadstep_core::EquipmentSlot::Weapon => "weapon",
    dreadstep_core::EquipmentSlot::Armor => "armor",
  }
}

pub(crate) fn equipment_status_suffix(item: Item, equipped: bool) -> &'static str {
  if !equipped {
    return "";
  }
  match item.equipment_slot() {
    Some(dreadstep_core::EquipmentSlot::Weapon) => " [wielded]",
    Some(dreadstep_core::EquipmentSlot::Armor) => " [worn]",
    None => " [equipped]",
  }
}

#[cfg(test)]
mod tests {
  use super::item_value;
  use dreadstep_core::{ItemDefinitionId, ItemId, ItemRarity};

  #[test]
  fn item_json_projects_explicit_rarity() {
    let item = dreadstep_core::Item::new(ItemId::new(1), ItemDefinitionId::new(2))
      .with_rarity(ItemRarity::Rare);
    assert_eq!(item_value(item)["rarity"], "rare");
  }
}
