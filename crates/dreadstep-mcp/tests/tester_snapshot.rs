//! Contract tests for in-memory tester savepoints and restore.

use dreadstep_mcp::Session;
use dreadstep_protocol::{ActorId, CommandRequest};

#[test]
fn snapshot_is_read_only_and_restore_returns_to_captured_state() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  let initial_world = session.observe();
  let initial_replay = session.get_replay();
  let savepoint = session.snapshot();

  assert_eq!(savepoint.seed(), 7);
  assert_eq!(session.observe(), initial_world);
  assert_eq!(session.get_replay(), initial_replay);

  session
    .act(CommandRequest::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("attack should be accepted");
  assert_ne!(session.observe(), initial_world);
  assert_ne!(session.get_replay(), initial_replay);

  let mut different_seed = Session::start_run(99).expect("fixed scenario should be valid");
  different_seed.restore(savepoint.clone());
  assert_eq!(different_seed.seed(), 7);
  assert_eq!(different_seed.observe(), initial_world);
  assert_eq!(different_seed.get_replay(), initial_replay);

  session.restore(savepoint);
  assert_eq!(session.observe(), initial_world);
  assert_eq!(session.get_replay(), initial_replay);
  assert!(session.get_history().is_empty());
}

#[test]
fn restoring_a_savepoint_repeats_the_same_protocol_transition() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .act(CommandRequest::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("attack should be accepted");
  let savepoint = session.snapshot();

  let first = session
    .act(CommandRequest::Wait {
      actor: ActorId::new(2),
    })
    .expect("enemy wait should be accepted");
  let first_world = session.observe();
  let first_replay = session.get_replay();

  session.restore(savepoint);
  let repeated = session
    .act(CommandRequest::Wait {
      actor: ActorId::new(2),
    })
    .expect("enemy wait should be accepted after restore");

  assert_eq!(repeated, first);
  assert_eq!(session.observe(), first_world);
  assert_eq!(session.get_replay(), first_replay);
}
