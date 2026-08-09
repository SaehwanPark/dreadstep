//! Headless ECS scene synchronization behavior.

use std::collections::BTreeMap;

use bevy::ecs::world::World;
use dreadstep_bevy::{
  PresentationState, SceneActor, SceneGroundItem, SceneInventoryItem, SceneTile, sync_scene,
};
use dreadstep_content::starter_floor;
use dreadstep_core::{
  ActionTime, Actor, ActorId, ActorKind, Command, GridMap, HitPoints, Item, ItemDefinitionId,
  ItemId, Position, Tile, WorldState,
};

fn combat_state() -> PresentationState {
  let map = GridMap::from_tiles(2, 1, vec![Tile::Floor, Tile::Floor]).expect("map should validate");
  let world = WorldState::new(
    map,
    vec![
      Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(3),
      ),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(1, 0),
        HitPoints::new(1),
      ),
    ],
  )
  .expect("world should validate");
  PresentationState::new(7, world)
}

fn ground_world() -> WorldState {
  let map = GridMap::from_tiles(
    3,
    2,
    vec![
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
    ],
  )
  .expect("map should validate");
  let mut world = WorldState::new(
    map,
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 1)),
    ],
  )
  .expect("world should validate");
  world
    .give_item(
      ActorId::new(1),
      Item::new(ItemId::new(1), ItemDefinitionId::new(101)),
    )
    .expect("first item should be given");
  world
    .give_item(
      ActorId::new(1),
      Item::new(ItemId::new(2), ItemDefinitionId::new(102)),
    )
    .expect("second item should be given");
  world
    .drop_item(ActorId::new(1), ItemId::new(1))
    .expect("first item should be dropped");
  world
    .drop_item(ActorId::new(1), ItemId::new(2))
    .expect("second item should be dropped");
  world
    .give_item(
      ActorId::new(2),
      Item::new(ItemId::new(3), ItemDefinitionId::new(103)),
    )
    .expect("third item should be given");
  world
    .drop_item(ActorId::new(2), ItemId::new(3))
    .expect("third item should be dropped");
  world
}

fn inventory_world() -> WorldState {
  let mut world = ground_world();
  world
    .give_item(
      ActorId::new(1),
      Item::new(ItemId::new(4), ItemDefinitionId::new(104)),
    )
    .expect("fourth item should be given");
  world
    .give_item(
      ActorId::new(1),
      Item::new(ItemId::new(5), ItemDefinitionId::new(105)),
    )
    .expect("fifth item should be given");
  world
    .give_item(
      ActorId::new(2),
      Item::new(ItemId::new(6), ItemDefinitionId::new(106)),
    )
    .expect("sixth item should be given");
  world
}

fn keyed_tiles(scene: &mut World) -> BTreeMap<(i32, i32), (bevy::ecs::entity::Entity, SceneTile)> {
  scene
    .query::<(bevy::ecs::entity::Entity, &SceneTile)>()
    .iter(scene)
    .map(|(entity, tile)| ((tile.position().x(), tile.position().y()), (entity, *tile)))
    .collect()
}

fn keyed_actors(scene: &mut World) -> BTreeMap<ActorId, (bevy::ecs::entity::Entity, SceneActor)> {
  scene
    .query::<(bevy::ecs::entity::Entity, &SceneActor)>()
    .iter(scene)
    .map(|(entity, actor)| (actor.id(), (entity, *actor)))
    .collect()
}

fn keyed_ground_items(
  scene: &mut World,
) -> BTreeMap<ItemId, (bevy::ecs::entity::Entity, Position, ItemDefinitionId, usize)> {
  scene
    .query::<(bevy::ecs::entity::Entity, &SceneGroundItem)>()
    .iter(scene)
    .map(|(entity, item)| {
      (
        item.id(),
        (
          entity,
          item.position(),
          item.definition(),
          item.stack_index(),
        ),
      )
    })
    .collect()
}

