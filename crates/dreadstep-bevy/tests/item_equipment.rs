//! Contract tests for the typed equipment field across Bevy scene and cue projections.

use bevy::app::App;
use bevy::input::{ButtonInput, keyboard::KeyCode};
use dreadstep_bevy::{
  PresentationAnimationCue, PresentationAnimationCues, PresentationAudioCue, PresentationAudioCues,
  PresentationInput, PresentationMessage, PresentationMessages, PresentationPlugin,
  PresentationRuntime, PresentationState, SceneActor, SceneInventoryItem,
};
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, GridMap, Item, ItemDefinitionId, ItemId, Position, Tile,
  WorldState,
};

fn equipment_runtime() -> PresentationRuntime {
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
  PresentationRuntime::new(PresentationState::new(7, world))
}

fn equipment_app() -> App {
  let mut app = App::new();
  app.insert_resource(equipment_runtime());
  app.insert_resource(PresentationInput::new(ActorId::new(1)));
  app.insert_resource(PresentationMessages::new());
  app.insert_resource(PresentationAudioCues::new());
  app.insert_resource(PresentationAnimationCues::new());
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);
  app
}

#[test]
fn equipment_updates_complete_actor_inventory_and_typed_cues() {
  let mut app = equipment_app();
  app.update();
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    })
    .expect("owned item should equip");
  app.update();

  let actors: Vec<_> = app
    .world_mut()
    .query::<&SceneActor>()
    .iter(app.world())
    .copied()
    .collect();
  assert_eq!(actors.len(), 1);
  assert_eq!(actors[0].id(), ActorId::new(1));
  assert_eq!(actors[0].equipped_item(), Some(ItemId::new(4)));
  assert_eq!(actors[0].position(), Position::new(0, 0));
  assert_eq!(
    app
      .world_mut()
      .query::<&SceneInventoryItem>()
      .iter(app.world())
      .map(|item| item.id())
      .collect::<Vec<_>>(),
    vec![ItemId::new(4)]
  );
  assert_eq!(
    app.world().resource::<PresentationMessages>().messages(),
    &[PresentationMessage::ItemEquipped {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    }]
  );
  assert_eq!(
    app.world().resource::<PresentationAudioCues>().cues(),
    &[PresentationAudioCue::ItemEquipped {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    }]
  );
  assert_eq!(
    app.world().resource::<PresentationAnimationCues>().cues(),
    &[PresentationAnimationCue::ItemEquipped {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    }]
  );
}
