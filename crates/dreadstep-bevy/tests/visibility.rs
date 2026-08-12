//! Deterministic presentation-only field-of-view projection.

use bevy::app::App;
use bevy::camera::visibility::Visibility;
use dreadstep_bevy::{
  PresentationBevySpriteProjection, PresentationInput, PresentationPlugin,
  PresentationRenderCommandPlan, PresentationRenderNodeProjection, PresentationRenderProjection,
  PresentationRuntime, PresentationSpriteProjection, PresentationState, PresentationVisibility,
  SceneActor, SceneGroundItem, SceneRenderNode, SceneTile,
};
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, Direction, GridMap, HitPoints, Item, ItemDefinitionId,
  ItemId, Position, Tile, WorldState,
};

const PLAYER: ActorId = ActorId::new(1);
const ENEMY: ActorId = ActorId::new(2);

fn visibility_app() -> App {
  let map = GridMap::from_tiles(
    5,
    3,
    vec![
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Cover,
      Tile::Cover,
      Tile::Floor,
      Tile::Wall,
      Tile::Floor,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
    ],
  )
  .expect("visibility map should validate");
  let mut world = WorldState::new(
    map,
    vec![
      Actor::new(PLAYER, ActorKind::Player, Position::new(0, 1)),
      Actor::with_hit_points(
        ENEMY,
        ActorKind::Enemy,
        Position::new(2, 1),
        HitPoints::new(2),
      ),
    ],
  )
  .expect("visibility world should validate");
  world
    .give_item(
      PLAYER,
      Item::new(ItemId::new(10), ItemDefinitionId::new(101)),
    )
    .expect("inventory item should validate");
  world
    .give_item(
      ENEMY,
      Item::new(ItemId::new(11), ItemDefinitionId::new(102)),
    )
    .expect("ground item should validate");
  world
    .drop_item(ENEMY, ItemId::new(11))
    .expect("ground item should drop");

  let mut app = App::new();
  app.insert_resource(PresentationRuntime::new(PresentationState::new(7, world)));
  app.insert_resource(PresentationInput::new(PLAYER));
  app.insert_resource(PresentationVisibility::new(PLAYER, 1));
  app.insert_resource(PresentationRenderProjection::new());
  app.insert_resource(PresentationSpriteProjection::new());
  app.insert_resource(PresentationRenderCommandPlan::new());
  app.insert_resource(PresentationRenderNodeProjection::new());
  app.insert_resource(PresentationBevySpriteProjection::new());
  app.add_plugins(PresentationPlugin);
  app.update();
  app
}

fn node_visibility(app: &mut App, node: SceneRenderNode) -> Visibility {
  let entity = node_entity(app, node);
  node_visibility_at(app, entity)
}

fn node_entity(app: &mut App, node: SceneRenderNode) -> bevy::ecs::entity::Entity {
  app
    .world_mut()
    .query::<(bevy::ecs::entity::Entity, &SceneRenderNode)>()
    .iter(app.world())
    .find_map(|(entity, candidate)| (*candidate == node).then_some(entity))
    .expect("render node entity should exist")
}

fn node_visibility_at(app: &mut App, node_entity: bevy::ecs::entity::Entity) -> Visibility {
  *app
    .world_mut()
    .get::<Visibility>(node_entity)
    .expect("render node should retain visibility")
}

fn actor_node(app: &mut App, actor: ActorId) -> SceneRenderNode {
  let nodes = app
    .world_mut()
    .query::<&SceneRenderNode>()
    .iter(app.world())
    .copied()
    .collect::<Vec<_>>();
  nodes
    .into_iter()
    .find(|node| {
      app
        .world()
        .get::<SceneActor>(node.source_entity())
        .is_some_and(|record| record.id() == actor)
    })
    .expect("actor render node should exist")
}

fn ground_node(app: &mut App, item_id: ItemId) -> SceneRenderNode {
  let nodes = app
    .world_mut()
    .query::<&SceneRenderNode>()
    .iter(app.world())
    .copied()
    .collect::<Vec<_>>();
  nodes
    .into_iter()
    .find(|node| {
      app
        .world()
        .get::<SceneGroundItem>(node.source_entity())
        .is_some_and(|item| item.id() == item_id)
    })
    .expect("ground render node should exist")
}