fn keyed_inventory_items(
  scene: &mut World,
) -> BTreeMap<ItemId, (bevy::ecs::entity::Entity, ActorId, ItemDefinitionId, usize)> {
  scene
    .query::<(bevy::ecs::entity::Entity, &SceneInventoryItem)>()
    .iter(scene)
    .map(|(entity, item)| {
      (
        item.id(),
        (
          entity,
          item.owner(),
          item.definition(),
          item.inventory_index(),
        ),
      )
    })
    .collect()
}

#[test]
fn sync_creates_scene_entities_and_preserves_keys_across_updates() {
  let mut state = PresentationState::start_run(7).expect("content should validate");
  let initial = state.snapshot();
  let mut scene = World::new();
  sync_scene(&mut scene, &initial);

  assert_eq!(scene.query::<&SceneTile>().iter(&scene).count(), 35);
  assert_eq!(scene.query::<&SceneActor>().iter(&scene).count(), 4);
  assert!(initial.ground_items().is_empty());
  assert_eq!(scene.query::<&SceneGroundItem>().iter(&scene).count(), 0);
  assert_eq!(scene.query::<&SceneInventoryItem>().iter(&scene).count(), 0);
  let player_entity = scene
    .query::<(bevy::ecs::entity::Entity, &SceneActor)>()
    .iter(&scene)
    .find(|(_, actor)| actor.id() == ActorId::new(1))
    .map(|(entity, _)| entity)
    .expect("player scene entity should exist");
  let tile_entity = scene
    .query::<(bevy::ecs::entity::Entity, &SceneTile)>()
    .iter(&scene)
    .find(|(_, tile)| tile.position() == Position::new(0, 0))
    .map(|(entity, _)| entity)
    .expect("origin tile scene entity should exist");

  sync_scene(&mut scene, &initial);
  let repeated_entity = scene
    .query::<(bevy::ecs::entity::Entity, &SceneActor)>()
    .iter(&scene)
    .find(|(_, actor)| actor.id() == ActorId::new(1))
    .map(|(entity, _)| entity)
    .expect("player scene entity should remain");
  let repeated_tile = scene
    .query::<(bevy::ecs::entity::Entity, &SceneTile)>()
    .iter(&scene)
    .find(|(_, tile)| tile.position() == Position::new(0, 0))
    .map(|(entity, _)| entity)
    .expect("origin tile scene entity should remain");
  assert_eq!(repeated_entity, player_entity);
  assert_eq!(repeated_tile, tile_entity);

  let output = state
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: dreadstep_core::Direction::East,
    })
    .expect("player should move");
  sync_scene(&mut scene, output.snapshot());
  let player = scene
    .query::<(bevy::ecs::entity::Entity, &SceneActor)>()
    .iter(&scene)
    .find(|(_, actor)| actor.id() == ActorId::new(1))
    .expect("updated player scene entity should exist");
  assert_eq!(player.0, player_entity);
  assert_eq!(player.1.position(), Position::new(2, 1));
  assert_eq!(player.1.ready_at(), ActionTime::new(1));
}

