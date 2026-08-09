//! Contract tests for the typed equipment field across Bevy scene and cue projections.

use bevy::input::{ButtonInput, keyboard::KeyCode};
use bevy::{app::App, ecs::entity::Entity};
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
  world
    .give_item(
      ActorId::new(1),
      Item::new(ItemId::new(5), ItemDefinitionId::new(105)),
    )
    .expect("second item should be accepted");
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
    vec![ItemId::new(4), ItemId::new(5)]
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

#[test]
#[expect(
  clippy::too_many_lines,
  reason = "the contract intentionally compares retained identity and three ordered projections"
)]
fn replacement_and_unequip_preserve_scene_identity_and_ordered_projections() {
  let mut app = equipment_app();
  app.update();
  let initial_actor_entity = app
    .world_mut()
    .query::<(Entity, &SceneActor)>()
    .iter(app.world())
    .map(|(entity, actor)| (actor.id(), entity))
    .find(|(actor, _)| *actor == ActorId::new(1))
    .map(|(_, entity)| entity)
    .expect("actor mirror should exist");
  let initial_inventory_entities: Vec<_> = app
    .world_mut()
    .query::<(Entity, &SceneInventoryItem)>()
    .iter(app.world())
    .map(|(entity, item)| (item.id(), entity))
    .collect();

  for command in [
    Command::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    },
    Command::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(5),
    },
  ] {
    app
      .world_mut()
      .resource_mut::<PresentationRuntime>()
      .execute(command)
      .expect("equipment command should be accepted");
    app.update();
  }

  let actor_after_replacement = app
    .world_mut()
    .query::<(Entity, &SceneActor)>()
    .iter(app.world())
    .find(|(_, actor)| actor.id() == ActorId::new(1))
    .map(|(entity, actor)| (entity, *actor))
    .expect("actor mirror should remain");
  assert_eq!(actor_after_replacement.0, initial_actor_entity);
  assert_eq!(
    actor_after_replacement.1.equipped_item(),
    Some(ItemId::new(5))
  );
  let replacement_inventory_entities: Vec<_> = app
    .world_mut()
    .query::<(Entity, &SceneInventoryItem)>()
    .iter(app.world())
    .map(|(entity, item)| (item.id(), entity))
    .collect();
  assert_eq!(replacement_inventory_entities, initial_inventory_entities);
  assert_eq!(
    app.world().resource::<PresentationMessages>().messages(),
    &[
      PresentationMessage::ItemUnequipped {
        actor: ActorId::new(1),
        item: ItemId::new(4),
      },
      PresentationMessage::ItemEquipped {
        actor: ActorId::new(1),
        item: ItemId::new(5),
      },
    ]
  );
  assert_eq!(
    app.world().resource::<PresentationAudioCues>().cues(),
    &[
      PresentationAudioCue::ItemUnequipped {
        actor: ActorId::new(1),
        item: ItemId::new(4),
      },
      PresentationAudioCue::ItemEquipped {
        actor: ActorId::new(1),
        item: ItemId::new(5),
      },
    ]
  );
  assert_eq!(
    app.world().resource::<PresentationAnimationCues>().cues(),
    &[
      PresentationAnimationCue::ItemUnequipped {
        actor: ActorId::new(1),
        item: ItemId::new(4),
      },
      PresentationAnimationCue::ItemEquipped {
        actor: ActorId::new(1),
        item: ItemId::new(5),
      },
    ]
  );

  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Unequip {
      actor: ActorId::new(1),
    })
    .expect("equipment should be removable");
  app.update();
  let actor_after_unequip = app
    .world_mut()
    .query::<(Entity, &SceneActor)>()
    .iter(app.world())
    .find(|(_, actor)| actor.id() == ActorId::new(1))
    .map(|(entity, actor)| (entity, *actor))
    .expect("actor mirror should remain after unequip");
  assert_eq!(actor_after_unequip.0, initial_actor_entity);
  assert_eq!(actor_after_unequip.1.equipped_item(), None);
  assert_eq!(
    app.world().resource::<PresentationMessages>().messages(),
    &[PresentationMessage::ItemUnequipped {
      actor: ActorId::new(1),
      item: ItemId::new(5),
    }]
  );
  assert_eq!(
    app.world().resource::<PresentationAudioCues>().cues(),
    &[PresentationAudioCue::ItemUnequipped {
      actor: ActorId::new(1),
      item: ItemId::new(5),
    }]
  );
  assert_eq!(
    app.world().resource::<PresentationAnimationCues>().cues(),
    &[PresentationAnimationCue::ItemUnequipped {
      actor: ActorId::new(1),
      item: ItemId::new(5),
    }]
  );
}
