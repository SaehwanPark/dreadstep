//! Stable item labels shared by the inventory frame and comparison line.

use dreadstep_core::{EquipmentEffect, Item, ItemEffect, ThrowableEffect};

use crate::CellColor;

pub(crate) fn item_kind_label_and_color(item: &Item) -> (String, CellColor) {
  if matches!(item.throwable_effect(), Some(ThrowableEffect::Chill)) {
    return (
      format!("{}flask", item.rarity().display_prefix()),
      CellColor::Cyan,
    );
  }
  let (label, color) = match item.effect() {
    ItemEffect::Heal { amount } => (format!("heal+{}", amount.value()), CellColor::Green),
    ItemEffect::RestoreAmmunition { amount } => {
      (format!("ammo+{}", amount.value()), CellColor::Yellow)
    }
    ItemEffect::None => match item.equipment_effect() {
      Some(EquipmentEffect::MinimumMeleeReach { reach }) => {
        (format!("reach{}", reach.value()), CellColor::Magenta)
      }
      Some(EquipmentEffect::MeleeDamage { amount }) => {
        (format!("damage+{}", amount.value()), CellColor::Red)
      }
      Some(EquipmentEffect::RangedDamage { amount }) => {
        (format!("ranged+{}", amount.value()), CellColor::Yellow)
      }
      Some(EquipmentEffect::DamageReduction { amount }) => {
        (format!("armor-{}", amount.value()), CellColor::Cyan)
      }
      None => ("item".to_string(), CellColor::Default),
    },
  };
  let affix = item.affix().map_or_else(String::new, |affix| {
    format!(" [{} +{}]", affix.wire_name(), affix.amount().value())
  });
  (
    format!("{}{}{}", item.rarity().display_prefix(), label, affix),
    color,
  )
}

#[cfg(test)]
mod tests {
  use super::item_kind_label_and_color;
  use dreadstep_core::{Damage, Item, ItemAffix, ItemDefinitionId, ItemId, ItemRarity};

  #[test]
  fn non_common_rarity_is_visible_in_item_label() {
    let item = Item::new(ItemId::new(1), ItemDefinitionId::new(2)).with_rarity(ItemRarity::Magic);
    assert_eq!(item_kind_label_and_color(&item).0, "magic item");
  }

  #[test]
  fn affix_is_visible_in_item_label() {
    let item =
      Item::with_equipment_damage(ItemId::new(1), ItemDefinitionId::new(2), Damage::new(1))
        .with_affix(ItemAffix::MeleeDamage {
          amount: Damage::new(2),
        });
    assert_eq!(
      item_kind_label_and_color(&item).0,
      "damage+1 [melee_damage +2]"
    );
  }
}