#[test]
fn sync_projects_complete_ground_items_and_preserves_item_identity() {
  let snapshot = PresentationState::new(7, ground_world()).snapshot();
  let expected_stacks = vec![
    (
      Position::new(0, 0),
      vec![
        (ItemId::new(1), ItemDefinitionId::new(101)),
        (ItemId::new(2), ItemDefinitionId::new(102)),
      ],
    ),
    (
      Position::new(2, 1),
      vec![(ItemId::new(3), ItemDefinitionId::new(103))],
    ),
  ];
  let actual_stacks: Vec<_> = snapshot
    .ground_items()
    .iter()
    .map(|stack| {
      (
        stack.position(),
        stack
          .items()
          .iter()
          .map(|item| (item.id(), item.definition()))
          .collect::<Vec<_>>(),
      )
    })
    .collect();
  assert_eq!(actual_stacks, expected_stacks);

  let mut scene = World::new();
  sync_scene(&mut scene, &snapshot);

  let first_items = keyed_ground_items(&mut scene);
  let expected_items = BTreeMap::from([
    (
      ItemId::new(1),
      (Position::new(0, 0), ItemDefinitionId::new(101), 0),
    ),
    (
      ItemId::new(2),
      (Position::new(0, 0), ItemDefinitionId::new(102), 1),
    ),
    (
      ItemId::new(3),
      (Position::new(2, 1), ItemDefinitionId::new(103), 0),
    ),
  ]);
  let actual_items: BTreeMap<_, _> = first_items
    .iter()
    .map(|(item_id, (_, position, definition, stack_index))| {
      (*item_id, (*position, *definition, *stack_index))
    })
    .collect();
  assert_eq!(actual_items, expected_items);

  let item_two_entity = first_items[&ItemId::new(2)].0;
  let mut updated_world = ground_world();
  updated_world
    .pickup_item(ActorId::new(1), ItemId::new(2))
    .expect("second item should be picked up");
  updated_world
    .teleport(ActorId::new(1), Position::new(1, 1))
    .expect("actor should teleport on a floor tile");
  updated_world
    .drop_item(ActorId::new(1), ItemId::new(2))
    .expect("second item should be dropped at the new position");
  let updated = PresentationState::new(7, updated_world).snapshot();
  sync_scene(&mut scene, &updated);

  let updated_items = keyed_ground_items(&mut scene);
  assert_eq!(updated_items[&ItemId::new(2)].0, item_two_entity);
  assert_eq!(
    (
      updated_items[&ItemId::new(2)].1,
      updated_items[&ItemId::new(2)].2,
      updated_items[&ItemId::new(2)].3,
    ),
    (Position::new(1, 1), ItemDefinitionId::new(102), 0)
  );
}

#[test]
fn sync_projects_complete_inventory_items_and_updates_owner_and_order() {
  let full = PresentationState::new(7, inventory_world()).snapshot();
  let expected_inventories = vec![
    (
      ActorId::new(1),
      vec![
        (ItemId::new(4), ItemDefinitionId::new(104)),
        (ItemId::new(5), ItemDefinitionId::new(105)),
      ],
    ),
    (
      ActorId::new(2),
      vec![(ItemId::new(6), ItemDefinitionId::new(106))],
    ),
  ];
  let actual_inventories: Vec<_> = full
    .actors()
    .iter()
    .filter_map(|actor| {
      let items: Vec<_> = actor
        .inventory()
        .iter()
        .map(|item| (item.id(), item.definition()))
        .collect();
      (!items.is_empty()).then_some((actor.id(), items))
    })
    .collect();
  assert_eq!(actual_inventories, expected_inventories);

  let mut scene = World::new();
  sync_scene(&mut scene, &full);
  let before_tiles = keyed_tiles(&mut scene);
  let before_actors = keyed_actors(&mut scene);
  let before_ground = keyed_ground_items(&mut scene);
  let before_inventory = keyed_inventory_items(&mut scene);
  let expected_projection = BTreeMap::from([
    (
      ItemId::new(4),
      (ActorId::new(1), ItemDefinitionId::new(104), 0),
    ),
    (
      ItemId::new(5),
      (ActorId::new(1), ItemDefinitionId::new(105), 1),
    ),
    (
      ItemId::new(6),
      (ActorId::new(2), ItemDefinitionId::new(106), 0),
    ),
  ]);
  let actual_projection: BTreeMap<_, _> = before_inventory
    .iter()
    .map(|(item_id, (_, owner, definition, index))| (*item_id, (*owner, *definition, *index)))
    .collect();
  assert_eq!(actual_projection, expected_projection);

  let item_five_entity = before_inventory[&ItemId::new(5)].0;
  let item_four_entity = before_inventory[&ItemId::new(4)].0;
  let mut updated_world = inventory_world();
  updated_world
    .transfer_item(ActorId::new(1), ActorId::new(2), ItemId::new(4))
    .expect("transfer should update owner and order");
  let updated = PresentationState::new(7, updated_world).snapshot();
  sync_scene(&mut scene, &updated);

  let updated_inventory = keyed_inventory_items(&mut scene);
  assert_eq!(updated_inventory[&ItemId::new(5)].0, item_five_entity);
  assert_eq!(
    (
      updated_inventory[&ItemId::new(5)].1,
      updated_inventory[&ItemId::new(5)].2,
      updated_inventory[&ItemId::new(5)].3,
    ),
    (ActorId::new(1), ItemDefinitionId::new(105), 0)
  );
  assert_eq!(updated_inventory[&ItemId::new(4)].0, item_four_entity);
  assert_eq!(
    (
      updated_inventory[&ItemId::new(4)].1,
      updated_inventory[&ItemId::new(4)].2,
      updated_inventory[&ItemId::new(4)].3,
    ),
    (ActorId::new(2), ItemDefinitionId::new(104), 1)
  );
  assert_eq!(keyed_tiles(&mut scene), before_tiles);
  assert_eq!(keyed_actors(&mut scene), before_actors);
  assert_eq!(keyed_ground_items(&mut scene), before_ground);

  let reduced = PresentationState::new(7, ground_world()).snapshot();
  sync_scene(&mut scene, &reduced);
  assert_eq!(scene.query::<&SceneInventoryItem>().iter(&scene).count(), 0);
  assert!(scene.get_entity(item_four_entity).is_err());
  assert!(scene.get_entity(item_five_entity).is_err());
  assert_eq!(keyed_tiles(&mut scene), before_tiles);
  assert_eq!(keyed_actors(&mut scene), before_actors);
  assert_eq!(keyed_ground_items(&mut scene), before_ground);
}

