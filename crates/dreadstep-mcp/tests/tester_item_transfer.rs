//! MCP tester item-transfer behavior.

use dreadstep_mcp::{Session, SessionError};
use dreadstep_protocol::{ActorId, ItemDefinitionId, ItemId, WorldError};

#[test]
fn transfer_projects_ordered_inventory_without_player_replay() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .give_item(ActorId::new(1), ItemId::new(1), ItemDefinitionId::new(10))
    .expect("first item should be accepted");
  session
    .give_item(ActorId::new(1), ItemId::new(2), ItemDefinitionId::new(20))
    .expect("second item should be accepted");
  let history = session.get_history();
  let replay = session.get_replay();

  session
    .transfer_item(ActorId::new(1), ActorId::new(2), ItemId::new(1))
    .expect("owned item should transfer");

  let source = session.inspect(ActorId::new(1)).expect("source exists");
  let target = session.inspect(ActorId::new(2)).expect("target exists");
  assert_eq!(source.inventory().len(), 1);
  assert_eq!(source.inventory()[0].id(), ItemId::new(2));
  assert_eq!(target.inventory().len(), 1);
  assert_eq!(target.inventory()[0].id(), ItemId::new(1));
  assert_eq!(session.get_history(), history);
  assert_eq!(session.get_replay(), replay);
}

#[test]
fn transfer_rejection_is_typed_and_atomic() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .give_item(ActorId::new(1), ItemId::new(1), ItemDefinitionId::new(10))
    .expect("item should be accepted");
  let before = session.inspect_world();
  let history = session.get_history();
  let replay = session.get_replay();

  assert_eq!(
    session.transfer_item(ActorId::new(1), ActorId::new(2), ItemId::new(2)),
    Err(SessionError::WorldRejected(WorldError::ItemNotOwned {
      actor: ActorId::new(1),
      item: ItemId::new(2),
    }))
  );
  assert_eq!(session.inspect_world(), before);
  assert_eq!(session.get_history(), history);
  assert_eq!(session.get_replay(), replay);
}
