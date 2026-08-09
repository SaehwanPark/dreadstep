//! MCP tester item-drop behavior.

use dreadstep_mcp::{Session, SessionError};
use dreadstep_protocol::{ActorId, ItemDefinitionId, ItemId, WorldError};

#[test]
fn drop_projects_ground_item_without_player_replay() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .give_item(ActorId::new(1), ItemId::new(1), ItemDefinitionId::new(10))
    .expect("item should be accepted");
  let history = session.get_history();
  let replay = session.get_replay();

  session
    .drop_item(ActorId::new(1), ItemId::new(1))
    .expect("owned item should drop");

  let snapshot = session.inspect_world();
  assert_eq!(snapshot.actors()[0].inventory(), &[]);
  assert_eq!(snapshot.ground_items().len(), 1);
  assert_eq!(
    snapshot.ground_items()[0].position(),
    snapshot.actors()[0].position()
  );
  assert_eq!(snapshot.ground_items()[0].items()[0].id(), ItemId::new(1));
  assert_eq!(session.get_history(), history);
  assert_eq!(session.get_replay(), replay);
}

#[test]
fn drop_rejection_is_typed_and_atomic() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  let before = session.inspect_world();
  let history = session.get_history();
  let replay = session.get_replay();

  assert_eq!(
    session.drop_item(ActorId::new(1), ItemId::new(1)),
    Err(SessionError::WorldRejected(WorldError::ItemNotOwned {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    }))
  );
  assert_eq!(session.inspect_world(), before);
  assert_eq!(session.get_history(), history);
  assert_eq!(session.get_replay(), replay);
}
