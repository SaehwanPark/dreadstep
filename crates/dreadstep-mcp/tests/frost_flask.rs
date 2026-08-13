//! Contract tests for Frost Flask throws through the bounded MCP session.

use dreadstep_mcp::Session;
use dreadstep_protocol::{ActorId, CommandRequest, Event, ItemId, StatusKind};

#[test]
fn authored_frost_flask_throws_project_status_events_and_replay_history() {
  let mut session = Session::start_item_run(7).expect("authored item scenario should be valid");
  let request = CommandRequest::Throw {
    actor: ActorId::new(1),
    item: ItemId::new(104),
    target: ActorId::new(2),
  };
  assert!(session.legal_actions().contains(&request));

  let output = session.act(request).expect("Frost Flask should throw");
  assert_eq!(
    output.events(),
    &[
      Event::ItemThrown {
        actor: ActorId::new(1),
        item: ItemId::new(104),
        target: ActorId::new(2),
      },
      Event::StatusApplied {
        actor: ActorId::new(2),
        status: StatusKind::Chilled,
        remaining_actions: 2,
      },
    ]
  );
  assert!(
    output
      .snapshot()
      .actors()
      .iter()
      .find(|actor| actor.id() == ActorId::new(1))
      .expect("player should remain in snapshot")
      .inventory()
      .iter()
      .all(|item| item.id() != ItemId::new(104))
  );
  assert_eq!(session.history(), vec![request]);
  assert_eq!(session.get_replay().commands(), &[request]);
}
