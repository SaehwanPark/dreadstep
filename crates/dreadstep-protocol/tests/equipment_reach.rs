//! Protocol projection tests for the bounded equipment-derived reach effect.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Damage, GridMap, Item, ItemAffix, ItemDefinitionId, MeleeReach,
  Position, Tile, WorldState,
};
use dreadstep_protocol::{ItemId, ItemRarity, ItemSnapshot, PROTOCOL_VERSION, WorldSnapshot};
use serde_json::json;

#[test]
fn item_snapshot_projects_equipment_effect_and_protocol_version_bumps() {
  let item = Item::with_equipment_effect(
    dreadstep_core::ItemId::new(103),
    ItemDefinitionId::new(4),
    MeleeReach::new(2).expect("reach should be positive"),
  );
  let snapshot = ItemSnapshot::from_item(item);
  assert_eq!(snapshot.id().value(), 103);
  assert!(matches!(
    snapshot.equipment_effect(),
    Some(dreadstep_protocol::EquipmentEffect::MinimumMeleeReach { reach })
      if reach.value() == 2
  ));
  assert_eq!(
    snapshot.equipment_slot(),
    Some(dreadstep_protocol::EquipmentSlot::Weapon)
  );
  assert_eq!(PROTOCOL_VERSION, 36);
}

#[test]
fn item_snapshot_projects_equipment_affix() {
  let item = Item::with_equipment_damage(
    dreadstep_core::ItemId::new(105),
    ItemDefinitionId::new(6),
    Damage::new(1),
  )
  .with_affix(ItemAffix::MeleeDamage {
    amount: Damage::new(2),
  });
  let snapshot = ItemSnapshot::from_item(item);
  assert!(matches!(
    snapshot.affix(),
    Some(dreadstep_protocol::ItemAffix::MeleeDamage { amount }) if amount.value() == 2
  ));
}

#[test]
fn item_snapshot_projects_melee_damage_effect() {
  let item = Item::with_equipment_damage(
    dreadstep_core::ItemId::new(105),
    ItemDefinitionId::new(6),
    Damage::new(1),
  );
  let snapshot = ItemSnapshot::from_item(item);
  assert!(matches!(
    snapshot.equipment_effect(),
    Some(dreadstep_protocol::EquipmentEffect::MeleeDamage { amount })
      if amount.value() == 1
  ));
}

#[test]
fn item_snapshot_projects_explicit_rarity() {
  let item = Item::with_equipment_damage(
    dreadstep_core::ItemId::new(105),
    ItemDefinitionId::new(6),
    Damage::new(1),
  )
  .with_rarity(dreadstep_core::ItemRarity::Rare);
  let snapshot = ItemSnapshot::from_item(item);
  assert_eq!(snapshot.rarity(), ItemRarity::Rare);
}

#[test]
fn item_snapshot_projects_ranged_damage_effect() {
  let item = Item::with_ranged_damage(
    dreadstep_core::ItemId::new(106),
    ItemDefinitionId::new(7),
    Damage::new(1),
  );
  let snapshot = ItemSnapshot::from_item(item);
  assert!(matches!(
    snapshot.equipment_effect(),
    Some(dreadstep_protocol::EquipmentEffect::RangedDamage { amount })
      if amount.value() == 1
  ));
}

#[test]
fn item_snapshot_projects_weapon_and_armor_roles() {
  let weapon = ItemSnapshot::from_item(Item::with_equipment_damage(
    dreadstep_core::ItemId::new(105),
    ItemDefinitionId::new(6),
    Damage::new(1),
  ));
  let armor = ItemSnapshot::from_item(Item::with_damage_reduction(
    dreadstep_core::ItemId::new(106),
    ItemDefinitionId::new(7),
    Damage::new(1),
  ));
  assert_eq!(
    weapon.equipment_slot(),
    Some(dreadstep_protocol::EquipmentSlot::Weapon)
  );
  assert_eq!(
    armor.equipment_slot(),
    Some(dreadstep_protocol::EquipmentSlot::Armor)
  );
}

#[test]
fn actor_snapshot_projects_independent_weapon_and_armor_slots() {
  let map = GridMap::from_tiles(1, 1, vec![Tile::Floor]).unwrap();
  let mut world = WorldState::new(
    map,
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .unwrap();
  world
    .give_item(
      ActorId::new(1),
      Item::with_equipment_damage(
        dreadstep_core::ItemId::new(105),
        ItemDefinitionId::new(6),
        Damage::new(1),
      ),
    )
    .unwrap();
  world
    .give_item(
      ActorId::new(1),
      Item::with_damage_reduction(
        dreadstep_core::ItemId::new(106),
        ItemDefinitionId::new(7),
        Damage::new(1),
      ),
    )
    .unwrap();
  world
    .execute(dreadstep_core::Command::Equip {
      actor: ActorId::new(1),
      item: dreadstep_core::ItemId::new(105),
    })
    .unwrap();
  world
    .execute(dreadstep_core::Command::Equip {
      actor: ActorId::new(1),
      item: dreadstep_core::ItemId::new(106),
    })
    .unwrap();
  let actor = WorldSnapshot::from_world(&world).actors()[0].clone();
  assert_eq!(actor.equipped_weapon().map(ItemId::value), Some(105));
  assert_eq!(actor.equipped_armor().map(ItemId::value), Some(106));
}

#[test]
fn item_snapshot_projects_damage_reduction_effect() {
  let item = Item::with_damage_reduction(
    dreadstep_core::ItemId::new(105),
    ItemDefinitionId::new(6),
    Damage::new(1),
  );
  let snapshot = ItemSnapshot::from_item(item);
  assert!(matches!(
    snapshot.equipment_effect(),
    Some(dreadstep_protocol::EquipmentEffect::DamageReduction { amount })
      if amount.value() == 1
  ));
}

#[test]
fn actor_snapshot_projects_effective_reach_after_equipment() {
  let map = GridMap::from_tiles(3, 1, vec![Tile::Floor, Tile::Floor, Tile::Floor]).unwrap();
  let mut world = WorldState::new(
    map,
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .unwrap();
  world
    .give_item(
      ActorId::new(1),
      Item::with_equipment_effect(
        dreadstep_core::ItemId::new(103),
        ItemDefinitionId::new(4),
        MeleeReach::new(2).unwrap(),
      ),
    )
    .unwrap();
  world
    .execute(dreadstep_core::Command::Equip {
      actor: ActorId::new(1),
      item: dreadstep_core::ItemId::new(103),
    })
    .unwrap();
  let snapshot = WorldSnapshot::from_world(&world);
  assert_eq!(snapshot.actors()[0].melee_reach().value(), 2);
}

#[test]
fn non_consumable_rejection_keeps_a_stable_protocol_error_shape() {
  let error =
    dreadstep_protocol::CommandError::from(dreadstep_core::CommandError::ItemNotConsumable {
      actor: ActorId::new(1),
      item: dreadstep_core::ItemId::new(103),
    });
  assert_eq!(
    serde_json::to_value(error).expect("error should serialize"),
    json!({"item_not_consumable": {"actor": 1, "item": 103}})
  );
}
