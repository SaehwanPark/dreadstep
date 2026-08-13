//! Contract tests for the bounded equipment-derived melee-reach effect.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, CommandError, Event, GridMap, Item, ItemDefinitionId, ItemId,
  MeleeReach, Position, Tile, WorldState,
};

fn world_with_weapon() -> WorldState {
  let map = GridMap::from_tiles(
    5,
    3,
    vec![
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
    ],
  )
  .expect("reach map should validate");
  let mut world = WorldState::new(
    map,
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(1, 1)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(3, 1)),
    ],
  )
  .expect("reach world should validate");
  world
    .give_item(
      ActorId::new(1),
      Item::with_equipment_effect(
        ItemId::new(103),
        ItemDefinitionId::new(4),
        MeleeReach::new(2).expect("reach should be positive"),
      ),
    )
    .expect("weapon should be owned");
  world
}

#[test]
fn equipment_raises_effective_reach_and_distance_two_attack_is_typed() {
  let mut world = world_with_weapon();
  assert_eq!(
    world.actor(ActorId::new(1)).unwrap().melee_reach(),
    MeleeReach::DEFAULT
  );
  assert_eq!(
    world.execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }),
    Err(CommandError::AttackOutOfRange {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
    })
  );
  world
    .execute(Command::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(103),
    })
    .expect("weapon should equip");
  assert_eq!(
    world.actor(ActorId::new(1)).unwrap().melee_reach().value(),
    2
  );
  world
    .execute(Command::Wait {
      actor: ActorId::new(2),
    })
    .expect("enemy should yield");
  let result = world
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("equipped reach should allow distance two");
  assert!(matches!(result.events(), [Event::Attacked { .. }]));
}

#[test]
fn equipment_is_not_consumable_and_unequip_restores_base_reach() {
  let mut world = world_with_weapon();
  assert_eq!(
    world.execute(Command::UseItem {
      actor: ActorId::new(1),
      item: ItemId::new(103),
    }),
    Err(CommandError::ItemNotConsumable {
      actor: ActorId::new(1),
      item: ItemId::new(103),
    })
  );
  world
    .execute(Command::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(103),
    })
    .expect("weapon should equip");
  world
    .execute(Command::Wait {
      actor: ActorId::new(2),
    })
    .expect("enemy should yield before unequip");
  assert_eq!(
    world.execute(Command::UseItem {
      actor: ActorId::new(1),
      item: ItemId::new(103),
    }),
    Err(CommandError::ItemNotConsumable {
      actor: ActorId::new(1),
      item: ItemId::new(103),
    })
  );
  world
    .execute(Command::Unequip {
      actor: ActorId::new(1),
    })
    .expect("weapon should unequip after enemy yield");
  assert_eq!(
    world.actor(ActorId::new(1)).unwrap().melee_reach(),
    MeleeReach::DEFAULT
  );
}

#[test]
fn equipment_is_excluded_from_use_legal_actions_and_changes_digest() {
  let mut without_weapon = WorldState::new(
    GridMap::from_tiles(3, 1, vec![Tile::Floor, Tile::Floor, Tile::Floor]).unwrap(),
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .unwrap();
  let baseline = without_weapon.digest();
  without_weapon
    .give_item(
      ActorId::new(1),
      Item::with_equipment_effect(ItemId::new(103), ItemDefinitionId::new(4), MeleeReach::TWO),
    )
    .unwrap();
  let legal = without_weapon.legal_commands();
  assert!(legal.contains(&Command::Equip {
    actor: ActorId::new(1),
    item: ItemId::new(103),
  }));
  assert!(!legal.contains(&Command::UseItem {
    actor: ActorId::new(1),
    item: ItemId::new(103),
  }));
  assert_ne!(baseline, without_weapon.digest());
}