#[test]
fn bounded_traversal_keeps_cover_walkable_and_reveals_wall_boundary() {
  let mut app = visibility_app();
  let visibility = app.world().resource::<PresentationVisibility>();
  assert!(visibility.is_active());
  assert_eq!(visibility.actor(), PLAYER);
  assert_eq!(visibility.radius(), 1);
  assert_eq!(
    visibility.visible_positions(),
    [
      Position::new(0, 0),
      Position::new(1, 0),
      Position::new(0, 1),
      Position::new(1, 1),
      Position::new(0, 2),
      Position::new(1, 2),
    ]
  );
  assert!(visibility.is_visible(Position::new(1, 0)));
  assert!(visibility.is_visible(Position::new(0, 1)));
  assert!(visibility.is_visible(Position::new(1, 1)));
  assert!(!visibility.is_visible(Position::new(2, 1)));

  let mut positions = app
    .world_mut()
    .query::<&SceneTile>()
    .iter(app.world())
    .map(|tile| tile.position())
    .collect::<Vec<_>>();
  positions.sort_by_key(|position| (position.y(), position.x()));
  assert_eq!(positions.len(), 15);
}

#[test]
fn out_of_view_nodes_are_hidden_without_despawning_typed_mirrors() {
  let mut app = visibility_app();
  let player = actor_node(&mut app, PLAYER);
  let enemy = actor_node(&mut app, ENEMY);
  let ground = ground_node(&mut app, ItemId::new(11));
  let player_entity = node_entity(&mut app, player);
  let enemy_entity = node_entity(&mut app, enemy);
  let ground_entity = node_entity(&mut app, ground);

  assert_eq!(node_visibility(&mut app, player), Visibility::Inherited);
  assert_eq!(node_visibility(&mut app, enemy), Visibility::Hidden);
  assert_eq!(node_visibility(&mut app, ground), Visibility::Hidden);
  assert_eq!(
    app
      .world_mut()
      .query::<&SceneActor>()
      .iter(app.world())
      .count(),
    2
  );
  assert_eq!(
    app
      .world_mut()
      .query::<&SceneGroundItem>()
      .iter(app.world())
      .count(),
    1
  );

  app.world_mut().remove_resource::<PresentationRuntime>();
  app.update();
  assert!(
    app
      .world()
      .get::<SceneRenderNode>(player_entity)
      .expect("player node should be retained")
      .is_visible()
  );
  assert!(
    app
      .world()
      .get::<SceneRenderNode>(enemy_entity)
      .expect("enemy node should be retained")
      .is_visible()
  );
  assert!(
    app
      .world()
      .get::<SceneRenderNode>(ground_entity)
      .expect("ground node should be retained")
      .is_visible()
  );
  assert_eq!(
    node_visibility_at(&mut app, enemy_entity),
    Visibility::Inherited
  );
}

#[test]
fn movement_and_controlled_actor_refresh_projection_and_missing_input_restores_visibility() {
  let mut app = visibility_app();
  let before = app
    .world()
    .resource::<PresentationVisibility>()
    .visible_positions()
    .to_vec();
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Move {
      actor: PLAYER,
      direction: Direction::East,
    })
    .expect("player movement should be accepted");
  app.update();
  let after = app
    .world()
    .resource::<PresentationVisibility>()
    .visible_positions()
    .to_vec();
  assert_ne!(before, after);
  assert!(
    app
      .world()
      .resource::<PresentationVisibility>()
      .is_visible(Position::new(2, 1))
  );
  let enemy_node = actor_node(&mut app, ENEMY);
  let enemy_entity = node_entity(&mut app, enemy_node);
  assert!(
    app
      .world()
      .get::<SceneRenderNode>(enemy_entity)
      .expect("enemy node should be retained")
      .is_visible()
  );

  app
    .world_mut()
    .insert_resource(PresentationInput::new(ENEMY));
  app.update();
  let visibility = app.world().resource::<PresentationVisibility>();
  assert_eq!(visibility.actor(), ENEMY);
  assert!(visibility.is_visible(Position::new(2, 1)));
  assert!(!visibility.is_visible(Position::new(0, 1)));

  app.world_mut().remove_resource::<PresentationVisibility>();
  app.update();
  assert!(
    app
      .world()
      .get_resource::<PresentationVisibility>()
      .is_none()
  );
  assert!(
    app
      .world()
      .get::<SceneRenderNode>(enemy_entity)
      .expect("enemy node should remain after visibility removal")
      .is_visible()
  );
  assert_eq!(
    node_visibility_at(&mut app, enemy_entity),
    Visibility::Inherited
  );

  app.insert_resource(PresentationVisibility::new(ENEMY, 1));
  app.update();
  assert!(app.world().resource::<PresentationVisibility>().is_active());
  app.world_mut().remove_resource::<PresentationInput>();
  app.update();
  assert!(!app.world().resource::<PresentationVisibility>().is_active());
  assert_eq!(
    node_visibility_at(&mut app, enemy_entity),
    Visibility::Inherited
  );
}
