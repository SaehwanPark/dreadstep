//! Contract tests for deterministic single-item consumption across the Bevy boundary.

use bevy::input::{ButtonInput, keyboard::KeyCode};
use bevy::{app::App, ecs::entity::Entity};
use dreadstep_bevy::{
  PresentationAnimationCue, PresentationAnimationCues, PresentationAudioCue, PresentationAudioCues,
  PresentationInput, PresentationMessage, PresentationMessages, PresentationPlugin,
  PresentationRuntime, PresentationState, SceneActor, SceneGroundItem, SceneInventoryItem,
};
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, GridMap, HealingAmount, HitPoints, Item, ItemDefinitionId,
  ItemEffect, ItemId, Position, Tile, WorldState,
};

fn consumption_runtime() -> PresentationRuntime {
  let map = GridMap::filled(1, 1, Tile::Floor).expect("map should validate");
  let mut world = WorldState::new(
    map,
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .expect("world should validate");
  for (item, definition) in [(4, 104), (5, 105)] {
    world
      .give_item(
        ActorId::new(1),
        Item::new(ItemId::new(item), ItemDefinitionId::new(definition)),
      )
      .expect("item should be accepted");
  }
  PresentationRuntime::new(PresentationState::new(7, world))
}

fn consumption_app() -> App {
  let mut app = App::new();
  app.insert_resource(consumption_runtime());
  app.insert_resource(PresentationInput::new(ActorId::new(1)));
  app.insert_resource(PresentationMessages::new());
  app.insert_resource(PresentationAudioCues::new());
  app.insert_resource(PresentationAnimationCues::new());
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);
  app
}

fn pickup_app() -> App {
  let map = GridMap::filled(1, 1, Tile::Floor).expect("map should validate");
  let mut world = WorldState::new(
    map,
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .expect("world should validate");
  world
    .give_item(
      ActorId::new(1),
      Item::new(ItemId::new(4), ItemDefinitionId::new(104)),
    )
    .expect("item should be accepted");
  world
    .drop_item(ActorId::new(1), ItemId::new(4))
    .expect("item should be dropped");
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::new(PresentationState::new(7, world)));
  app.insert_resource(PresentationInput::new(ActorId::new(1)));
  app.insert_resource(PresentationMessages::new());
  app.insert_resource(PresentationAudioCues::new());
  app.insert_resource(PresentationAnimationCues::new());
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);
  app
}

fn healing_consumption_app() -> App {
  let map = GridMap::filled(1, 1, Tile::Floor).expect("map should validate");
  let mut world = WorldState::new(
    map,
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .expect("world should validate");
  world
    .give_item(
      ActorId::new(1),
      Item::with_effect(
        ItemId::new(4),
        ItemDefinitionId::new(104),
        ItemEffect::Heal {
          amount: HealingAmount::new(5).expect("positive healing should validate"),
        },
      ),
    )
    .expect("item should be accepted");
  world
    .set_hit_points(ActorId::new(1), HitPoints::new(8))
    .expect("tester damage should be accepted");
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::new(PresentationState::new(7, world)));
  app.insert_resource(PresentationInput::new(ActorId::new(1)));
  app.insert_resource(PresentationMessages::new());
  app.insert_resource(PresentationAudioCues::new());
  app.insert_resource(PresentationAnimationCues::new());
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);
  app
}