#[test]
fn sync_deduplicates_inventory_items_by_deterministic_lowest_entity() {
  let snapshot = PresentationState::new(7, inventory_world()).snapshot();
  let mut scene = World::new();
  sync_scene(&mut scene, &snapshot);
  let (original_entity, duplicate_item) = scene
    .query::<(bevy::ecs::entity::Entity, &SceneInventoryItem)>()
    .iter(&scene)
    .find(|(_, item)| item.id() == ItemId::new(4))
    .map(|(entity, item)| (entity, *item))
    .expect("inventory item should exist");
  let duplicate_entity = scene.spawn(duplicate_item).id();

  assert_eq!(scene.query::<&SceneInventoryItem>().iter(&scene).count(), 4);
  sync_scene(&mut scene, &snapshot);

  let inventory_items = keyed_inventory_items(&mut scene);
  let expected_survivor = std::cmp::min(original_entity, duplicate_entity);
  let expected_stale = std::cmp::max(original_entity, duplicate_entity);
  assert_eq!(inventory_items[&ItemId::new(4)].0, expected_survivor);
  assert!(scene.get_entity(expected_stale).is_err());
  assert_eq!(
    (
      inventory_items[&ItemId::new(4)].1,
      inventory_items[&ItemId::new(4)].2,
      inventory_items[&ItemId::new(4)].3,
    ),
    (ActorId::new(1), ItemDefinitionId::new(104), 0)
  );
  assert_eq!(inventory_items.len(), 3);
}

