//! Contract tests for read-only player actor inspection.

use dreadstep_mcp::Session;
use dreadstep_protocol::{ActorId, CommandRequest, LifeState};

#[test]
fn inspect_returns_known_actor_from_the_current_snapshot() {
  let session = Session::start_run(7).expect("fixed scenario should be valid");
  let inspected = session
    .inspect(ActorId::new(1))
    .expect("player actor should be present");
  let snapshot = session.observe();
  let from_snapshot = snapshot
    .actors()
    .iter()
    .find(|actor| actor.id() == ActorId::new(1))
    .expect("player actor should be in snapshot");

  assert_eq!(&inspected, from_snapshot);
  assert_eq!(inspected.life(), LifeState::Alive);
}

#[test]
fn inspect_unknown_actor_is_absent_and_does_not_mutate_session() {
  let session = Session::start_run(7).expect("fixed scenario should be valid");
  let snapshot = session.observe();
  let replay = session.get_replay();

  assert_eq!(session.inspect(ActorId::new(99)), None);
  assert_eq!(session.observe(), snapshot);
  assert_eq!(session.get_replay(), replay);
}

#[test]
fn inspect_retains_dead_actor_after_valid_combat() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  let attack = CommandRequest::Attack {
    actor: ActorId::new(1),
    target: ActorId::new(2),
  };
  let wait = CommandRequest::Wait {
    actor: ActorId::new(2),
  };

  session
    .act(attack)
    .expect("first attack should be accepted");
  session.act(wait).expect("enemy wait should be accepted");
  session
    .act(attack)
    .expect("second attack should be accepted");

  let inspected = session
    .inspect(ActorId::new(2))
    .expect("dead actor record should remain inspectable");
  assert_eq!(inspected.life(), LifeState::Dead);
  assert_eq!(inspected.hit_points().value(), 0);
}
