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
