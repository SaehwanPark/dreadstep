//! Contract tests for MCP tester item ownership.

use dreadstep_mcp::{Session, SessionError};
use dreadstep_protocol::{ActorId, ItemDefinitionId, ItemId, WorldError};

#[test]
fn give_item_projects_ordered_inventory_without_recording_player_replay() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  let history = session.get_history();
  let replay = session.get_replay();

  session
    .give_item(ActorId::new(1), ItemId::new(1), ItemDefinitionId::new(10))
    .expect("item should be accepted");
  session
    .give_item(ActorId::new(1), ItemId::new(2), ItemDefinitionId::new(20))
    .expect("second item should be accepted");

  let actor = session
    .inspect(ActorId::new(1))
    .expect("actor should be inspectable");
  assert_eq!(actor.inventory().len(), 2);
  assert_eq!(actor.inventory()[0].id(), ItemId::new(1));
  assert_eq!(actor.inventory()[0].definition(), ItemDefinitionId::new(10));
  assert_eq!(actor.inventory()[1].id(), ItemId::new(2));
  assert_eq!(actor.inventory()[1].definition(), ItemDefinitionId::new(20));
  assert_eq!(session.get_history(), history);
  assert_eq!(session.get_replay(), replay);
}

#[test]
fn duplicate_item_rejection_is_typed_and_atomic() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .give_item(ActorId::new(1), ItemId::new(1), ItemDefinitionId::new(10))
    .expect("item should be accepted");
  let before = session.inspect_world();
  let history = session.get_history();
  let replay = session.get_replay();

  assert_eq!(
    session.give_item(ActorId::new(2), ItemId::new(1), ItemDefinitionId::new(99)),
    Err(SessionError::WorldRejected(WorldError::DuplicateItemId(
      ItemId::new(1)
    )))
  );
  assert_eq!(session.inspect_world(), before);
  assert_eq!(session.get_history(), history);
  assert_eq!(session.get_replay(), replay);

  assert_eq!(
    session.give_item(ActorId::new(9), ItemId::new(2), ItemDefinitionId::new(20)),
    Err(SessionError::WorldRejected(WorldError::UnknownActor(
      ActorId::new(9)
    )))
  );
  assert_eq!(session.inspect_world(), before);
  assert_eq!(session.get_history(), history);
  assert_eq!(session.get_replay(), replay);
}

#[test]
fn capacity_rejection_is_typed_and_atomic() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  for id in 1..=4 {
    session
      .give_item(
        ActorId::new(1),
        ItemId::new(id),
        ItemDefinitionId::new(id + 100),
      )
      .expect("capacity-sized inventory should be accepted");
  }
  let before = session.inspect_world();
  let history = session.get_history();
  let replay = session.get_replay();

  assert_eq!(
    session.give_item(ActorId::new(1), ItemId::new(99), ItemDefinitionId::new(199)),
    Err(SessionError::WorldRejected(WorldError::InventoryFull(
      ActorId::new(1)
    )))
  );
  assert_eq!(session.inspect_world(), before);
  assert_eq!(session.get_history(), history);
  assert_eq!(session.get_replay(), replay);
}
