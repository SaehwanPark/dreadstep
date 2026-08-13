//! Authored starter-floor item placement behavior.

use dreadstep_content::{
  ContentError, ItemCatalogDefinition, StarterItemPlacement, starter_floor,
  starter_floor_definition, starter_item_floor, starter_item_floor_definition,
};
use dreadstep_core::{
  ActorId, AmmunitionAmount, HealingAmount, Item, ItemDefinitionId, ItemEffect, ItemId, MeleeReach,
  ThrowableEffect, WorldState,
};

#[test]
fn starter_item_floor_is_complete_and_repeatable() {
  let from_definition = starter_item_floor_definition()
    .build()
    .expect("authored starter item floor should validate");
  let from_wrapper = starter_item_floor().expect("starter item floor wrapper should validate");
  let default = starter_floor().expect("default starter floor should validate");
  let actors: Vec<_> = from_definition.actors().collect();

  assert_eq!(from_wrapper, from_definition);
  assert_ne!(from_definition.map(), default.map());
  assert_eq!(
    from_definition
      .map()
      .tile_at(dreadstep_core::Position::new(4, 3)),
    Some(dreadstep_core::Tile::Breakable)
  );
  let actor_projection = |world: &WorldState| {
    world
      .actors()
      .map(|actor| {
        (
          actor.id(),
          actor.kind(),
          actor.position(),
          actor.hit_points(),
          actor.ready_at(),
          actor.is_alive(),
        )
      })
      .collect::<Vec<_>>()
  };
  assert_eq!(
    actor_projection(&from_definition),
    actor_projection(&default)
  );
  assert_eq!(from_definition.current_time(), default.current_time());
  assert_eq!(from_definition.next_actor(), default.next_actor());
  assert_eq!(
    actors[0].inventory(),
    &[
      Item::with_effect(
        ItemId::new(101),
        ItemDefinitionId::new(2),
        ItemEffect::Heal {
          amount: HealingAmount::new(3).expect("starter healing amount should be positive"),
        },
      ),
      Item::with_equipment_effect(
        ItemId::new(103),
        ItemDefinitionId::new(4),
        MeleeReach::new(2).expect("starter weapon reach should be positive"),
      ),
      Item::with_throwable_effect(
        ItemId::new(104),
        ItemDefinitionId::new(5),
        ThrowableEffect::Chill,
      ),
      Item::with_effect(
        ItemId::new(102),
        ItemDefinitionId::new(3),
        ItemEffect::RestoreAmmunition {
          amount: AmmunitionAmount::new(2).expect("starter ammunition amount should be positive"),
        },
      ),
    ]
  );
  assert_eq!(
    actors[1].inventory(),
    &[Item::new(ItemId::new(100), ItemDefinitionId::new(1))]
  );
  assert!(actors[2..].iter().all(|actor| actor.inventory().is_empty()));
  assert_eq!(
    actors[3].enemy_behavior(),
    dreadstep_core::EnemyBehavior::Brute
  );
  assert!(from_definition.ground_items().is_empty());
  assert_eq!(
    from_definition.digest(),
    starter_item_floor()
      .expect("repeated starter item floor should validate")
      .digest()
  );
}

