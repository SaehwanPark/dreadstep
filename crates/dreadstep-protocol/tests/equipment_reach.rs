//! Protocol projection tests for the bounded equipment-derived reach effect.

use dreadstep_core::{
  Actor, ActorId, ActorKind, GridMap, Item, ItemDefinitionId, MeleeReach, Position, Tile,
  WorldState,
};
use dreadstep_protocol::{ItemSnapshot, PROTOCOL_VERSION, WorldSnapshot};
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
  assert_eq!(PROTOCOL_VERSION, 29);
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