#[test]
fn sync_deduplicates_public_mirror_entities_by_stable_key() {
  let snapshot = PresentationState::start_run(7)
    .expect("content should validate")
    .snapshot();
  let mut scene = World::new();
  sync_scene(&mut scene, &snapshot);
  let tile = *scene
    .query::<&SceneTile>()
    .iter(&scene)
    .next()
    .expect("tile should exist");
  let actor = *scene
    .query::<&SceneActor>()
    .iter(&scene)
    .next()
    .expect("actor should exist");
  let ground_snapshot = PresentationState::new(7, ground_world()).snapshot();
  sync_scene(&mut scene, &ground_snapshot);
  let (original_item_entity, ground_item) = scene
    .query::<(bevy::ecs::entity::Entity, &SceneGroundItem)>()
    .iter(&scene)
    .find(|(_, item)| item.id() == ItemId::new(1))
    .map(|(entity, item)| (entity, *item))
    .expect("first ground item should exist");
  scene.spawn(tile);
  scene.spawn(actor);
  let duplicate_item_entity = scene.spawn(ground_item).id();
  assert_eq!(scene.query::<&SceneTile>().iter(&scene).count(), 7);
  assert_eq!(scene.query::<&SceneActor>().iter(&scene).count(), 3);
  assert_eq!(scene.query::<&SceneGroundItem>().iter(&scene).count(), 4);

  sync_scene(&mut scene, &ground_snapshot);

  assert_eq!(scene.query::<&SceneTile>().iter(&scene).count(), 6);
  assert_eq!(scene.query::<&SceneActor>().iter(&scene).count(), 2);
  assert_eq!(scene.query::<&SceneGroundItem>().iter(&scene).count(), 3);
  let ground_items = keyed_ground_items(&mut scene);
  assert_ne!(original_item_entity, duplicate_item_entity);
  let expected_survivor = std::cmp::min(original_item_entity, duplicate_item_entity);
  let expected_stale = std::cmp::max(original_item_entity, duplicate_item_entity);
  assert_eq!(ground_items[&ItemId::new(1)].0, expected_survivor);
  assert!(scene.get_entity(expected_stale).is_err());
  assert_eq!(
    (
      ground_items[&ItemId::new(1)].1,
      ground_items[&ItemId::new(1)].2,
      ground_items[&ItemId::new(1)].3,
    ),
    (Position::new(0, 0), ItemDefinitionId::new(101), 0)
  );
}

