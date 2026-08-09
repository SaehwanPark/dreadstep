//! Contract tests for the read-only tester world inspection accessor.

use dreadstep_mcp::Session;
use dreadstep_protocol::{ActorId, CommandRequest};

#[test]
fn inspect_world_matches_observe_without_mutating_a_new_session() {
  let session = Session::start_run(7).expect("fixed scenario should be valid");
  let history = session.get_history();
  let replay = session.get_replay();

  assert_eq!(session.inspect_world(), session.observe());
  assert_eq!(session.get_history(), history);
  assert_eq!(session.get_replay(), replay);
}

#[test]
fn inspect_world_matches_observe_after_an_accepted_action() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .act(CommandRequest::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("attack should be accepted");

  assert_eq!(session.inspect_world(), session.observe());
}

#[test]
fn equivalent_sessions_expose_equal_inspect_world_snapshots() {
  let mut first = Session::start_run(7).expect("fixed scenario should be valid");
  let mut second = Session::start_run(7).expect("fixed scenario should be valid");
  let request = CommandRequest::Wait {
    actor: ActorId::new(1),
  };
  first.act(request).expect("wait should be accepted");
  second.act(request).expect("wait should be accepted");

  assert_eq!(first.inspect_world(), second.inspect_world());
}
