//! Contract tests for MCP tester actor spawning.

use dreadstep_mcp::{Session, SessionError};
use dreadstep_protocol::{ActorId, ActorKind, HitPoints, Position, WorldError};

#[test]
fn spawn_adds_a_protocol_actor_visible_through_inspection() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  let history = session.get_history();
  let replay = session.get_replay();
  session
    .spawn(
      ActorId::new(3),
      ActorKind::Enemy,
      Position::new(2, 0),
      HitPoints::new(2),
    )
    .expect("valid spawn should succeed");

  let spawned = session
    .inspect(ActorId::new(3))
    .expect("spawned actor should be inspectable");
  assert_eq!(session.get_history(), history);
  assert_eq!(session.get_replay(), replay);
  assert_eq!(spawned.kind(), ActorKind::Enemy);
  assert_eq!(spawned.position(), Position::new(2, 0));
  assert_eq!(spawned.hit_points(), HitPoints::new(2));
}

#[test]
fn rejected_spawn_is_typed_and_atomic() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  let before = session.inspect_world();
  let history = session.get_history();
  let replay = session.get_replay();

  assert_eq!(
    session.spawn(
      ActorId::new(1),
      ActorKind::Enemy,
      Position::new(2, 0),
      HitPoints::new(2),
    ),
    Err(SessionError::WorldRejected(WorldError::DuplicateActorId(
      ActorId::new(1)
    )))
  );
  assert_eq!(session.inspect_world(), before);
  assert_eq!(session.get_history(), history);
  assert_eq!(session.get_replay(), replay);
}
