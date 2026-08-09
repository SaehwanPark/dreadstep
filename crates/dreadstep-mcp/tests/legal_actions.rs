//! Contract tests for MCP legal-action conversion.

use dreadstep_mcp::Session;
use dreadstep_protocol::{ActorId, CommandRequest, Direction};

#[test]
fn session_exposes_protocol_actions_for_the_scheduled_player() {
  let session = Session::start_run(7).expect("fixed scenario should be valid");
  let actions = session.legal_actions();

  assert_eq!(actions.len(), 6);
  assert_eq!(
    actions[0],
    CommandRequest::Move {
      actor: ActorId::new(1),
      direction: Direction::North,
    }
  );
  assert_eq!(
    actions[5],
    CommandRequest::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }
  );
}

#[test]
fn equivalent_sessions_expose_equal_legal_action_lists() {
  let first = Session::start_run(7).expect("fixed scenario should be valid");
  let second = Session::start_run(7).expect("fixed scenario should be valid");

  assert_eq!(first.legal_actions(), second.legal_actions());
}

#[test]
fn legal_action_discovery_is_read_only_for_world_history_and_replay() {
  let session = Session::start_run(7).expect("fixed scenario should be valid");
  let before_snapshot = session.observe();
  let before_history = session.history();
  let before_replay = session.get_replay();

  let actions = session.legal_actions();

  assert_eq!(actions.len(), 6);
  assert_eq!(session.observe(), before_snapshot);
  assert_eq!(session.history(), before_history);
  assert_eq!(session.get_replay(), before_replay);
}