#[test]
fn consumption_removes_one_inventory_mirror_and_emits_all_typed_cues() {
  let mut app = consumption_app();
  app.update();
  let actor_before = app
    .world_mut()
    .query::<(Entity, &SceneActor)>()
    .iter(app.world())
    .find(|(_, actor)| actor.id() == ActorId::new(1))
    .map(|(entity, actor)| (entity, *actor))
    .expect("actor mirror should exist");
  let retained_item_before = app
    .world_mut()
    .query::<(Entity, &SceneInventoryItem)>()
    .iter(app.world())
    .find(|(_, item)| item.id() == ItemId::new(5))
    .map(|(entity, item)| (entity, *item))
    .expect("remaining item mirror should exist");

  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::UseItem {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    })
    .expect("owned unequipped item should be consumed");
  app.update();

  let actor_after = app
    .world_mut()
    .query::<(Entity, &SceneActor)>()
    .iter(app.world())
    .find(|(_, actor)| actor.id() == ActorId::new(1))
    .map(|(entity, actor)| (entity, *actor))
    .expect("actor mirror should remain");
  assert_eq!(actor_after.0, actor_before.0);
  assert_eq!(actor_after.1.id(), actor_before.1.id());
  assert_eq!(actor_after.1.kind(), actor_before.1.kind());
  assert_eq!(actor_after.1.position(), actor_before.1.position());
  assert_eq!(actor_after.1.hit_points(), actor_before.1.hit_points());
  assert_eq!(
    actor_after.1.ready_at().value(),
    actor_before.1.ready_at().value() + 1
  );
  assert_eq!(actor_after.1.equipped_item(), None);
  assert_eq!(actor_after.1.is_alive(), actor_before.1.is_alive());
  let remaining_items: Vec<_> = app
    .world_mut()
    .query::<(Entity, &SceneInventoryItem)>()
    .iter(app.world())
    .map(|(entity, item)| (entity, *item))
    .collect();
  assert_eq!(remaining_items.len(), 1);
  assert_eq!(remaining_items[0].0, retained_item_before.0);
  assert_eq!(remaining_items[0].1.id(), retained_item_before.1.id());
  assert_eq!(remaining_items[0].1.owner(), retained_item_before.1.owner());
  assert_eq!(
    remaining_items[0].1.definition(),
    retained_item_before.1.definition()
  );
  assert_eq!(remaining_items[0].1.inventory_index(), 0);
  assert_eq!(
    app.world().resource::<PresentationMessages>().messages(),
    &[PresentationMessage::ItemConsumed {
      actor: ActorId::new(1),
      item: ItemId::new(4),
      healing: None,
    }]
  );
  assert_eq!(
    app.world().resource::<PresentationAudioCues>().cues(),
    &[PresentationAudioCue::ItemConsumed {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    }]
  );
  assert_eq!(
    app.world().resource::<PresentationAnimationCues>().cues(),
    &[PresentationAnimationCue::ItemConsumed {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    }]
  );
}

#[test]
fn pickup_moves_scene_item_and_preserves_typed_presentation_cues() {
  let mut app = pickup_app();
  app.update();
  assert_eq!(
    app
      .world_mut()
      .query::<&SceneGroundItem>()
      .iter(app.world())
      .count(),
    1
  );
  assert_eq!(
    app
      .world_mut()
      .query::<&SceneInventoryItem>()
      .iter(app.world())
      .count(),
    0
  );

  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Pickup {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    })
    .expect("ground item should be picked up");
  app.update();

  assert_eq!(
    app
      .world_mut()
      .query::<&SceneGroundItem>()
      .iter(app.world())
      .count(),
    0
  );
  let inventory = app
    .world_mut()
    .query::<&SceneInventoryItem>()
    .iter(app.world())
    .next()
    .copied()
    .expect("picked item should become an inventory mirror");
  assert_eq!(inventory.id(), ItemId::new(4));
  assert_eq!(inventory.owner(), ActorId::new(1));
  assert_eq!(inventory.inventory_index(), 0);
  assert_eq!(
    app.world().resource::<PresentationMessages>().messages(),
    &[PresentationMessage::ItemPickedUp {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    }]
  );
  assert_eq!(
    app.world().resource::<PresentationAudioCues>().cues(),
    &[PresentationAudioCue::ItemPickedUp {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    }]
  );
  assert_eq!(
    app.world().resource::<PresentationAnimationCues>().cues(),
    &[PresentationAnimationCue::ItemPickedUp {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    }]
  );
}

#[test]
fn healing_consumption_preserves_optional_recovery_in_typed_message() {
  let mut app = healing_consumption_app();
  app.update();
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::UseItem {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    })
    .expect("healing item should be consumed");
  app.update();

  assert_eq!(
    app.world().resource::<PresentationMessages>().messages(),
    &[PresentationMessage::ItemConsumed {
      actor: ActorId::new(1),
      item: ItemId::new(4),
      healing: Some(dreadstep_core::HealingResult::new(2, HitPoints::new(10))),
    }]
  );
}