#[test]
fn authored_items_preserve_actor_order_and_complete_data() {
  let definition = starter_floor_definition()
    .with_item_catalog(ItemCatalogDefinition::new(vec![
      ItemDefinitionId::new(6),
      ItemDefinitionId::new(9),
      ItemDefinitionId::new(7),
      ItemDefinitionId::new(8),
    ]))
    .with_items(vec![
      StarterItemPlacement::new(
        ActorId::new(1),
        Item::new(ItemId::new(42), ItemDefinitionId::new(8)),
      ),
      StarterItemPlacement::new(
        ActorId::new(2),
        Item::new(ItemId::new(41), ItemDefinitionId::new(7)),
      ),
      StarterItemPlacement::new(
        ActorId::new(1),
        Item::new(ItemId::new(40), ItemDefinitionId::new(9)),
      ),
      StarterItemPlacement::new(
        ActorId::new(2),
        Item::new(ItemId::new(43), ItemDefinitionId::new(6)),
      ),
    ]);

  let world = definition.build().expect("authored items should validate");
  let actors: Vec<_> = world.actors().collect();
  assert_eq!(
    actors[0].inventory(),
    &[
      Item::new(ItemId::new(42), ItemDefinitionId::new(8)),
      Item::new(ItemId::new(40), ItemDefinitionId::new(9)),
    ]
  );
  assert_eq!(
    actors[1].inventory(),
    &[
      Item::new(ItemId::new(41), ItemDefinitionId::new(7)),
      Item::new(ItemId::new(43), ItemDefinitionId::new(6)),
    ]
  );
  assert_eq!(
    world.digest(),
    definition
      .build()
      .expect("repeated authored items should validate")
      .digest()
  );
  let reordered_catalog = definition
    .clone()
    .with_item_catalog(ItemCatalogDefinition::new(vec![
      ItemDefinitionId::new(8),
      ItemDefinitionId::new(7),
      ItemDefinitionId::new(9),
      ItemDefinitionId::new(6),
    ]));
  assert_eq!(
    world,
    reordered_catalog
      .build()
      .expect("catalog order should not affect the core world")
  );
}

#[test]
fn default_starter_floor_remains_item_free() {
  let from_definition = starter_floor_definition()
    .build()
    .expect("default starter floor should validate");
  let from_wrapper = dreadstep_content::starter_floor().expect("starter wrapper should validate");

  assert_eq!(from_wrapper, from_definition);
  for world in [from_definition, from_wrapper] {
    assert!(world.actors().all(|actor| actor.inventory().is_empty()));
    assert!(world.ground_items().is_empty());
  }
}

#[test]
fn invalid_item_placements_are_typed_and_atomic() {
  let unknown_actor = starter_floor_definition()
    .with_item_catalog(ItemCatalogDefinition::new(vec![ItemDefinitionId::new(7)]))
    .with_items(vec![StarterItemPlacement::new(
      ActorId::new(99),
      Item::new(ItemId::new(41), ItemDefinitionId::new(7)),
    )]);
  assert_eq!(
    unknown_actor.build(),
    Err(ContentError::World(
      dreadstep_core::WorldError::UnknownActor(ActorId::new(99),)
    ))
  );

  let duplicate = starter_floor_definition()
    .with_item_catalog(ItemCatalogDefinition::new(vec![
      ItemDefinitionId::new(7),
      ItemDefinitionId::new(8),
    ]))
    .with_items(vec![
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

#[test]
fn catalog_rejects_duplicate_and_unknown_definitions_before_world_build() {
  let duplicate_catalog =
    dreadstep_content::StarterFloorDefinition::new(0, 0, Vec::new(), Vec::new())
      .with_item_catalog(ItemCatalogDefinition::new(vec![
        ItemDefinitionId::new(7),
        ItemDefinitionId::new(7),
      ]))
      .with_items(vec![StarterItemPlacement::new(
        ActorId::new(1),
        Item::new(ItemId::new(41), ItemDefinitionId::new(7)),
      )]);
  assert_eq!(
    duplicate_catalog.build(),
    Err(ContentError::DuplicateItemDefinitionId(
      ItemDefinitionId::new(7)
    ))
  );

  let unknown_definition =
    dreadstep_content::StarterFloorDefinition::new(0, 0, Vec::new(), Vec::new())
      .with_item_catalog(ItemCatalogDefinition::new(vec![ItemDefinitionId::new(7)]))
      .with_items(vec![StarterItemPlacement::new(
        ActorId::new(1),
        Item::new(ItemId::new(41), ItemDefinitionId::new(8)),
      )]);
  assert_eq!(
    unknown_definition.build(),
    Err(ContentError::UnknownItemDefinitionId(
      ItemDefinitionId::new(8)
    ))
  );
}
