//! Authored starter-floor item placement behavior.

use dreadstep_content::{ContentError, StarterItemPlacement, starter_floor_definition};
use dreadstep_core::{ActorId, Item, ItemDefinitionId, ItemId};

#[test]
fn authored_items_preserve_actor_order_and_complete_data() {
  let definition = starter_floor_definition().with_items(vec![
    StarterItemPlacement::new(
      ActorId::new(1),
      Item::new(ItemId::new(41), ItemDefinitionId::new(7)),
    ),
    StarterItemPlacement::new(
      ActorId::new(1),
      Item::new(ItemId::new(42), ItemDefinitionId::new(8)),
    ),
    StarterItemPlacement::new(
      ActorId::new(2),
      Item::new(ItemId::new(43), ItemDefinitionId::new(9)),
    ),
  ]);

  let world = definition.build().expect("authored items should validate");
  let actors: Vec<_> = world.actors().collect();
  assert_eq!(
    actors[0].inventory(),
    &[
      Item::new(ItemId::new(41), ItemDefinitionId::new(7)),
      Item::new(ItemId::new(42), ItemDefinitionId::new(8)),
    ]
  );
  assert_eq!(
    actors[1].inventory(),
    &[Item::new(ItemId::new(43), ItemDefinitionId::new(9))]
  );
  assert_eq!(
    world.digest(),
    definition
      .build()
      .expect("repeated authored items should validate")
      .digest()
  );
}

#[test]
fn default_starter_floor_remains_item_free() {
  let world = starter_floor_definition()
    .build()
    .expect("default starter floor should validate");

  assert!(world.actors().all(|actor| actor.inventory().is_empty()));
  assert!(world.ground_items().is_empty());
}

#[test]
fn invalid_item_placements_are_typed_and_atomic() {
  let unknown_actor = starter_floor_definition().with_items(vec![StarterItemPlacement::new(
    ActorId::new(99),
    Item::new(ItemId::new(41), ItemDefinitionId::new(7)),
  )]);
  assert_eq!(
    unknown_actor.build(),
    Err(ContentError::World(
      dreadstep_core::WorldError::UnknownActor(ActorId::new(99),)
    ))
  );

  let duplicate = starter_floor_definition().with_items(vec![
    StarterItemPlacement::new(
      ActorId::new(1),
      Item::new(ItemId::new(41), ItemDefinitionId::new(7)),
    ),
    StarterItemPlacement::new(
      ActorId::new(2),
      Item::new(ItemId::new(41), ItemDefinitionId::new(8)),
    ),
  ]);
  assert_eq!(
    duplicate.build(),
    Err(ContentError::World(
      dreadstep_core::WorldError::DuplicateItemId(ItemId::new(41),)
    ))
  );
}
