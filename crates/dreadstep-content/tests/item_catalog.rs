//! Deterministic authored item-definition catalog behavior.

use dreadstep_content::{ContentError, ItemCatalogDefinition, starter_item_catalog};
use dreadstep_core::ItemDefinitionId;

#[test]
fn starter_catalog_has_stable_order_and_known_unknown_lookup() {
  let catalog = starter_item_catalog().expect("starter item catalog should validate");

  assert_eq!(
    catalog.definitions(),
    &[
      ItemDefinitionId::new(1),
      ItemDefinitionId::new(2),
      ItemDefinitionId::new(3),
      ItemDefinitionId::new(4),
      ItemDefinitionId::new(5),
      ItemDefinitionId::new(6),
      ItemDefinitionId::new(7),
    ]
  );
  assert!(catalog.contains(ItemDefinitionId::new(2)));
  assert!(!catalog.contains(ItemDefinitionId::new(99)));
}

#[test]
fn authored_order_is_preserved_and_repeat_construction_is_equal() {
  let definition = ItemCatalogDefinition::new(vec![
    ItemDefinitionId::new(3),
    ItemDefinitionId::new(1),
    ItemDefinitionId::new(2),
  ]);
  let catalog = definition.build().expect("unique IDs should validate");

  assert_eq!(
    catalog.definitions(),
    &[
      ItemDefinitionId::new(3),
      ItemDefinitionId::new(1),
      ItemDefinitionId::new(2),
    ]
  );
  assert_eq!(
    starter_item_catalog().expect("starter item catalog should validate"),
    starter_item_catalog().expect("same starter content should validate")
  );
}

#[test]
fn duplicate_definition_ids_are_rejected_before_catalog_creation() {
  let definition =
    ItemCatalogDefinition::new(vec![ItemDefinitionId::new(7), ItemDefinitionId::new(7)]);

  assert_eq!(
    definition.build(),
    Err(ContentError::DuplicateItemDefinitionId(
      ItemDefinitionId::new(7)
    ))
  );
}
