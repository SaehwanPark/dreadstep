//! MCP tester item-pickup behavior.

use dreadstep_mcp::{Session, SessionError};
use dreadstep_protocol::{ActorId, ItemDefinitionId, ItemId, WorldError};

#[test]
fn pickup_projects_inventory_without_player_replay() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .give_item(ActorId::new(1), ItemId::new(1), ItemDefinitionId::new(10))
    .expect("item should be accepted");
  let expected = session.inspect_world();
  session
    .drop_item(ActorId::new(1), ItemId::new(1))
    .expect("item should drop");
  let history = session.get_history();
  let replay = session.get_replay();

  session
    .pickup_item(ActorId::new(1), ItemId::new(1))
    .expect("ground item should be picked up");

  assert_eq!(session.inspect_world(), expected);
  assert_eq!(session.get_history(), history);
  assert_eq!(session.get_replay(), replay);
}

#[test]
fn pickup_rejection_is_typed_and_atomic() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  let before = session.inspect_world();
  let history = session.get_history();
  let replay = session.get_replay();

  assert_eq!(
    session.pickup_item(ActorId::new(1), ItemId::new(1)),
    Err(SessionError::WorldRejected(WorldError::ItemNotOnGround {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    }))
  );
  assert_eq!(session.inspect_world(), before);
  assert_eq!(session.get_history(), history);
  assert_eq!(session.get_replay(), replay);
}