#[test]
fn sync_removes_entities_absent_from_a_later_snapshot() {
  let full = combat_state().snapshot();
  let mut scene = World::new();
  sync_scene(&mut scene, &full);
  assert_eq!(scene.query::<&SceneTile>().iter(&scene).count(), 2);
  assert_eq!(scene.query::<&SceneActor>().iter(&scene).count(), 2);
  assert_eq!(scene.query::<&SceneGroundItem>().iter(&scene).count(), 0);

  let map = GridMap::filled(1, 1, Tile::Floor).expect("map should validate");
  let reduced_world = WorldState::new(
    map,
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .expect("reduced world should validate");
  let reduced = PresentationState::new(7, reduced_world).snapshot();
  sync_scene(&mut scene, &reduced);

  assert_eq!(scene.query::<&SceneTile>().iter(&scene).count(), 1);
  assert_eq!(scene.query::<&SceneActor>().iter(&scene).count(), 1);
  assert_eq!(scene.query::<&SceneGroundItem>().iter(&scene).count(), 0);
  assert_eq!(
    scene
      .query::<&SceneActor>()
      .iter(&scene)
      .next()
      .expect("player should remain")
      .id(),
    ActorId::new(1)
  );
}

#[test]
fn sync_removes_picked_up_items_while_preserving_other_scene_mirrors() {
  let full = PresentationState::new(7, ground_world()).snapshot();
  let mut reduced_world = ground_world();
  reduced_world
    .pickup_item(ActorId::new(1), ItemId::new(1))
    .expect("first item should be picked up");
  let reduced = PresentationState::new(7, reduced_world).snapshot();

  let mut scene = World::new();
  sync_scene(&mut scene, &full);
  let before_tiles = keyed_tiles(&mut scene);
  let before_actors = keyed_actors(&mut scene);
  let before_ground_items = keyed_ground_items(&mut scene);
  let item_one_entity = scene
    .query::<(bevy::ecs::entity::Entity, &SceneGroundItem)>()
    .iter(&scene)
    .find(|(_, item)| item.id() == ItemId::new(1))
    .map(|(entity, _)| entity)
    .expect("first ground item should exist");
  let item_three_entity = scene
    .query::<(bevy::ecs::entity::Entity, &SceneGroundItem)>()
    .iter(&scene)
    .find(|(_, item)| item.id() == ItemId::new(3))
    .map(|(entity, _)| entity)
    .expect("third ground item should exist");
  let item_two_entity = scene
    .query::<(bevy::ecs::entity::Entity, &SceneGroundItem)>()
    .iter(&scene)
    .find(|(_, item)| item.id() == ItemId::new(2))
    .map(|(entity, _)| entity)
    .expect("second ground item should exist");
  let player_entity = scene
    .query::<(bevy::ecs::entity::Entity, &SceneActor)>()
    .iter(&scene)
    .find(|(_, actor)| actor.id() == ActorId::new(1))
    .map(|(entity, _)| entity)
    .expect("player should exist");

  sync_scene(&mut scene, &reduced);

  assert_eq!(scene.query::<&SceneTile>().iter(&scene).count(), 6);
  assert_eq!(scene.query::<&SceneActor>().iter(&scene).count(), 2);
  assert_eq!(scene.query::<&SceneGroundItem>().iter(&scene).count(), 2);
  assert_eq!(keyed_tiles(&mut scene), before_tiles);
  assert_eq!(keyed_actors(&mut scene), before_actors);
  assert!(
    scene
      .query::<&SceneGroundItem>()
      .iter(&scene)
      .all(|item| item.id() != ItemId::new(1))
  );
  assert!(scene.get_entity(item_one_entity).is_err());
  assert!(
    scene
      .get_entity(before_ground_items[&ItemId::new(1)].0)
      .is_err()
  );
  assert_eq!(
    scene
      .query::<(bevy::ecs::entity::Entity, &SceneGroundItem)>()
      .iter(&scene)
      .find(|(_, item)| item.id() == ItemId::new(3))
      .map(|(entity, _)| entity),
    Some(item_three_entity)
  );
  let after_ground_items = keyed_ground_items(&mut scene);
  assert_eq!(after_ground_items[&ItemId::new(2)].0, item_two_entity);
  assert_eq!(
    (
      after_ground_items[&ItemId::new(2)].1,
      after_ground_items[&ItemId::new(2)].2,
      after_ground_items[&ItemId::new(2)].3,
    ),
    (Position::new(0, 0), ItemDefinitionId::new(102), 0)
  );
  assert_eq!(
    scene
      .query::<(bevy::ecs::entity::Entity, &SceneGroundItem)>()
      .iter(&scene)
      .find(|(_, item)| item.id() == ItemId::new(2))
      .map(|(entity, item)| (entity, item.stack_index())),
    Some((item_two_entity, 0))
  );
  assert_eq!(
    scene
      .query::<(bevy::ecs::entity::Entity, &SceneActor)>()
      .iter(&scene)
      .find(|(_, actor)| actor.id() == ActorId::new(1))
      .map(|(entity, _)| entity),
    Some(player_entity)
  );
}

#[test]
fn sync_retains_dead_actor_records_for_presentation() {
  let mut state = combat_state();
  let mut scene = World::new();
  sync_scene(&mut scene, &state.snapshot());
  let output = state
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("adjacent attack should succeed");
  sync_scene(&mut scene, output.snapshot());

  let enemy = scene
    .query::<&SceneActor>()
    .iter(&scene)
    .find(|actor| actor.id() == ActorId::new(2))
    .expect("dead actor should remain represented");
  assert!(!enemy.is_alive());
  assert_eq!(enemy.hit_points(), HitPoints::new(0));
  assert_eq!(scene.query::<&SceneActor>().iter(&scene).count(), 2);
}

#[test]
fn starter_snapshot_uses_typed_scene_values() {
  let snapshot =
    PresentationState::new(1, starter_floor().expect("content should validate")).snapshot();
  let mut scene = World::new();
  sync_scene(&mut scene, &snapshot);

  let tile = scene
    .query::<&SceneTile>()
    .iter(&scene)
    .next()
    .expect("tile should exist");
  assert_eq!(tile.position(), Position::new(0, 0));
  assert_eq!(tile.terrain(), Tile::Wall);
}
